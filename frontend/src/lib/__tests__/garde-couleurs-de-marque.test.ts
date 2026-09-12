import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Le vert de marque ne s'écrit qu'à un seul endroit.
 *
 * C'est le troisième critère de fin de #797 :
 *
 * > Les jetons sont dans `@theme`, et **aucune couleur de marque n'est écrite
 * > en dur ailleurs**.
 *
 * ── Pourquoi cette garde, alors que #790 a déjà supprimé le fichier ─────
 *
 * `tailwind.config.mjs` déclarait `primary-600: #059669` là où le bloc
 * `@theme` dit `#15803d`, sans directive `@config` pour le charger : il
 * mentait, et il a été supprimé. Mais supprimer la source du mensonge ne
 * retire pas les copies qui en étaient sorties.
 *
 * Relevé le 2026-09-12, six mois après que ce vert a cessé d'être celui du
 * produit : **`#059669` survivait à six endroits**, dont `public/manifest.json`
 * où il donnait au chrome de la PWA une couleur que plus rien n'utilise.
 *
 * ── Pourquoi `#15803d` est interdit AUSSI ──────────────────────────────
 *
 * C'est la bonne couleur, et c'est précisément le piège : une valeur juste
 * écrite en dur ne se remarque pas, et devient fausse le jour où la marque
 * change — exactement ce qui vient d'arriver à `#059669`. Un jeton n'a de
 * valeur que s'il est le seul chemin.
 *
 * ── Le cliquet ─────────────────────────────────────────────────────────
 *
 * Interdiction, donc zéro dans `src/`. Deux fichiers sont exemptés, et pour
 * des raisons opposées :
 *
 * - `src/styles/global.css` — c'est LA source de vérité, le bloc `@theme` ;
 * - `src/layouts/Layout.astro` et `public/manifest.json` — la balise
 *   `theme-color` et le manifeste sont lus par le système d'exploitation
 *   AVANT que la moindre feuille de style ne soit chargée. Une `var()` y
 *   serait ignorée en silence. Ils sont donc autorisés à porter la valeur,
 *   mais un test dédié vérifie qu'ils portent **la même**, ce qui est le vrai
 *   risque : #796 a corrigé la balise et laissé le manifeste derrière.
 */

/** Les deux verts de marque : l'actuel, et celui qui ne l'est plus. */
const VERT_ACTUEL = "#15803d";
const VERT_MORT = "#059669";
const MARQUE = /#(?:15803d|059669)\b/gi;

const RACINE = join(process.cwd(), "src");

/** La source de vérité, et les deux fichiers lus avant toute feuille. */
const EXEMPTES = new Set([
  join(RACINE, "styles", "global.css"),
  join(RACINE, "layouts", "Layout.astro"),
]);

function sources(dossier: string, sortie: string[] = []): string[] {
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      // Une garde ne se lit pas elle-même : ce fichier cite les deux hexas
      // qu'il interdit, et se ferait échouer sur sa propre documentation.
      if (entree === "__tests__" || entree === "node_modules") continue;
      sources(chemin, sortie);
    } else if (/\.(svelte|astro|css|ts|json)$/.test(entree)) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

function occurrences(): string[] {
  const trouves: string[] = [];
  for (const chemin of sources(RACINE)) {
    if (EXEMPTES.has(chemin)) continue;
    const source = readFileSync(chemin, "utf-8");
    for (const m of source.matchAll(MARQUE)) {
      const ligne = source.slice(0, m.index).split("\n").length;
      const relatif = chemin.slice(process.cwd().length + 1);
      trouves.push(`${relatif}:${ligne}  ${m[0]}`);
    }
  }
  return trouves;
}

describe("le vert de marque ne s'écrit qu'au bloc @theme", () => {
  it("n'apparaît en dur nulle part dans src/", () => {
    const trouves = occurrences();
    expect(
      trouves,
      `${trouves.length} couleur(s) de marque écrite(s) en dur :\n\n` +
        trouves.map((t) => `  ${t}`).join("\n") +
        `\n\nLe bloc @theme de global.css est la seule source de vérité. ` +
        `Employer var(--color-primary) ou var(--color-primary-hover), ou la ` +
        `classe Tailwind correspondante.\n\n` +
        `${VERT_MORT} est pire qu'une duplication : ce n'est PLUS le vert du ` +
        `produit. Il vient du tailwind.config.mjs supprimé par #790, qui ` +
        `n'était même pas chargé. Ces lignes peignent donc une couleur que ` +
        `plus rien d'autre n'emploie (#797).`,
    ).toEqual([]);
  });

  it("le manifeste de la PWA annonce le vert actuel", () => {
    const manifeste = JSON.parse(
      readFileSync(join(process.cwd(), "public", "manifest.json"), "utf-8"),
    );
    expect(
      manifeste.theme_color?.toLowerCase(),
      `Le manifeste annonce ${manifeste.theme_color}, le produit est ` +
        `${VERT_ACTUEL}.\n\n` +
        `Le manifeste et la balise theme-color de Layout.astro sont lus par ` +
        `le système AVANT toute feuille de style : ils portent la valeur en ` +
        `clair, et rien ne les tient d'accord. #796 a corrigé la balise et ` +
        `laissé le manifeste derrière (#797).`,
    ).toBe(VERT_ACTUEL);
  });

  it("les trois porteurs de la valeur en clair disent la même chose", () => {
    // Le manifeste d'astro.config.mjs est aujourd'hui SERVI SANS ÊTRE LU :
    // `@vite-pwa` en produit `dist/manifest.webmanifest`, mais Layout.astro
    // pointe `<link rel="manifest" href="/manifest.json">`, donc c'est le
    // fichier de `public/` que le navigateur charge.
    //
    // Un fichier généré, servi, et référencé par personne est exactement le
    // genre d'endroit où une couleur fausse survit des mois : relevé le
    // 2026-09-12, il annonçait `#0F766E`, un turquoise qui n'a jamais été la
    // marque. Le jour où quelqu'un branchera ce manifeste-là, il livrerait
    // cette couleur sans qu'aucun test ne bronche.
    const config = readFileSync(join(process.cwd(), "astro.config.mjs"), "utf-8");
    const m = config.match(/theme_color:\s*"(#[0-9a-f]{6})"/i);
    expect(m, "Aucun theme_color dans le manifeste d'astro.config.mjs").not.toBeNull();
    expect(m![1].toLowerCase()).toBe(VERT_ACTUEL);
  });

  it("la balise theme-color dit la même chose que le manifeste", () => {
    const layout = readFileSync(
      join(RACINE, "layouts", "Layout.astro"),
      "utf-8",
    );
    const m = layout.match(/name="theme-color"\s+content="(#[0-9a-f]{6})"/i);
    expect(m, "Aucune balise theme-color dans Layout.astro").not.toBeNull();
    expect(m![1].toLowerCase()).toBe(VERT_ACTUEL);
  });

  it("lit bien les fichiers, et n'est pas verte par vacuité", () => {
    const fichiers = sources(RACINE);
    expect(fichiers.length).toBeGreaterThan(100);
    // Le motif doit mordre sur la source de vérité, sans quoi il ne mordrait
    // sur rien : une garde dont le détecteur est cassé passe au vert partout.
    const theme = readFileSync(join(RACINE, "styles", "global.css"), "utf-8");
    expect([...theme.matchAll(MARQUE)].length).toBeGreaterThan(0);
  });
});
