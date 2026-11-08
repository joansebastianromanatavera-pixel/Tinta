//! Navigation documents: the EPUB 3 `nav.xhtml` (table of contents and
//! landmarks) and the EPUB 2 `toc.ncx` that older readers and Kindle
//! conversions still rely on.

use crate::contents::{Contents, TocEntry};
use crate::xml::escape;
use std::fmt::Write as _;
use tinta_core::Metadata;

pub(crate) const TOC_LABEL: &str = "Índice";
pub(crate) const START_LABEL: &str = "Inicio";

pub(crate) fn nav(contents: &Contents, language: &str) -> String {
    let mut toc = String::new();
    write_entries(&mut toc, &contents.toc);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{language}" lang="{language}">
<head>
<meta charset="UTF-8"/>
<title>{TOC_LABEL}</title>
</head>
<body>
<nav epub:type="toc" id="toc" role="doc-toc"><h1>{TOC_LABEL}</h1>{toc}</nav>
<nav epub:type="landmarks" id="landmarks" hidden="hidden"><ol><li><a epub:type="toc" href="nav.xhtml#toc">{TOC_LABEL}</a></li><li><a epub:type="bodymatter" href="{start}">{START_LABEL}</a></li></ol></nav>
</body>
</html>
"#,
        start = contents.start_href,
    )
}

fn write_entries(out: &mut String, entries: &[TocEntry]) {
    out.push_str("<ol>");
    for entry in entries {
        let _ = write!(
            out,
            r#"<li><a href="{}">{}</a>"#,
            entry.href,
            escape(&entry.title)
        );
        if !entry.children.is_empty() {
            write_entries(out, &entry.children);
        }
        out.push_str("</li>");
    }
    out.push_str("</ol>");
}

pub(crate) fn ncx(
    contents: &Contents,
    metadata: &Metadata,
    identifier: &str,
    language: &str,
) -> String {
    let mut map = String::new();
    let mut play_order = 0;
    write_nav_points(&mut map, &contents.toc, &mut play_order);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1" xml:lang="{language}">
<head>
<meta name="dtb:uid" content="{identifier}"/>
<meta name="dtb:depth" content="{depth}"/>
<meta name="dtb:totalPageCount" content="0"/>
<meta name="dtb:maxPageNumber" content="0"/>
</head>
<docTitle><text>{title}</text></docTitle>
<navMap>{map}</navMap>
</ncx>
"#,
        identifier = escape(identifier),
        depth = contents.depth(),
        title = escape(&metadata.title),
    )
}

fn write_nav_points(out: &mut String, entries: &[TocEntry], play_order: &mut usize) {
    for entry in entries {
        *play_order += 1;
        let _ = write!(
            out,
            r#"<navPoint id="nav-{order}" playOrder="{order}"><navLabel><text>{title}</text></navLabel><content src="{href}"/>"#,
            order = play_order,
            title = escape(&entry.title),
            href = entry.href,
        );
        write_nav_points(out, &entry.children, play_order);
        out.push_str("</navPoint>");
    }
}
