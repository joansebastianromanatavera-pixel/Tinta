//! Round trip through the real pdfium engine: laid-out pages are written as
//! PDF files, read back with pdfium and rebuilt into books.
//!
//! Needs the pdfium library: run `scripts/fetch-pdfium.sh` once.

mod support;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use support::{document, page, pdf};
use tinta_pdf::layout::Document;
use tinta_pdf::{read, PdfError, PdfiumLibrary};

const CHAPTER_SIZE: f32 = 18.0;
const BODY: &str = "Había una vez, en un lugar no muy lejano, una casa junto al";

fn library() -> PdfiumLibrary {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pdfium");
    PdfiumLibrary::load(&[vendor]).expect("pdfium no está instalado: corre scripts/fetch-pdfium.sh")
}

fn write_pdf(name: &str, document: &Document) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.pdf"));
    std::fs::write(&path, pdf::write(document)).expect("write test PDF");
    path
}

fn ignore_progress(_: usize, _: usize) {}

fn novel() -> Document {
    let mut doc = document(vec![
        page()
            .heading("La partida", CHAPTER_SIZE)
            .indented(BODY)
            .body("mar donde vivía una familia de pescadores que")
            .footer("1")
            .build(),
        page()
            .header("LA PARTIDA")
            .body("salía cada mañana antes del alba y volvía")
            .indented(BODY)
            .last("río.")
            .footer("2")
            .build(),
        page()
            .heading("El regreso", CHAPTER_SIZE)
            .indented(BODY)
            .last("puerto.")
            .footer("3")
            .build(),
    ]);
    doc.title = Some("Mar de fondo".into());
    doc.author = Some("Ana Pérez".into());
    doc
}

#[test]
fn a_real_pdf_keeps_metadata_chapters_and_joined_paragraphs() {
    let path = write_pdf("novel-structure", &novel());

    let book = read(&library(), &path, &ignore_progress).expect("readable PDF");

    assert_eq!(book.metadata.title, "Mar de fondo");
    assert_eq!(book.metadata.authors, ["Ana Pérez"]);
    let titles: Vec<_> = book.chapters().map(|c| c.title.as_str()).collect();
    assert_eq!(titles, ["La partida", "El regreso"]);
    let first = &book.parts[0].chapters[0].blocks;
    assert_eq!(first.len(), 2, "the paragraph crossing the page is one");
}

#[test]
fn progress_is_reported_for_every_page() {
    let path = write_pdf("progress", &novel());
    let seen = RefCell::new(Vec::new());

    read(&library(), &path, &|done, total| {
        seen.borrow_mut().push((done, total));
    })
    .expect("readable PDF");

    assert_eq!(seen.into_inner(), [(1, 3), (2, 3), (3, 3)]);
}

#[test]
fn a_pdf_without_text_is_reported_as_empty() {
    let path = write_pdf("blank", &document(vec![page().build()]));

    let error = read(&library(), &path, &ignore_progress).unwrap_err();

    assert!(matches!(error, PdfError::Empty), "{error:?}");
}

#[test]
fn a_file_that_is_not_a_pdf_is_invalid() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("not-a.pdf");
    std::fs::write(&path, b"esto no es un PDF").unwrap();

    let error = read(&library(), &path, &ignore_progress).unwrap_err();

    assert!(matches!(error, PdfError::Invalid), "{error:?}");
}

#[test]
fn a_missing_file_is_unreadable() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-existe.pdf");

    let error = read(&library(), &path, &ignore_progress).unwrap_err();

    assert!(matches!(error, PdfError::Unreadable), "{error:?}");
}
