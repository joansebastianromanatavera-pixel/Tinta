// Film grain, painted once into a canvas tile: every pixel a random grey
// with a little alpha, like fine black-and-white sand. Greys follow a
// bell curve around mid-grey, so most grains are subtle and a few are
// bright or dark — the way real grain looks. Painted at the device pixel
// ratio so each grain is one physical pixel and stays crisp.

const TILE_CSS_PX = 256;
const GRAIN_ALPHA = 36;
const MID_GREY = 128;
const MAX_CHANNEL = 255;
const BELL_SAMPLES = 3;

/** Approximately normal value in -1..1 (average of uniform samples). */
function bell() {
  let sum = 0;
  for (let i = 0; i < BELL_SAMPLES; i += 1) sum += Math.random();
  return (sum / BELL_SAMPLES) * 2 - 1;
}

export function paintGrain(element) {
  if (!element) return;
  const ratio = window.devicePixelRatio || 1;
  const size = Math.round(TILE_CSS_PX * ratio);
  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const context = canvas.getContext("2d");
  const image = context.createImageData(size, size);
  const pixels = image.data;

  for (let i = 0; i < pixels.length; i += 4) {
    const grey = Math.round(MID_GREY + bell() * MID_GREY);
    pixels[i] = Math.min(grey, MAX_CHANNEL);
    pixels[i + 1] = pixels[i];
    pixels[i + 2] = pixels[i];
    pixels[i + 3] = GRAIN_ALPHA;
  }

  context.putImageData(image, 0, 0);
  element.style.backgroundImage = `url(${canvas.toDataURL("image/png")})`;
  element.style.backgroundSize = `${TILE_CSS_PX}px ${TILE_CSS_PX}px`;
}
