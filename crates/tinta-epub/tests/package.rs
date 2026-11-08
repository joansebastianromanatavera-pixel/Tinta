//! The EPUB package as a Kindle and epubcheck see it.

use std::io::{Cursor, Read};
use tinta_core::{Block, Book, Chapter, Metadata, Part, Series};
use tinta_epub::{to_bytes, EpubOptions};
use zip::{CompressionMethod, ZipArchive};

const IDENTIFIER: &str = "urn:uuid:6f1c2d3e-0000-4000-8000-000000000001";
const MODIFIED: &str = "2026-11-07T12:00:00Z";
const FONT_EXTENSIONS: [&str; 4] = ["ttf", "otf", "woff", "woff2"];

fn options() -> EpubOptions {
    EpubOptions {
        identifier: IDENTIFIER.into(),
        modified: MODIFIED.into(),
    }
}

fn chapter(title: &str, blocks: Vec<Block>) -> Chapter {
    Chapter {
        title: title.into(),
        blocks,
    }
}

fn opening(text: &str) -> Block {
    Block::Paragraph {
        text: text.into(),
        opening: true,
    }
}

fn novel() -> Book {
    Book {
        metadata: Metadata {
            title: "Mar & fondo".into(),
            authors: vec!["Ana Pérez".into(), "Luis Gil".into()],
            language: Some("es".into()),
            series: Some(Series {
                name: "Trilogía del mar".into(),
                index: 2,
            }),
        },
        parts: vec![
            Part {
                title: Some("Primera parte".into()),
                chapters: vec![
                    chapter(
                        "La partida",
                        vec![
                            opening("Era una mañana de sol."),
                            Block::paragraph("Salimos."),
                        ],
                    ),
                    chapter(
                        "El viaje",
                        vec![
                            opening("—¿Vienes?"),
                            Block::Heading {
                                level: 2,
                                text: "En alta mar".into(),
                            },
                        ],
                    ),
                ],
            },
            Part {
                title: Some("Segunda parte".into()),
                chapters: vec![chapter(
                    "El regreso",
                    vec![
                        opening("Volvimos <tarde>."),
                        Block::Quote {
                            text: "Nunca más.".into(),
                        },
                    ],
                )],
            },
        ],
    }
}

struct Epub(ZipArchive<Cursor<Vec<u8>>>);

impl Epub {
    fn of(book: &Book) -> Self {
        let bytes = to_bytes(book, &options()).expect("packaged");
        Self(ZipArchive::new(Cursor::new(bytes)).expect("valid zip"))
    }

