// Living sand: the background is a field of fine grains, rendered by a
// WebGL shader at one grain per physical pixel.
//
// The pointer is a lamp held just above the surface. Grains near it catch
// the light — each grain has its own tilt, so some glint and some stay in
// shadow, like the texture of paper under a reading lamp. Moving the
// pointer brushes the sand: grains along the path are nudged aside and
// settle back as the trail fades.
//
// It renders only while something changes, then sleeps.

const MAX_TRAIL = 32;
const TRAIL_LIFE_S = 1.4;
const TRAIL_SPACING_PX = 10;
const LIGHT_STIFFNESS = 60;
const MAX_FRAME_S = 1 / 30;
const SETTLE_EPSILON = 0.002;

const VERTEX_SHADER = `
attribute vec2 aPosition;
void main() {
  gl_Position = vec4(aPosition, 0.0, 1.0);
}`;

const FRAGMENT_SHADER = `
precision highp float;

uniform vec2 uResolution;
uniform vec2 uLight;
uniform float uLightStrength;
uniform vec3 uTrail[${MAX_TRAIL}];
uniform float uScale;
uniform float uSeed;

const float BRUSH_RADIUS = 34.0;
const float BRUSH_PUSH = 3.0;
const float LAMP_HEIGHT = 150.0;
const float LAMP_REACH = 280.0;

float hash(vec2 p) {
  p = fract(p * vec2(123.34, 456.21) + uSeed);
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}

// Studio light: a soft white bloom from the top right, a faint fill low left.
float sceneLight(vec2 frag) {
  float size = max(uResolution.x, uResolution.y);
  vec2 key = vec2(uResolution.x * 0.86, uResolution.y * 1.12);
  float k = 1.0 - smoothstep(0.0, size * 0.9, length(frag - key));
  vec2 fill = vec2(uResolution.x * -0.08, uResolution.y * -0.18);
  float f = 1.0 - smoothstep(0.0, size * 0.6, length(frag - fill));
  return 0.025 + 0.6 * pow(k, 1.7) + 0.06 * f * f;
}

void main() {
  vec2 frag = gl_FragCoord.xy;

  // The brush: grains along the pointer's path are pushed outward.
  vec2 push = vec2(0.0);
  float stir = 0.0;
  float radius = BRUSH_RADIUS * uScale;
  for (int i = 0; i < ${MAX_TRAIL}; i++) {
    vec3 t = uTrail[i];
    if (t.z <= 0.0) continue;
    vec2 d = frag - t.xy;
    float w = exp(-dot(d, d) / (radius * radius)) * t.z;
    push += normalize(d + 0.0001) * w;
    stir += w;
  }
  vec2 cell = floor(frag + push * BRUSH_PUSH * uScale);

  // Each grain: a height and a tilt (normal) from its neighbours.
  float h = hash(cell);
  vec3 normal = normalize(vec3(
    hash(cell - vec2(1.0, 0.0)) - hash(cell + vec2(1.0, 0.0)),
    hash(cell - vec2(0.0, 1.0)) - hash(cell + vec2(0.0, 1.0)),
    0.85));

  float base = sceneLight(frag);
  float grain = (h - 0.5) * (0.07 + base * 0.12);

  // The lamp: raking light that reveals the grain, with the odd glint.
  vec3 toLamp = vec3(uLight - frag, LAMP_HEIGHT * uScale);
  vec3 l = normalize(toLamp);
  float dist = length(uLight - frag) / uScale;
  float reach = exp(-dist * dist / (LAMP_REACH * LAMP_REACH)) * uLightStrength;
  float diffuse = max(dot(normal, l), 0.0);
  float glint = pow(max(dot(reflect(-l, normal), vec3(0.0, 0.0, 1.0)), 0.0), 48.0);
  float lamp = reach * (0.05 + 0.09 * diffuse) + reach * glint * 0.28;

  float settle = stir * 0.05 * (h - 0.35);
  float luminance = base + grain + lamp + settle;
  gl_FragColor = vec4(vec3(clamp(luminance, 0.0, 1.0)), 1.0);
}`;

function compile(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    throw new Error(gl.getShaderInfoLog(shader) ?? "shader compile failed");
  }
  return shader;
}

