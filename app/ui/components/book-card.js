// Card for the book that is about to be converted, listing the pipeline
// phases it will go through.
import { stagger } from "motion";
import { move, show, hide, springs } from "../motion.js";

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

export function createBookCard({ onClose }) {
  const card = document.getElementById("card");
  const phases = document.getElementById("phases");
  document.getElementById("card-close").addEventListener("click", onClose);

  return {
    async present(item, phaseNames) {
      document.getElementById("card-title").textContent = item.title;
      document.getElementById("card-subtitle").textContent = item.subtitle;
      document.getElementById("card-cover").textContent = item.badge;
      phases.replaceChildren(
        ...phaseNames.map((name) => Object.assign(document.createElement("li"), { textContent: name })),
      );
      show(card);
      move(
        phases.children,
        { opacity: [0, 1], x: [-8, 0] },
        { ...springs.gentle, delay: stagger(0.05, { startDelay: 0.12 }) },
      );
    },
    dismiss: () => hide(card),
  };
}
