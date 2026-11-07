use std::collections::HashSet;
use tinta_kindle::{find, BookFormat, Connection, KfxSupport, CATALOG, EMAIL_FORMAT};

#[test]
fn catalog_starts_at_fifth_generation() {
    let oldest = CATALOG.iter().map(|m| m.year).min().unwrap();
    assert_eq!(oldest, 2012);
}

#[test]
fn ids_are_unique() {
    let ids: HashSet<_> = CATALOG.iter().map(|m| m.id).collect();
    assert_eq!(ids.len(), CATALOG.len());
}

#[test]
fn screens_are_portrait_with_sane_density() {
    for m in CATALOG {
        assert!(
            m.screen.height > m.screen.width,
            "{} must be portrait",
            m.id
        );
        assert!(
            (150..=330).contains(&m.screen.ppi),
            "{} ppi out of range",
            m.id
        );
    }
}

#[test]
fn kfx_capable_models_get_kfx_only_when_a_generator_exists() {
    let pw5 = find("paperwhite-5").unwrap();
    assert_eq!(pw5.usb_format(KfxSupport::Available), BookFormat::Kfx);
    assert_eq!(pw5.usb_format(KfxSupport::Unavailable), BookFormat::Azw3);
}

#[test]
fn models_without_kfx_always_get_azw3() {
    let kindle5 = find("kindle-5").unwrap();
    assert_eq!(kindle5.usb_format(KfxSupport::Available), BookFormat::Azw3);
}

#[test]
fn models_from_2024_use_mtp() {
    assert!(CATALOG
        .iter()
        .filter(|m| m.year >= 2024)
        .all(|m| m.connection == Connection::Mtp));
}

#[test]
fn images_fit_the_exact_screen() {
    assert_eq!(find("scribe-2").unwrap().image_box(), (1860, 2480));
}

#[test]
fn email_always_sends_epub() {
    assert_eq!(EMAIL_FORMAT.extension(), "epub");
}

#[test]
fn unknown_model_is_none() {
    assert!(find("kindle-dx").is_none());
}
