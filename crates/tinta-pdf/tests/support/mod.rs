//! Builders that lay out synthetic pages the way a typeset book would.

#![allow(dead_code)]

pub mod pdf;

use tinta_pdf::layout::{Document, Page, TextLine};

pub const PAGE_WIDTH: f32 = 432.0;
pub const PAGE_HEIGHT: f32 = 648.0;
pub const MARGIN: f32 = 54.0;
pub const BODY_SIZE: f32 = 11.0;
pub const LEADING: f32 = 14.0;
pub const HEADER_SIZE: f32 = 8.0;
pub const HEADER_TOP: f32 = 30.0;
pub const FOOTER_TOP: f32 = 610.0;
pub const INDENT: f32 = 16.0;
pub const DROP_CAP_SIZE: f32 = 30.0;
const DROP_CAP_RIGHT: f32 = MARGIN + 10.0;
const WORD_SPACE: f32 = 6.0;
/// Space below a heading, as a fraction of its font size.
const HEADING_SPACE_AFTER: f32 = 0.5;
/// Average glyph width as a fraction of the font size.
const GLYPH_WIDTH: f32 = 0.5;

pub struct PageBuilder {
    lines: Vec<TextLine>,
    cursor: f32,
}

pub fn page() -> PageBuilder {
    PageBuilder {
        lines: Vec::new(),
        cursor: MARGIN,
    }
}

pub fn document(pages: Vec<Page>) -> Document {
    Document {
        title: None,
        author: None,
        pages,
    }
}

fn text_line(text: &str, left: f32, right: f32, top: f32, size: f32) -> TextLine {
    TextLine {
        text: text.to_owned(),
        left,
        right,
        top,
        bottom: top + size,
        font_size: size,
        bold: false,
    }
}

fn natural_width(text: &str, size: f32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let chars = text.chars().count() as f32;
    chars * size * GLYPH_WIDTH
}

impl PageBuilder {
    /// A full-width body line, as in the middle of a justified paragraph.
    pub fn body(mut self, text: &str) -> Self {
        self.push(text_line(
            text,
            MARGIN,
            PAGE_WIDTH - MARGIN,
            self.cursor,
            BODY_SIZE,
        ));
        self
    }

    /// First line of a paragraph, indented like in a printed book.
    pub fn indented(mut self, text: &str) -> Self {
        let line = text_line(
            text,
            MARGIN + INDENT,
            PAGE_WIDTH - MARGIN,
            self.cursor,
            BODY_SIZE,
        );
        self.push(line);
        self
    }

    /// A two-line paragraph: an indented first line and a short last one.
    pub fn paragraph(self, first: &str, last: &str) -> Self {
        self.indented(first).last(last)
    }

    /// Last line of a paragraph: it stops short of the right margin.
    pub fn last(mut self, text: &str) -> Self {
        let right = MARGIN + natural_width(text, BODY_SIZE);
        self.push(text_line(text, MARGIN, right, self.cursor, BODY_SIZE));
        self
    }

    pub fn heading(mut self, text: &str, size: f32) -> Self {
        let right = MARGIN + natural_width(text, size);
        let line = text_line(text, MARGIN, right, self.cursor, size);
        self.cursor = line.bottom + size * HEADING_SPACE_AFTER;
        self.lines.push(line);
        self
    }

    /// A drop cap: a large initial letter with the first lines of text to its
    /// right, starting at the same height.
    pub fn drop_cap(mut self, letter: &str) -> Self {
        let line = text_line(letter, MARGIN, DROP_CAP_RIGHT, self.cursor, DROP_CAP_SIZE);
        self.lines.push(line);
        self
    }

    /// A line wrapped around a drop cap, starting right after it.
    pub fn beside_cap(self, text: &str) -> Self {
        self.wrapped_line(text, DROP_CAP_RIGHT)
    }

    /// A line beside a drop cap that is a whole word, like `I` in
    /// `I suppose`: a word space separates them.
    pub fn beside_cap_after_space(self, text: &str) -> Self {
        self.wrapped_line(text, DROP_CAP_RIGHT + WORD_SPACE)
    }

    fn wrapped_line(mut self, text: &str, left: f32) -> Self {
        self.push(text_line(
            text,
            left,
            PAGE_WIDTH - MARGIN,
            self.cursor,
            BODY_SIZE,
        ));
        self
    }

    pub fn bold(mut self, text: &str) -> Self {
        let right = MARGIN + natural_width(text, BODY_SIZE);
        let mut line = text_line(text, MARGIN, right, self.cursor, BODY_SIZE);
        line.bold = true;
        self.push(line);
        self
    }

    /// Vertical whitespace of one blank line.
    pub fn gap(mut self) -> Self {
        self.cursor += LEADING;
        self
    }

    pub fn header(mut self, text: &str) -> Self {
        let right = MARGIN + natural_width(text, HEADER_SIZE);
        let line = text_line(text, MARGIN, right, HEADER_TOP, HEADER_SIZE);
        self.lines.insert(0, line);
        self
    }

    pub fn footer(mut self, text: &str) -> Self {
        let center = PAGE_WIDTH / 2.0;
        let line = text_line(text, center, center + HEADER_SIZE, FOOTER_TOP, HEADER_SIZE);
        self.lines.push(line);
        self
    }

    pub fn build(self) -> Page {
        Page {
            width: PAGE_WIDTH,
            height: PAGE_HEIGHT,
            lines: self.lines,
            image_count: 0,
        }
    }

    fn push(&mut self, line: TextLine) {
        self.cursor = line.top + LEADING;
        self.lines.push(line);
    }
}
