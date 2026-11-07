//! Turns the lines of every page into a flat run of headings and paragraphs.
//!
//! Lines of a paragraph are joined with `\n`: the typography phase of the
//! core pipeline later rejoins words hyphenated across lines. A paragraph
//! starts on a first-line indent, on extra vertical space, or after a short
//! line that closes a sentence. Page breaks alone never split a paragraph.

use crate::layout::{Page, TextLine};
use crate::metrics::Metrics;

/// Indent, in ems, that marks the first line of a paragraph.
const INDENT_EM: f32 = 0.8;
/// Extra space, in ems, between block-style paragraphs.
const PARAGRAPH_GAP_EM: f32 = 0.5;
/// A line that stops this many ems before the right margin is short.
const SHORT_LINE_EM: f32 = 2.0;
/// A gap wider than this, in ems, between a drop cap and its text means the
/// cap is a whole word (`I suppose`), not the first letter of one.
const DROP_CAP_WORD_SPACE_EM: f32 = 0.3;
/// Lines wrapped around a drop cap start within this many ems of each other.
const WRAP_EDGE_TOLERANCE_EM: f32 = 0.1;
/// Heading lines closer than this many of their own size are one title.
const HEADING_MERGE_GAP_RATIO: f32 = 2.5;
/// Characters that close a sentence, possibly followed by a closing quote.
const SENTENCE_END: [char; 4] = ['.', '!', '?', '…'];
const CLOSING_MARKS: [char; 5] = ['"', '”', '»', ')', '’'];
/// Characters after which a stacked heading continues without a period.
const TITLE_JOINERS: [char; 6] = ['.', ':', '—', '–', '-', ','];

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Piece {
    Heading { text: String, size: f32 },
    Paragraph(String),
}

/// Geometry of the text block of one page.
struct Frame {
    left: f32,
    right: f32,
}

/// The last body line seen, possibly on a previous page.
struct PreviousLine {
    page: usize,
    bottom: f32,
    closes_paragraph: bool,
}

/// The last heading line seen, while no text follows it yet.
struct PreviousHeading {
    page: usize,
    bottom: f32,
}

pub(crate) fn assemble(pages: &[Page], metrics: Metrics) -> Vec<Piece> {
    let pages: Vec<Vec<TextLine>> = pages
        .iter()
        .map(|page| merge_drop_caps(&page.lines, metrics))
        .collect();
    let frames = Frames::measure(&pages, metrics);
    let mut builder = Builder::new(metrics);
    for (index, lines) in pages.iter().enumerate() {
        for line in lines {
            builder.push(index, line, frames.for_page(index));
        }
    }
    builder.finish()
}

struct Builder {
    metrics: Metrics,
    pieces: Vec<Piece>,
    paragraph: Option<String>,
    previous_line: Option<PreviousLine>,
    previous_heading: Option<PreviousHeading>,
}

impl Builder {
    fn new(metrics: Metrics) -> Self {
        Self {
            metrics,
            pieces: Vec::new(),
            paragraph: None,
            previous_line: None,
            previous_heading: None,
        }
    }

    fn push(&mut self, page: usize, line: &TextLine, frame: &Frame) {
        if self.metrics.is_heading_size(line.font_size) {
            self.push_heading(page, line);
        } else {
            self.push_body(page, line, frame);
        }
    }

    fn push_heading(&mut self, page: usize, line: &TextLine) {
        self.flush_paragraph();
        self.previous_line = None;
        let continues = self.previous_heading.as_ref().is_some_and(|previous| {
            previous.page == page
                && previous.bottom + line.font_size * HEADING_MERGE_GAP_RATIO >= line.top
        });
        match self.pieces.last_mut() {
            Some(Piece::Heading { text, size }) if continues => {
                join_title(text, &line.text);
                *size = size.max(line.font_size);
            }
            _ => self.pieces.push(Piece::Heading {
                text: line.text.trim().to_owned(),
                size: line.font_size,
            }),
        }
        self.previous_heading = Some(PreviousHeading {
            page,
            bottom: line.bottom,
        });
    }

    fn push_body(&mut self, page: usize, line: &TextLine, frame: &Frame) {
        self.previous_heading = None;
        let text = line.text.trim();
        if self.starts_paragraph(page, line, frame) {
            self.flush_paragraph();
            self.paragraph = Some(text.to_owned());
        } else if let Some(paragraph) = &mut self.paragraph {
            paragraph.push('\n');
            paragraph.push_str(text);
        }
        self.previous_line = Some(PreviousLine {
            page,
            bottom: line.bottom,
            closes_paragraph: line.right < frame.right - self.metrics.em(SHORT_LINE_EM)
                && ends_sentence(text),
        });
    }

    fn starts_paragraph(&self, page: usize, line: &TextLine, frame: &Frame) -> bool {
        let Some(previous) = &self.previous_line else {
            return true;
        };
        let indented = line.left > frame.left + self.metrics.em(INDENT_EM);
        let spaced = previous.page == page
            && line.top - previous.bottom
                > self.metrics.line_gap + self.metrics.em(PARAGRAPH_GAP_EM);
        self.paragraph.is_none() || indented || spaced || previous.closes_paragraph
    }

    fn flush_paragraph(&mut self) {
        if let Some(text) = self.paragraph.take() {
            self.pieces.push(Piece::Paragraph(text));
        }
    }

    fn finish(mut self) -> Vec<Piece> {
        self.flush_paragraph();
        self.pieces
    }
}

