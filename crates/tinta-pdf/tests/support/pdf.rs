//! A minimal PDF writer, enough to turn laid-out test pages into real PDF
//! files: one standard font (Helvetica, regular and bold), `WinAnsi` text and
//! an info dictionary. It lets tests round-trip pages through pdfium.

use std::fmt::Write as _;
use tinta_pdf::layout::{Document, TextLine};

const REGULAR_FONT: &str = "F1";
const BOLD_FONT: &str = "F2";
/// Objects 1–4: catalog, page tree, regular font and bold font.
const FIXED_OBJECTS: usize = 4;
const INFO_OBJECT_OFFSET: usize = 1;

pub fn write(document: &Document) -> Vec<u8> {
    let mut writer = Writer::default();
    let page_count = document.pages.len();
    let first_page = FIXED_OBJECTS + 1;
    let kids = (0..page_count).fold(String::new(), |mut kids, i| {
        let _ = write!(kids, "{} 0 R ", first_page + i * 2);
        kids
    });
    writer.object(b"<< /Type /Catalog /Pages 2 0 R >>");
    writer.object(format!("<< /Type /Pages /Kids [{kids}] /Count {page_count} >>").as_bytes());
    writer.object(font("Helvetica").as_bytes());
    writer.object(font("Helvetica-Bold").as_bytes());
    for (index, page) in document.pages.iter().enumerate() {
        let content_id = first_page + index * 2 + 1;
        writer.object(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] \
                 /Resources << /Font << /{REGULAR_FONT} 3 0 R /{BOLD_FONT} 4 0 R >> >> \
                 /Contents {content_id} 0 R >>",
                page.width, page.height
            )
            .as_bytes(),
        );
        writer.stream(&content(&page.lines, page.height));
    }
    let info = writer.objects() + INFO_OBJECT_OFFSET;
    writer.object(&info_dictionary(document));
    writer.finish(info)
}

fn font(name: &str) -> String {
    format!("<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>")
}

fn content(lines: &[TextLine], page_height: f32) -> Vec<u8> {
    let mut out = Vec::new();
    for line in lines {
        let font = if line.bold { BOLD_FONT } else { REGULAR_FONT };
        let baseline = page_height - line.bottom;
        out.extend_from_slice(
            format!(
                "BT /{font} {} Tf {} {} Td ",
                line.font_size, line.left, baseline
            )
            .as_bytes(),
        );
        out.extend(pdf_string(&line.text));
        out.extend_from_slice(b" Tj ET\n");
    }
    out
}

fn info_dictionary(document: &Document) -> Vec<u8> {
    let mut out = b"<< ".to_vec();
    for (key, value) in [("Title", &document.title), ("Author", &document.author)] {
        if let Some(value) = value {
            out.extend(format!("/{key} ").as_bytes());
            out.extend(pdf_string(value));
            out.push(b' ');
        }
    }
    out.extend(b">>");
    out
}

/// A literal string in `WinAnsi`, which matches Latin-1 for Spanish letters.
fn pdf_string(text: &str) -> Vec<u8> {
    let mut out = vec![b'('];
    for c in text.chars() {
        let byte = u8::try_from(u32::from(c)).expect("test text must be Latin-1");
        if matches!(byte, b'(' | b')' | b'\\') {
            out.push(b'\\');
        }
        out.push(byte);
    }
    out.push(b')');
    out
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
    offsets: Vec<usize>,
}

impl Writer {
    fn objects(&self) -> usize {
        self.offsets.len()
    }

    fn object(&mut self, body: &[u8]) {
        self.begin();
        self.bytes.extend_from_slice(body);
        self.bytes.extend_from_slice(b"\nendobj\n");
    }

    fn stream(&mut self, data: &[u8]) {
        self.begin();
        self.bytes
            .extend(format!("<< /Length {} >>\nstream\n", data.len()).as_bytes());
        self.bytes.extend_from_slice(data);
        self.bytes.extend_from_slice(b"\nendstream\nendobj\n");
    }

    fn begin(&mut self) {
        if self.bytes.is_empty() {
            self.bytes.extend_from_slice(b"%PDF-1.4\n");
        }
        self.offsets.push(self.bytes.len());
        let id = self.offsets.len();
        self.bytes.extend(format!("{id} 0 obj\n").as_bytes());
    }

    fn finish(mut self, info: usize) -> Vec<u8> {
        let xref = self.bytes.len();
        let mut table = format!("xref\n0 {}\n0000000000 65535 f \n", self.offsets.len() + 1);
        for offset in &self.offsets {
            let _ = writeln!(table, "{offset:010} 00000 n ");
        }
        let _ = write!(
            table,
            "trailer\n<< /Size {} /Root 1 0 R /Info {info} 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            self.offsets.len() + 1
        );
        self.bytes.extend(table.as_bytes());
        self.bytes
    }
}
