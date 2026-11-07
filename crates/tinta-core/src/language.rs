//! Language detection over the text of a book.

use crate::model::{Block, Book};
use std::collections::HashMap;
use whatlang::Lang;

/// Paragraphs sampled across the book: enough to be sure, cheap to scan.
const SAMPLE_PARAGRAPHS: usize = 60;
/// Below this confidence (0–1) the guess is not trusted.
const MIN_CONFIDENCE: f64 = 0.5;
/// Fewer characters than this in the winning language are not enough to
/// tell languages apart.
const MIN_SAMPLE_CHARS: usize = 40;

/// BCP 47 tag of the language most of the book is written in.
///
/// Each sampled paragraph votes for its language with its length, so a
/// foreign epigraph or a licence page does not decide for the whole book.
pub fn detect(book: &Book) -> Option<&'static str> {
    let mut votes: HashMap<Lang, usize> = HashMap::new();
    for paragraph in sample(book) {
        let Some(info) = whatlang::detect(paragraph) else {
            continue;
        };
        if info.confidence() >= MIN_CONFIDENCE {
            *votes.entry(info.lang()).or_default() += paragraph.chars().count();
        }
    }
    let (lang, chars) = votes.into_iter().max_by_key(|&(_, chars)| chars)?;
    (chars >= MIN_SAMPLE_CHARS).then(|| bcp47(lang))
}

/// Paragraphs spread evenly through the book.
fn sample(book: &Book) -> Vec<&str> {
    let paragraphs: Vec<&str> = book
        .chapters()
        .flat_map(|chapter| &chapter.blocks)
        .filter_map(|block| match block {
            Block::Paragraph { text, .. } | Block::Quote { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    let stride = paragraphs.len().div_ceil(SAMPLE_PARAGRAPHS).max(1);
    paragraphs.into_iter().step_by(stride).collect()
}

/// Two-letter ISO 639-1 code, as BCP 47 requires when one exists.
#[allow(clippy::too_many_lines)]
fn bcp47(lang: Lang) -> &'static str {
    match lang {
        Lang::Afr => "af",
        Lang::Aka => "ak",
        Lang::Amh => "am",
        Lang::Ara => "ar",
        Lang::Aze => "az",
        Lang::Bel => "be",
        Lang::Ben => "bn",
        Lang::Bul => "bg",
        Lang::Cat => "ca",
        Lang::Ces => "cs",
        Lang::Cmn => "zh",
        Lang::Cym => "cy",
        Lang::Dan => "da",
        Lang::Deu => "de",
        Lang::Ell => "el",
        Lang::Eng => "en",
        Lang::Epo => "eo",
        Lang::Est => "et",
        Lang::Fin => "fi",
        Lang::Fra => "fr",
        Lang::Guj => "gu",
        Lang::Heb => "he",
        Lang::Hin => "hi",
        Lang::Hrv => "hr",
        Lang::Hun => "hu",
        Lang::Hye => "hy",
        Lang::Ind => "id",
        Lang::Ita => "it",
        Lang::Jav => "jv",
        Lang::Jpn => "ja",
        Lang::Kan => "kn",
        Lang::Kat => "ka",
        Lang::Khm => "km",
        Lang::Kor => "ko",
        Lang::Lat => "la",
        Lang::Lav => "lv",
        Lang::Lit => "lt",
        Lang::Mal => "ml",
        Lang::Mar => "mr",
        Lang::Mkd => "mk",
        Lang::Mya => "my",
        Lang::Nep => "ne",
        Lang::Nld => "nl",
        Lang::Nob => "nb",
        Lang::Ori => "or",
        Lang::Pan => "pa",
        Lang::Pes => "fa",
        Lang::Pol => "pl",
        Lang::Por => "pt",
        Lang::Ron => "ro",
        Lang::Rus => "ru",
        Lang::Sin => "si",
        Lang::Slk => "sk",
        Lang::Slv => "sl",
        Lang::Sna => "sn",
        Lang::Spa => "es",
        Lang::Srp => "sr",
        Lang::Swe => "sv",
        Lang::Tam => "ta",
        Lang::Tel => "te",
        Lang::Tgl => "tl",
        Lang::Tha => "th",
        Lang::Tuk => "tk",
        Lang::Tur => "tr",
        Lang::Ukr => "uk",
        Lang::Urd => "ur",
        Lang::Uzb => "uz",
        Lang::Vie => "vi",
        Lang::Yid => "yi",
        Lang::Zul => "zu",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_detectable_language_has_a_two_letter_tag() {
        for lang in Lang::all() {
            let tag = bcp47(*lang);
            assert_eq!(tag.len(), 2, "{lang:?}");
            assert!(tag.chars().all(|c| c.is_ascii_lowercase()), "{lang:?}");
        }
    }
}
