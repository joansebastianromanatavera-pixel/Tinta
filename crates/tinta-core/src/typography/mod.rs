//! Deep typographic cleanup.
//!
//! PDF extraction leaves tell-tale artefacts: broken ligatures, invisible
//! soft hyphens, words split across lines and straight quotes. Each rule is a
//! small pure function; [`clean`] applies them all in the right order.

const LIGATURES: [(char, &str); 7] = [
    ('\u{FB00}', "ff"),
    ('\u{FB01}', "fi"),
    ('\u{FB02}', "fl"),
    ('\u{FB03}', "ffi"),
    ('\u{FB04}', "ffl"),
    ('\u{FB05}', "st"),
    ('\u{FB06}', "st"),
];

/// Invisible characters that only leak from layout engines.
/// The zero-width joiner (U+200D) is kept because emoji sequences need it.
const INVISIBLE: [char; 5] = ['\u{00AD}', '\u{200B}', '\u{200C}', '\u{2060}', '\u{FEFF}'];

const NON_BREAKING_SPACES: [char; 3] = ['\u{00A0}', '\u{202F}', '\u{2007}'];

const ELLIPSIS: &str = "…";
const EM_DASH: &str = "—";

/// Applies every typographic rule. Idempotent.
pub fn clean(text: &str) -> String {
    let text = remove_invisible(text);
    let text = fix_ligatures(&text);
    let text = join_broken_lines(&text);
    let text = collapse_spaces(&text);
    let text = normalize_punctuation(&text);
    smart_quotes(&text)
}

pub fn fix_ligatures(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match LIGATURES.iter().find(|(ligature, _)| *ligature == c) {
            Some((_, expanded)) => out.push_str(expanded),
            None => out.push(c),
        }
    }
    out
}

pub fn remove_invisible(text: &str) -> String {
    text.chars().filter(|c| !INVISIBLE.contains(c)).collect()
}

/// Rebuilds paragraphs broken by the line wrapping of the source page.
///
/// A word hyphenated at a line end is rejoined when the next line starts in
/// lowercase; otherwise lines are joined with a space.
pub fn join_broken_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let starts_lowercase = line.chars().next().is_some_and(char::is_lowercase);
        if ends_with_word_hyphen(&out) && starts_lowercase {
            out.pop();
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push(' ');
        }
        out.push_str(line);
    }
    out
}

fn ends_with_word_hyphen(text: &str) -> bool {
    let mut tail = text.chars().rev();
    tail.next() == Some('-') && tail.next().is_some_and(char::is_alphabetic)
}

/// Collapses runs of whitespace into one space and trims the ends.
/// Non-breaking spaces are intentional and preserved.
pub fn collapse_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for c in text.chars() {
        if c.is_whitespace() && !NON_BREAKING_SPACES.contains(&c) {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(c);
    }
    out
}

/// Ellipses, em dashes and the Spanish dialogue dash.
pub fn normalize_punctuation(text: &str) -> String {
    let text = text.replace("...", ELLIPSIS).replace("--", EM_DASH);
    dialogue_dash(&text)
}

fn dialogue_dash(text: &str) -> String {
    let Some(rest) = text.strip_prefix('-') else {
        return text.to_owned();
    };
    let speech = rest.trim_start();
    let opens_speech = speech
        .chars()
        .next()
        .is_some_and(|c| c.is_uppercase() || matches!(c, '¿' | '¡' | '“' | '"'));
    if opens_speech {
        format!("{EM_DASH}{speech}")
    } else {
        text.to_owned()
    }
}

/// Curly quotes and apostrophes.
pub fn smart_quotes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous: Option<char> = None;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let opens = previous.is_none_or(opens_quote_after);
        let curled = match c {
            '"' if opens => '“',
            '"' => '”',
            '\'' if previous.is_some_and(char::is_alphanumeric)
                && chars.peek().is_some_and(|n| n.is_alphanumeric()) =>
            {
                '’'
            }
            '\'' if opens => '‘',
            '\'' => '’',
            other => other,
        };
        out.push(curled);
        previous = Some(c);
    }
    out
}

fn opens_quote_after(c: char) -> bool {
    c.is_whitespace() || matches!(c, '(' | '[' | '{' | '—' | '–' | '¿' | '¡')
}
