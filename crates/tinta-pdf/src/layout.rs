//! Positioned text as it sits on the pages of a PDF.
//!
//! This is the boundary between the PDF engine and the reconstruction logic:
//! the adapter fills it, everything else only reads it. Coordinates are in
//! PDF points and grow downwards from the top-left corner of the page.

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    /// Title from the PDF metadata, if any.
    pub title: Option<String>,
    /// Author from the PDF metadata, if any.
    pub author: Option<String>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Page {
    pub width: f32,
    pub height: f32,
    pub lines: Vec<TextLine>,
    /// Number of raster images drawn on the page.
    pub image_count: usize,
}

/// One visual line of text: glyphs that share a baseline.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextLine {
    pub text: String,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    /// Dominant font size of the line, in points.
    pub font_size: f32,
    /// Every glyph of the line is set in a bold face.
    pub bold: bool,
}

impl TextLine {
    pub fn char_count(&self) -> usize {
        self.text.chars().filter(|c| !c.is_whitespace()).count()
    }

    /// Vertical whitespace between this line and one below it.
    pub fn gap_to(&self, below: &TextLine) -> f32 {
        below.top - self.bottom
    }

    pub fn overlaps_vertically(&self, other: &TextLine) -> bool {
        self.top < other.bottom && other.top < self.bottom
    }
}

impl Page {
    /// Sorts lines into reading order: top to bottom, then left to right.
    pub(crate) fn sort_lines(&mut self) {
        self.lines
            .sort_by(|a, b| a.top.total_cmp(&b.top).then(a.left.total_cmp(&b.left)));
    }

    pub fn char_count(&self) -> usize {
        self.lines.iter().map(TextLine::char_count).sum()
    }
}
