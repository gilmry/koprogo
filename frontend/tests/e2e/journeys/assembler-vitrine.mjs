// assembler-vitrine.mjs — agrège vidéos et narrations en une galerie HTML.
//
// Transposé de `.foyer/kit-actix/harness/demo/assembler-vitrine.mjs`.
//
// La galerie est **statique et autonome** : aucune dépendance réseau, aucune
// police distante, aucun script. Elle doit s'ouvrir depuis un artefact de CI
// téléchargé, sur une machine hors ligne, sans rien installer — sinon
// l'arbitre ne la regardera pas, et la preuve sera perdue faute d'être
// atteignable.
//
// Les **chapitres horodatés** sont la raison d'être de ce fichier : chaque
// entrée de narration devient un bouton qui déplace la lecture. L'arbitre saute
// à l'étape au lieu de regarder le film en entier. C'est ce qui rend l'arbitre
// unique tenable sur 87 branches (cf. #875).
import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";

const ICI = dirname(new URL(import.meta.url).pathname);
const DOSSIER_VIDEOS = resolve(ICI, "vitrine/videos");
const SORTIE = resolve(ICI, "vitrine/index.html");

if (!existsSync(DOSSIER_VIDEOS)) {
  console.error("  ⚠️  aucune vidéo : lancer enregistrer-vitrine.mjs d'abord.");
  process.exit(0); // non bloquant, par construction
}

const echapper = (s) =>
  String(s).replace(
    /[&<>"']/g,
    (c) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      })[c],
  );

const metas = readdirSync(DOSSIER_VIDEOS)
  .filter((f) => f.endsWith(".json"))
  .map((f) => JSON.parse(readFileSync(resolve(DOSSIER_VIDEOS, f), "utf8")))
  .sort((a, b) => a.slug.localeCompare(b.slug));

const cartes = metas
  .map((m) => {
    const chapitres = (m.narration ?? [])
      .map(
        (n) =>
          `<li><button type="button" data-t="${n.t}">` +
          `<span class="t">${String(n.t).padStart(3, "0")}s</span>` +
          `${n.acteur ? `<span class="acteur">${echapper(n.acteur)}</span>` : ""}` +
          `<span class="txt">${echapper(n.texte)}</span></button></li>`,
      )
      .join("");
    const acteurs = (m.acteurs ?? [])
      .map((a) => `<span class="badge">${echapper(a)}</span>`)
      .join("");
    const alerte = m.interrompu
      ? `<p class="alerte">Parcours interrompu : ${echapper(m.interrompu)}. ` +
        `Cette vitrine est partielle — elle montre où ça s'arrête.</p>`
      : "";
    return `
      <article class="carte">
        <h2>${echapper(m.title)}</h2>
        <p class="propos">${echapper(m.propos ?? "")}</p>
        <p class="acteurs">${acteurs}</p>
        ${alerte}
        <video controls preload="metadata" src="videos/${echapper(m.slug)}.webm"></video>
        <ol class="chapitres">${chapitres}</ol>
      </article>`;
  })
  .join("");

const html = `<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Vitrine KoproGo — preuve de valeur</title>
<style>
  :root { color-scheme: light dark; }
  body { font: 16px/1.5 system-ui, sans-serif; margin: 0; background: #f6f7f9; color: #111827; }
  header { background: #111827; color: #fff; padding: 24px; }
  header h1 { margin: 0 0 6px; font-size: 22px; }
  header p { margin: 0; opacity: .82; font-size: 14px; max-width: 70ch; }
  main { max-width: 920px; margin: 24px auto; padding: 0 16px; display: grid; gap: 24px; }
  .carte { background: #fff; border-radius: 12px; box-shadow: 0 1px 4px rgba(0,0,0,.08); padding: 16px; }
  .carte h2 { margin: 0 0 6px; font-size: 18px; }
  .propos { margin: 0 0 10px; color: #4b5563; font-size: 14px; }
  .badge { display: inline-block; background: #eef2ff; color: #3730a3; border-radius: 999px;
           padding: 2px 10px; font-size: 12px; margin-right: 6px; }
  .alerte { background: #fef2f2; color: #991b1b; border-left: 3px solid #dc2626;
            padding: 8px 12px; font-size: 14px; border-radius: 4px; }
  video { width: 100%; border-radius: 8px; background: #000; margin-top: 10px; }
  .chapitres { margin: 12px 0 0; padding: 0; list-style: none; }
  .chapitres button { display: flex; gap: 10px; align-items: baseline; width: 100%;
                      text-align: left; background: none; border: 0; padding: 5px 6px;
                      font: inherit; font-size: 14px; color: #374151; cursor: pointer;
                      border-radius: 6px; }
  .chapitres button:hover, .chapitres button:focus { background: #f3f4f6; }
  .chapitres .t { color: #6b7280; font-variant-numeric: tabular-nums; flex: 0 0 auto; }
  .chapitres .acteur { color: #3730a3; flex: 0 0 auto; font-size: 12px; }
  footer { text-align: center; color: #6b7280; font-size: 13px; padding: 24px; }
  @media (prefers-color-scheme: dark) {
    body { background: #0b0f19; color: #e5e7eb; }
    .carte { background: #111827; box-shadow: none; }
    .propos, .chapitres button { color: #9ca3af; }
    .chapitres button:hover, .chapitres button:focus { background: #1f2937; }
  }
</style>
</head>
<body>
<header>
  <h1>Vitrine KoproGo — preuve de valeur</h1>
  <p>Le parcours de référence, rejoué en cadence et narré. Ce n'est pas un
     verdict : le vert et le rouge viennent des gates. Ceci répond à l'autre
     question — à quoi ça ressemble pour un humain.</p>
</header>
<main>${cartes || "<p>Aucun parcours enregistré.</p>"}</main>
<footer>Généré par <code>assembler-vitrine.mjs</code> — Méthode Foyer,
        documentation vivante. ${metas.length} parcours.</footer>
<script>
  // Les chapitres déplacent la lecture. C'est ce qui rend la preuve bon
  // marché à lire : l'arbitre saute à l'étape au lieu de tout regarder.
  document.querySelectorAll(".carte").forEach((carte) => {
    const video = carte.querySelector("video");
    carte.querySelectorAll(".chapitres button").forEach((b) => {
      b.addEventListener("click", () => {
        video.currentTime = Number(b.dataset.t) || 0;
        video.play();
      });
    });
  });
</script>
</body>
</html>
`;

writeFileSync(SORTIE, html);
console.log(`  🖼  ${SORTIE} — ${metas.length} parcours`);
