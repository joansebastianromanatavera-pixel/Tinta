//! Identifier and modification date for a packaged book.

use crate::EpubOptions;
use std::time::{SystemTime, UNIX_EPOCH};
use tinta_core::Book;
use uuid::Uuid;

/// Namespace of Tinta's name-based book identifiers (UUID v5).
const BOOK_NAMESPACE: Uuid = Uuid::from_u128(0x5f0e_7a1c_3b2d_4e8f_9a61_2c7d_0b4e_8f13);
/// Separates title and authors in the identifier name, so "Ab" + "c"
/// and "A" + "bc" never collide.
const NAME_SEPARATOR: char = '\u{1F}';
const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_MINUTE: u64 = 60;

impl EpubOptions {
    /// Options for `book` packaged at `now`. The identifier depends only on
    /// the title and authors, so re-sending the same book keeps its identity.
    pub fn for_book(book: &Book, now: SystemTime) -> Self {
        Self {
            identifier: format!("urn:uuid:{}", book_uuid(book)),
            modified: utc_timestamp(now),
        }
    }
}

fn book_uuid(book: &Book) -> Uuid {
    let mut name = book.metadata.title.trim().to_lowercase();
    for author in &book.metadata.authors {
        name.push(NAME_SEPARATOR);
        name.push_str(&author.trim().to_lowercase());
    }
    Uuid::new_v5(&BOOK_NAMESPACE, name.as_bytes())
}

/// `CCYY-MM-DDThh:mm:ssZ`, as EPUB requires for `dcterms:modified`.
fn utc_timestamp(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (year, month, day) = civil_from_days(seconds / SECONDS_PER_DAY);
    let of_day = seconds % SECONDS_PER_DAY;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        of_day / SECONDS_PER_HOUR,
        of_day % SECONDS_PER_HOUR / SECONDS_PER_MINUTE,
        of_day % SECONDS_PER_MINUTE
    )
}

/// Gregorian date of a day count since 1970-01-01, after Howard Hinnant's
/// `civil_from_days` (<https://howardhinnant.github.io/date_algorithms.html>).
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    const DAYS_TO_ERA_START: u64 = 719_468;
    const DAYS_PER_ERA: u64 = 146_097;
    let z = days + DAYS_TO_ERA_START;
    let era = z / DAYS_PER_ERA;
    let day_of_era = z % DAYS_PER_ERA;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}