/// Text block frames for even and odd pages, which mirror their margins in
/// print-ready PDFs. Measured over the whole document, so a page of
/// dialogue where every line is indented still knows where the margin is.
struct Frames([Frame; 2]);

impl Frames {
    fn measure(pages: &[Vec<TextLine>], metrics: Metrics) -> Self {
        let parity = |p: usize| {
            pages
                .iter()
                .skip(p)
                .step_by(2)
                .flatten()
                .filter(|line| !metrics.is_heading_size(line.font_size))
                .collect::<Vec<_>>()
        };
        let (even, odd) = (parity(0), parity(1));
        let even_frame = Frame::measure(&even).or_else(|| Frame::measure(&odd));
        let odd_frame = Frame::measure(&odd).or_else(|| Frame::measure(&even));
        let fallback = || Frame {
            left: 0.0,
            right: f32::MAX,
        };
        Self([
            even_frame.unwrap_or_else(fallback),
            odd_frame.unwrap_or_else(fallback),
        ])
    }

    fn for_page(&self, index: usize) -> &Frame {
        &self.0[index % 2]
    }
}

/// Percentile of line ends taken as the right margin, robust to the odd
/// line that overflows it (a long URL, for instance).
const RIGHT_EDGE_PERCENTILE: usize = 90;
const PERCENT: usize = 100;

impl Frame {
    fn measure(lines: &[&TextLine]) -> Option<Self> {
        let left = most_common_left(lines)?;
        let mut rights: Vec<f32> = lines.iter().map(|line| line.right).collect();
        rights.sort_by(f32::total_cmp);
        let index = ((rights.len() - 1) * RIGHT_EDGE_PERCENTILE).div_ceil(PERCENT);
        Some(Self {
            left,
            right: rights[index],
        })
    }
}

/// The most frequent left edge; on a tie, the leftmost one, since first
/// lines are indented to the right of the margin.
fn most_common_left(lines: &[&TextLine]) -> Option<f32> {
    let mut lefts: Vec<f32> = lines.iter().map(|line| line.left.round()).collect();
    lefts.sort_by(f32::total_cmp);
    let mut best: Option<(f32, usize)> = None;
    for group in lefts.chunk_by(|a, b| (a - b).abs() < f32::EPSILON) {
        if best.is_none_or(|(_, count)| group.len() > count) {
            best = Some((group[0], group.len()));
        }
    }
    best.map(|(left, _)| left)
}

/// Folds a drop cap into the first line of text beside it. The lines
/// wrapped around the cap (they share the first line's left edge) move back
/// to the cap's edge, so the room the cap takes is not mistaken for a
/// paragraph indent; a real indent inside the cap's box is kept.
fn merge_drop_caps(lines: &[TextLine], metrics: Metrics) -> Vec<TextLine> {
    let mut merged: Vec<TextLine> = lines.to_vec();
    let caps: Vec<usize> = (0..merged.len())
        .filter(|&i| is_drop_cap(&merged[i], metrics))
        .collect();
    for &cap_index in caps.iter().rev() {
        let cap = merged[cap_index].clone();
        let beside: Vec<usize> = (0..merged.len())
            .filter(|&i| {
                i != cap_index
                    && !metrics.is_heading_size(merged[i].font_size)
                    && merged[i].left >= cap.left
                    && merged[i].overlaps_vertically(&cap)
            })
            .collect();
        let Some(&first) = beside.first() else {
            continue;
        };
        let spaced = merged[first].left - cap.right > metrics.em(DROP_CAP_WORD_SPACE_EM);
        let prefix = format!("{}{}", cap.text.trim(), if spaced { " " } else { "" });
        merged[first].text.insert_str(0, &prefix);
        let wrap_edge = merged[first].left;
        for &i in &beside {
            if (merged[i].left - wrap_edge).abs() <= metrics.em(WRAP_EDGE_TOLERANCE_EM) {
                merged[i].left = cap.left;
            }
        }
        merged.remove(cap_index);
    }
    merged
}

/// A single large letter, possibly after an opening quote or dash
/// (`‘T`, `«E`, `—A`).
fn is_drop_cap(line: &TextLine, metrics: Metrics) -> bool {
    let mut chars = line
        .text
        .trim()
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .chars();
    metrics.is_heading_size(line.font_size)
        && chars.next().is_some_and(char::is_alphabetic)
        && chars.next().is_none()
}

fn ends_sentence(text: &str) -> bool {
    text.trim_end_matches(CLOSING_MARKS).ends_with(SENTENCE_END)
}

/// Joins a heading line to the title above it: a wrapped title continues
/// with a space, a stacked label (`Capítulo 1` / `La partida`) with a period.
fn join_title(title: &mut String, next: &str) {
    let next = next.trim();
    let wrapped = next.chars().next().is_some_and(char::is_lowercase);
    if !wrapped && !title.ends_with(TITLE_JOINERS) {
        title.push('.');
    }
    title.push(' ');
    title.push_str(next);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_ends_inside_closing_quotes() {
        assert!(ends_sentence("—¿Vienes?»"));
        assert!(ends_sentence("Fin.”"));
        assert!(!ends_sentence("y entonces"));
    }

    #[test]
    fn stacked_titles_are_joined_with_a_period() {
        let mut title = "Capítulo 1".to_owned();
        join_title(&mut title, "La partida");
        assert_eq!(title, "Capítulo 1. La partida");
    }

    #[test]
    fn titles_ending_in_punctuation_are_not_doubled() {
        let mut title = "Capítulo 1:".to_owned();
        join_title(&mut title, "La partida");
        assert_eq!(title, "Capítulo 1: La partida");
    }
}
