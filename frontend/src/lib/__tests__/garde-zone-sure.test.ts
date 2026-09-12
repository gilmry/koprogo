import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Cliquet : les zones sûres ne sont pas des marges décoratives.
 *
 * ── Le défaut que cette garde tient ────────────────────────────────────────
 *
 * Deux composants réservaient une marge sous l'indicateur d'accueil de
 * l'iPhone avec `env(safe-area-inset-bottom)` : la barre d'onglets, présente
 * sur chaque écran mobile, et la feuille du bas.
 *
 * `env(safe-area-inset-*)` vaut **zéro** tant que la balise viewport ne porte
 * pas `viewport-fit=cover`. Elle ne le portait pas. Les deux marges étaient
 * donc écrites juste et **inertes** : la dernière rangée de pixels de la barre
 * d'onglets tombait sous l'indicateur, dont la zone de glissement recouvre les
 * cibles.
 *
 * Rien ne pouvait le signaler. Une balise `<meta>` dans un gabarit Astro et
 * une classe Tailwind dans un composant Svelte n'ont aucune raison de se
 * connaître, et aucune des deux n'est fausse prise seule.
 *
 * ── Ce que poser `viewport-fit=cover` déplace ──────────────────────────────
 *
 * Le HAUT de l'écran passe aussi sous l'encoche. L'en-tête mobile était en
 * `fixed top-0 h-14`, et `Layout.astro` lui réservait une cale `h-14` : deux
 * `14` écrits séparément, qu'il fallait désormais faire grandir tous les deux
 * de la hauteur exacte de l'encoche. Ils passent par un jeton unique,
 * `--spacing-entete-mobile`, et cette garde refuse qu'ils s'en détachent.
 *
 * ── Pourquoi SEUL `Layout.astro` est visé ──────────────────────────────────
 *
 * Deux pages déclarent leur propre `<html>` hors du gabarit partagé :
 * `src/pages/c.astro` et `src/pages/contractor-report/index.astro`, les écrans
 * du prestataire ouverts par lien magique. Elles n'affichent ni barre
 * d'onglets ni feuille du bas, donc **aucune marge de zone sûre**.
 *
 * Leur poser `viewport-fit=cover` par souci d'uniformité créerait le défaut au
 * lieu de le corriger : le contenu passerait sous l'encoche sans rien pour le
 * redescendre. La balise ne se pose que là où quelque chose la compense.
 */

const RACINE = process.cwd();
const EXTENSIONS = new Set([".svelte", ".astro", ".css", ".ts"]);

/**
 * Les gardes ne se lisent pas elles-mêmes.
 *
 * Sans cette exclusion, ce cliquet compte le `calc()` de son propre
 * commentaire d'explication et échoue sur lui. C'est la TROISIÈME fois dans ce
 * chantier qu'un cliquet se retourne contre le texte qui le justifie, et les
 * deux parades précédentes — dépouiller `<!-- -->`, puis `//` — n'ont tenu que
 * jusqu'à la forme de commentaire suivante.
 *
 * La règle qui tient est celle-ci : un fichier de test ne part pas au
 * navigateur, donc il ne peut porter aucun défaut de style. Il n'est pas une
 * preuve, quoi qu'il contienne.
 */
const EXCLUS = /(^|[\\/])__tests__([\\/]|$)/;

function fichiers(depuis: string, acc: string[] = []): string[] {
  for (const entree of readdirSync(join(RACINE, depuis))) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    if (EXCLUS.test(join(depuis, entree))) continue;
    const relatif = join(depuis, entree);
    if (statSync(join(RACINE, relatif)).isDirectory()) fichiers(relatif, acc);
    else if (EXTENSIONS.has(extname(entree))) acc.push(relatif);
  }
  return acc;
}

const SOURCES = fichiers("src");
const LAYOUT = "src/layouts/Layout.astro";

/**
 * Un usage RÉEL de zone sûre, par opposition à une prose qui en parle.
 *
 * La première version de ce contrôle comptait `env(safe-area-inset-` partout,
 * et voyait donc le commentaire de `Layout.astro` qui explique le correctif.
 * Le témoin l'a montré : en supprimant les quatre vrais usages, la garde
 * restait verte sur une phrase.
 *
 * Deux filtres, dont aucun ne suffit seul :
 *
 * - un usage nomme un côté concret, jamais `safe-area-inset-*` — le joker
 *   n'existe pas en CSS et n'apparaît que dans une explication ;
 * - il n'est pas sur une ligne de commentaire.
 *
 * **Limite connue, écrite pour qu'on ne la découvre pas par surprise** : un
 * commentaire qui nommerait `env(safe-area-inset-bottom)` en milieu de ligne
 * de code passerait encore. Compter du texte ne remplacera jamais la lecture
 * de la feuille de style produite ; ce contrôle vise le cas réaliste, pas
 * l'adversaire.
 */
