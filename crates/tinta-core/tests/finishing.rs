//! The finishing phases: language detection and final validation.

use tinta_core::phases::{DetectLanguage, Validate};
use tinta_core::{Block, Book, Chapter, Metadata, Part, Phase};

const SPANISH: &str = "En un lugar de la Mancha, de cuyo nombre no quiero acordarme, no ha \
    mucho tiempo que vivía un hidalgo de los de lanza en astillero, adarga antigua, rocín \
    flaco y galgo corredor.";
const ENGLISH: &str = "It is a truth universally acknowledged, that a single man in \
    possession of a good fortune, must be in want of a wife. However little known the \
    feelings or views of such a man may be on his first entering a neighbourhood.";

fn book_with(texts: &[&str]) -> Book {
    Book {
        metadata: Metadata {
            title: "Libro".into(),
            ..Metadata::default()
        },
        parts: vec![Part {
            title: None,
            chapters: vec![Chapter {
                title: "Uno".into(),
                blocks: texts.iter().map(|text| Block::paragraph(*text)).collect(),
            }],
        }],
    }
}

// --- DetectLanguage ------------------------------------------------------

#[test]
fn detects_spanish_as_a_two_letter_tag() {
    let book = DetectLanguage.apply(book_with(&[SPANISH])).unwrap();

    assert_eq!(book.metadata.language.as_deref(), Some("es"));
}

#[test]
fn detects_english() {
    let book = DetectLanguage.apply(book_with(&[ENGLISH])).unwrap();

    assert_eq!(book.metadata.language.as_deref(), Some("en"));
}

#[test]
fn the_language_of_most_of_the_text_wins() {
    let book = DetectLanguage
        .apply(book_with(&[ENGLISH, SPANISH, SPANISH, SPANISH]))
        .unwrap();

    assert_eq!(book.metadata.language.as_deref(), Some("es"));
}

#[test]
fn a_language_already_known_is_kept() {
    let mut book = book_with(&[ENGLISH]);
    book.metadata.language = Some("pt-BR".into());

    let book = DetectLanguage.apply(book).unwrap();

    assert_eq!(book.metadata.language.as_deref(), Some("pt-BR"));
}

#[test]
fn too_little_text_leaves_the_language_unknown() {
    let book = DetectLanguage.apply(book_with(&["Ok."])).unwrap();

    assert_eq!(book.metadata.language, None);
}

// --- Validate ------------------------------------------------------------

#[test]
fn a_complete_book_is_valid() {
    let book = book_with(&[SPANISH]);

    assert_eq!(Validate.apply(book.clone()), Ok(book));
}

#[test]
fn a_book_without_title_is_invalid() {
    let mut book = book_with(&[SPANISH]);
    book.metadata.title = "  ".into();

    let error = Validate.apply(book).unwrap_err();

    assert_eq!(error.0, "El libro no tiene título");
}

#[test]
fn a_chapter_without_title_is_invalid() {
    let mut book = book_with(&[SPANISH]);
    book.parts[0].chapters[0].title = String::new();

    let error = Validate.apply(book).unwrap_err();

    assert_eq!(error.0, "El capítulo 1 no tiene título");
}

#[test]
fn a_book_without_any_text_is_invalid() {
    let mut book = book_with(&[]);
    book.parts[0].chapters[0].blocks = vec![Block::Image {
        source: "cover.jpg".into(),
        alt: String::new(),
    }];

    let error = Validate.apply(book).unwrap_err();

    assert_eq!(error.0, "El libro no tiene texto");
}
