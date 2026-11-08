//! The conversion use case: a source file becomes a Kindle-ready book.
//!
//! Progress is one fraction across the whole run, weighted by how long each
//! stage takes: reading the PDF dominates, the pipeline phases and the
//! packaging are quick.

use serde::{Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use tinta_core::phases::standard_pipeline;
use tinta_core::{Book, PipelineError, Progress};
use tinta_epub::{EpubError, EpubOptions};
use tinta_pdf::{PdfError, PdfiumLibrary};

/// Share of the bar spent reading pages.
const READING_SHARE: f32 = 0.70;
/// Share spent in the pipeline phases; packaging takes the rest.
const PHASES_SHARE: f32 = 0.25;
const PACKAGING_START: f32 = READING_SHARE + PHASES_SHARE;
pub const READING_LABEL: &str = "Leyendo el PDF";
pub const PACKAGING_LABEL: &str = "Empaquetando";
pub const SUPPORTED_EXTENSION: &str = "pdf";
pub const OUTPUT_EXTENSION: &str = "epub";
/// Characters Windows, macOS or Linux reject in file names.
const FORBIDDEN_FILE_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const MAX_FILE_STEM_CHARS: usize = 120;
const FALLBACK_FILE_STEM: &str = "libro";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProgressEvent {
    /// Completed share of the whole conversion, `0.0..=1.0`.
    pub fraction: f32,
    /// What is happening now, ready to show.
    pub stage: &'static str,
}

/// What the interface shows once the book is ready.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Converted {
    pub title: String,
    pub authors: Vec<String>,
    pub language: Option<String>,
    pub chapters: usize,
    pub bytes: u64,
    pub file_name: String,
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Por ahora Tinta convierte archivos PDF. EPUB, Word y más formatos llegarán pronto.")]
    Unsupported,
    #[error("{0}")]
    Pdf(#[from] PdfError),
    #[error("{}", .0.source)]
    Pipeline(#[from] PipelineError),
    #[error("{0}")]
    Epub(#[from] EpubError),
    #[error("No se pudo guardar el libro: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ya hay una conversión en curso.")]
    Busy,
    #[error("Todavía no hay un libro convertido.")]
    NothingToSave,
}

impl ConversionError {
    /// Stable identifier the interface uses to pick the error state.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::Pdf(PdfError::Scanned) => "scanned",
            Self::Pdf(PdfError::Encrypted) => "encrypted",
            Self::Pdf(PdfError::Empty) | Self::Pipeline(_) => "empty",
            Self::Pdf(PdfError::LibraryMissing) => "engine",
            Self::Pdf(_) => "unreadable",
            Self::Epub(_) | Self::Io(_) => "io",
            Self::Busy => "busy",
            Self::NothingToSave => "nothing",
        }
    }
}

impl Serialize for ConversionError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ConversionError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

/// Converts `source` into an EPUB inside `output_dir`.
///
/// # Errors
/// See [`ConversionError`].
pub fn convert(
    library: &PdfiumLibrary,
    source: &Path,
    output_dir: &Path,
    report: &dyn Fn(ProgressEvent),
) -> Result<(PathBuf, Converted), ConversionError> {
    if !is_supported(source) {
        return Err(ConversionError::Unsupported);
    }
    report(reading_progress(0, 1));
    let raw = tinta_pdf::read(library, source, &|done, total| {
        report(reading_progress(done, total));
    })?;
    let book = standard_pipeline().run(raw, &|progress: Progress| {
        report(phase_progress(progress));
    })?;
    report(ProgressEvent {
        fraction: PACKAGING_START,
        stage: PACKAGING_LABEL,
    });
    let bytes = tinta_epub::to_bytes(&book, &EpubOptions::for_book(&book, SystemTime::now()))?;
    std::fs::create_dir_all(output_dir)?;
    let file_name = format!("{}.{OUTPUT_EXTENSION}", file_stem(&book.metadata.title));
    let path = output_dir.join(&file_name);
    std::fs::write(&path, &bytes)?;
    report(ProgressEvent {
        fraction: 1.0,
        stage: PACKAGING_LABEL,
    });
    Ok((path, summary(&book, bytes.len(), file_name)))
}

fn summary(book: &Book, bytes: usize, file_name: String) -> Converted {
    Converted {
        title: book.metadata.title.clone(),
        authors: book.metadata.authors.clone(),
        language: book.metadata.language.clone(),
        chapters: book.chapter_count(),
        bytes: u64::try_from(bytes).unwrap_or(u64::MAX),
        file_name,
    }
}

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(SUPPORTED_EXTENSION))
}

