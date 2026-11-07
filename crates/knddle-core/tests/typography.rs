use knddle_core::typography::{
    clean, collapse_spaces, fix_ligatures, join_broken_lines, normalize_punctuation,
    remove_invisible, smart_quotes,
};

#[test]
fn expands_broken_ligatures() {
    assert_eq!(
        fix_ligatures("ﬁn de la ﬂor, eﬀecto, oﬃcina"),
        "fin de la flor, effecto, officina"
    );
}

#[test]
fn removes_soft_hyphens_and_zero_width_characters() {
    assert_eq!(
        remove_invisible("con\u{00AD}ver\u{200B}sión\u{FEFF}"),
        "conversión"
    );
}

#[test]
fn keeps_zero_width_joiner_used_by_emoji() {
    let family = "👨\u{200D}👩\u{200D}👧";
    assert_eq!(remove_invisible(family), family);
}

#[test]
fn joins_words_split_by_hyphen_at_line_end() {
    assert_eq!(
        join_broken_lines("la conver-\nsión del libro"),
        "la conversión del libro"
    );
}

#[test]
fn keeps_hyphen_when_next_line_starts_with_uppercase() {
    assert_eq!(join_broken_lines("Hispano-\nAmérica"), "Hispano-América");
}

#[test]
fn joins_lines_broken_mid_sentence_with_a_space() {
    assert_eq!(
        join_broken_lines("era una noche\r\noscura y fría"),
        "era una noche oscura y fría"
    );
}

#[test]
fn collapses_repeated_whitespace_and_trims() {
    assert_eq!(collapse_spaces("  hola   \t mundo  "), "hola mundo");
}

#[test]
fn preserves_non_breaking_spaces() {
    assert_eq!(collapse_spaces("10\u{00A0}km"), "10\u{00A0}km");
}

#[test]
fn converts_ellipsis_and_double_hyphen() {
    assert_eq!(
        normalize_punctuation("Pues... no -- dijo"),
        "Pues… no — dijo"
    );
}

#[test]
fn converts_leading_hyphen_into_dialogue_dash() {
    assert_eq!(normalize_punctuation("- ¿Vienes?"), "—¿Vienes?");
    assert_eq!(normalize_punctuation("-Hola"), "—Hola");
}

#[test]
fn leaves_negative_numbers_untouched() {
    assert_eq!(normalize_punctuation("-5 grados"), "-5 grados");
}

#[test]
fn curls_double_quotes() {
    assert_eq!(
        smart_quotes(r#"dijo "hola" y "adiós""#),
        "dijo “hola” y “adiós”"
    );
}

#[test]
fn curls_apostrophes_inside_words() {
    assert_eq!(smart_quotes("it's 'fine'"), "it’s ‘fine’");
}

#[test]
fn clean_applies_every_rule() {
    let raw = "  \"Era el ﬁn...\" -- pensó, mien-\ntras  la\u{00AD} noche\ncaía ";
    assert_eq!(clean(raw), "“Era el fin…” — pensó, mientras la noche caía");
}

#[test]
fn clean_is_idempotent() {
    let once = clean("\"Hola\" -- dijo... con-\nversando");
    assert_eq!(clean(&once), once);
}
