//! Rebuilding a book from the positioned lines of its pages.

mod support;

use support::{document, page};
use tinta_core::{Block, Book};
use tinta_pdf::rebuild;

const CHAPTER_SIZE: f32 = 18.0;
const PART_SIZE: f32 = 24.0;
const TITLE_SIZE: f32 = 30.0;
const SECTION_SIZE: f32 = 14.0;
const FALLBACK_TITLE: &str = "archivo";
/// A full first line of body text, so pages carry more body than headings,
/// as real books do.
const BODY: &str = "Había una vez, en un lugar no muy lejano, una casa junto al";

fn paragraphs(book: &Book) -> Vec<String> {
    book.chapters()
        .flat_map(|chapter| chapter.blocks.iter())
        .filter_map(|block| match block {
            Block::Paragraph { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// The closing line of each paragraph, which tells pages apart.
fn last_lines(book: &Book) -> Vec<String> {
    paragraphs(book)
        .iter()
        .map(|text| text.lines().last().unwrap_or_default().to_owned())
        .collect()
}

fn chapter_titles(book: &Book) -> Vec<String> {
    book.chapters()
        .map(|chapter| chapter.title.clone())
        .collect()
}

// --- Paragraphs ----------------------------------------------------------

#[test]
fn wrapped_lines_form_one_paragraph() {
    let doc = document(vec![page()
        .indented("Era una mañana")
        .body("de sol en la")
        .last("ciudad.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(paragraphs(&book), ["Era una mañana\nde sol en la\nciudad."]);
}

#[test]
fn first_line_indent_starts_a_new_paragraph() {
    let doc = document(vec![page()
        .indented("Primero")
        .last("termina.")
        .indented("Segundo")
        .last("también.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        paragraphs(&book),
        ["Primero\ntermina.", "Segundo\ntambién."]
    );
}

#[test]
fn vertical_gap_starts_a_new_paragraph_in_block_style() {
    let doc = document(vec![page()
        .body("Primero sin")
        .last("sangría.")
        .gap()
        .body("Segundo sin")
        .last("sangría.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        paragraphs(&book),
        ["Primero sin\nsangría.", "Segundo sin\nsangría."]
    );
}

#[test]
fn paragraph_continues_across_a_page_break() {
    let doc = document(vec![
        page().indented("Una frase que").body("sigue en la").build(),
        page().body("página siguiente").last("y acaba.").build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        paragraphs(&book),
        ["Una frase que\nsigue en la\npágina siguiente\ny acaba."]
    );
}

#[test]
fn short_final_line_ends_the_paragraph_at_a_page_break() {
    let doc = document(vec![
        page().indented("Acaba justo").last("aquí.").build(),
        page().body("Nuevo párrafo sin").last("sangría.").build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(paragraphs(&book).len(), 2);
}

// --- Running headers, footers and page numbers --------------------------

#[test]
fn running_headers_repeated_across_pages_are_removed() {
    let pages = ["A.", "B.", "C."]
        .iter()
        .map(|text| {
            page()
                .header("Editorial Tinta")
                .paragraph(BODY, text)
                .build()
        })
        .collect();

    let book = rebuild(&document(pages), FALLBACK_TITLE);

    assert_eq!(last_lines(&book), ["A.", "B.", "C."]);
}

#[test]
fn page_numbers_are_removed_even_in_a_short_document() {
    let doc = document(vec![
        page().paragraph(BODY, "fin.").footer("7").build(),
        page().paragraph(BODY, "fin.").footer("- 8 -").build(),
        page().header("xii").paragraph(BODY, "fin.").build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(last_lines(&book), ["fin.", "fin.", "fin."]);
}

#[test]
fn headers_with_changing_page_numbers_count_as_repeated() {
    let doc = document(vec![
        page().header("12  LA CASA").paragraph(BODY, "fin.").build(),
        page().header("LA CASA  13").paragraph(BODY, "fin.").build(),
        page().header("14  LA CASA").paragraph(BODY, "fin.").build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(last_lines(&book), ["fin.", "fin.", "fin."]);
}

#[test]
fn running_header_with_the_chapter_title_is_removed_even_if_rare() {
    let doc = document(vec![
        page()
            .heading("El viaje", CHAPTER_SIZE)
            .paragraph(BODY, "temprano.")
            .build(),
        page().header("EL VIAJE").paragraph(BODY, "tarde.").build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(last_lines(&book), ["temprano.", "tarde."]);
}

#[test]
fn a_one_off_line_at_the_top_of_a_page_is_kept() {
    let doc = document(vec![
        page()
            .header("Nota del editor")
            .paragraph(BODY, "fin.")
            .build(),
        page().paragraph(BODY, "fin.").build(),
        page().paragraph(BODY, "fin.").build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(paragraphs(&book)[0], "Nota del editor");
}

#[test]
fn repeated_chapter_headings_at_the_top_of_pages_are_kept() {
    let pages = (1..=3)
        .map(|n| {
            page()
                .heading(&format!("Capítulo {n}"), CHAPTER_SIZE)
                .paragraph(BODY, "fin.")
                .build()
        })
        .collect();

    let book = rebuild(&document(pages), FALLBACK_TITLE);

    assert_eq!(
        chapter_titles(&book),
        ["Capítulo 1", "Capítulo 2", "Capítulo 3"]
    );
}

// --- Structure -----------------------------------------------------------

#[test]
fn larger_lines_start_chapters() {
    let doc = document(vec![
        page()
            .heading("Uno", CHAPTER_SIZE)
            .paragraph(BODY, "A.")
            .build(),
        page()
            .heading("Dos", CHAPTER_SIZE)
            .paragraph(BODY, "B.")
            .build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["Uno", "Dos"]);
    assert_eq!(book.parts.len(), 1);
    assert_eq!(book.parts[0].title, None);
}

#[test]
fn stacked_heading_lines_merge_into_one_title() {
    let doc = document(vec![page()
        .heading("Capítulo 1", SECTION_SIZE)
        .heading("La partida", CHAPTER_SIZE)
        .paragraph(BODY, "A.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["Capítulo 1. La partida"]);
}

#[test]
fn a_heading_wrapped_over_two_lines_joins_with_a_space() {
    let doc = document(vec![page()
        .heading("La larga noche de los", CHAPTER_SIZE)
        .heading("tiempos perdidos", CHAPTER_SIZE)
        .paragraph(BODY, "A.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        chapter_titles(&book),
        ["La larga noche de los tiempos perdidos"]
    );
}

#[test]
fn headings_that_open_straight_into_chapters_become_parts() {
    let doc = document(vec![
        page().heading("Primera parte", PART_SIZE).build(),
        page()
            .heading("Uno", CHAPTER_SIZE)
            .paragraph(BODY, "A.")
            .build(),
        page()
            .heading("Dos", CHAPTER_SIZE)
            .paragraph(BODY, "B.")
            .build(),
        page().heading("Segunda parte", PART_SIZE).build(),
        page()
            .heading("Tres", CHAPTER_SIZE)
            .paragraph(BODY, "C.")
            .build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    let parts: Vec<_> = book
        .parts
        .iter()
        .map(|part| (part.title.clone(), part.chapters.len()))
        .collect();
    assert_eq!(
        parts,
        [
            (Some("Primera parte".to_owned()), 2),
            (Some("Segunda parte".to_owned()), 1)
        ]
    );
}

#[test]
fn smaller_headings_inside_chapters_become_section_headings() {
    let doc = document(vec![
        page()
            .heading("Uno", CHAPTER_SIZE)
            .paragraph(BODY, "Intro.")
            .heading("Sección", SECTION_SIZE)
            .paragraph(BODY, "A.")
            .build(),
        page()
            .heading("Dos", CHAPTER_SIZE)
            .paragraph(BODY, "B.")
            .build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["Uno", "Dos"]);
    let first = &book.parts[0].chapters[0].blocks;
    assert!(first.contains(&Block::Heading {
        level: 2,
        text: "Sección".into()
    }));
}

#[test]
fn title_page_sets_the_title_and_is_not_a_chapter() {
    let doc = document(vec![
        page().heading("Mi novela", TITLE_SIZE).build(),
        page()
            .heading("Uno", CHAPTER_SIZE)
            .paragraph(BODY, "A.")
            .build(),
        page()
            .heading("Dos", CHAPTER_SIZE)
            .paragraph(BODY, "B.")
            .build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(book.metadata.title, "Mi novela");
    assert_eq!(chapter_titles(&book), ["Uno", "Dos"]);
}

#[test]
fn text_before_the_first_heading_opens_a_chapter_named_after_the_book() {
    let mut doc = document(vec![
        page().paragraph(BODY, "Para mi madre.").build(),
        page()
            .heading("Uno", CHAPTER_SIZE)
            .paragraph(BODY, "A.")
            .build(),
    ]);
    doc.title = Some("Cuentos".into());

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["Cuentos", "Uno"]);
}

#[test]
fn a_document_without_headings_is_a_single_chapter() {
    let doc = document(vec![page().paragraph(BODY, "Solo texto.").build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), [FALLBACK_TITLE]);
    assert_eq!(book.metadata.title, FALLBACK_TITLE);
}

#[test]
fn embedded_metadata_wins_over_the_title_page() {
    let mut doc = document(vec![
        page().heading("MI NOVELA", TITLE_SIZE).build(),
        page()
            .heading("Uno", CHAPTER_SIZE)
            .paragraph(BODY, "A.")
            .build(),
    ]);
    doc.title = Some("Mi novela".into());
    doc.author = Some("Ana Pérez".into());

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(book.metadata.title, "Mi novela");
    assert_eq!(book.metadata.authors, ["Ana Pérez"]);
}

#[test]
fn a_large_single_letter_is_a_drop_cap_not_a_heading() {
    let doc = document(vec![page()
        .heading("Uno", CHAPTER_SIZE)
        .drop_cap("E")
        .body("ra una vez")
        .last("un cuento.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["Uno"]);
    assert_eq!(paragraphs(&book), ["Era una vez\nun cuento."]);
}

#[test]
fn a_roman_numeral_above_the_text_is_a_chapter_not_a_drop_cap() {
    let doc = document(vec![
        page()
            .heading("I", CHAPTER_SIZE)
            .paragraph(BODY, "A.")
            .build(),
        page()
            .heading("II", CHAPTER_SIZE)
            .paragraph(BODY, "B.")
            .build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["I", "II"]);
}

#[test]
fn a_lone_short_line_in_the_middle_of_text_is_not_a_page_number() {
    let doc = document(vec![page()
        .indented(BODY)
        .body("hasta contar")
        .last("3")
        .indented(BODY)
        .last("y salió.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        paragraphs(&book),
        [
            format!("{BODY}\nhasta contar\n3"),
            format!("{BODY}\ny salió.")
        ]
    );
}

#[test]
fn a_drop_cap_after_an_opening_quote_is_still_a_drop_cap() {
    let doc = document(vec![page()
        .heading("Uno", CHAPTER_SIZE)
        .drop_cap("‘T")
        .body("here is no good")
        .last("telling me.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["Uno"]);
    assert_eq!(paragraphs(&book), ["‘There is no good\ntelling me."]);
}

#[test]
fn a_single_heading_larger_than_the_chapters_does_not_take_their_level() {
    let doc = document(vec![
        page()
            .heading("Uno", CHAPTER_SIZE)
            .paragraph(BODY, "A.")
            .build(),
        page()
            .heading("Dos", CHAPTER_SIZE)
            .paragraph(BODY, "B.")
            .build(),
        page()
            .heading("Interludio", PART_SIZE)
            .paragraph(BODY, "C.")
            .build(),
        page()
            .heading("Tres", CHAPTER_SIZE)
            .paragraph(BODY, "D.")
            .build(),
    ]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(chapter_titles(&book), ["Uno", "Dos", "Interludio", "Tres"]);
}

#[test]
fn every_line_wrapped_around_a_drop_cap_belongs_to_the_opening_paragraph() {
    let doc = document(vec![page()
        .heading("Uno", CHAPTER_SIZE)
        .drop_cap("T")
        .beside_cap("he studio was filled")
        .beside_cap("when the light")
        .body("of the garden")
        .last("lilac.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        paragraphs(&book),
        ["The studio was filled\nwhen the light\nof the garden\nlilac."]
    );
}

#[test]
fn a_drop_cap_that_is_a_whole_word_keeps_its_space() {
    let doc = document(vec![page()
        .heading("Uno", CHAPTER_SIZE)
        .drop_cap("I")
        .beside_cap_after_space("suppose you have")
        .beside_cap_after_space("heard the news")
        .last("Basil.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        paragraphs(&book),
        ["I suppose you have\nheard the news\nBasil."]
    );
}

#[test]
fn an_indented_paragraph_still_inside_the_drop_cap_box_keeps_its_indent() {
    let doc = document(vec![page()
        .heading("Uno", CHAPTER_SIZE)
        .drop_cap("I")
        .beside_cap("t was on the seventh")
        .beside_cap("of November.")
        .indented("He was walking home")
        .last("alone.")
        .build()]);

    let book = rebuild(&doc, FALLBACK_TITLE);

    assert_eq!(
        paragraphs(&book),
        [
            "It was on the seventh\nof November.",
            "He was walking home\nalone."
        ]
    );
}
