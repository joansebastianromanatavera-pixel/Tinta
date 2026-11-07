// Bridge to the Rust core. Falls back to sample data in a plain browser so
// the interface can be designed and tested without the desktop shell.
const inTauri = "__TAURI_INTERNALS__" in window;

export const SUPPORTED_EXTENSIONS = [
  "pdf", "epub", "mobi", "azw3", "docx", "odt", "rtf", "txt", "md", "html", "htm", "fb2",
];

const SAMPLE_PHASES = ["Capítulos vacíos", "Limpieza tipográfica", "Inicio de capítulos"];

async function invoke(command) {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke(command);
}

export async function pipelinePhases() {
  return inTauri ? invoke("pipeline_phases") : SAMPLE_PHASES;
}

/** Opens the native file picker. Resolves to a path, or null if cancelled. */
export async function pickBookFile() {
  if (!inTauri) return pickInBrowser();
  const { open } = await import("@tauri-apps/plugin-dialog");
  return open({
    multiple: false,
    directory: false,
    filters: [{ name: "Libros", extensions: SUPPORTED_EXTENSIONS }],
  });
}

function pickInBrowser() {
  return new Promise((resolve) => {
    const input = Object.assign(document.createElement("input"), {
      type: "file",
      accept: SUPPORTED_EXTENSIONS.map((ext) => `.${ext}`).join(","),
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
