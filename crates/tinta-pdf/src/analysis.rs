//! First look at a PDF: does it carry real text, or only pictures of pages?

use crate::layout::Document;

/// Below this many characters per page on average, the text layer is too
/// thin to be the book itself (it may be page numbers or an OCR stub).
const MIN_CHARS_PER_PAGE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Content {
    /// Selectable text that can be rebuilt into a book.
    Text,
    /// Scanned pages: images without a usable text layer.
    Scanned,
    /// Neither text nor images.
    Empty,
}

pub fn classify(document: &Document) -> Content {
    let pages = document.pages.len();
    if pages == 0 {
        return Content::Empty;
    }
    let chars: usize = document
        .pages
        .iter()
        .map(crate::layout::Page::char_count)
        .sum();
    let pages_with_images = document
        .pages
        .iter()
        .filter(|page| page.image_count > 0)
        .count();
    if chars / pages >= MIN_CHARS_PER_PAGE {
        Content::Text
    } else if pages_with_images * 2 >= pages {
        Content::Scanned
    } else if chars == 0 {
        Content::Empty
    } else {
        Content::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Page, TextLine};

    fn page(text: &str, image_count: usize) -> Page {
        Page {
            lines: vec![TextLine {
                text: text.into(),
                ..TextLine::default()
            }],
            image_count,
            ..Page::default()
        }
    }

    #[test]
    fn pages_full_of_text_are_text() {
        let doc = Document {
            pages: vec![page(&"palabra ".repeat(50), 0)],
            ..Document::default()
        };
        assert_eq!(classify(&doc), Content::Text);
    }

    #[test]
    fn images_with_almost_no_text_are_a_scan() {
        let doc = Document {
            pages: vec![page("12", 1), page("13", 1)],
            ..Document::default()
        };
        assert_eq!(classify(&doc), Content::Scanned);
    }

    #[test]
    fn nothing_at_all_is_empty() {
        let doc = Document {
            pages: vec![page("", 0)],
            ..Document::default()
        };
        assert_eq!(classify(&doc), Content::Empty);
        assert_eq!(classify(&Document::default()), Content::Empty);
    }

    #[test]
    fn a_short_text_document_is_still_text() {
        let doc = Document {
            pages: vec![page("Una nota breve.", 0)],
            ..Document::default()
        };
        assert_eq!(classify(&doc), Content::Text);
    }
}
