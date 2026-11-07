// The two primary actions: Insertar (file) and Enlace (link). The Enlace
// button morphs into a text field in place.
import { move, springs, shake, pressable } from "../motion.js";
import { pickBookFile } from "../api.js";

const ALLOWED_PROTOCOLS = ["http:", "https:"];

export function parseBookUrl(raw) {
  const value = raw.trim();
  if (!value) return null;
  const candidate = /^[a-z]+:\/\//i.test(value) ? value : `https://${value}`;
  try {
    const url = new URL(candidate);
    const hasDomain = url.hostname.includes(".");
    return ALLOWED_PROTOCOLS.includes(url.protocol) && hasDomain ? url : null;
  } catch {
    return null;
  }
}

export function createActions({ onFile, onLink }) {
  const insert = document.getElementById("insert");
  const link = document.getElementById("link");
  const field = document.getElementById("link-field");
  const input = document.getElementById("link-input");
  const buttons = [insert, link];
  let fieldOpen = false;

  buttons.forEach(pressable);

  insert.addEventListener("click", async () => {
    const path = await pickBookFile();
    if (path) onFile(path);
  });

  link.addEventListener("click", openField);

  field.addEventListener("submit", (event) => {
    event.preventDefault();
    const url = parseBookUrl(input.value);
    if (!url) {
      shake(field);
      input.focus();
      return;
    }
    onLink(url);
  });

  input.addEventListener("keydown", (event) => {
    if (event.key === "Escape") closeField();
  });

  input.addEventListener("blur", () => {
    if (!input.value.trim()) closeField();
  });

  async function openField() {
    if (fieldOpen) return;
    fieldOpen = true;
    buttons.forEach((b) => move(b, { opacity: 0, scale: 0.9 }, springs.snappy));
    buttons.forEach((b) => (b.style.pointerEvents = "none"));
    field.style.display = "flex";
    move(field, { opacity: [0, 1], scaleX: [0.6, 1], scaleY: [0.9, 1] }, springs.bouncy);
    input.focus();
  }

  async function closeField() {
    if (!fieldOpen) return;
    fieldOpen = false;
    input.value = "";
    await move(field, { opacity: 0, scaleX: 0.7, scaleY: 0.92 }, springs.snappy);
    field.style.display = "none";
    buttons.forEach((b) => {
      b.style.pointerEvents = "";
      move(b, { opacity: 1, scale: 1 }, springs.bouncy);
    });
  }

  return { reset: closeField };
}
