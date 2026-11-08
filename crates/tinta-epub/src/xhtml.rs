//! XHTML for part title pages and chapters.
//!
//! The markup is deliberately plain so the Kindle can apply the reader's
//! font, size and margins. The opening paragraph of a chapter carries a
//! drop cap and a small-caps lead-in, as in a professionally set book.

use crate::xml::escape;
use std::fmt::Write as _;
use tinta_core::{Block, Chapter};

/// Words of the opening paragraph set in small caps.
const LEAD_IN_WORDS: usize = 3;
/// Opening quotes that travel with the drop cap (`‘T`, `«E`).
const OPENING_QUOTES: [char; 6] = ['‘', '“', '«', '"', '\'', '‹'];
/// Dashes that open dialogue: such paragraphs get no drop cap.
const DIALOGUE_DASHES: [char; 3] = ['—', '–', '-'];
/// A lead-in stops after the word that closes a clause.
const CLAUSE_END: [char; 8] = [',', '.', ';', ':', '!', '?', '…', ')'];
const DEEPEST_HEADING: u8 = 6;

pub(crate) fn chapter(chapter: &Chapter, language: &str) -> String {
    let mut body = format!(
        r#"<section epub:type="chapter" role="doc-chapter"><h1>{}</h1>"#,
        escape(&chapter.title)
    );
    for block in &chapter.blocks {
        body.push_str(&self::block(block));
    }
    body.push_str("</section>");
    document(&chapter.title, language, &body)
}

pub(crate) fn part(title: &str, language: &str) -> String {
    let body = format!(
        r#"<section epub:type="part" role="doc-part"><h1 class="part">{}</h1></section>"#,
        escape(title)
    );
    document(title, language, &body)
}

fn document(title: &str, language: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{language}" lang="{language}">
<head>
<meta charset="UTF-8"/>
<title>{title}</title>
<link rel="stylesheet" type="text/css" href="../style.css"/>
</head>
<body epub:type="bodymatter">
{body}
</body>
</html>
"#,
        title = escape(title),
    )
}

fn block(block: &Block) -> String {
    match block {
        Block::Paragraph {
            text,
            opening: true,
        } => format!(r#"<p class="opening">{}</p>"#, opening(text)),
        Block::Paragraph { text, .. } => format!("<p>{}</p>", escape(text)),
        Block::Heading { level, text } => {
            let level = (*level).clamp(2, DEEPEST_HEADING);
            format!("<h{level}>{}</h{level}>", escape(text))
        }
        Block::Quote { text } => format!("<blockquote><p>{}</p></blockquote>", escape(text)),
        Block::Image { source, alt } => format!(
            r#"<figure><img src="../images/{}" alt="{}"/></figure>"#,
            escape(source),
            escape(alt)
        ),
    }
}

/// `<span class="drop-cap">E</span><span class="lead-in">ra una mañana</span> de sol.`
fn opening(text: &str) -> String {
    let (lead_in, rest) = split_lead_in(text);
    let (drop_cap, lead_in) = split_drop_cap(lead_in);
    let mut html = String::new();
    if !drop_cap.is_empty() {
        let _ = write!(
            html,
            r#"<span class="drop-cap">{}</span>"#,
            escape(drop_cap)
        );
    }
    if !lead_in.is_empty() {
        let _ = write!(html, r#"<span class="lead-in">{}</span>"#, escape(lead_in));
    }
    html.push_str(&escape(rest));
    html
}

/// The first few words, ending early at the end of a clause.
fn split_lead_in(text: &str) -> (&str, &str) {
    let mut words = 0;
    let mut in_word = false;
    for (index, c) in text.char_indices() {
        if c.is_whitespace() {
            if in_word {
                words += 1;
                let previous = text[..index].chars().next_back();
                if words == LEAD_IN_WORDS || previous.is_some_and(|p| CLAUSE_END.contains(&p)) {
                    return text.split_at(index);
                }
            }
            in_word = false;
        } else {
            in_word = true;
        }
    }
    (text, "")
}

/// The first letter, with any opening quote before it. Dialogue and text
/// that does not start with a letter get no drop cap.
fn split_drop_cap(lead_in: &str) -> (&str, &str) {
    if lead_in.starts_with(DIALOGUE_DASHES) {
        return ("", lead_in);
    }
    let after_quotes = lead_in.trim_start_matches(OPENING_QUOTES);
    let mut chars = after_quotes.chars();
    match chars.next() {
        Some(letter) if letter.is_alphabetic() => {
            let end = lead_in.len() - after_quotes.len() + letter.len_utf8();
            lead_in.split_at(end)
        }
        _ => ("", lead_in),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lead_in_takes_three_words() {
        assert_eq!(
            split_lead_in("Era una mañana de sol."),
            ("Era una mañana", " de sol.")
        );
    }

    #[test]
    fn lead_in_stops_at_the_end_of_a_clause() {
        assert_eq!(
            split_lead_in("Sí, claro que sí."),
            ("Sí,", " claro que sí.")
        );
    }

    #[test]
    fn a_short_paragraph_is_all_lead_in() {
        assert_eq!(split_lead_in("Fin."), ("Fin.", ""));
    }

    #[test]
    fn drop_cap_carries_the_opening_quote() {
        assert_eq!(split_drop_cap("‘There is"), ("‘T", "here is"));
        assert_eq!(split_drop_cap("«Ñandú"), ("«Ñ", "andú"));
    }

    #[test]
    fn no_drop_cap_for_dialogue_or_numbers() {
        assert_eq!(split_drop_cap("—¿Vienes?"), ("", "—¿Vienes?"));
        assert_eq!(split_drop_cap("1984 fue"), ("", "1984 fue"));
    }

    #[test]
    fn a_one_letter_word_is_its_own_drop_cap() {
        assert_eq!(
            opening("I suppose you have heard"),
            concat!(
                r#"<span class="drop-cap">I</span><span class="lead-in"> suppose you</span>"#,
                " have heard"
            )
        );
    }
}