function createProgram(gl) {
  const program = gl.createProgram();
  gl.attachShader(program, compile(gl, gl.VERTEX_SHADER, VERTEX_SHADER));
  gl.attachShader(program, compile(gl, gl.FRAGMENT_SHADER, FRAGMENT_SHADER));
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(program) ?? "program link failed");
  }
  return program;
}

/**
 * Starts the sand on `canvas`. Returns false if WebGL is unavailable, so
 * the caller can fall back to the static CSS scene.
 */
export function initSand(canvas) {
  const gl = canvas?.getContext("webgl", { antialias: false, alpha: false });
  if (!gl) return false;

  let program;
  try {
    program = createProgram(gl);
  } catch (error) {
    console.error("Knddle sand:", error);
    return false;
  }
  gl.useProgram(program);

  const quad = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, quad);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
  const position = gl.getAttribLocation(program, "aPosition");
  gl.enableVertexAttribArray(position);
  gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);

  const uniform = (name) => gl.getUniformLocation(program, name);
  const u = {
    resolution: uniform("uResolution"),
    light: uniform("uLight"),
    lightStrength: uniform("uLightStrength"),
    trail: uniform("uTrail"),
    scale: uniform("uScale"),
    seed: uniform("uSeed"),
  };
  gl.uniform1f(u.seed, Math.random() * 100);

  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  const trail = new Float32Array(MAX_TRAIL * 3);
  let trailHead = 0;
  let scale = 1;
  let lightX = 0;
  let lightY = 0;
  let strength = 0;
  let strengthTarget = 0;
  let strengthVelocity = 0;
  let lastPoint = null;
  let frame = 0;
  let lastTime = 0;

  const resize = () => {
    scale = window.devicePixelRatio || 1;
    canvas.width = Math.round(window.innerWidth * scale);
    canvas.height = Math.round(window.innerHeight * scale);
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.uniform2f(u.resolution, canvas.width, canvas.height);
    gl.uniform1f(u.scale, scale);
    draw();
  };

  const draw = () => {
    gl.uniform2f(u.light, lightX, lightY);
    gl.uniform1f(u.lightStrength, strength);
    gl.uniform3fv(u.trail, trail);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  };

  const addTrailPoint = (x, y) => {
    trail[trailHead * 3] = x;
    trail[trailHead * 3 + 1] = y;
    trail[trailHead * 3 + 2] = 1;
    trailHead = (trailHead + 1) % MAX_TRAIL;
  };

  /** Fades the trail and eases the lamp; returns true while anything moves. */
  const advance = (dt) => {
    let alive = false;
    for (let i = 2; i < trail.length; i += 3) {
      if (trail[i] <= 0) continue;
      trail[i] = Math.max(0, trail[i] - dt / TRAIL_LIFE_S);
      alive ||= trail[i] > 0;
    }
    const damping = 2 * Math.sqrt(LIGHT_STIFFNESS);
    strengthVelocity += (-LIGHT_STIFFNESS * (strength - strengthTarget) - damping * strengthVelocity) * dt;
    strength += strengthVelocity * dt;
    const easing = Math.abs(strength - strengthTarget) > SETTLE_EPSILON;
    return alive || easing;
  };

  const tick = (time) => {
    const dt = Math.min((time - lastTime) / 1000, MAX_FRAME_S);
    lastTime = time;
    const moving = advance(dt);
    draw();
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
      lightX = event.clientX * scale;
      lightY = (window.innerHeight - event.clientY) * scale;
      strengthTarget = 1;
      if (!reducedMotion.matches) brushTo(lightX, lightY);
      wake();
    },
    { passive: true },
  );

  // Interpolates along fast strokes so the brushed path stays continuous.
  const brushTo = (x, y) => {
    if (!lastPoint) {
      lastPoint = { x, y };
      addTrailPoint(x, y);
      return;
    }
    const dx = x - lastPoint.x;
    const dy = y - lastPoint.y;
    const steps = Math.floor(Math.hypot(dx, dy) / (TRAIL_SPACING_PX * scale));
    for (let i = 1; i <= Math.min(steps, MAX_TRAIL); i += 1) {
      addTrailPoint(lastPoint.x + (dx * i) / steps, lastPoint.y + (dy * i) / steps);
    }
    if (steps > 0) lastPoint = { x, y };
  };

  document.documentElement.addEventListener("pointerleave", () => {
    strengthTarget = 0;
    lastPoint = null;
    wake();
  });

  window.addEventListener("resize", resize);
  resize();
  return true;
}
