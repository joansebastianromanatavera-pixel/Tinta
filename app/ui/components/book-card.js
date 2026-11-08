// Card for the book being converted. It walks through three designed
// states: converting (real progress and current stage), done (summary and
// Guardar) and error (what happened and how to go on).
import { move, show, hide, springs, crossfadeText } from "../motion.js";
import { createProgressRing } from "./progress-ring.js";

const KIND_LABELS = {
  pdf: "Documento PDF",
  epub: "Libro EPUB",
  mobi: "Libro MOBI",
  azw3: "Libro AZW3",
  docx: "Documento de Word",
  odt: "Documento ODT",
  rtf: "Documento RTF",
  txt: "Texto",
  md: "Markdown",
  html: "Página HTML",
  htm: "Página HTML",
  fb2: "Libro FB2",
};

/** Headline of each error state, keyed by the error kind from Rust. */
const ERROR_TITLES = {
  unsupported: "Formato no disponible todavía",
  link: "Enlaces muy pronto",
  scanned: "Es un PDF escaneado",
  encrypted: "PDF protegido",
  empty: "No encontré texto",
  engine: "Falta el motor de PDF",
  unreadable: "No se pudo leer el archivo",
  io: "No se pudo guardar",
  busy: "Espera un momento",
};
const FALLBACK_ERROR_TITLE = "Algo salió mal";
/** Extra guidance shown under some errors. */
const ERROR_HINTS = {
  scanned: "El reconocimiento de texto (OCR) llegará en una versión próxima.",
  engine: "Ejecuta scripts/fetch-pdfium.sh y vuelve a abrir Tinta.",
};

const READY_LABEL = "Listo para tu Kindle";
const READY_NOTE =
  "Envíalo a tu Kindle con Send to Kindle, desde la web o la app de Amazon. Copiado por USB, el Kindle no abre EPUB.";
const SAVE_LABEL = "Guardar";
const SAVED_LABEL = "Guardado";
const STARTING_LABEL = "Preparando";

const languageNames = new Intl.DisplayNames(["es"], { type: "language" });
const authorList = new Intl.ListFormat("es", { type: "conjunction" });
const sizeFormat = new Intl.NumberFormat("es", { maximumFractionDigits: 1 });
const BYTES_PER_KB = 1024;
const BYTES_PER_MB = BYTES_PER_KB * 1024;

export function describeFile(path) {
  const name = path.split(/[\\/]/).pop();
  const dot = name.lastIndexOf(".");
  const ext = dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
  return {
    title: dot > 0 ? name.slice(0, dot) : name,
    subtitle: KIND_LABELS[ext] ?? "Archivo",
    badge: ext.toUpperCase().slice(0, 4) || "DOC",
  };
}

export function describeLink(url) {
  return { title: url.hostname.replace(/^www\./, ""), subtitle: "Enlace web", badge: "WEB" };
}

export function summarize(result) {
  const chapters = result.chapters === 1 ? "1 capítulo" : `${result.chapters} capítulos`;
  const language = result.language ? languageNames.of(result.language) : null;
  return [chapters, language, formatSize(result.bytes)].filter(Boolean).join(" · ");
}

export function formatSize(bytes) {
  if (bytes >= BYTES_PER_MB) return `${sizeFormat.format(bytes / BYTES_PER_MB)} MB`;
  return `${sizeFormat.format(Math.max(1, Math.round(bytes / BYTES_PER_KB)))} KB`;
}

export function createBookCard({ onClose, onAgain, onSave }) {
  const card = document.getElementById("card");
  const title = document.getElementById("card-title");
  const subtitle = document.getElementById("card-subtitle");
  const cover = document.getElementById("card-cover");
  const label = document.getElementById("status-label");
  const detail = document.getElementById("status-detail");
  const note = document.getElementById("card-note");
  const actions = document.getElementById("card-actions");
  const again = document.getElementById("card-again");
  const save = document.getElementById("card-save");
  const saveLabel = document.getElementById("card-save-label");
  const ring = createProgressRing(document.getElementById("ring"));

  document.getElementById("card-close").addEventListener("click", onClose);
  again.addEventListener("click", onAgain);
  save.addEventListener("click", onSave);

  function reveal(element) {
    element.hidden = false;
    move(element, { opacity: [0, 1], y: [6, 0] }, springs.gentle);
  }

  return {
    /** Converting state: the item and an empty ring. */
    present(item) {
      title.textContent = item.title;
      subtitle.textContent = item.subtitle;
      cover.textContent = item.badge;
      label.textContent = STARTING_LABEL;
      detail.textContent = "";
      note.hidden = true;
      actions.hidden = true;
      save.disabled = false;
      saveLabel.textContent = SAVE_LABEL;
      ring.reset();
      return show(card);
    },

    progress({ fraction, stage }) {
      ring.set(fraction);
      crossfadeText(label, stage);
    },

    succeed(result) {
      ring.complete();
      crossfadeText(title, result.title);
      if (result.authors.length) crossfadeText(subtitle, authorList.format(result.authors));
      crossfadeText(label, READY_LABEL);
      detail.textContent = summarize(result);
      note.textContent = READY_NOTE;
      again.hidden = true;
      save.hidden = false;
      reveal(note);
      reveal(actions);
    },

    fail(error) {
      ring.fail();
      crossfadeText(label, ERROR_TITLES[error.kind] ?? FALLBACK_ERROR_TITLE);
      detail.textContent = error.message ?? "";
      note.textContent = ERROR_HINTS[error.kind] ?? "";
      if (note.textContent) reveal(note);
      again.hidden = false;
      save.hidden = true;
      reveal(actions);
    },

    saved() {
      save.disabled = true;
      crossfadeText(saveLabel, SAVED_LABEL);
    },

    saveFailed(error) {
      crossfadeText(label, ERROR_TITLES.io);
      detail.textContent = error.message ?? "";
    },

    dismiss: () => hide(card),
  };
}
