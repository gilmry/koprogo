import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Un tableau qui ne tient pas dans l'écran doit défiler, pas être coupé.
 *
 * ── Les deux défauts, et pourquoi l'un est bien pire ────────────────────
 *
 * **Le débordement** — un tableau sans conteneur défilant sort de son cadre
 * sur un téléphone. C'est laid, et ça se voit : le contenu dépasse,
 * l'utilisateur comprend qu'il manque quelque chose.
 *
 * **Le découpage silencieux** — un conteneur en `overflow-hidden` masque ce
 * qui dépasse. Pas de barre, pas d'ombre, aucun signe. L'écran a l'air
 * complet.
 *
 * Trois tableaux du dépôt étaient dans ce second état au 2026-09-10, dont
 * `EtatDateList` et ses NEUF colonnes. Un syndic qui consultait ses états
 * datés sur son téléphone ne voyait pas la colonne « Échéance », et ne savait
 * pas qu'elle existait. L'`overflow-hidden` était là pour arrondir les coins
 * de la carte ; il coupait le tableau en prime.
 *
 * ── Défiler, pas empiler ────────────────────────────────────────────────
 *
 * Empiler les colonnes en cartes sur mobile perdrait l'ALIGNEMENT des
 * chiffres, qui est tout l'intérêt d'un tableau : on compare des montants et
 * des dates d'une ligne à l'autre, et cette comparaison se fait à l'œil, en
 * colonne.
 *
 * ── Ce que la mesure a d'abord donné, et pourquoi c'était faux ──────────
 *
 * Premier comptage : dix tableaux fautifs. Mon détecteur ne cherchait que les
 * classes Tailwind, et quatre fichiers déclarent `overflow-x: auto` en CSS
 * scopé. Après correction : six. Quatrième fois dans la journée qu'une mesure
 * affinée évitait un chiffre trop gros.
 *
 * Cette garde accepte donc les DEUX formes.
 *
 * ── Ce qu'elle ne peut pas voir ─────────────────────────────────────────
 *
 * Elle vérifie qu'un défilement est DÉCLARÉ dans le fichier, pas qu'il porte
 * sur le bon élément. Un `overflow-x` sur un conteneur sans rapport la
 * satisferait. C'est une garde grossière contre une régression franche, pas
 * une preuve de mise en page — celle-là demande un rendu, et
 * `Accessibility.spec.ts` ne visite aujourd'hui que `/login` (#865).
 */

const RACINE = join(process.cwd(), "src");

function gabarits(dossier: string, sortie: string[] = []): string[] {
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      if (entree === "__tests__" || entree === "node_modules") continue;
      gabarits(chemin, sortie);
    } else if (/\.(svelte|astro)$/.test(entree)) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

/** Les deux formes de déclaration : classe utilitaire, ou CSS scopé. */
function declareUnDefilement(source: string): boolean {
  return (
    source.includes("overflow-x") ||
    source.includes("overflow: auto") ||
    source.includes("overflow:auto")
  );
}

function releve(): string[] {
  const fautifs: string[] = [];
  for (const chemin of gabarits(RACINE)) {
    const source = readFileSync(chemin, "utf-8");
    if (declareUnDefilement(source)) continue;

    const tableaux = source.match(/<table\b/g);
    if (!tableaux) continue;

    fautifs.push(
      `  ${chemin.replace(process.cwd() + "/", "")} — ${tableaux.length} tableau(x)`,
    );
  }
  return fautifs;
}

describe("les tableaux défilent au lieu d'être coupés", () => {
  it("n'ajoute aucun tableau sans défilement", () => {
    expect(
      releve().join("\n"),
      "Ces fichiers portent un `<table>` sans déclarer le moindre " +
        "défilement horizontal. Sur un téléphone, le tableau déborde — ou, " +
        "pire, il est COUPÉ EN SILENCE si un conteneur porte " +
        "`overflow-hidden` : pas de barre, pas d'ombre, l'écran a l'air " +
        "complet.\n\n" +
        "Le motif :\n" +
        '  <div class="overflow-x-auto" tabindex="0" role="region">\n' +
        '    <table class="min-w-[520px] w-full">\n\n' +
        "Le `min-w` compte autant : sans lui les colonnes se compriment " +
        "jusqu'à l'illisible, et une table qu'on ne peut pas lire est pire " +
        "que celle qui défile.\n\n" +
        "Le `tabindex` non plus n'est pas optionnel — une zone défilante " +
        "doit être atteignable au clavier (WCAG 2.1.1). Svelte le refusera " +
        "sur un élément non interactif ; axe-core l'exige par " +
        "`scrollable-region-focusable`. Le `svelte-ignore` doit porter sa " +
        "raison.",
    ).toBe("");
  });

  it("lit bien les gabarits, et trouve bien des tableaux", () => {
    // Vérification d'aveuglement, deux volets.
    //
    // Le premier : un répertoire renommé rendrait la liste vide, et le zéro
    // ci-dessus voudrait dire « je n'ai rien regardé ».
    const fichiers = gabarits(RACINE);
    expect(fichiers.length).toBeGreaterThan(100);

    // Le second : le motif `<table` doit encore reconnaître un tableau. Sans
    // cela, le relevé serait vide quoi qu'il arrive.
    const avecTableau = fichiers.filter((f) =>
      /<table\b/.test(readFileSync(f, "utf-8")),
    );
    expect(
      avecTableau.length,
      "plus aucun tableau dans le dépôt : le motif a changé, ou les listes " +
        "ont été réécrites. Vérifiez avant de vous réjouir.",
    ).toBeGreaterThan(5);
  });
});
