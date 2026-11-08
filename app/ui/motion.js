// Motion presets. Every animation is a spring; when the system asks for
// reduced motion, springs become short opacity fades.
import { animate } from "motion";

// Calm springs: near-critical damping, so things settle without bouncing.
export const springs = {
  snappy: { type: "spring", stiffness: 520, damping: 44 },
  gentle: { type: "spring", stiffness: 240, damping: 30 },
  soft: { type: "spring", stiffness: 300, damping: 30 },
};

const REDUCED_FADE = { duration: 0.15, ease: "easeOut" };
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

/** Animates with a spring, or a plain fade if reduced motion is on. */
export function move(element, keyframes, spring = springs.gentle) {
  if (!reducedMotion.matches) return animate(element, keyframes, spring);
  const fadeOnly = "opacity" in keyframes ? { opacity: keyframes.opacity } : {};
  return animate(element, fadeOnly, REDUCED_FADE);
}

export function show(element, display = "block") {
  element.style.display = display;
  return move(element, { opacity: [0, 1], y: [10, 0], scale: [0.985, 1] });
}

export async function hide(element) {
  await move(element, { opacity: 0, y: 8, scale: 0.98 }, springs.snappy);
  element.style.display = "none";
}

/** Horizontal shake that signals invalid input. */
export function shake(element) {
  if (reducedMotion.matches) return Promise.resolve();
  return animate(element, { x: [0, -9, 8, -5, 3, 0] }, { duration: 0.42, ease: "easeOut" });
}

/** Press feedback for buttons: a quick scale-down that springs back. */
export function pressable(button) {
  button.addEventListener("pointerdown", () => move(button, { scale: 0.97 }, springs.snappy));
  const release = () => move(button, { scale: 1 }, springs.soft);
  button.addEventListener("pointerup", release);
  button.addEventListener("pointerleave", release);
}

/** Text an element is fading towards, while a crossfade is running. */
const pendingText = new WeakMap();

/**
 * Swaps an element's text with a quick fade out and in. Calls made while a
 * fade runs only update the target, so fast-changing stages never queue up:
 * the element always lands on the latest text.
 */
export async function crossfadeText(element, text) {
  if (pendingText.has(element)) {
    pendingText.set(element, text);
    return;
  }
  if (element.textContent === text) return;
  pendingText.set(element, text);
  while (element.textContent !== pendingText.get(element)) {
    await move(element, { opacity: 0 }, springs.snappy);
    element.textContent = pendingText.get(element);
    await move(element, { opacity: 1 }, springs.soft);
  }
  pendingText.delete(element);
}
