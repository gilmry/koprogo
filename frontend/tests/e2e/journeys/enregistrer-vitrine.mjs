// enregistrer-vitrine.mjs — harnais n°2 : la preuve de valeur.
//
// ── Pourquoi un harnais SÉPARÉ ────────────────────────────────────────────
//
// `skills/documentation-vivante.md`, élément 3 :
//
//   « La preuve de valeur n'est PAS un test : elle ne doit pas faire tomber le
//     build. C'est un harnais distinct (sa propre config, son propre runner)
//     qui rejoue le parcours, capture les étapes et la vidéo, et publie une
//     galerie HTML en artefact CI (non bloquant). La mélanger au gate E2E […]
//     c'est lui confier la responsabilité qu'on ne peut pas lui confier (le
//     verdict) — et ça finira coupé du pipeline. »
//
// C'est exactement ce que `.claude/scripts/slow-down-tests.sh` faisait :
// modifier les fichiers du gate pour insérer les pauses, puis les restaurer.
// Ce script-ci n'a aucun effet sur le gate. Il rejoue le MÊME parcours, en
// cadence, et rend une galerie.
//
// Sortie : vitrine/videos/<slug>.webm + <slug>.json (les chapitres).
import { chromium } from "playwright";
import {
  mkdirSync,
  existsSync,
  readdirSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { resolve, dirname } from "node:path";
import { pathToFileURL } from "node:url";

const ICI = dirname(new URL(import.meta.url).pathname);
const DOSSIER_VIDEOS = resolve(ICI, "vitrine/videos");
const BASE = process.env.PLAYWRIGHT_BASE_URL ?? "http://localhost";

// Le parcours et la scène sont en TypeScript : on passe par le transpileur de
// Vite plutôt que d'en tenir une copie JavaScript. Une copie dériverait, et
// c'est précisément ce que l'élément 1 du skill interdit.
const { createServer } = await import("vite");
const vite = await createServer({
  server: { middlewareMode: true },
  appType: "custom",
  logLevel: "error",
});

async function charger(relatif) {
  return vite.ssrLoadModule(pathToFileURL(resolve(ICI, relatif)).pathname);
}

const { Scene } = await charger("scene.ts");
const { perimetreMultiRole } = await charger("perimetre-multi-role.journey.ts");

const parcours = perimetreMultiRole;

if (!existsSync(DOSSIER_VIDEOS)) mkdirSync(DOSSIER_VIDEOS, { recursive: true });

const navigateur = await chromium.launch({
  args: ["--no-sandbox", "--disable-dev-shm-usage"],
});
const contexte = await navigateur.newContext({
  baseURL: BASE,
  viewport: { width: 1280, height: 720 },
  recordVideo: { dir: DOSSIER_VIDEOS, size: { width: 1280, height: 720 } },
  locale: "fr-BE",
});
const page = await contexte.newPage();
const scene = new Scene(page);

let echec = null;
try {
  await scene.raconter(parcours.propos);
  for (const etape of parcours.etapes) {
    await scene.raconter(etape.description, etape.acteur);
    await etape.action(scene);
    // Pas d'assertion ici : ce harnais ne rend PAS de verdict. Si le parcours
    // casse, c'est le gate E2E qui le dit — lui seul en a la responsabilité.
  }
} catch (e) {
  // On enregistre quand même ce qui a été filmé : une vitrine partielle
  // montre où ça s'arrête, ce qu'un échec silencieux ne montrerait pas.
  echec = e;
  console.error(`  ⚠️  parcours interrompu : ${e.message}`);
}

await contexte.close(); // déclenche l'écriture de la vidéo
await navigateur.close();
await vite.close();

// Playwright nomme les vidéos aléatoirement : on fixe un slug stable pour que
// la galerie et les chapitres se retrouvent.
const brutes = readdirSync(DOSSIER_VIDEOS).filter((f) => f.endsWith(".webm"));
const derniere = brutes
  .map((f) => resolve(DOSSIER_VIDEOS, f))
  .filter((f) => !f.endsWith(`${parcours.slug}.webm`))
  .sort()
  .pop();
const cible = resolve(DOSSIER_VIDEOS, `${parcours.slug}.webm`);
if (derniere) renameSync(derniere, cible);

writeFileSync(
  resolve(DOSSIER_VIDEOS, `${parcours.slug}.json`),
  JSON.stringify(
    {
      slug: parcours.slug,
      title: parcours.titre,
      propos: parcours.propos,
      acteurs: [...new Set(parcours.etapes.map((e) => e.acteur))],
      interrompu: echec ? String(echec.message) : null,
      narration: scene.narration,
    },
    null,
    2,
  ),
);

console.log(
  `  🎬 ${cible} — ${scene.narration.length} chapitres` +
    (echec ? " (parcours interrompu, vitrine partielle)" : ""),
);
