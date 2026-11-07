//! Adapter over the pdfium engine: the only module that touches it.
//!
//! It reads every glyph with its position, size and weight and groups them
//! into the [`TextLine`]s of the layout model. pdfium is a native library
//! loaded at runtime; see `scripts/fetch-pdfium.sh`.

use crate::analysis::{classify, Content};
use crate::layout::{Document, Page, TextLine};
use crate::metrics::size_key;
use crate::rebuild;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use thiserror::Error;
use tinta_core::Book;

/// Overrides the folder searched for the pdfium library.
pub const LIBRARY_DIR_VARIABLE: &str = "TINTA_PDFIUM_DIR";
/// Fonts at or above this weight are bold.
const BOLD_WEIGHT: u32 = 600;
const BOLD_NAME_MARKERS: [&str; 4] = ["bold", "black", "heavy", "semibold"];
/// A glyph whose baseline moves more than this fraction of its size has
/// started a new line.
const BASELINE_JUMP_RATIO: f32 = 0.5;
/// Rotated glyphs (margin notes, watermarks) are not part of the text flow.
const MAX_TEXT_ANGLE_DEGREES: f32 = 1.0;
/// pdfium reports the hyphen that breaks a word at the end of a line as
/// this control character.
const ENGINE_HYPHEN: char = '\u{2}';

/// What a character from the engine means for line building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Glyph {
    Visible(char),
    Space,
    LineBreak,
}

fn normalize_glyph(c: char) -> Option<Glyph> {
    match c {
        '\n' | '\r' => Some(Glyph::LineBreak),
        ENGINE_HYPHEN => Some(Glyph::Visible('-')),
        c if c.is_whitespace() => Some(Glyph::Space),
        c if c.is_control() => None,
        c => Some(Glyph::Visible(c)),
    }
}

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("No se encontró el motor de PDF (pdfium). Vuelve a instalar Tinta.")]
    LibraryMissing,
    #[error("El archivo no es un PDF válido o está dañado.")]
    Invalid,
    #[error("El PDF está protegido con contraseña.")]
    Encrypted,
    #[error("No se pudo abrir el archivo.")]
    Unreadable,
    #[error("Este PDF es un escaneo: sus páginas son imágenes sin texto.")]
    Scanned,
    #[error("El PDF no tiene texto.")]
    Empty,
    #[error("No se pudo leer la página {0}.")]
    Page(usize),
}

/// The pdfium engine, bound once per process.
pub struct PdfiumLibrary(&'static Pdfium);

static ENGINE: OnceLock<Pdfium> = OnceLock::new();
static BINDING: Mutex<()> = Mutex::new(());

impl PdfiumLibrary {
    /// Binds the pdfium library, looking first in the folder named by
    /// [`LIBRARY_DIR_VARIABLE`], then in `search_dirs`, then in the system.
    ///
    /// # Errors
    /// [`PdfError::LibraryMissing`] when no usable library is found.
    pub fn load(search_dirs: &[PathBuf]) -> Result<Self, PdfError> {
        let _guard = BINDING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(engine) = ENGINE.get() {
            return Ok(Self(engine));
        }
        let bindings = bind(search_dirs).ok_or(PdfError::LibraryMissing)?;
        Ok(Self(ENGINE.get_or_init(|| Pdfium::new(bindings))))
    }
}

fn bind(search_dirs: &[PathBuf]) -> Option<Box<dyn PdfiumLibraryBindings>> {
    let from_env = std::env::var_os(LIBRARY_DIR_VARIABLE).map(PathBuf::from);
    from_env
        .iter()
        .chain(search_dirs)
        .find_map(|dir| {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dir)).ok()
        })
        .or_else(|| Pdfium::bind_to_system_library().ok())
}

/// Reads a PDF into a book. `on_page` receives `(pages_read, page_count)`.
///
/// # Errors
/// See [`PdfError`]: unreadable, encrypted, scanned or empty documents.
pub fn read(
    library: &PdfiumLibrary,
    path: &Path,
    on_page: &dyn Fn(usize, usize),
) -> Result<Book, PdfError> {
    let document = extract(library, path, on_page)?;
    match classify(&document) {
        Content::Text => {
            let fallback = path
                .file_stem()
                .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
            Ok(rebuild(&document, &fallback))
        }
        Content::Scanned => Err(PdfError::Scanned),
        Content::Empty => Err(PdfError::Empty),
    }
}

/// Extracts the positioned lines of every page.
///
/// # Errors
/// See [`PdfError`].
pub fn extract(
    library: &PdfiumLibrary,
    path: &Path,
    on_page: &dyn Fn(usize, usize),
) -> Result<Document, PdfError> {
    let pdf = library
        .0
        .load_pdf_from_file(path, None)
        .map_err(|error| open_error(&error))?;
    let metadata = |tag| {
        pdf.metadata()
            .get(tag)
            .map(|entry| entry.value().to_owned())
    };
    let total = usize::try_from(pdf.pages().len()).unwrap_or_default();
    let mut pages = Vec::with_capacity(total);
    for (index, page) in pdf.pages().iter().enumerate() {
        pages.push(extract_page(&page).ok_or(PdfError::Page(index + 1))?);
        on_page(index + 1, total);
    }
    Ok(Document {
        title: metadata(PdfDocumentMetadataTagType::Title),
        author: metadata(PdfDocumentMetadataTagType::Author),
        pages,
    })
}

