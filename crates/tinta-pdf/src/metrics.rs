//! Typographic measurements of the whole document: the body text size and
//! the usual gap between lines. Every other heuristic is relative to them,
//! so the rules work the same for a pocket novel and a large-format essay.

use crate::layout::Page;
use std::collections::BTreeMap;

/// Font sizes closer than this are considered the same size.
const SIZE_STEP: f32 = 0.5;
const DEFAULT_BODY_SIZE: f32 = 11.0;
/// Lines this much larger than the body text are headings.
const HEADING_SIZE_RATIO: f32 = 1.15;
/// Lines up to this much larger than the body still count as body text,
/// e.g. running headers set in small caps.
const BODY_SIZE_TOLERANCE: f32 = 1.05;
/// Fallback gap between lines when the document has too few lines to
/// measure it, as a fraction of the body size.
const DEFAULT_LINE_GAP_EM: f32 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Metrics {
    pub body_size: f32,
    /// Typical whitespace between two consecutive lines of a paragraph.
    pub line_gap: f32,
}

impl Metrics {
    pub fn measure(pages: &[Page]) -> Self {
        let body_size = body_size(pages);
        Self {
            body_size,
            line_gap: line_gap(pages, body_size),
        }
    }

    pub fn is_heading_size(self, size: f32) -> bool {
        size >= self.body_size * HEADING_SIZE_RATIO
    }

    pub fn is_body_size(self, size: f32) -> bool {
        size <= self.body_size * BODY_SIZE_TOLERANCE
    }

    /// Lengths expressed in ems of the body text.
    pub fn em(self, ems: f32) -> f32 {
        self.body_size * ems
    }
}

/// Font sizes bucketed so that sizes from the same style compare equal.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn size_key(size: f32) -> u32 {
    (size.max(0.0) / SIZE_STEP).round() as u32
}

#[allow(clippy::cast_precision_loss)]
fn size_from_key(key: u32) -> f32 {
    key as f32 * SIZE_STEP
}

/// The size used by most characters of the document.
fn body_size(pages: &[Page]) -> f32 {
    let mut chars_per_size: BTreeMap<u32, usize> = BTreeMap::new();
    for line in pages.iter().flat_map(|page| &page.lines) {
        *chars_per_size.entry(size_key(line.font_size)).or_default() += line.char_count();
    }
    chars_per_size
        .into_iter()
        .max_by_key(|&(_, chars)| chars)
        .map_or(DEFAULT_BODY_SIZE, |(key, _)| size_from_key(key))
}

/// Median gap between consecutive body lines on the same page. It can be
/// negative: engines report glyph boxes that overlap the next line.
fn line_gap(pages: &[Page], body_size: f32) -> f32 {
    let body_key = size_key(body_size);
    let mut gaps: Vec<f32> = pages
        .iter()
        .flat_map(|page| page.lines.windows(2))
        .filter(|pair| pair.iter().all(|line| size_key(line.font_size) == body_key))
        .map(|pair| pair[0].gap_to(&pair[1]))
        .collect();
    if gaps.is_empty() {
        return body_size * DEFAULT_LINE_GAP_EM;
    }
    gaps.sort_by(f32::total_cmp);
    gaps[gaps.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::TextLine;

    fn line(text: &str, top: f32, size: f32) -> TextLine {
        TextLine {
            text: text.into(),
            top,
            bottom: top + size,
            font_size: size,
            ..TextLine::default()
        }
    }

    #[test]
    fn body_size_counts_characters_not_lines() {
        let page = Page {
            lines: vec![
                line("Título", 0.0, 20.0),
                line("Pie", 30.0, 8.0),
                line("Pie", 40.0, 8.0),
                line("Un cuerpo de texto mucho más largo", 50.0, 11.0),
            ],
            ..Page::default()
        };

        let metrics = Metrics::measure(&[page]);

        assert!((metrics.body_size - 11.0).abs() < f32::EPSILON);
    }

    #[test]
    fn line_gap_is_the_median_between_body_lines() {
        let page = Page {
            lines: vec![
                line("a", 0.0, 10.0),
                line("b", 12.0, 10.0),
                line("c", 24.0, 10.0),
                line("d", 60.0, 10.0),
            ],
            ..Page::default()
        };

        let metrics = Metrics::measure(&[page]);

        assert!((metrics.line_gap - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn overlapping_line_boxes_give_a_negative_gap() {
        let page = Page {
            lines: vec![
                line("a", 0.0, 10.0),
                line("b", 9.0, 10.0),
                line("c", 18.0, 10.0),
            ],
            ..Page::default()
        };

        let metrics = Metrics::measure(&[page]);

        assert!((metrics.line_gap + 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_document_falls_back_to_defaults() {
        let metrics = Metrics::measure(&[]);

        assert!((metrics.body_size - DEFAULT_BODY_SIZE).abs() < f32::EPSILON);
    }
}
