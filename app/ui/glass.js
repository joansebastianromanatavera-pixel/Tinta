// Pointer-driven light for liquid glass surfaces.
//
// Every element marked [data-glass] gets a specular highlight that follows
// the pointer, a rim lit from the pointer's direction and — for panels — a
// subtle 3D tilt. All motion runs through critically damped springs in one
// requestAnimationFrame loop that sleeps when everything has settled.

const TILT_MAX_DEG = 4;
const PERSPECTIVE_PX = 1100;
const HOVER_MARGIN_PX = 120;
const LIGHT_IDLE = 0.35;
const LIGHT_ACTIVE = 1;
const SETTLE_EPSILON = 0.01;
const MAX_FRAME_S = 1 / 30;

const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

class Spring {
  constructor(value = 0, stiffness = 170, damping = 26) {
    this.value = value;
    this.target = value;
    this.velocity = 0;
    this.stiffness = stiffness;
    this.damping = damping;
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
    this.tilts = element.dataset.glass === "panel";
    this.light = document.createElement("span");
    this.light.className = "glass__light";
    this.light.setAttribute("aria-hidden", "true");
    element.prepend(this.light);
    this.x = new Spring(0, 220, 28);
    this.y = new Spring(0, 220, 28);
    this.intensity = new Spring(0, 160, 24);
    this.tiltX = new Spring(0, 120, 18);
    this.tiltY = new Spring(0, 120, 18);
    this.springs = [this.x, this.y, this.intensity, this.tiltX, this.tiltY];
    // Children marked [data-depth] float above the glass and shift with the
    // tilt, so the content reads as layered inside a thick slab.
    this.layers = this.tilts ? [...element.querySelectorAll("[data-depth]")] : [];

  }

  aim(pointerX, pointerY) {
    const rect = this.element.getBoundingClientRect();
    if (rect.width === 0) return;
    const localX = pointerX - rect.left;
    const localY = pointerY - rect.top;
    const near =
      localX > -HOVER_MARGIN_PX &&
      localY > -HOVER_MARGIN_PX &&
      localX < rect.width + HOVER_MARGIN_PX &&
      localY < rect.height + HOVER_MARGIN_PX;
    const inside = localX >= 0 && localY >= 0 && localX <= rect.width && localY <= rect.height;

    this.x.target = clamp(localX, -HOVER_MARGIN_PX, rect.width + HOVER_MARGIN_PX);
    this.y.target = clamp(localY, -HOVER_MARGIN_PX, rect.height + HOVER_MARGIN_PX);
    this.intensity.target = near ? LIGHT_ACTIVE : LIGHT_IDLE;

    const normX = (localX / rect.width) * 2 - 1;
    const normY = (localY / rect.height) * 2 - 1;
    this.tiltY.target = this.tilts && inside ? normX * TILT_MAX_DEG : 0;
    this.tiltX.target = this.tilts && inside ? -normY * TILT_MAX_DEG : 0;

    const angle = Math.atan2(localX - rect.width / 2, -(localY - rect.height / 2));
    this.element.style.setProperty("--light-angle", `${angle}rad`);
  }

  rest() {
    this.intensity.target = LIGHT_IDLE;
    this.tiltX.target = 0;
    this.tiltY.target = 0;
  }

  /** Advances the springs; returns true while still moving. */
  step(dt) {
    this.springs.forEach((spring) => spring.step(dt));
    this.render();
    return !this.springs.every((spring) => spring.settled);
  }

  render() {
    this.light.style.transform = `translate3d(${this.x.value}px, ${this.y.value}px, 0)`;
    this.light.style.opacity = String(clamp(this.intensity.value, 0, 1));
    if (!this.tilts) return;
    this.element.style.transform =
      `perspective(${PERSPECTIVE_PX}px) rotateX(${this.tiltX.value}deg) rotateY(${this.tiltY.value}deg)`;
    const shiftX = this.tiltY.value / TILT_MAX_DEG;
    const shiftY = -this.tiltX.value / TILT_MAX_DEG;
    for (const layer of this.layers) {
      const depth = Number(layer.dataset.depth);
      layer.style.transform = `translate3d(${shiftX * depth}px, ${shiftY * depth}px, 0)`;
    }
  }
}

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
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
      if (reducedMotion.matches) return;
      surfaces.forEach((surface) => surface.aim(event.clientX, event.clientY));
      wake();
    },
    { passive: true },
  );

  document.documentElement.addEventListener("pointerleave", () => {
    surfaces.forEach((surface) => surface.rest());
    wake();
  });

  new MutationObserver(() => root.querySelectorAll("[data-glass]").forEach(adopt)).observe(
    document.body,
    { childList: true, subtree: true },
  );

  surfaces.forEach((surface) => surface.render());
}

