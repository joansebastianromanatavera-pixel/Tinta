//! Knddle core: the book domain model and the phased conversion pipeline.
//!
//! This crate is pure domain logic: no file, network or device IO.

pub mod model;
pub mod phases;
pub mod pipeline;
pub mod typography;

pub use model::{Block, Book, Chapter, Metadata, Part, Series};
pub use pipeline::{Phase, PhaseError, Pipeline, PipelineError, Progress, ProgressSink};
