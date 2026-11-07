//! Composition root: wires the core crates into Tauri commands.

use tinta_core::phases::{DropEmptyChapters, MarkOpeningParagraphs, TypographyCleanup};
use tinta_core::Pipeline;
use tinta_kindle::{KindleModel, CATALOG};

/// The conversion pipeline with every phase implemented so far.
fn conversion_pipeline() -> Pipeline {
    Pipeline::new()
        .with(DropEmptyChapters)
        .with(TypographyCleanup)
        .with(MarkOpeningParagraphs)
}

#[tauri::command]
fn pipeline_phases() -> Vec<&'static str> {
    conversion_pipeline().phase_names()
}

#[tauri::command]
fn kindle_catalog() -> Vec<KindleModel> {
    CATALOG.to_vec()
}

/// Starts the desktop app.
///
/// # Panics
/// Panics if the Tauri runtime cannot start (e.g. no webview available).
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![pipeline_phases, kindle_catalog])
        .run(tauri::generate_context!())
        .expect("failed to start Tinta");
}
