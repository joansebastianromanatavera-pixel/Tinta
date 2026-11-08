//! EPUB 3 packager for Tinta.
//!
//! Produces the EPUB a Kindle treats like a bought book: a two-level table
//! of contents, "Go to beginning" on the first chapter, series metadata,
//! a drop cap and small caps opening each chapter, and no embedded fonts so
//! the reader's own settings always apply. The same XHTML and CSS feed the
//! AZW3 writer.

mod contents;
mod navigation;
mod options;
mod package;
mod xhtml;
mod xml;

use contents::Contents;
use std::io::{Cursor, Seek, Write};
use thiserror::Error;
use tinta_core::{Block, Book};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

/// Language tag for books whose language could not be detected.
const UNDETERMINED_LANGUAGE: &str = "und";
const MIMETYPE: &str = "application/epub+zip";
const STYLESHEET: &str = include_str!("style.css");
const CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles>
<rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
</rootfiles>
</container>
"#;

/// Values that must come from outside the pure packager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpubOptions {
    /// Unique, stable identifier, e.g. `urn:uuid:…`.
    pub identifier: String,
    /// Last modification, `CCYY-MM-DDThh:mm:ssZ`.
    pub modified: String,
}

#[derive(Debug, Error)]
pub enum EpubError {
    #[error("El libro no tiene capítulos.")]
    NoChapters,
    #[error("Las imágenes todavía no se pueden incluir en el EPUB.")]
    ImagesNotSupported,
    #[error("No se pudo escribir el EPUB: {0}")]
    Write(#[from] std::io::Error),
    #[error("No se pudo comprimir el EPUB: {0}")]
    Zip(#[from] zip::result::ZipError),
}

/// Packages the book into EPUB bytes.
///
/// # Errors
/// See [`EpubError`].
pub fn to_bytes(book: &Book, options: &EpubOptions) -> Result<Vec<u8>, EpubError> {
    let mut buffer = Cursor::new(Vec::new());
    write(book, options, &mut buffer)?;
    Ok(buffer.into_inner())
}

/// Packages the book into any seekable writer, such as a file.
///
/// # Errors
/// See [`EpubError`].
pub fn write(book: &Book, options: &EpubOptions, out: impl Write + Seek) -> Result<(), EpubError> {
    if book.chapter_count() == 0 {
        return Err(EpubError::NoChapters);
    }
    if has_images(book) {
        return Err(EpubError::ImagesNotSupported);
    }
    let language = book
        .metadata
        .language
        .as_deref()
        .unwrap_or(UNDETERMINED_LANGUAGE);
    let contents = Contents::lay_out(book, language);
    let mut zip = ZipWriter::new(out);
    let stored = file_options(CompressionMethod::Stored);
    let deflated = file_options(CompressionMethod::Deflated);
    add(&mut zip, "mimetype", MIMETYPE, stored)?;
    add(&mut zip, "META-INF/container.xml", CONTAINER, deflated)?;
    let opf = package::opf(&contents, &book.metadata, options, language);
    add(&mut zip, "OEBPS/content.opf", &opf, deflated)?;
    let ncx = navigation::ncx(&contents, &book.metadata, &options.identifier, language);
    add(&mut zip, "OEBPS/toc.ncx", &ncx, deflated)?;
    add(
        &mut zip,
        "OEBPS/nav.xhtml",
        &navigation::nav(&contents, language),
        deflated,
    )?;
    add(&mut zip, "OEBPS/style.css", STYLESHEET, deflated)?;
    for document in &contents.documents {
        let name = format!("OEBPS/{}", document.href);
        add(&mut zip, &name, &document.xhtml, deflated)?;
    }
    zip.finish()?;
    Ok(())
}

/// Fixed timestamps keep the output byte-for-byte reproducible.
fn file_options(method: CompressionMethod) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(method)
        .last_modified_time(DateTime::default())
}

fn add<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    name: &str,
    contents: &str,
    options: SimpleFileOptions,
) -> Result<(), EpubError> {
    zip.start_file(name, options)?;
    zip.write_all(contents.as_bytes())?;
    Ok(())
}

fn has_images(book: &Book) -> bool {
    book.chapters()
        .flat_map(|chapter| &chapter.blocks)
        .any(|block| matches!(block, Block::Image { .. }))
}
