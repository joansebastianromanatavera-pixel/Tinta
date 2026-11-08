//! Identifier and modification date derived for a book.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tinta_core::{Book, Metadata};
use tinta_epub::EpubOptions;

const NOV_7_2026_NOON: u64 = 1_794_052_800;
const LEAP_DAY_2024_LAST_SECOND: u64 = 1_709_251_199;
const URN_PREFIX: &str = "urn:uuid:";
const UUID_LENGTH: usize = 36;
const UUID_VERSION_POSITION: usize = 14;

fn book(title: &str, authors: &[&str]) -> Book {
    Book {
        metadata: Metadata {
            title: title.into(),
            authors: authors.iter().map(|a| (*a).to_owned()).collect(),
            ..Metadata::default()
        },
        parts: Vec::new(),
    }
}

fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[test]
fn the_same_book_always_gets_the_same_identifier() {
    let first = EpubOptions::for_book(&book("Niebla", &["Unamuno"]), at(0));
    let again = EpubOptions::for_book(&book("Niebla", &["Unamuno"]), at(NOV_7_2026_NOON));

    assert_eq!(first.identifier, again.identifier);
}

#[test]
fn different_books_get_different_identifiers() {
    let niebla = EpubOptions::for_book(&book("Niebla", &["Unamuno"]), at(0));
    let other_author = EpubOptions::for_book(&book("Niebla", &["Otro"]), at(0));
    let other_title = EpubOptions::for_book(&book("Paz", &["Unamuno"]), at(0));

    assert_ne!(niebla.identifier, other_author.identifier);
    assert_ne!(niebla.identifier, other_title.identifier);
}

#[test]
fn identifier_is_a_name_based_uuid_urn() {
    let options = EpubOptions::for_book(&book("Niebla", &["Unamuno"]), at(0));

    let uuid = options.identifier.strip_prefix(URN_PREFIX).expect("urn");
    assert_eq!(uuid.len(), UUID_LENGTH);
    assert_eq!(&uuid[UUID_VERSION_POSITION..=UUID_VERSION_POSITION], "5");
}

#[test]
fn modified_is_utc_with_seconds() {
    let book = book("Niebla", &[]);

    assert_eq!(
        EpubOptions::for_book(&book, at(0)).modified,
        "1970-01-01T00:00:00Z"
    );
    assert_eq!(
        EpubOptions::for_book(&book, at(NOV_7_2026_NOON)).modified,
        "2026-11-07T12:00:00Z"
    );
    assert_eq!(
        EpubOptions::for_book(&book, at(LEAP_DAY_2024_LAST_SECOND)).modified,
        "2024-02-29T23:59:59Z"
    );
}
