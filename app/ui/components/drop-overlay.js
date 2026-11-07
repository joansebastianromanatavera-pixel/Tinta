// Full-window drop target. The ring "breathes" while a file hovers.
import { animate } from "motion";
import { move, springs } from "../motion.js";

const BREATH = { duration: 2.4, repeat: Infinity, ease: "easeInOut" };

export function createDropOverlay() {
  const overlay = document.getElementById("drop");
  const ring = document.getElementById("drop-ring");
  let breathing = null;

  return {
    enter() {
      move(overlay, { opacity: 1 }, springs.snappy);
      move(ring, { scale: [0.85, 1] }, springs.soft);
      if (!window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
        breathing = animate(ring, { scale: [1, 1.04, 1] }, BREATH);
      }
    },
    async leave({ absorbed = false } = {}) {
      breathing?.stop();
      breathing = null;
      if (absorbed) await move(ring, { scale: 0.6, opacity: 0 }, springs.snappy);
      await move(overlay, { opacity: 0 }, springs.snappy);
      ring.style.opacity = "";
      ring.style.transform = "";
    },
  };
}
