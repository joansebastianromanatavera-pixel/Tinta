// Motion presets. Every animation is a spring; when the system asks for
// reduced motion, springs become short opacity fades.
import { animate } from "motion";

export const springs = {
  snappy: { type: "spring", stiffness: 520, damping: 38 },
  gentle: { type: "spring", stiffness: 260, damping: 30 },
  bouncy: { type: "spring", stiffness: 340, damping: 20 },
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
  return move(element, { opacity: [0, 1], y: [14, 0], scale: [0.97, 1] });
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
  button.addEventListener("pointerdown", () => move(button, { scale: 0.96 }, springs.snappy));
  const release = () => move(button, { scale: 1 }, springs.bouncy);
  button.addEventListener("pointerup", release);
  button.addEventListener("pointerleave", release);
}
