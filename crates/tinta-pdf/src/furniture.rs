//! Removes page furniture: running headers, footers and page numbers.
//!
//! Only lines at the edge of a page are candidates, and only when blank
//! space sets them apart from the text, as typesetters do. A candidate is
//! removed when it is a page number, repeats across many pages (ignoring the
//! page number inside it) or repeats a chapter or book title. Lines larger
//! than the body text are never removed, so chapter headings survive.

use crate::layout::{Page, TextLine};
use crate::metrics::Metrics;
use std::collections::{HashMap, HashSet};

/// How many lines at the top and at the bottom of a page can be furniture.
const MAX_EDGE_LINES: usize = 2;
/// Extra whitespace, in ems, that separates furniture from the text.
const ISOLATION_EM: f32 = 1.0;
/// A header must repeat on at least this many pages…
const MIN_REPEATS: usize = 3;
/// …and on at least this fraction of them, so that a short line of dialogue
/// that happens to open a few pages of a long novel is kept.
const MIN_REPEAT_RATIO: f32 = 0.25;
const MAX_ROMAN_NUMERAL_LEN: usize = 7;

pub(crate) fn strip(pages: &mut [Page], metrics: Metrics, titles: &HashSet<String>) {
    let repeated = repeated_edge_texts(pages, metrics);
    let is_furniture = |line: &TextLine| {
        metrics.is_body_size(line.font_size)
            && (is_page_number(&line.text) || {
                let key = text_key(&line.text);
                !key.is_empty() && (repeated.contains(&key) || titles.contains(&key))
            })
    };
    for page in pages {
        strip_page(page, metrics, &is_furniture);
    }
}

fn strip_page(page: &mut Page, metrics: Metrics, is_furniture: &impl Fn(&TextLine) -> bool) {
    let isolation = metrics.line_gap + metrics.em(ISOLATION_EM);
    for _ in 0..MAX_EDGE_LINES {
        let removable = match page.lines.as_slice() {
            [first, next, ..] => first.gap_to(next) > isolation && is_furniture(first),
            [only] => is_furniture(only),
            [] => false,
        };
        if !removable {
            break;
        }
        page.lines.remove(0);
    }
    for _ in 0..MAX_EDGE_LINES {
        let removable = match page.lines.as_slice() {
            [.., previous, last] => previous.gap_to(last) > isolation && is_furniture(last),
            [only] => is_furniture(only),
            [] => false,
        };
        if !removable {
            break;
        }
        page.lines.pop();
    }
}

/// Texts that appear at the edge of enough pages to be running headers.
fn repeated_edge_texts(pages: &[Page], metrics: Metrics) -> HashSet<String> {
    let mut pages_per_text: HashMap<String, usize> = HashMap::new();
    for page in pages {
        let keys: HashSet<String> = edge_lines(page)
            .filter(|line| metrics.is_body_size(line.font_size))
            .map(|line| text_key(&line.text))
            .filter(|key| !key.is_empty())
            .collect();
        for key in keys {
            *pages_per_text.entry(key).or_default() += 1;
        }
    }
    let threshold = min_repeats(pages.len());
    pages_per_text
        .into_iter()
        .filter(|&(_, count)| count >= threshold)
        .map(|(key, _)| key)
        .collect()
}

fn edge_lines(page: &Page) -> impl Iterator<Item = &TextLine> {
    let lines = &page.lines;
    let tail_start = lines
        .len()
        .saturating_sub(MAX_EDGE_LINES)
        .max(MAX_EDGE_LINES);
    lines
        .iter()
        .take(MAX_EDGE_LINES)
        .chain(lines.iter().skip(tail_start))
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn min_repeats(page_count: usize) -> usize {
    let by_ratio = (page_count as f32 * MIN_REPEAT_RATIO).ceil() as usize;
    by_ratio.max(MIN_REPEATS)
}

/// Comparable form of a text: lowercase letters only, single spaces.
/// Page numbers and punctuation vanish, so `12 · LA CASA` and `La casa 13`
/// share the key `la casa`.
pub(crate) fn text_key(text: &str) -> String {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_page_number(text: &str) -> bool {
    let core: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '-' | '–' | '—' | '.' | '·'))
        .collect();
    let arabic = !core.is_empty() && core.chars().all(|c| c.is_ascii_digit());
    arabic || is_roman_numeral(&core)
}

const ROMAN_NUMERALS: [(u32, &str); 13] = [
    (1000, "m"),
    (900, "cm"),
    (500, "d"),
    (400, "cd"),
    (100, "c"),
    (90, "xc"),
    (50, "l"),
    (40, "xl"),
    (10, "x"),
    (9, "ix"),
    (5, "v"),
    (4, "iv"),
    (1, "i"),
];

/// Accepts only canonical numerals, so words such as `civil` or `mil`,
/// written with the same letters, are not mistaken for page numbers.
fn is_roman_numeral(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.is_empty() || lower.len() > MAX_ROMAN_NUMERAL_LEN {
        return false;
    }
    let mut rest = lower.as_str();
    let mut value = 0;
    for (amount, symbol) in ROMAN_NUMERALS {
        while let Some(tail) = rest.strip_prefix(symbol) {
            value += amount;
            rest = tail;
        }
    }
    rest.is_empty() && to_roman(value) == lower
}

fn to_roman(mut value: u32) -> String {
    let mut out = String::new();
    for (amount, symbol) in ROMAN_NUMERALS {
        while value >= amount {
            out.push_str(symbol);
            value -= amount;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_numbers_in_their_usual_shapes() {
        for text in ["7", "- 8 -", "— 112 —", "xii", "XIV", "· 3 ·"] {
            assert!(is_page_number(text), "{text}");
        }
    }

    #[test]
    fn words_are_not_page_numbers() {
        for text in ["", "Capítulo", "7 de mayo", "civil", "mil", "iiii", "vx"] {
            assert!(!is_page_number(text), "{text}");
        }
    }

    #[test]
    fn text_key_ignores_numbers_case_and_punctuation() {
        assert_eq!(text_key("12 · LA CASA"), "la casa");
        assert_eq!(text_key("La casa, 13"), "la casa");
        assert_eq!(text_key("— 7 —"), "");
    }

    #[test]
    fn repeat_threshold_grows_with_long_books() {
        assert_eq!(min_repeats(4), MIN_REPEATS);
        assert_eq!(min_repeats(300), 75);
    }
}
