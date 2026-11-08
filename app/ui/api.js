// Bridge to the Rust core. Falls back to sample data in a plain browser so
// the interface can be designed and tested without the desktop shell.
const inTauri = "__TAURI_INTERNALS__" in window;

/** Formats the converter reads today; the rest arrive in later releases. */
export const CONVERTIBLE_EXTENSIONS = ["pdf"];
const OUTPUT_EXTENSION = "epub";

/** Stages and timing of the simulated conversion used in a plain browser. */
const SAMPLE_STAGES = [
  "Leyendo el PDF",
  "Capítulos vacíos",
  "Limpieza tipográfica",
  "Inicio de capítulos",
  "Idioma",
  "Validación",
  "Empaquetando",
];
const SAMPLE_STEP_MS = 90;
const SAMPLE_STEPS = 24;
const SAMPLE_RESULT = {
  title: "El retrato de Dorian Gray",
  authors: ["Oscar Wilde"],
  language: "es",
  chapters: 13,
  bytes: 133245,
  fileName: "El retrato de Dorian Gray.epub",
};

/**
 * Converts a book. `onProgress` receives `{ fraction, stage }` as it goes.
 * Resolves to the book summary; rejects with `{ kind, message }`.
 */
export async function convertBook(path, onProgress) {
  if (!inTauri) return simulateConversion(onProgress);
  const { invoke, Channel } = await import("@tauri-apps/api/core");
  const channel = new Channel();
  channel.onmessage = onProgress;
  return invoke("convert_book", { path, onProgress: channel });
}

/** Asks where to save the converted book. Resolves to the path, or null. */
export async function saveBook(fileName) {
  if (!inTauri) return `~/Descargas/${fileName}`;
  const { save } = await import("@tauri-apps/plugin-dialog");
  const { invoke } = await import("@tauri-apps/api/core");
  const { downloadDir, join } = await import("@tauri-apps/api/path");
  const destination = await save({
    defaultPath: await join(await downloadDir(), fileName),
    filters: [{ name: "Libro EPUB", extensions: [OUTPUT_EXTENSION] }],
  });
  if (!destination) return null;
  await invoke("save_book", { destination });
  return destination;
}

function simulateConversion(onProgress) {
  return new Promise((resolve) => {
    let step = 0;
    const timer = setInterval(() => {
      step += 1;
      const fraction = step / SAMPLE_STEPS;
      const stage = SAMPLE_STAGES[Math.min(SAMPLE_STAGES.length - 1, Math.floor(fraction * SAMPLE_STAGES.length))];
      onProgress({ fraction, stage });
      if (step === SAMPLE_STEPS) {
        clearInterval(timer);
        resolve(SAMPLE_RESULT);
      }
    }, SAMPLE_STEP_MS);
  });
}

/** Opens the native file picker. Resolves to a path, or null if cancelled. */
export async function pickBookFile() {
  if (!inTauri) return pickInBrowser();
  const { open } = await import("@tauri-apps/plugin-dialog");
  return open({
    multiple: false,
    directory: false,
    filters: [{ name: "Libros PDF", extensions: CONVERTIBLE_EXTENSIONS }],
  });
}

function pickInBrowser() {
  return new Promise((resolve) => {
    const input = Object.assign(document.createElement("input"), {
      type: "file",
      accept: CONVERTIBLE_EXTENSIONS.map((ext) => `.${ext}`).join(","),
    });
    input.addEventListener("change", () => resolve(input.files[0]?.name ?? null));
    input.addEventListener("cancel", () => resolve(null));
    input.click();
  });
}

/** Subscribes to native drag & drop. `handlers` = { enter, leave, drop(paths) }. */
export async function onFileDrop(handlers) {
  if (!inTauri) return;
  const { getCurrentWebview } = await import("@tauri-apps/api/webview");
  await getCurrentWebview().onDragDropEvent(({ payload }) => {
    if (payload.type === "enter") handlers.enter();
    else if (payload.type === "leave") handlers.leave();
    else if (payload.type === "drop") handlers.drop(payload.paths);
  });
}