/** Gentle parallax of the ambient scene against the pointer. */
export function initAmbientParallax(scene, depthPx = 18) {
  if (!scene || reducedMotion.matches) return;
  const x = new Spring(0, 40, 14);
  const y = new Spring(0, 40, 14);
  let frame = 0;
  let lastTime = 0;

  const tick = (time) => {
    const dt = Math.min((time - lastTime) / 1000, MAX_FRAME_S);
    lastTime = time;
    x.step(dt);
    y.step(dt);
    scene.style.transform = `translate3d(${x.value}px, ${y.value}px, 0)`;
    frame = x.settled && y.settled ? 0 : requestAnimationFrame(tick);
  };

  window.addEventListener(
    "pointermove",
    (event) => {
      x.target = -((event.clientX / window.innerWidth) * 2 - 1) * depthPx;
      y.target = -((event.clientY / window.innerHeight) * 2 - 1) * depthPx;
      if (!frame) {
        lastTime = performance.now();
        frame = requestAnimationFrame(tick);
      }
    },
    { passive: true },
  );
}

const GLOW_VISIBLE = 1;
const AMBIENT_OVERSCAN = 0.1;

/** A light behind the glass that follows the pointer, so every glass
 *  surface glows where the user is looking. */
export function initPointerGlow(glow) {
  if (!glow || reducedMotion.matches) return;
  const x = new Spring(window.innerWidth / 2, 60, 16);
  const y = new Spring(window.innerHeight / 2, 60, 16);
  const strength = new Spring(0, 40, 14);
  runWhileMoving([x, y, strength], () => {
    const offsetX = window.innerWidth * AMBIENT_OVERSCAN;
    const offsetY = window.innerHeight * AMBIENT_OVERSCAN;
    glow.style.transform = `translate3d(${x.value + offsetX}px, ${y.value + offsetY}px, 0)`;
    glow.style.opacity = String(clamp(strength.value, 0, 1));
  }, (wake) => {
    window.addEventListener("pointermove", (event) => {
      x.target = event.clientX;
      y.target = event.clientY;
      strength.target = GLOW_VISIBLE;
      wake();
    }, { passive: true });
    document.documentElement.addEventListener("pointerleave", () => {
      strength.target = 0;
      wake();
    });
  });
}

const DROPLET_PARALLAX_PX = 26;
const DROPLET_REPEL_RADIUS_PX = 240;
const DROPLET_REPEL_PX = 46;

/** Glass droplets drift against the pointer (parallax) and flee from it
 *  when it comes close, like drops on a surface. */
export function initDroplets(anchors) {
  if (reducedMotion.matches) return;
  const droplets = [...anchors].map((anchor) => ({
    anchor,
    depth: Number(anchor.dataset.droplet),
    x: new Spring(0, 70, 11),
    y: new Spring(0, 70, 11),
  }));
  const springs = droplets.flatMap((d) => [d.x, d.y]);

  runWhileMoving(springs, () => {
    for (const d of droplets) {
      d.anchor.style.transform = `translate3d(${d.x.value}px, ${d.y.value}px, 0)`;
    }
  }, (wake) => {
    window.addEventListener("pointermove", (event) => {
      const normX = (event.clientX / window.innerWidth) * 2 - 1;
      const normY = (event.clientY / window.innerHeight) * 2 - 1;
      for (const d of droplets) {
        const rect = d.anchor.getBoundingClientRect();
        const centerX = rect.left + rect.width / 2 - d.x.value;
        const centerY = rect.top + rect.height / 2 - d.y.value;
        const dx = centerX - event.clientX;
        const dy = centerY - event.clientY;
        const distance = Math.hypot(dx, dy) || 1;
        const push = Math.max(0, 1 - distance / DROPLET_REPEL_RADIUS_PX) * DROPLET_REPEL_PX;
        d.x.target = -normX * DROPLET_PARALLAX_PX * d.depth + (dx / distance) * push * d.depth;
        d.y.target = -normY * DROPLET_PARALLAX_PX * d.depth + (dy / distance) * push * d.depth;
      }
      wake();
    }, { passive: true });
  });
}

/** Runs `render` every frame while any spring moves; sleeps otherwise.
 *  `subscribe(wake)` attaches the listeners that change spring targets. */
function runWhileMoving(springs, render, subscribe) {
  let frame = 0;
  let lastTime = 0;
  const tick = (time) => {
    const dt = Math.min((time - lastTime) / 1000, MAX_FRAME_S);
    lastTime = time;
    springs.forEach((spring) => spring.step(dt));
    render();
    frame = springs.every((spring) => spring.settled) ? 0 : requestAnimationFrame(tick);
  };
  subscribe(() => {
    if (frame) return;
    lastTime = performance.now();
    frame = requestAnimationFrame(tick);
  });
  render();
}
