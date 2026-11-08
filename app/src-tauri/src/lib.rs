//! Composition root: wires the core crates into Tauri commands.

mod conversion;

use conversion::{ConversionError, Converted, ProgressEvent};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tinta_kindle::{KindleModel, CATALOG};
use tinta_pdf::PdfiumLibrary;

/// Folder, inside the app resources, that holds the pdfium library.
const PDFIUM_RESOURCE_DIR: &str = "pdfium";
/// Folder, inside the app cache, where converted books wait to be saved.
const BOOKS_CACHE_DIR: &str = "books";

#[derive(Default)]
struct AppState {
    converting: AtomicBool,
    last_book: Mutex<Option<PathBuf>>,
}

/// Releases the "converting" flag however the conversion ends.
struct ConvertingGuard<'a>(&'a AtomicBool);

impl Drop for ConvertingGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[tauri::command]
async fn convert_book(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    on_progress: Channel<ProgressEvent>,
) -> Result<Converted, ConversionError> {
    if state.converting.swap(true, Ordering::AcqRel) {
        return Err(ConversionError::Busy);
    }
    let _guard = ConvertingGuard(&state.converting);
    let source = PathBuf::from(path);
    let output_dir = books_dir(&app)?;
    let search_dirs = pdfium_dirs(&app);
    let (book_path, converted) = tauri::async_runtime::spawn_blocking(move || {
        let library = PdfiumLibrary::load(&search_dirs)?;
        conversion::convert(&library, &source, &output_dir, &|event| {
            let _ = on_progress.send(event);
        })
    })
    .await
    .map_err(|error| ConversionError::Io(std::io::Error::other(error.to_string())))??;
    *lock(&state.last_book) = Some(book_path);
    Ok(converted)
}

/// Copies the last converted book to where the user chose to save it.
#[tauri::command]
async fn save_book(state: State<'_, AppState>, destination: String) -> Result<(), ConversionError> {
    let source = lock(&state.last_book)
        .clone()
        .ok_or(ConversionError::NothingToSave)?;
    tauri::async_runtime::spawn_blocking(move || std::fs::copy(source, destination))
        .await
        .map_err(|error| ConversionError::Io(std::io::Error::other(error.to_string())))??;
    Ok(())
}

#[tauri::command]
fn kindle_catalog() -> Vec<KindleModel> {
    CATALOG.to_vec()
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn books_dir(app: &AppHandle) -> Result<PathBuf, ConversionError> {
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|error| ConversionError::Io(std::io::Error::other(error.to_string())))?;
    Ok(cache.join(BOOKS_CACHE_DIR))
}

/// Where pdfium may live: bundled with the app, next to the executable,
/// and, in development builds, the repository's `vendor/pdfium`.
fn pdfium_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(resources) = app.path().resource_dir() {
        dirs.push(resources.join(PDFIUM_RESOURCE_DIR));
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        dirs.push(exe_dir);
    }
    if cfg!(debug_assertions) {
        dirs.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pdfium"));
    }
    dirs
}

/// Starts the desktop app.
///
/// # Panics
/// Panics if the Tauri runtime cannot start (e.g. no webview available).
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            convert_book,
            save_book,
            kindle_catalog
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Tinta");
}