    fn text(&mut self, name: &str) -> String {
        let mut text = String::new();
        self.0
            .by_name(name)
            .unwrap_or_else(|_| panic!("{name} missing"))
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    fn names(&self) -> Vec<String> {
        self.0.file_names().map(str::to_owned).collect()
    }
}

// --- Container -----------------------------------------------------------

#[test]
fn mimetype_comes_first_and_uncompressed() {
    let mut epub = Epub::of(&novel());

    let first = epub.0.by_index(0).unwrap();
    assert_eq!(first.name(), "mimetype");
    assert_eq!(first.compression(), CompressionMethod::Stored);
    drop(first);
    assert_eq!(epub.text("mimetype"), "application/epub+zip");
}

#[test]
fn container_points_to_the_package_document() {
    let mut epub = Epub::of(&novel());

    assert!(epub
        .text("META-INF/container.xml")
        .contains(r#"full-path="OEBPS/content.opf""#));
}

#[test]
fn no_fonts_are_embedded() {
    let mut epub = Epub::of(&novel());

    let names = epub.names();
    let is_font = |name: &String| {
        std::path::Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| FONT_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
    };
    assert!(!names.iter().any(is_font), "{names:?}");
    assert!(!epub.text("OEBPS/style.css").contains("@font-face"));
    assert!(!epub.text("OEBPS/style.css").contains("font-family"));
}

// --- Metadata ------------------------------------------------------------

#[test]
fn package_carries_title_authors_language_and_modified_date() {
    let opf = Epub::of(&novel()).text("OEBPS/content.opf");

    assert!(opf.contains(r#"<dc:identifier id="book-id">urn:uuid:6f1c2d3e"#));
    assert!(opf.contains("<dc:title>Mar &amp; fondo</dc:title>"));
    assert!(opf.contains(r#"<dc:creator id="creator-1">Ana Pérez</dc:creator>"#));
    assert!(opf.contains(r#"<dc:creator id="creator-2">Luis Gil</dc:creator>"#));
    assert!(opf.contains("<dc:language>es</dc:language>"));
    assert!(opf.contains(r#"<meta property="dcterms:modified">2026-11-07T12:00:00Z</meta>"#));
}

#[test]
fn series_is_declared_for_epub3_and_for_calibre_readers() {
    let opf = Epub::of(&novel()).text("OEBPS/content.opf");

    assert!(opf
        .contains(r#"<meta property="belongs-to-collection" id="series">Trilogía del mar</meta>"#));
    assert!(opf.contains(r##"<meta refines="#series" property="collection-type">series</meta>"##));
    assert!(opf.contains(r##"<meta refines="#series" property="group-position">2</meta>"##));
    assert!(opf.contains(r#"<meta name="calibre:series" content="Trilogía del mar"/>"#));
    assert!(opf.contains(r#"<meta name="calibre:series_index" content="2"/>"#));
}

#[test]
fn unknown_language_is_declared_as_undetermined() {
    let mut book = novel();
    book.metadata.language = None;

    let opf = Epub::of(&book).text("OEBPS/content.opf");

    assert!(opf.contains("<dc:language>und</dc:language>"));
}

// --- Navigation ----------------------------------------------------------

#[test]
fn table_of_contents_has_parts_with_their_chapters() {
    let nav = Epub::of(&novel()).text("OEBPS/nav.xhtml");

    let expected = concat!(
        r#"<li><a href="text/part-01.xhtml">Primera parte</a><ol>"#,
        r#"<li><a href="text/chapter-001.xhtml">La partida</a></li>"#,
        r#"<li><a href="text/chapter-002.xhtml">El viaje</a></li>"#,
        "</ol></li>",
    );
    assert!(nav.contains(expected), "{nav}");
}

#[test]
fn chapters_of_an_untitled_part_sit_at_the_top_level() {
    let mut book = novel();
    book.parts.truncate(1);
    book.parts[0].title = None;

    let mut epub = Epub::of(&book);
    let nav = epub.text("OEBPS/nav.xhtml");

    assert!(nav.contains(r#"<ol><li><a href="text/chapter-001.xhtml">La partida</a></li>"#));
    assert!(!epub.names().iter().any(|name| name.contains("part-")));
}

#[test]
fn go_to_beginning_points_to_the_first_chapter() {
    let mut epub = Epub::of(&novel());

    let nav = epub.text("OEBPS/nav.xhtml");
    assert!(nav.contains(r#"<a epub:type="bodymatter" href="text/chapter-001.xhtml">"#));
    let opf = epub.text("OEBPS/content.opf");
    assert!(
        opf.contains(r#"<reference type="text" title="Inicio" href="text/chapter-001.xhtml"/>"#)
    );
}

#[test]
fn ncx_mirrors_the_two_levels_for_older_readers() {
    let ncx = Epub::of(&novel()).text("OEBPS/toc.ncx");

    assert!(ncx.contains(
        r#"<meta name="dtb:uid" content="urn:uuid:6f1c2d3e-0000-4000-8000-000000000001"/>"#
    ));
    assert!(ncx.contains(r#"<meta name="dtb:depth" content="2"/>"#));
    assert_eq!(ncx.matches("<navPoint ").count(), 5);
}

#[test]
fn the_contents_page_is_in_the_spine_out_of_the_reading_flow() {
    let opf = Epub::of(&novel()).text("OEBPS/content.opf");

    assert!(opf.contains(r#"<spine toc="ncx">
<itemref idref="nav" linear="no"/>"#), "{opf}");
}

#[test]
fn spine_reads_part_pages_and_chapters_in_order() {
    let opf = Epub::of(&novel()).text("OEBPS/content.opf");

    let order: Vec<_> = [
        "part-01",
        "chapter-001",
        "chapter-002",
        "part-02",
        "chapter-003",
    ]
    .iter()
    .map(|id| opf.find(&format!(r#"<itemref idref="{id}"/>"#)).expect(id))
    .collect();
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
}

// --- Content -------------------------------------------------------------

#[test]
fn chapter_opens_with_a_drop_cap_and_small_caps() {
    let xhtml = Epub::of(&novel()).text("OEBPS/text/chapter-001.xhtml");

    assert!(xhtml.contains("<h1>La partida</h1>"));
    assert!(xhtml.contains(
        r#"<p class="opening"><span class="drop-cap">E</span><span class="lead-in">ra una mañana</span> de sol.</p>"#
    ), "{xhtml}");
    assert!(xhtml.contains("<p>Salimos.</p>"));
}

#[test]
fn dialogue_openings_get_small_caps_but_no_drop_cap() {
    let xhtml = Epub::of(&novel()).text("OEBPS/text/chapter-002.xhtml");

    assert!(
        xhtml.contains(r#"<p class="opening"><span class="lead-in">—¿Vienes?</span></p>"#),
        "{xhtml}"
    );
    assert!(xhtml.contains("<h2>En alta mar</h2>"));
}

#[test]
fn text_is_escaped_and_quotes_become_blockquotes() {
    let xhtml = Epub::of(&novel()).text("OEBPS/text/chapter-003.xhtml");

    assert!(xhtml.contains("&lt;tarde&gt;"));
    assert!(xhtml.contains("<blockquote><p>Nunca más.</p></blockquote>"));
}

#[test]
fn documents_declare_the_book_language() {
    let xhtml = Epub::of(&novel()).text("OEBPS/text/chapter-001.xhtml");

    assert!(xhtml.contains(r#"xml:lang="es" lang="es""#));
}

#[test]
fn part_pages_show_the_part_title() {
    let xhtml = Epub::of(&novel()).text("OEBPS/text/part-02.xhtml");

    assert!(xhtml.contains(r#"<section epub:type="part" role="doc-part"><h1 class="part">Segunda parte</h1></section>"#), "{xhtml}");
}

#[test]
fn packaging_is_deterministic() {
    let book = novel();

    assert_eq!(
        to_bytes(&book, &options()).unwrap(),
        to_bytes(&book, &options()).unwrap()
    );
}