function aUneZoneSure(source: string): boolean {
  return source.split("\n").some((ligne) => {
    const nue = ligne.trim();
    if (nue.startsWith("//") || nue.startsWith("*") || nue.startsWith("<!--")) {
      return false;
    }
    return /env\(safe-area-inset-(top|bottom|left|right)\b/.test(ligne);
  });
}

describe("les zones sûres sont effectivement activées", () => {
  it("trouve bien des usages de zone sûre, et ne passe pas faute d'en trouver", () => {
    // Vérification d'aveuglement : si plus personne n'utilise `env()`, la
    // règle suivante devient vraie sans rien garder. Mieux vaut qu'elle
    // échoue et qu'on la retire sciemment.
    const usages = SOURCES.filter((f) =>
      aUneZoneSure(readFileSync(join(RACINE, f), "utf8")),
    );
    expect(
      usages.length,
      "Plus aucun fichier n'utilise `env(safe-area-inset-*)`. Si les zones " +
        "sûres ne sont plus gérées, retirez cette garde ET `viewport-fit=cover` " +
        "— le laisser posé sans marge fait passer le contenu SOUS l'encoche.",
    ).toBeGreaterThan(0);
  });

  it("pose `viewport-fit=cover` dès qu'une zone sûre est lue", () => {
    const source = readFileSync(join(RACINE, LAYOUT), "utf8");
    const meta = source.match(/<meta[^>]*name="viewport"[\s\S]*?\/>/);

    // Deuxième aveuglement : sans balise trouvée, l'assertion suivante
    // porterait sur une chaîne vide.
    expect(
      meta,
      `Aucune balise viewport trouvée dans ${LAYOUT}. Elle a été déplacée : ` +
        "pointez cette garde sur son nouvel emplacement.",
    ).not.toBeNull();

    expect(
      meta?.[0],
      `La balise viewport de ${LAYOUT} ne porte pas \`viewport-fit=cover\`.\n\n` +
        "Sans lui, `env(safe-area-inset-*)` vaut ZÉRO partout, et les marges " +
        "de la barre d'onglets et de la feuille du bas ne réservent rien. " +
        "Elles restent écrites, testées, et sans effet.",
    ).toContain("viewport-fit=cover");
  });

  it("fait grandir l'en-tête mobile ET sa cale du même jeton", () => {
    const entete = readFileSync(
      join(RACINE, "src/components/navigation/Navigation.svelte"),
      "utf8",
    );
    const gabarit = readFileSync(join(RACINE, LAYOUT), "utf8");

    // L'en-tête est en `fixed top-0` : sous `viewport-fit=cover` il commence
    // au bord physique de l'écran, donc sous l'encoche.
    expect(
      entete,
      "L'en-tête mobile ne descend plus son contenu de l'encoche " +
        "(`pt-[env(safe-area-inset-top,0px)]`). En `fixed top-0` sous " +
        "`viewport-fit=cover`, son contenu passe sous la barre d'état.",
    ).toContain("pt-[env(safe-area-inset-top,0px)]");

    for (const [nom, source] of [
      ["l'en-tête mobile", entete],
      ["la cale de Layout.astro", gabarit],
    ] as const) {
      expect(
        source,
        `${nom} n'utilise plus \`h-entete-mobile\`.\n\n` +
          "Les deux doivent mesurer exactement la même chose. Écrits " +
          "séparément, ils dérivent, et la page glisse sous l'en-tête de la " +
          "hauteur de l'encoche — un décalage qu'aucun écran sans encoche ne " +
          "montre.",
      ).toContain("h-entete-mobile");
    }
  });

  it("fait grandir la barre d'onglets ET la réserve de `main` du même jeton", () => {
    const barre = readFileSync(
      join(RACINE, "src/components/navigation/TabBarMobile.svelte"),
      "utf8",
    );
    const gabarit = readFileSync(join(RACINE, LAYOUT), "utf8");

    // Le défaut d'origine : `h-[74px]` avec `pb-[env(...)]` DEDANS. Tailwind
    // pose `box-sizing: border-box`, donc la marge de zone sûre était prise
    // SUR les 74 px au lieu de s'y ajouter. Sur un iPhone sans bouton, l'inset
    // vaut ~34 px : les cibles retombaient à ~40 px, sous le minimum de 44,
    // pendant que le commentaire du composant promettait « 56 px utilisables ».
    for (const [nom, source] of [
      ["la barre d'onglets", barre],
      ["la réserve de `main` dans Layout.astro", gabarit],
    ] as const) {
      expect(
        source,
        `${nom} n'utilise plus \`barre-onglets\`.\n\n` +
          "Une hauteur en dur des deux côtés se désaccorde dès que l'encoche " +
          "entre en jeu : soit les cibles rétrécissent, soit le dernier " +
          "élément de chaque page revit derrière la barre.",
      ).toContain("barre-onglets");
    }

    expect(
      barre,
      "La barre d'onglets ne pousse plus son contenu au-dessus de " +
        "l'indicateur d'accueil (`pb-[env(safe-area-inset-bottom,0px)]`).",
    ).toContain("pb-[env(safe-area-inset-bottom,0px)]");
  });

  it("donne un repli à tout `env()` enfermé dans un `calc()`", () => {
    // Hors `calc()`, un `env()` non résolu invalide sa seule déclaration :
    // une marge disparaît, ce qui est bénin. DANS un `calc()`, il invalide
    // tout le calcul, donc le jeton entier — et une cale de hauteur `auto`
    // laisse la page remonter sous l'en-tête.
    const fautes: string[] = [];
    for (const f of SOURCES) {
      const source = readFileSync(join(RACINE, f), "utf8");
      for (const calc of source.match(
        /calc\([^;{}"]*env\([^)]*\)[^;{}"]*\)/g,
      ) ?? []) {
        if (!/env\(\s*safe-area-inset-[a-z]+\s*,/.test(calc)) {
          fautes.push(`${f} — ${calc.trim()}`);
        }
      }
    }
    expect(
      fautes,
      "Un `env(safe-area-inset-*)` sans repli est enfermé dans un `calc()`.\n\n" +
        "Là où `env()` ne résout pas, le calcul entier devient invalide et le " +
        "jeton ne vaut plus rien. Écrivez `env(safe-area-inset-top, 0px)`.\n\n" +
        fautes.join("\n"),
    ).toEqual([]);
  });
});
