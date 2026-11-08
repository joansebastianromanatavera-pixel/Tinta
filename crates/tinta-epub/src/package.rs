//! The package document (`content.opf`): metadata, manifest, spine and the
//! EPUB 2 guide the Kindle reads for "Go to beginning".

use crate::contents::Contents;
use crate::navigation::{START_LABEL, TOC_LABEL};
use crate::xml::escape;
use crate::EpubOptions;
use std::fmt::Write as _;
use tinta_core::Metadata;

pub(crate) fn opf(
    contents: &Contents,
    metadata: &Metadata,
    options: &EpubOptions,
    language: &str,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id" xml:lang="{language}">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
{metadata}</metadata>
<manifest>
<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
<item id="style" href="style.css" media-type="text/css"/>
{manifest}</manifest>
<spine toc="ncx">
{spine}</spine>
<guide>
<reference type="toc" title="{TOC_LABEL}" href="nav.xhtml"/>
<reference type="text" title="{START_LABEL}" href="{start}"/>
</guide>
</package>
"#,
        metadata = metadata_elements(metadata, options, language),
        manifest = manifest(contents),
        spine = spine(contents),
        start = contents.start_href,
    )
}

fn metadata_elements(metadata: &Metadata, options: &EpubOptions, language: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        r#"<dc:identifier id="book-id">{}</dc:identifier>"#,
        escape(&options.identifier)
    );
    let _ = writeln!(out, "<dc:title>{}</dc:title>", escape(&metadata.title));
    for (index, author) in metadata.authors.iter().enumerate() {
        let id = format!("creator-{}", index + 1);
        let _ = writeln!(
            out,
            r#"<dc:creator id="{id}">{}</dc:creator>"#,
            escape(author)
        );
        let _ = writeln!(
            out,
            r##"<meta refines="#{id}" property="role" scheme="marc:relators">aut</meta>"##
        );
    }
    let _ = writeln!(out, "<dc:language>{language}</dc:language>");
    let _ = writeln!(
        out,
        r#"<meta property="dcterms:modified">{}</meta>"#,
        escape(&options.modified)
    );
    if let Some(series) = &metadata.series {
        let name = escape(&series.name);
        let _ = writeln!(
            out,
            r#"<meta property="belongs-to-collection" id="series">{name}</meta>"#
        );
        let _ = writeln!(
            out,
            r##"<meta refines="#series" property="collection-type">series</meta>"##
        );
        let _ = writeln!(
            out,
            r##"<meta refines="#series" property="group-position">{}</meta>"##,
            series.index
        );
        let _ = writeln!(out, r#"<meta name="calibre:series" content="{name}"/>"#);
        let _ = writeln!(
            out,
            r#"<meta name="calibre:series_index" content="{}"/>"#,
            series.index
        );
    }
    out
}

fn manifest(contents: &Contents) -> String {
    contents
        .documents
        .iter()
        .fold(String::new(), |mut out, document| {
            let _ = writeln!(
                out,
                r#"<item id="{}" href="{}" media-type="application/xhtml+xml"/>"#,
                document.id, document.href
            );
            out
        })
}

fn spine(contents: &Contents) -> String {
    contents
        .documents
        .iter()
        .fold(String::new(), |mut out, document| {
            let _ = writeln!(out, r#"<itemref idref="{}"/>"#, document.id);
            out
        })
}