fn open_error(error: &PdfiumError) -> PdfError {
    match error {
        PdfiumError::PdfiumLibraryInternalError(
            PdfiumInternalError::PasswordError | PdfiumInternalError::SecurityError,
        ) => PdfError::Encrypted,
        PdfiumError::IoError(_)
        | PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::FileError) => {
            PdfError::Unreadable
        }
        _ => PdfError::Invalid,
    }
}

fn extract_page(page: &PdfPage) -> Option<Page> {
    let height = page.height().value;
    let text = page.text().ok()?;
    let mut lines = LineCollector::new(height);
    for glyph in text.chars().iter() {
        lines.push(&glyph);
    }
    let image_count = page
        .objects()
        .iter()
        .filter(|object| object.object_type() == PdfPageObjectType::Image)
        .count();
    Some(Page {
        width: page.width().value,
        height,
        lines: lines.finish(),
        image_count,
    })
}

/// Groups glyphs, in content order, into lines.
struct LineCollector {
    page_height: f32,
    lines: Vec<TextLine>,
    current: Option<LineDraft>,
}

struct LineDraft {
    text: String,
    baseline: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    /// Font size of every visible glyph, to pick the dominant one.
    sizes: Vec<f32>,
    all_bold: bool,
}

impl LineCollector {
    fn new(page_height: f32) -> Self {
        Self {
            page_height,
            lines: Vec::new(),
            current: None,
        }
    }

    fn push(&mut self, glyph: &PdfPageTextChar) {
        let c = match glyph.unicode_char().and_then(normalize_glyph) {
            None => return,
            Some(Glyph::LineBreak) => return self.end_line(),
            Some(Glyph::Space) => {
                if let Some(line) = &mut self.current {
                    line.text.push(' ');
                }
                return;
            }
            Some(Glyph::Visible(c)) => c,
        };
        let rotated = glyph
            .angle_degrees()
            .is_ok_and(|angle| angle.abs() > MAX_TEXT_ANGLE_DEGREES);
        let Ok(bounds) = glyph.loose_bounds() else {
            return;
        };
        if rotated {
            return;
        }
        self.push_visible(c, &bounds, glyph.scaled_font_size().value, is_bold(glyph));
    }

    fn push_visible(&mut self, c: char, bounds: &PdfRect, size: f32, bold: bool) {
        let baseline = bounds.bottom().value;
        let jumped = self.current.as_ref().is_some_and(|line| {
            (line.baseline - baseline).abs() > size * BASELINE_JUMP_RATIO
                || bounds.right().value < line.left
        });
        if jumped {
            self.end_line();
        }
        let top = self.page_height - bounds.top().value;
        let bottom = self.page_height - baseline;
        let line = self.current.get_or_insert_with(|| LineDraft {
            text: String::new(),
            baseline,
            left: bounds.left().value,
            right: bounds.right().value,
            top,
            bottom,
            sizes: Vec::new(),
            all_bold: true,
        });
        line.text.push(c);
        line.left = line.left.min(bounds.left().value);
        line.right = line.right.max(bounds.right().value);
        line.top = line.top.min(top);
        line.bottom = line.bottom.max(bottom);
        line.sizes.push(size);
        line.all_bold &= bold;
    }

    fn end_line(&mut self) {
        if let Some(draft) = self.current.take() {
            if let Some(line) = draft.into_line() {
                self.lines.push(line);
            }
        }
    }

    fn finish(mut self) -> Vec<TextLine> {
        self.end_line();
        self.lines
    }
}

impl LineDraft {
    fn into_line(mut self) -> Option<TextLine> {
        let text = self.text.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.is_empty() {
            return None;
        }
        Some(TextLine {
            font_size: self.dominant_size(),
            text,
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
            bold: self.all_bold,
        })
    }

    /// The size of most glyphs, so a superscript note mark does not count.
    fn dominant_size(&mut self) -> f32 {
        self.sizes.sort_by(f32::total_cmp);
        self.sizes
            .chunk_by(|a, b| size_key(*a) == size_key(*b))
            .max_by_key(|group| group.len())
            .map_or(0.0, |group| group[0])
    }
}

fn is_bold(glyph: &PdfPageTextChar) -> bool {
    let heavy = match glyph.font_weight() {
        Some(PdfFontWeight::Custom(weight)) => weight >= BOLD_WEIGHT,
        Some(
            PdfFontWeight::Weight600
            | PdfFontWeight::Weight700Bold
            | PdfFontWeight::Weight800
            | PdfFontWeight::Weight900,
        ) => true,
        _ => false,
    };
    heavy || {
        let name = glyph.font_name().to_lowercase();
        BOLD_NAME_MARKERS.iter().any(|marker| name.contains(marker))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_hyphen_marker_becomes_a_hyphen() {
        assert_eq!(normalize_glyph(ENGINE_HYPHEN), Some(Glyph::Visible('-')));
    }

    #[test]
    fn control_characters_are_dropped_and_breaks_end_lines() {
        assert_eq!(normalize_glyph('\u{1}'), None);
        assert_eq!(normalize_glyph('\n'), Some(Glyph::LineBreak));
        assert_eq!(normalize_glyph('\t'), Some(Glyph::Space));
        assert_eq!(normalize_glyph('a'), Some(Glyph::Visible('a')));
    }
}
