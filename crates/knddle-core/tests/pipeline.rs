use knddle_core::phases::{DropEmptyChapters, MarkOpeningParagraphs, TypographyCleanup};
use knddle_core::{Block, Book, Chapter, Metadata, Part, Phase, PhaseError, Pipeline, Progress};
use std::cell::RefCell;

fn chapter(title: &str, blocks: Vec<Block>) -> Chapter {
    Chapter {
        title: title.into(),
        blocks,
    }
}

fn book(chapters: Vec<Chapter>) -> Book {
    Book {
        metadata: Metadata {
            title: "Prueba".into(),
            ..Metadata::default()
        },
        parts: vec![Part {
            title: None,
            chapters,
        }],
    }
}

#[test]
fn runs_phases_in_order_and_reports_progress() {
    let reported = RefCell::new(Vec::new());
    let pipeline = Pipeline::new()
        .with(TypographyCleanup)
        .with(MarkOpeningParagraphs);

    let result = pipeline
        .run(
            book(vec![chapter("Uno", vec![Block::paragraph("ﬁn")])]),
            &|p: Progress| {
                reported.borrow_mut().push((p.phase, p.index, p.total));
            },
        )
        .unwrap();

    assert_eq!(
        reported.into_inner(),
        vec![
            ("Limpieza tipográfica", 0, 2),
            ("Inicio de capítulos", 1, 2)
        ]
    );
    assert_eq!(
        result.parts[0].chapters[0].blocks[0],
        Block::Paragraph {
            text: "fin".into(),
            opening: true
        }
    );
}

#[test]
fn progress_fraction_is_relative_to_total() {
    let progress = Progress {
        phase: "x",
        index: 1,
        total: 4,
    };
    assert!((progress.fraction() - 0.25).abs() < f32::EPSILON);
}

struct Failing;

impl Phase for Failing {
    fn name(&self) -> &'static str {
        "Falla"
    }
    fn apply(&self, _: Book) -> Result<Book, PhaseError> {
        Err(PhaseError("boom".into()))
    }
}

#[test]
fn stops_at_first_failing_phase_and_names_it() {
    let pipeline = Pipeline::new().with(Failing).with(TypographyCleanup);
    let error = pipeline
        .run(Book::default(), &|_: Progress| {})
        .unwrap_err();
    assert_eq!(error.phase, "Falla");
}

#[test]
fn marks_only_first_paragraph_of_each_chapter_as_opening() {
    let input = book(vec![chapter(
        "Uno",
        vec![
            Block::Heading {
                level: 2,
                text: "Uno".into(),
            },
            Block::paragraph("primero"),
            Block::paragraph("segundo"),
        ],
    )]);

    let output = MarkOpeningParagraphs.apply(input).unwrap();
    let openings: Vec<bool> = output.parts[0].chapters[0]
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Paragraph { opening, .. } => Some(*opening),
            _ => None,
        })
        .collect();

    assert_eq!(openings, vec![true, false]);
}

#[test]
fn cleans_chapter_titles_and_all_text_blocks() {
    let input = book(vec![chapter(
        "  El  ﬁnal ",
        vec![
            Block::Quote {
                text: "\"cita\"".into(),
            },
            Block::Image {
                source: "a.png".into(),
                alt: "ﬁg".into(),
            },
        ],
    )]);

    let output = TypographyCleanup.apply(input).unwrap();
    let ch = &output.parts[0].chapters[0];

    assert_eq!(ch.title, "El final");
    assert_eq!(
        ch.blocks[0],
        Block::Quote {
            text: "“cita”".into()
        }
    );
    assert_eq!(
        ch.blocks[1],
        Block::Image {
            source: "a.png".into(),
            alt: "ﬁg".into()
        }
    );
}

#[test]
fn drops_empty_chapters_and_parts() {
    let input = Book {
        metadata: Metadata::default(),
        parts: vec![
            Part {
                title: Some("I".into()),
                chapters: vec![chapter("vacío", vec![])],
            },
            Part {
                title: Some("II".into()),
                chapters: vec![chapter("ok", vec![Block::paragraph("x")])],
            },
        ],
    };

    let output = DropEmptyChapters.apply(input).unwrap();

    assert_eq!(output.parts.len(), 1);
    assert_eq!(output.chapter_count(), 1);
}

#[test]
fn fails_when_book_has_no_content() {
    assert!(DropEmptyChapters
        .apply(book(vec![chapter("vacío", vec![])]))
        .is_err());
}
