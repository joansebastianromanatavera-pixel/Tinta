//! The book laid out as EPUB documents: one XHTML file per part title page
//! and per chapter, plus the table of contents that links them.

use crate::xhtml;
use tinta_core::Book;

pub(crate) struct Document {
    pub id: String,
    pub href: String,
    pub xhtml: String,
}

pub(crate) struct TocEntry {
    pub title: String,
    pub href: String,
    pub children: Vec<TocEntry>,
}

pub(crate) struct Contents {
    /// Documents in reading order.
    pub documents: Vec<Document>,
    pub toc: Vec<TocEntry>,
    /// Where "Go to beginning" lands: the first chapter.
    pub start_href: String,
}

impl Contents {
    pub fn lay_out(book: &Book, language: &str) -> Self {
        let mut documents = Vec::new();
        let mut toc = Vec::new();
        let mut chapter_number = 0;
        for (part_index, part) in book.parts.iter().enumerate() {
            let mut entries = Vec::new();
            for chapter in &part.chapters {
                chapter_number += 1;
                let id = format!("chapter-{chapter_number:03}");
                let href = format!("text/{id}.xhtml");
                entries.push(TocEntry {
                    title: chapter.title.clone(),
                    href: href.clone(),
                    children: Vec::new(),
                });
                documents.push(Document {
                    xhtml: xhtml::chapter(chapter, language),
                    id,
                    href,
                });
            }
            match &part.title {
                Some(title) => {
                    let id = format!("part-{:02}", part_index + 1);
                    let href = format!("text/{id}.xhtml");
                    let first_chapter = documents.len() - part.chapters.len();
                    documents.insert(
                        first_chapter,
                        Document {
                            xhtml: xhtml::part(title, language),
                            id,
                            href: href.clone(),
                        },
                    );
                    toc.push(TocEntry {
                        title: title.clone(),
                        href,
                        children: entries,
                    });
                }
                None => toc.extend(entries),
            }
        }
        let start_href = documents
            .iter()
            .find(|document| document.id.starts_with("chapter-"))
            .map(|document| document.href.clone())
            .unwrap_or_default();
        Self {
            documents,
            toc,
            start_href,
        }
    }

    /// Two when parts group chapters, one otherwise.
    pub fn depth(&self) -> usize {
        if self.toc.iter().any(|entry| !entry.children.is_empty()) {
            2
        } else {
            1
        }
    }
}
