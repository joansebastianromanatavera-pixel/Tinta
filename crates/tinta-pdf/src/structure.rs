//! Builds the two-level table of contents from the heading sizes.
//!
//! Each distinct heading size is a level. The largest size is a part only
//! when every heading of that size opens straight into a heading of the next
//! size (a part page followed by its first chapter). The chapter level is the
//! largest remaining size that repeats. Smaller sizes become section
//! headings inside chapters.
//! A lone, largest heading that opens the document is the title page.

use crate::metrics::size_key;
use crate::paragraphs::Piece;
use std::collections::BTreeMap;
use tinta_core::{Block, Book, Chapter, Metadata, Part};

/// Section headings start at this level; the chapter title is level 1.
const FIRST_SECTION_LEVEL: u8 = 2;
const DEEPEST_HEADING_LEVEL: u8 = 6;
/// The chapter level is the largest heading size used at least this many
/// times; larger one-off headings (an interlude, an epilogue in a bigger
/// face) are chapters too instead of stealing the level.
const MIN_CHAPTER_HEADINGS: usize = 2;

/// What a heading of a given size means in the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rank {
    Part,
    Chapter,
    Section(u8),
}

pub(crate) struct Outline {
    /// Title read from the title page, if the document has one.
    pub title_page: Option<String>,
    pieces: Vec<Piece>,
    ranks: BTreeMap<u32, Rank>,
}

impl Outline {
    pub fn new(mut pieces: Vec<Piece>) -> Self {
        let title_page = take_title_page(&mut pieces);
        let ranks = rank_sizes(&pieces);
        Self {
            title_page,
            pieces,
            ranks,
        }
    }

    /// Lays the pieces out as parts and chapters. Text that comes before
    /// the first chapter heading goes into a chapter named after the book.
    pub fn into_book(self, metadata: Metadata) -> Book {
        let book_title = metadata.title.clone();
        let mut layout = Layout::new(&book_title);
        for piece in self.pieces {
            match piece {
                Piece::Paragraph(text) => layout.push_block(Block::paragraph(text)),
                Piece::Heading { text, size } => match self.ranks[&size_key(size)] {
                    Rank::Part => layout.open_part(text),
                    Rank::Chapter => layout.open_chapter(text),
                    Rank::Section(level) => layout.push_block(Block::Heading { level, text }),
                },
            }
        }
        Book {
            metadata,
            parts: layout.finish(),
        }
    }
}

fn heading_counts(pieces: &[Piece]) -> BTreeMap<u32, usize> {
    let mut counts = BTreeMap::new();
    for piece in pieces {
        if let Piece::Heading { size, .. } = piece {
            *counts.entry(size_key(*size)).or_default() += 1;
        }
    }
    counts
}

fn take_title_page(pieces: &mut Vec<Piece>) -> Option<String> {
    let counts = heading_counts(pieces);
    let heading_total: usize = counts.values().sum();
    let largest = counts.keys().next_back().copied()?;
    match pieces.first() {
        Some(Piece::Heading { size, .. })
            if size_key(*size) == largest && counts[&largest] == 1 && heading_total > 1 =>
        {
            match pieces.remove(0) {
                Piece::Heading { text, .. } => Some(text),
                Piece::Paragraph(_) => None,
            }
        }
        _ => None,
    }
}

fn rank_sizes(pieces: &[Piece]) -> BTreeMap<u32, Rank> {
    let counts = heading_counts(pieces);
    let mut sizes: Vec<u32> = counts.keys().rev().copied().collect();
    let mut ranks = BTreeMap::new();
    if let [largest, next, ..] = sizes[..] {
        if opens_into(pieces, largest, next) {
            ranks.insert(largest, Rank::Part);
            sizes.remove(0);
        }
    }
    let Some(chapter) = sizes
        .iter()
        .position(|size| counts[size] >= MIN_CHAPTER_HEADINGS)
        .or((!sizes.is_empty()).then_some(0))
    else {
        return ranks;
    };
    for &size in &sizes[..=chapter] {
        ranks.insert(size, Rank::Chapter);
    }
    for (&size, level) in sizes[chapter + 1..].iter().zip(FIRST_SECTION_LEVEL..) {
        ranks.insert(size, Rank::Section(level.min(DEEPEST_HEADING_LEVEL)));
    }
    ranks
}

/// Whether every heading of size `outer` is immediately followed by a
/// heading of size `inner`.
fn opens_into(pieces: &[Piece], outer: u32, inner: u32) -> bool {
    let is_heading_of = |piece: Option<&Piece>, key: u32| matches!(piece, Some(Piece::Heading { size, .. }) if size_key(*size) == key);
    pieces
        .iter()
        .enumerate()
        .filter(|(_, piece)| is_heading_of(Some(piece), outer))
        .all(|(index, _)| is_heading_of(pieces.get(index + 1), inner))
}

/// Accumulates parts and chapters while walking the pieces in order.
struct Layout<'t> {
    book_title: &'t str,
    parts: Vec<Part>,
    part: Part,
    chapter: Option<Chapter>,
}

impl<'t> Layout<'t> {
    fn new(book_title: &'t str) -> Self {
        Self {
            book_title,
            parts: Vec::new(),
            part: Part::default(),
            chapter: None,
        }
    }

    fn push_block(&mut self, block: Block) {
        let book_title = self.book_title;
        self.chapter
            .get_or_insert_with(|| Chapter {
                title: book_title.to_owned(),
                blocks: Vec::new(),
            })
            .blocks
            .push(block);
    }

    fn open_chapter(&mut self, title: String) {
        self.close_chapter();
        self.chapter = Some(Chapter {
            title,
            blocks: Vec::new(),
        });
    }

    fn open_part(&mut self, title: String) {
        self.close_part();
        self.part.title = Some(title);
    }

    fn close_chapter(&mut self) {
        if let Some(chapter) = self.chapter.take() {
            self.part.chapters.push(chapter);
        }
    }

    fn close_part(&mut self) {
        self.close_chapter();
        let part = std::mem::take(&mut self.part);
        if part.title.is_some() || !part.chapters.is_empty() {
            self.parts.push(part);
        }
    }

    fn finish(mut self) -> Vec<Part> {
        self.close_part();
        self.parts
    }
}
