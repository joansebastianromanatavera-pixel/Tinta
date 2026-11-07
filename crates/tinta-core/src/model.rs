//! The internal book model every source is normalised into.
//!
//! A book is organised as parts that contain chapters, giving the two-level
//! table of contents used by published books. Books without parts use a
//! single untitled part.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Book {
    pub metadata: Metadata,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub authors: Vec<String>,
    /// BCP 47 language tag, e.g. `es` or `en-US`.
    pub language: Option<String>,
    pub series: Option<Series>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Part {
    /// `None` when the book has no part divisions.
    pub title: Option<String>,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    Heading {
        level: u8,
        text: String,
    },
    /// `opening` marks the first paragraph of a chapter, rendered with a
    /// drop cap and small caps.
    Paragraph {
        text: String,
        opening: bool,
    },
    Quote {
        text: String,
    },
    Image {
        source: String,
        alt: String,
    },
}

impl Block {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self::Paragraph {
            text: text.into(),
            opening: false,
        }
    }

    /// Mutable access to the block's text, if it carries any.
    pub fn text_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Heading { text, .. } | Self::Paragraph { text, .. } | Self::Quote { text } => {
                Some(text)
            }
            Self::Image { .. } => None,
        }
    }
}

impl Book {
    pub fn chapters(&self) -> impl Iterator<Item = &Chapter> {
        self.parts.iter().flat_map(|part| part.chapters.iter())
    }

    pub fn chapters_mut(&mut self) -> impl Iterator<Item = &mut Chapter> {
        self.parts
            .iter_mut()
            .flat_map(|part| part.chapters.iter_mut())
    }

    pub fn chapter_count(&self) -> usize {
        self.parts.iter().map(|part| part.chapters.len()).sum()
    }
}
