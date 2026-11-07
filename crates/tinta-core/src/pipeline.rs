//! The phased conversion pipeline.
//!
//! Each phase does exactly one transformation of the [`Book`]. Phases are
//! independent, testable in isolation and composed in order by [`Pipeline`].

use crate::model::Book;
use thiserror::Error;

/// One step of the conversion pipeline.
pub trait Phase: Send + Sync {
    /// Human-readable name, shown in the UI while the phase runs.
    fn name(&self) -> &'static str;

    /// Transforms the book.
    ///
    /// # Errors
    /// Returns a [`PhaseError`] when the book cannot be transformed.
    fn apply(&self, book: Book) -> Result<Book, PhaseError>;
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0}")]
pub struct PhaseError(pub String);

#[derive(Debug, Error, PartialEq, Eq)]
#[error("phase `{phase}` failed: {source}")]
pub struct PipelineError {
    pub phase: &'static str,
    #[source]
    pub source: PhaseError,
}

/// Progress of a running pipeline, reported before each phase starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub phase: &'static str,
    /// Zero-based index of the phase about to run.
    pub index: usize,
    pub total: usize,
}

impl Progress {
    /// Completed fraction in `0.0..=1.0`.
    #[allow(clippy::cast_precision_loss)]
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.index as f32 / self.total as f32
    }
}

/// Receives progress updates, e.g. to forward them to the UI.
pub trait ProgressSink {
    fn report(&self, progress: Progress);
}

impl<F: Fn(Progress)> ProgressSink for F {
    fn report(&self, progress: Progress) {
        self(progress);
    }
}

#[derive(Default)]
pub struct Pipeline {
    phases: Vec<Box<dyn Phase>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with(mut self, phase: impl Phase + 'static) -> Self {
        self.phases.push(Box::new(phase));
        self
    }

    pub fn phase_names(&self) -> Vec<&'static str> {
        self.phases.iter().map(|phase| phase.name()).collect()
    }

    /// Runs every phase in order.
    ///
    /// # Errors
    /// Stops at the first failing phase and reports which one failed.
    pub fn run(&self, book: Book, sink: &impl ProgressSink) -> Result<Book, PipelineError> {
        let total = self.phases.len();
        self.phases
            .iter()
            .enumerate()
            .try_fold(book, |book, (index, phase)| {
                sink.report(Progress {
                    phase: phase.name(),
                    index,
                    total,
                });
                phase.apply(book).map_err(|source| PipelineError {
                    phase: phase.name(),
                    source,
                })
            })
    }
}
