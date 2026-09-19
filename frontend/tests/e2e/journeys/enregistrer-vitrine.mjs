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

// Chaque parcours de référence du dépôt, un par persona filmée. Ajouter une
// entrée ici suffit : la boucle plus bas et `assembler-vitrine.mjs` (qui lit
// tout `*.json` du dossier vidéos) prennent le reste en charge — aucun autre
// fichier à toucher pour qu'une nouvelle vitrine rejoigne la galerie.
const PARCOURS_A_FILMER = [
  { fichier: "perimetre-multi-role.journey.ts", export: "perimetreMultiRole" },
  { fichier: "coproprietaire.journey.ts", export: "coproprietaire" },
  { fichier: "comptable.journey.ts", export: "comptable" },
  { fichier: "administration.journey.ts", export: "administration" },
  { fichier: "conseil.journey.ts", export: "conseil" },
  { fichier: "prestataire.journey.ts", export: "prestataire" },
  { fichier: "moderation.journey.ts", export: "moderation" },
];

if (!existsSync(DOSSIER_VIDEOS)) mkdirSync(DOSSIER_VIDEOS, { recursive: true });

const navigateur = await chromium.launch({
  args: ["--no-sandbox", "--disable-dev-shm-usage"],
});

for (const { fichier, export: nomExport } of PARCOURS_A_FILMER) {
  const module = await charger(fichier);
  const parcours = module[nomExport];

  // Un contexte PAR parcours : `recordVideo` compresse la vidéo à la
  // fermeture du contexte, et deux parcours qui partageraient un contexte
  // finiraient dans le même fichier — exactement ce que le renommage
  // ci-dessous, écrit pour UN SEUL fichier `.webm` neuf, ne saurait démêler.
  const contexte = await navigateur.newContext({
    baseURL: BASE,
    viewport: { width: 1280, height: 720 },
    recordVideo: { dir: DOSSIER_VIDEOS, size: { width: 1280, height: 720 } },
    locale: "fr-BE",
  });
  const page = await contexte.newPage();

  // ── La console du navigateur, retenue avec le parcours ───────────────────
  //
  // Un parcours interrompu dit OÙ il s'est arrêté, jamais POURQUOI.
  //
  // Le 2026-09-18, les cinq parcours ont buté sur le voile de `RouteGuard` au
  // premier run de `vitrine.yml`, et TROIS correctifs successifs sont partis
  // sur des hypothèses — le délai du clic, puis celui de l'attente, puis la
  // logique du composant — parce que rien ne rapportait ce que la page disait
  // d'elle-même. Instrumenter coûte dix lignes ; deviner a coûté trois runs.
  //
  // On garde les erreurs, les avertissements et les requêtes échouées. Pas
  // les `log`, qui noieraient le signal. Trente entrées suffisent : ce qui
  // compte est le DÉBUT de la panne, pas sa répétition.
  const journalConsole = [];
  const PLAFOND_JOURNAL = 30;
  const retenir = (entree) => {
    if (journalConsole.length < PLAFOND_JOURNAL) journalConsole.push(entree);
  };
  page.on("console", (msg) => {
    if (msg.type() === "error" || msg.type() === "warning") {
      retenir(`[${msg.type()}] ${msg.text().slice(0, 300)}`);
    }
  });
  page.on("pageerror", (err) =>
    retenir(`[pageerror] ${String(err).slice(0, 300)}`),
  );
  page.on("requestfailed", (req) =>
    retenir(
      `[requestfailed] ${req.method()} ${req.url().slice(0, 160)} — ` +
        `${req.failure()?.errorText ?? "?"}`,
    ),
  );

  let echec = null;
  let scene = new Scene(page);
  try {
    // L'amorçage est DANS le try : s'il échoue, la vitrine doit le montrer
    // plutôt que de s'interrompre sans vidéo. C'est l'amorçage qui a rendu
    // la première vitrine muette, en CI, sur un monde que rien ne créait
    // (#876).
    scene = new Scene(page, await parcours.amorcer(page));
    await scene.raconter(parcours.propos);
    for (const etape of parcours.etapes) {
      await scene.raconter(etape.description, etape.acteur);
      await etape.action(scene);
      // Pas d'assertion ici : ce harnais ne rend PAS de verdict. Si le
      // parcours casse, c'est le gate E2E qui le dit — lui seul en a la
      // responsabilité.
    }
  } catch (e) {
    // On enregistre quand même ce qui a été filmé : une vitrine partielle
    // montre où ça s'arrête, ce qu'un échec silencieux ne montrerait pas.
    echec = e;
    console.error(`  ⚠️  ${parcours.slug} interrompu : ${e.message}`);
  }

  // Le chemin de la vidéo se DEMANDE à Playwright, avant la fermeture du
  // contexte : `video()` est attaché à la page, et c'est la seule source qui
  // désigne le bon fichier à coup sûr.
  //
  // Ce qu'il remplace, et pourquoi (constaté le 2026-09-19) : l'ancien code
  // listait les `.webm` du dossier, écartait le slug courant, triait et
  // prenait le DERNIER. Or Playwright nomme ses fichiers `page@<hash>.webm`,
  // et deux slugs trient APRÈS `page@` — `perimetre-multi-role` et
  // `prestataire` (« pe » et « pr » > « pa »). Le parcours suivant attrapait
  // donc une vidéo DÉJÀ renommée au lieu de la brute, et l'écrasait.
  //
  // Résultat mesuré sur la branche `vitrine-publiee` : cinq vidéos sur sept,
  // deux `page@<hash>.webm` orphelines, et un `index.html` pointant vers
  // deux fichiers inexistants. Le tri ne peut pas distinguer une brute d'une
  // renommée — il ne fallait pas le lui demander.
  const source = page.video();
  await contexte.close(); // déclenche l'écriture de la vidéo

  const cible = resolve(DOSSIER_VIDEOS, `${parcours.slug}.webm`);
  const brute = source ? await source.path() : null;
  if (brute && brute !== cible) renameSync(brute, cible);
  if (!existsSync(cible)) {
    console.error(
      `  ⚠️  ${parcours.slug} : aucune vidéo écrite (${brute ?? "pas de source"})`,
    );
  }

  writeFileSync(
    resolve(DOSSIER_VIDEOS, `${parcours.slug}.json`),
    JSON.stringify(
      {
        slug: parcours.slug,
        title: parcours.titre,
        propos: parcours.propos,
        acteurs: [...new Set(parcours.etapes.map((e) => e.acteur))],
        interrompu: echec ? String(echec.message) : null,
        // Ce que la PAGE a dit pendant le parcours. Vide quand tout va bien.
        console: journalConsole,
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
}

await navigateur.close();
await vite.close();
