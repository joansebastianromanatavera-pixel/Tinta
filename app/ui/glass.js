// The pointer is a light; glass surfaces reflect it.
//
// Every [data-glass] element gets a soft reflection that glides toward the
// pointer and a rim lit from the pointer's direction. Nothing tilts, scales
// or moves: only light changes. Springs are critically damped (no
// overshoot) and run in one requestAnimationFrame loop that sleeps when
// everything has settled.

const LIGHT_REACH_PX = 360;
const LIGHT_AMBIENT = 0.2;
const SETTLE_EPSILON = 0.005;
const MAX_FRAME_S = 1 / 30;
const POSITION_STIFFNESS = 150;
const INTENSITY_STIFFNESS = 90;

const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

/** Critically damped spring: reaches its target smoothly, never overshoots. */
class Spring {
  constructor(value, stiffness) {
    this.value = value;
    this.target = value;
    this.velocity = 0;
    this.stiffness = stiffness;
    this.damping = 2 * Math.sqrt(stiffness);
  }

  step(dt) {
    const force = -this.stiffness * (this.value - this.target) - this.damping * this.velocity;
    this.velocity += force * dt;
    this.value += this.velocity * dt;
  }

  get settled() {
    return (
      Math.abs(this.value - this.target) < SETTLE_EPSILON &&
      Math.abs(this.velocity) < SETTLE_EPSILON
    );
  }
}

class GlassSurface {
  constructor(element) {
    this.element = element;
    this.light = document.createElement("span");
    this.light.className = "glass__light";
    this.light.setAttribute("aria-hidden", "true");
    element.prepend(this.light);
    this.x = new Spring(0, POSITION_STIFFNESS);
    this.y = new Spring(0, POSITION_STIFFNESS);
    this.intensity = new Spring(0, INTENSITY_STIFFNESS);
    this.springs = [this.x, this.y, this.intensity];
  }

  /** Points the reflection at the light. */
  aim(lightX, lightY) {
    const rect = this.element.getBoundingClientRect();
    if (rect.width === 0) return;
    const localX = lightX - rect.left;
    const localY = lightY - rect.top;

    this.x.target = localX;
    this.y.target = localY;

    const outsideX = Math.max(0, -localX, localX - rect.width);
    const outsideY = Math.max(0, -localY, localY - rect.height);
    const falloff = Math.max(0, 1 - Math.hypot(outsideX, outsideY) / LIGHT_REACH_PX);
    this.intensity.target = falloff * falloff;

    const angle = Math.atan2(localX - rect.width / 2, -(localY - rect.height / 2));
    this.element.style.setProperty("--light-angle", `${angle}rad`);
  }

  dim() {
    this.intensity.target = 0;
  }

  /** Advances the springs; returns true while still moving. */
  step(dt) {
    this.springs.forEach((spring) => spring.step(dt));
    this.render();
    return !this.springs.every((spring) => spring.settled);
  }

  render() {
    const intensity = LIGHT_AMBIENT + (1 - LIGHT_AMBIENT) * this.intensity.value;
    this.light.style.transform = `translate3d(${this.x.value}px, ${this.y.value}px, 0)`;
    this.light.style.opacity = String(Math.min(Math.max(this.intensity.value, 0), 1));
    this.element.style.setProperty("--rim-strength", intensity.toFixed(3));
  }
}

/** Wires every [data-glass] surface (present and future) to the pointer. */
export function initGlass(root = document) {
  const surfaces = new Map();
  let frame = 0;
  let lastTime = 0;

  const adopt = (element) => {
    if (!surfaces.has(element)) surfaces.set(element, new GlassSurface(element));
  };
  root.querySelectorAll("[data-glass]").forEach(adopt);

  const tick = (time) => {
    const dt = Math.min((time - lastTime) / 1000, MAX_FRAME_S);
    lastTime = time;
    let moving = false;
    surfaces.forEach((surface) => {
      moving = surface.step(dt) || moving;
    });
    frame = moving ? requestAnimationFrame(tick) : 0;
  };

  const wake = () => {
    if (frame) return;
    lastTime = performance.now();
    frame = requestAnimationFrame(tick);
  };

  window.addEventListener(
    "pointermove",
    (event) => {
      surfaces.forEach((surface) => surface.aim(event.clientX, event.clientY));
      if (reducedMotion.matches) {
        surfaces.forEach((surface) => {
          surface.springs.forEach((spring) => (spring.value = spring.target));
          surface.render();
        });
        return;
      }
      wake();
    },
    { passive: true },
  );

  document.documentElement.addEventListener("pointerleave", () => {
    surfaces.forEach((surface) => surface.dim());
    wake();
  });

  new MutationObserver(() => root.querySelectorAll("[data-glass]").forEach(adopt)).observe(
    document.body,
    { childList: true, subtree: true },
  );

  surfaces.forEach((surface) => surface.render());
}
