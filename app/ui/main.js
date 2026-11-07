import "@fontsource-variable/inter";
import "@fontsource-variable/literata";
import { show, hide } from "./motion.js";
import { initGlass, initAmbientParallax } from "./glass.js";
import { pipelinePhases, onFileDrop } from "./api.js";
import { createActions } from "./components/actions.js";
import { createDropOverlay } from "./components/drop-overlay.js";
import { createBookCard, describeFile, describeLink } from "./components/book-card.js";

const hero = document.getElementById("hero");
let busy = false;

const card = createBookCard({ onClose: backToStart });
const actions = createActions({
  onFile: (path) => presentItem(describeFile(path)),
  onLink: (url) => presentItem(describeLink(url)),
});
const drop = createDropOverlay();

onFileDrop({
  enter: drop.enter,
  leave: () => drop.leave(),
  drop: async (paths) => {
    await drop.leave({ absorbed: true });
    if (paths.length) presentItem(describeFile(paths[0]));
  },
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && hero.style.display === "none") backToStart();
});

async function presentItem(item) {
  if (busy) return;
  busy = true;
  const [phases] = await Promise.all([pipelinePhases(), hide(hero)]);
  await card.present(item, phases);
  busy = false;
}

async function backToStart() {
  if (busy) return;
  busy = true;
  await card.dismiss();
  await actions.reset();
  initGlass();
initAmbientParallax(document.getElementById("ambient"));
show(hero, "flex");
  busy = false;
}

initGlass();
initAmbientParallax(document.getElementById("ambient"));
show(hero, "flex");
