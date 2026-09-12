import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Deux neutres sont des couleurs de TRAIT, jamais de texte.
 *
 * La remise de design pose la règle sans ambiguïté :
 *
 * > `#a8a29e` and `#c4c0bb` are **stroke/border colours only** — never text.
 * > This includes chevrons, which carry tap affordance and must reach
 * > icon-contrast. Any text below 14px uses `muted` or `muted-strong`.
 *
 * `#a8a29e` est `stone-400`. Sur le fond `page` (`#fafaf9`) il donne 2,5:1,
 * loin sous les 4,5:1 exigés. Le jeu de jetons porte exactement deux neutres
 * de texte, `muted` (4,80:1) et `muted-strong` (5,4:1), et ils n'existent que
 * pour ça.
 *
 * ── Pourquoi une garde statique et pas seulement axe-core ────────────────
 *
 * `Accessibility.spec.ts` exécute bien axe-core avec
 * `runOnly: ['color-contrast']`, mais **seulement sur `/login`**, et il ne
 * voit qu'un état de rendu à la fois : un texte gris dans une branche `{#if}`
 * non prise, une pastille d'état qui n'apparaît qu'en cas d'erreur, un chevron
 * dans une ligne conditionnelle lui échappent tous. Une garde statique lit le
 * source entier, y compris ce qui ne s'affiche presque jamais.
 *
 * Les deux se complètent : celle-ci attrape l'écriture, axe-core attrape le
 * rendu composé.
 *
 * ── Le cliquet ──────────────────────────────────────────────────────────
 *
 * Interdiction, donc zéro. Ces classes n'ont aucun emploi légitime comme
 * texte : `text-stone-400` a toujours une alternative qui passe le contraste.
 */

/**
 * Les classes Tailwind qui posent une COULEUR DE TEXTE sous le seuil.
 *
 * `stone-400` est `#a8a29e`, `stone-300` est `#d6d3d1` — le `border-strong`
 * des jetons, encore plus clair. `gray-400` et `slate-400` sont leurs
 * équivalents froids, que le produit n'utilise plus mais qui traînent.
 */
const TEXTE_INTERDIT =
  /\btext-(?:stone|gray|slate|zinc|neutral)-(?:100|200|300|400)\b/g;

/** Les hexadécimaux de trait, écrits en dur dans une propriété de texte. */
const HEXA_DE_TRAIT = /(?:color|fill)\s*:\s*(#a8a29e|#c4c0bb|#d6d3d1)\b/gi;

const RACINE = join(process.cwd(), "src");

function sources(dossier: string, sortie: string[] = []): string[] {
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      // Les tests décrivent le défaut qu'ils interdisent : les citer
      // ferait échouer la garde sur sa propre documentation.
      if (entree === "__tests__" || entree === "node_modules") continue;
      sources(chemin, sortie);
    } else if (/\.(svelte|astro|css|ts)$/.test(entree)) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

/**
 * Sur fond sombre, un neutre clair EST le bon choix de texte.
 *
 * `text-gray-400` (#9ca3af) sur `bg-gray-900` donne 7:1 ; le remplacer par
 * `muted` (#78716c) ferait TOMBER le contraste à 3,4:1. La règle vise les
 * neutres trop clairs sur fond clair, pas les neutres clairs tout court.
 */
const FOND_SOMBRE =
  /bg-(?:gray|slate|stone|zinc|neutral)-(?:700|800|900)|bg-black/;

/**
 * Un contrôle désactivé est exempté du contraste par WCAG 1.4.3, et le gris
 * pâle y est l'affordance même de l'inactivité. Le rehausser rendrait un
 * bouton mort indiscernable d'un bouton vivant — on corrigerait un chiffre
 * en cassant ce qu'il mesure.
 *
 * L'exception ne vaut QUE si le neutre est porté par la variante `disabled:`
 * elle-même. Ma première version cherchait le mot « disabled » dans une
 * fenêtre de ±220 caractères : dans `Pagination.svelte`, les attributs
 * `disabled={…}` des boutons « Précédent » et « Suivant » exemptaient alors
 * tout le voisinage, et le témoin ne mordait pas. Une garde qui ne mord pas
 * n'est pas une garde.
 */
const DESACTIVE_PORTE = /disabled:[a-z-]*$/;

/**
 * La portion de source qui entoure une occurrence, pour juger du fond.
 *
 * Le fond, lui, se lit bien en contexte : la couleur de fond d'un conteneur
 * est rarement sur le même élément que le texte qu'il porte.
 */
function contexte(source: string, index: number): string {
  return source.slice(Math.max(0, index - 220), index + 220);
}

/** Le préfixe collé à l'occurrence : `disabled:text-gray-400` par exemple. */
function prefixeImmediat(source: string, index: number): string {
  return source.slice(Math.max(0, index - 24), index);
}

function releve(): string[] {
  const fautifs: string[] = [];
  for (const chemin of sources(RACINE)) {
    // Les commentaires expliquent souvent POURQUOI une couleur est proscrite.
    // Les compter reviendrait à punir l'explication — et ce n'est pas
    // théorique : ma propre note dans `EvidenceUpload.svelte`, qui cite
    // `text-gray-400` pour dire pourquoi elle ne l'emploie pas, faisait
    // échouer cette garde. C'est la deuxième fois qu'un cliquet de ce dépôt
    // se retourne contre l'explication d'un défaut.
    //
    // Les trois formes de commentaire, donc. Le `(^|[^:])` du dernier motif
    // épargne `https://` et `file://`, qui ne sont pas des commentaires.
    const source = readFileSync(chemin, "utf-8")
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/(^|[^:])\/\/[^\n]*/g, "$1");

    for (const motif of [TEXTE_INTERDIT, HEXA_DE_TRAIT]) {
      for (const m of source.matchAll(motif)) {
        const index = m.index ?? 0;
        if (FOND_SOMBRE.test(contexte(source, index))) continue;
        if (DESACTIVE_PORTE.test(prefixeImmediat(source, index))) continue;
        const ligne = source.slice(0, m.index).split("\n").length;
        fautifs.push(
          `  ${chemin.replace(process.cwd() + "/", "")}:${ligne} — ${m[0]}`,
        );
      }
    }
  }
  return fautifs;
}

describe("les neutres de trait ne servent jamais de texte", () => {
  it("n'emploie aucun neutre trop clair comme couleur de texte", () => {
    expect(
      releve().join("\n"),
      "Ces neutres sont des couleurs de TRAIT — bordures, séparateurs, " +
        "contours. Employés comme texte ils tombent sous 4,5:1, et " +
        "`Accessibility.spec.ts` les refusera dès qu'il couvrira l'écran " +
        "concerné.\n\n" +
        "Les chevrons comptent : ils portent l'affordance du tap et doivent " +
        "atteindre le contraste d'icône.\n\n" +
        "Employez `text-muted` (#78716c, 4,80:1) ou `text-muted-strong` " +
        "(#6b6660, 5,4:1). Ces deux jetons n'existent que pour cela.",
    ).toBe("");
  });

  it("lit bien les fichiers, et n'est pas aveugle par construction", () => {
    // Vérification d'aveuglement : sans elle, une garde qui ne lirait plus
    // rien — répertoire renommé, extension oubliée — passerait au vert en
    // silence, et son zéro voudrait dire « je n'ai rien regardé ».
    const fichiers = sources(RACINE);
    expect(fichiers.length).toBeGreaterThan(100);
    expect(fichiers.some((f) => f.endsWith(".svelte"))).toBe(true);
    expect(fichiers.some((f) => f.endsWith("global.css"))).toBe(true);
  });
});
