//! PDF source for Tinta.
//!
//! Reading happens in two layers. The [`pdfium`] adapter is the only code
//! that talks to the PDF engine: it turns pages into positioned lines
//! ([`layout`]). Everything else is pure and rebuilds the book from those
//! lines: page furniture is removed, paragraphs are rejoined and heading
//! sizes become the two-level table of contents.

pub mod analysis;
pub mod layout;
pub mod pdfium;

mod furniture;
mod metrics;
mod paragraphs;
mod structure;

use furniture::text_key;
use layout::Document;
use metrics::Metrics;
use std::collections::HashSet;
use structure::Outline;
use tinta_core::{Book, Metadata};

pub use pdfium::{read, PdfError, PdfiumLibrary};

/// Rebuilds a book from the positioned lines of a document.
///
/// `fallback_title` (usually the file name) names the book when neither the
/// PDF metadata nor a title page provide one.
pub fn rebuild(document: &Document, fallback_title: &str) -> Book {
    let mut pages = document.pages.clone();
    pages.iter_mut().for_each(layout::Page::sort_lines);
    let metrics = Metrics::measure(&pages);
    let titles = known_titles(document, &pages, metrics);
    furniture::strip(&mut pages, metrics, &titles);
    let outline = Outline::new(paragraphs::assemble(&pages, metrics));
    let title = plausible(document.title.as_deref())
        .or(outline.title_page.clone())
        .unwrap_or_else(|| fallback_title.to_owned());
    let metadata = Metadata {
        title,
        authors: plausible(document.author.as_deref()).into_iter().collect(),
        ..Metadata::default()
    };
    outline.into_book(metadata)
}

/// Titles a running header may repeat: the book title and every heading.
fn known_titles(document: &Document, pages: &[layout::Page], metrics: Metrics) -> HashSet<String> {
    pages
        .iter()
        .flat_map(|page| &page.lines)
        .filter(|line| metrics.is_heading_size(line.font_size))
        .map(|line| line.text.as_str())
        .chain(document.title.as_deref())
        .map(text_key)
        .filter(|key| !key.is_empty())
        .collect()
}

/// Prefixes that word processors leave in the title field of exported PDFs.
const GENERATED_TITLE_PREFIXES: [&str; 2] = ["Microsoft Word - ", "untitled"];
const FILE_EXTENSIONS: [&str; 6] = [".doc", ".docx", ".odt", ".pdf", ".rtf", ".tex"];

/// Metadata text worth showing, discarding generated or empty values.
fn plausible(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    let lower = value.to_lowercase();
    let generated = GENERATED_TITLE_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(&prefix.to_lowercase()))
        || FILE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext));
    (!value.is_empty() && !generated).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::plausible;

    #[test]
    fn generated_metadata_is_ignored() {
        assert_eq!(plausible(Some("Microsoft Word - tesis.docx")), None);
        assert_eq!(plausible(Some("capitulo1.pdf")), None);
        assert_eq!(plausible(Some("   ")), None);
        assert_eq!(plausible(None), None);
    }

    #[test]
    fn real_titles_are_kept_trimmed() {
        assert_eq!(plausible(Some(" Niebla ")), Some("Niebla".into()));
    }
}
