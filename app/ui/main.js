import "@fontsource-variable/inter";
import "@fontsource-variable/literata";
import { show, hide } from "./motion.js";
import { initGlass } from "./glass.js";
import { paintGrain } from "./grain.js";
import { initSand } from "./sand.js";
import { convertBook, saveBook, onFileDrop } from "./api.js";
import { createActions } from "./components/actions.js";
import { createDropOverlay } from "./components/drop-overlay.js";
import { createBookCard, describeFile, describeLink } from "./components/book-card.js";

/** The ring needs this long to sweep calmly, even for an instant conversion. */
const MIN_CONVERSION_MS = 900;
const LINK_ERROR = {
  kind: "link",
  message: "Por ahora Tinta convierte archivos PDF. Los libros desde enlaces web llegarán pronto.",
};

const hero = document.getElementById("hero");
/** Guards the hero ↔ card transitions so they never overlap. */
let transitioning = false;
/** Identifies the conversion on screen; late results of older ones are ignored. */
let session = 0;
let converted = null;

const card = createBookCard({
  onClose: backToStart,
  onAgain: backToStart,
  onSave: saveConverted,
});
const actions = createActions({
  onFile: (path) => start(describeFile(path), (report) => convertBook(path, report)),
  onLink: (url) => start(describeLink(url), () => Promise.reject(LINK_ERROR)),
});
const drop = createDropOverlay();

onFileDrop({
  enter: drop.enter,
  leave: () => drop.leave(),
  drop: async (paths) => {
    await drop.leave({ absorbed: true });
    if (paths.length) start(describeFile(paths[0]), (report) => convertBook(paths[0], report));
  },
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && hero.style.display === "none") backToStart();
});

/** Shows the card for `item` and runs `task`, which reports progress. */
async function start(item, task) {
  if (transitioning || hero.style.display === "none") return;
  transitioning = true;
  const current = ++session;
  converted = null;
  await hide(hero);
  card.present(item);
  transitioning = false;

  const report = (event) => {
    if (current === session) card.progress(event);
  };
  const [outcome] = await Promise.allSettled([task(report), wait(MIN_CONVERSION_MS)]);
  if (current !== session) return;
  if (outcome.status === "fulfilled") {
    converted = outcome.value;
    card.succeed(converted);
  } else {
    card.fail(outcome.reason ?? {});
  }
}

async function saveConverted() {
  if (!converted) return;
  try {
    if (await saveBook(converted.fileName)) card.saved();
  } catch (error) {
    card.saveFailed(error ?? {});
  }
}

async function backToStart() {
  if (transitioning || hero.style.display !== "none") return;
  transitioning = true;
  session += 1;
  converted = null;
  await card.dismiss();
  await actions.reset();
  await show(hero, "flex");
  transitioning = false;
}

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

startScene();
initGlass();
show(hero, "flex");

/** Living sand when WebGL is available; a static grain scene otherwise. */
function startScene() {
  const ambient = document.getElementById("ambient");
  if (initSand(document.getElementById("sand"))) {
    ambient.classList.add("ambient--live");
  } else {
    paintGrain(document.getElementById("grain"));
  }
}
