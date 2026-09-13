import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";
import contrat from "./data-testid.contrat.json";

/**
 * Garde-fou : le contrat `data-testid` ne se perd pas dans la refonte.
 *
 * ── Pourquoi geler maintenant ──────────────────────────────────────────────
 *
 * La revue de design du 2026-09-06 le dit sans détour : « le contrat
 * `data-testid` n'est pas négociable ». La suite de tests encode des **règles
 * produit** — RBAC, invariants légaux — que les maquettes ne peuvent pas
 * exprimer. Renommer un identifiant, c'est effacer une règle sans s'en
 * apercevoir.
 *
 * La refonte à venir (#797 à #802) réécrit la barre latérale, les tableaux de
 * bord des quatre rôles, le motif de liste et le périmètre applicatif. C'est
 * précisément le moment où un identifiant disparaît « en passant ».
 *
 * Ce fichier fige donc l'état du 2026-09-06 : **882 identifiants littéraux et
 * 22 préfixes construits**, relevés dans le code de production.
 *
 * ── Ce que le contrat autorise ─────────────────────────────────────────────
 *
 * Il ne peut que **grandir**. Ajouter un identifiant ne demande rien. En
 * retirer un exige de modifier `data-testid.contrat.json`, ce qui se voit en
 * revue et appelle une justification — exactement comme les cliquets
 * `garde_ecriture` et `garde_champs_ignores` du backend bornent une dette
 * sans exiger sa disparition immédiate.
 *
 * ── Pourquoi les tests ne suffisent pas ────────────────────────────────────
 *
 * On pourrait croire qu'un identifiant supprimé fait échouer les tests qui
 * l'interrogent. C'est vrai des identifiants **testés**. Or le code en offre
 * 882 et les tests n'en interrogent que ~530 : plus de trois cents existent
 * sans filet, souvent posés pour un usage futur ou pour la recette humaine.
 * Ceux-là disparaîtraient sans un bruit.
 *
 * ── Les préfixes construits ────────────────────────────────────────────────
 *
 * `data-testid="building-selector-result-{acp.id}"` produit un identifiant
 * différent à chaque ligne. On ne peut pas figer la valeur, seulement le
 * préfixe stable — c'est lui que les tests emploient pour cibler une ligne
 * précise, et c'est lui qui doit survivre.
 */

const RACINE = join(process.cwd(), "src");
const EXTENSIONS = new Set([".svelte", ".astro"]);

function fichiersDeGabarit(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersDeGabarit(chemin));
    } else if (EXTENSIONS.has(extname(entree)) && !entree.includes(".test.")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

/** Ce que le code offre aujourd'hui : valeurs figées et préfixes construits. */
function contratActuel(): { litteraux: Set<string>; prefixes: Set<string> } {
  const litteraux = new Set<string>();
  const prefixes = new Set<string>();
  for (const chemin of fichiersDeGabarit(RACINE)) {
    const texte = readFileSync(chemin, "utf8");
    // Deux formes portent le MÊME contrat, et la seconde manquait.
    //
    // `data-testid="x"` pose l'attribut directement. `testId="x"` le passe à
    // un composant qui le repose sur son élément racine — motif que
    // `Button.svelte` emploie depuis longtemps via
    // `export { testId as 'data-testid' }`, et que `BoutonAction` et
    // `CadreLegal` reprennent.
    //
    // Ne lire que la première faisait disparaître du « contrat » des ancrages
    // qui vivent toujours dans le DOM rendu. La garde a signalé deux
    // identifiants perdus alors qu'ils ne l'étaient pas : elle mesurait le
    // texte du source, quand le contrat porte sur ce que le navigateur voit.
    for (const m of texte.matchAll(/(?:data-testid|testId)="([^"]+)"/g)) {
      const valeur = m[1];
      if (valeur.includes("{")) {
        const prefixe = valeur.split("{")[0].replace(/-+$/, "");
        if (prefixe) prefixes.add(prefixe);
      } else {
        litteraux.add(valeur);
      }
    }
  }
  return { litteraux, prefixes };
}

describe("le contrat data-testid ne rétrécit pas (#802)", () => {
  const actuel = contratActuel();

  it("conserve tous les identifiants littéraux figés le 2026-09-06", () => {
    const disparus = (contrat.litteraux as string[]).filter(
      (id) => !actuel.litteraux.has(id),
    );

    expect(
      disparus,
      `${disparus.length} identifiant(s) data-testid ont disparu du code.\n\n` +
        `Chacun peut porter une règle produit qu'aucune maquette n'exprime : ` +
        `un test de rôle, un invariant légal, un parcours de recette humaine. ` +
        `Le code en offre 882 et les tests n'en interrogent que ~530 : la ` +
        `disparition d'un identifiant non testé ne fait échouer personne.\n\n` +
        `Si la suppression est voulue, retirez l'entrée de ` +
        `\`data-testid.contrat.json\` dans le MÊME commit, en disant pourquoi.\n\n` +
        disparus.join("\n"),
    ).toEqual([]);
  });

  it("conserve les préfixes des identifiants construits", () => {
    const disparus = (contrat.prefixes_dynamiques as string[]).filter(
      (p) => !actuel.prefixes.has(p),
    );

    expect(
      disparus,
      `${disparus.length} préfixe(s) d'identifiant construit ont disparu.\n\n` +
        `Un préfixe comme \`building-selector-result\` est ce qui permet à un ` +
        `test de cibler une ligne précise d'une liste. Le perdre rend ces ` +
        `tests inécrivables.\n\n` +
        disparus.join("\n"),
    ).toEqual([]);
  });

  /// Le contrat doit rester une photographie du réel, pas une liste de vœux.
  it("ne fige aucun identifiant qui n'a jamais existé", () => {
    expect(contrat.litteraux.length).toBeGreaterThan(800);
    expect(contrat.prefixes_dynamiques.length).toBeGreaterThan(10);
  });
});
