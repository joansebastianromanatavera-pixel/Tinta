//! Converts a PDF into a Kindle-ready EPUB with the standard pipeline.
//!
//! Usage: `cargo run -p tinta-epub --example convert -- libro.pdf [salida.epub]`

use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use tinta_core::phases::standard_pipeline;
use tinta_core::Progress;
use tinta_epub::{to_bytes, EpubOptions};
use tinta_pdf::{read, PdfiumLibrary};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(input) = args.next().map(PathBuf::from) else {
        eprintln!("Uso: convert <libro.pdf> [salida.epub]");
        std::process::exit(2);
    };
    let output = args
        .next()
        .map_or_else(|| input.with_extension("epub"), PathBuf::from);
    if let Err(message) = convert(&input, &output) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn convert(input: &Path, output: &Path) -> Result<(), String> {
    let started = Instant::now();
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pdfium");
    let library = PdfiumLibrary::load(&[vendor]).map_err(|e| e.to_string())?;
    let raw = read(&library, input, &|_, _| {}).map_err(|e| e.to_string())?;
    let book = standard_pipeline()
        .run(raw, &|progress: Progress| eprintln!("· {}", progress.phase))
        .map_err(|e| e.to_string())?;
    let bytes = to_bytes(&book, &EpubOptions::for_book(&book, SystemTime::now()))
        .map_err(|e| e.to_string())?;
    std::fs::write(output, bytes).map_err(|e| e.to_string())?;
    eprintln!(
        "{} → {} ({} capítulos, idioma {}) en {:?}",
        input.display(),
        output.display(),
        book.chapter_count(),
        book.metadata.language.as_deref().unwrap_or("?"),
        started.elapsed()
    );
    Ok(())
}