#[allow(clippy::cast_precision_loss)]
pub fn reading_progress(done: usize, total: usize) -> ProgressEvent {
    let read = if total == 0 {
        0.0
    } else {
        done as f32 / total as f32
    };
    ProgressEvent {
        fraction: read.clamp(0.0, 1.0) * READING_SHARE,
        stage: READING_LABEL,
    }
}

pub fn phase_progress(progress: Progress) -> ProgressEvent {
    ProgressEvent {
        fraction: READING_SHARE + progress.fraction() * PHASES_SHARE,
        stage: progress.phase,
    }
}

/// A file name every system accepts, taken from the book title.
pub fn file_stem(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| {
            if FORBIDDEN_FILE_CHARS.contains(&c) || c.is_control() {
                ' '
            } else {
                c
            }
        })
        .collect();
    let stem: String = cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_FILE_STEM_CHARS)
        .collect();
    let stem = stem.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if stem.is_empty() {
        FALLBACK_FILE_STEM.to_owned()
    } else {
        stem.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn reading_fills_the_first_part_of_the_bar() {
        assert!(close(reading_progress(0, 10).fraction, 0.0));
        assert!(close(reading_progress(5, 10).fraction, READING_SHARE / 2.0));
        assert!(close(reading_progress(10, 10).fraction, READING_SHARE));
        assert_eq!(reading_progress(3, 10).stage, READING_LABEL);
    }

    #[test]
    fn a_document_without_pages_does_not_divide_by_zero() {
        assert!(close(reading_progress(0, 0).fraction, 0.0));
    }

    #[test]
    fn phases_continue_where_reading_stopped() {
        let first = phase_progress(Progress {
            phase: "Idioma",
            index: 0,
            total: 4,
        });
        let last = phase_progress(Progress {
            phase: "Validación",
            index: 3,
            total: 4,
        });

        assert!(close(first.fraction, READING_SHARE));
        assert_eq!(first.stage, "Idioma");
        assert!(last.fraction < PACKAGING_START);
    }

    #[test]
    fn file_names_drop_characters_systems_reject() {
        assert_eq!(file_stem("¿Qué es: esto/aquello?"), "¿Qué es esto aquello");
        assert_eq!(file_stem("  Niebla  "), "Niebla");
        assert_eq!(file_stem("..."), FALLBACK_FILE_STEM);
        assert_eq!(file_stem(""), FALLBACK_FILE_STEM);
    }

    #[test]
    fn long_titles_are_cut_to_a_safe_length() {
        let stem = file_stem(&"a".repeat(500));
        assert_eq!(stem.chars().count(), MAX_FILE_STEM_CHARS);
    }

    #[test]
    fn only_pdf_is_supported_for_now() {
        assert!(is_supported(Path::new("/libros/Niebla.PDF")));
        assert!(!is_supported(Path::new("/libros/Niebla.epub")));
        assert!(!is_supported(Path::new("/libros/Niebla")));
    }

    #[test]
    fn errors_reach_the_interface_with_a_kind_and_a_message() {
        let error = ConversionError::Pdf(PdfError::Scanned);

        assert_eq!(error.kind(), "scanned");
        assert_eq!(error.to_string(), PdfError::Scanned.to_string());
    }
}
