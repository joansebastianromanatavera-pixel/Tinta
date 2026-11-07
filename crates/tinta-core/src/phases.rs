//! Built-in pipeline phases that operate on the book model.

use crate::model::{Block, Book};
use crate::pipeline::{Phase, PhaseError};
use crate::typography;

/// Deep typographic cleanup of every text block (phase 6).
pub struct TypographyCleanup;

impl Phase for TypographyCleanup {
    fn name(&self) -> &'static str {
        "Limpieza tipográfica"
    }

    fn apply(&self, mut book: Book) -> Result<Book, PhaseError> {
        for chapter in book.chapters_mut() {
            chapter.title = typography::clean(&chapter.title);
            for text in chapter.blocks.iter_mut().filter_map(Block::text_mut) {
                *text = typography::clean(text);
            }
        }
        Ok(book)
    }
}

/// Marks the first paragraph of each chapter so it is rendered with a drop
/// cap and small caps, like a professionally typeset edition.
pub struct MarkOpeningParagraphs;

impl Phase for MarkOpeningParagraphs {
    fn name(&self) -> &'static str {
        "Inicio de capítulos"
    }

    fn apply(&self, mut book: Book) -> Result<Book, PhaseError> {
        for chapter in book.chapters_mut() {
            let mut paragraphs = chapter.blocks.iter_mut().filter_map(|block| match block {
                Block::Paragraph { opening, .. } => Some(opening),
                _ => None,
            });
            if let Some(first) = paragraphs.next() {
                *first = true;
            }
            paragraphs.for_each(|opening| *opening = false);
        }
        Ok(book)
    }
}

/// Removes chapters that contain no blocks, e.g. blank pages in a PDF.
pub struct DropEmptyChapters;

impl Phase for DropEmptyChapters {
    fn name(&self) -> &'static str {
        "Capítulos vacíos"
    }

    fn apply(&self, mut book: Book) -> Result<Book, PhaseError> {
        for part in &mut book.parts {
            part.chapters.retain(|chapter| !chapter.blocks.is_empty());
        }
        book.parts.retain(|part| !part.chapters.is_empty());
        if book.parts.is_empty() {
            return Err(PhaseError("El libro no tiene contenido".into()));
        }
        Ok(book)
    }
}
