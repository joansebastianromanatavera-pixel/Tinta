//! Prints the structure Tinta rebuilds from a PDF.
//!
//! Usage: `cargo run -p tinta-pdf --example dump -- libro.pdf [--page N]`
//!
//! With `--page N` it prints the raw lines of that page instead, with their
//! position, size and weight, to tune the reconstruction heuristics.

use std::path::{Path, PathBuf};
use tinta_core::Block;
use tinta_pdf::pdfium::extract;
use tinta_pdf::{read, PdfiumLibrary};

const PREVIEW_CHARS: usize = 160;

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Uso: dump <archivo.pdf>");
        std::process::exit(2);
    };
    let vendor: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pdfium");
    let library = PdfiumLibrary::load(&[vendor]).expect("pdfium");
    if let Some(page) = std::env::args()
        .nth(3)
        .and_then(|n| n.parse::<usize>().ok())
    {
        print_page(&library, Path::new(&path), page);
        return;
    }
    let started = std::time::Instant::now();
    let book = match read(&library, Path::new(&path), &|_, _| {}) {
        Ok(book) => book,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    println!(
        "Título: {}  Autores: {:?}",
        book.metadata.title, book.metadata.authors
    );
    println!("Leído en {:?}\n", started.elapsed());
    for part in &book.parts {
        if let Some(title) = &part.title {
            println!("# {title}");
        }
        for chapter in &part.chapters {
            println!("  ## {} ({} bloques)", chapter.title, chapter.blocks.len());
            for block in chapter.blocks.iter().take(2) {
                if let Block::Paragraph { text, .. } | Block::Heading { text, .. } = block {
                    let preview: String = text
                        .replace('\n', "⏎")
                        .chars()
                        .take(PREVIEW_CHARS)
                        .collect();
                    println!("     · {preview}");
                }
            }
        }
    }
}

fn print_page(library: &PdfiumLibrary, path: &Path, number: usize) {
    let document = extract(library, path, &|_, _| {}).expect("readable PDF");
    let page = &document.pages[number - 1];
    println!(
        "Página {number}: {}×{} pt, {} imágenes",
        page.width, page.height, page.image_count
    );
    for line in &page.lines {
        println!(
            "top {:>6.1} bot {:>6.1} left {:>6.1} right {:>6.1} size {:>5.1}{} | {}",
            line.top,
            line.bottom,
            line.left,
            line.right,
            line.font_size,
            if line.bold { " B" } else { "  " },
            line.text
        );
    }
}
