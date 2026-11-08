// Progress ring drawn with two masked half-rings that only rotate, so the
// animation is pure `transform` and stays smooth on WebKitGTK. The right
// half fills 0–50 %, the left half 50–100 %.
import { move, springs } from "../motion.js";

const BASE_ROTATION = 45; // aligns each half-ring's coloured edge with 12 o'clock
const HALF_TURN = 180;
const PERCENT = 100;

export function createProgressRing(root) {
  const right = root.querySelector("[data-ring-right]");
  const left = root.querySelector("[data-ring-left]");
  const value = root.querySelector("[data-ring-value]");
  const doneIcon = root.querySelector("[data-ring-done]");
  const errorIcon = root.querySelector("[data-ring-error]");
  const arcs = root.querySelector("[data-ring-arcs]");
  let shown = -1;

  /** Crossfades the ring centre to one of: the value, a check, a mark. */
  function reveal(visible) {
    for (const element of [value, doneIcon, errorIcon]) {
      const on = element === visible;
      move(element, { opacity: on ? 1 : 0, scale: on ? 1 : 0.8 }, springs.soft);
    }
  }

  function rotations(fraction) {
    const clamped = Math.min(1, Math.max(0, fraction));
    return {
      right: BASE_ROTATION + Math.min(clamped, 0.5) * 2 * HALF_TURN,
      left: BASE_ROTATION + Math.max(clamped - 0.5, 0) * 2 * HALF_TURN,
    };
  }

  function place(fraction, spring) {
    const turn = rotations(fraction);
    move(right, { rotate: turn.right }, spring);
    move(left, { rotate: turn.left }, spring);
    const percent = Math.round(Math.min(1, Math.max(0, fraction)) * PERCENT);
    if (percent !== shown) {
      shown = percent;
      value.textContent = String(percent);
      root.setAttribute("aria-valuenow", String(percent));
    }
  }

  return {
    /** Snaps back to empty, without animation. */
    reset() {
      shown = -1;
      place(0, { duration: 0 });
      move(arcs, { opacity: 1 }, { duration: 0 });
      reveal(value);
    },
    set(fraction) {
      place(fraction, springs.gentle);
    },
    complete() {
      place(1, springs.gentle);
      reveal(doneIcon);
    },
    fail() {
      move(arcs, { opacity: 0 }, springs.soft);
      reveal(errorIcon);
    },
  };
}
