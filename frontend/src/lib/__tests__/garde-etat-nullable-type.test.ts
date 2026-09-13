import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Un état nullable se déclare `$state<T | null>(null)`, jamais
 * `let x: T | null = $state(null)`.
 *
 * La seconde forme NE TYPE RIEN. `svelte-check` réduit la variable à `never`,
 * si bien que toute lecture d'un champ dans le bloc script produit
 * « Property ... does not exist on type 'never' ». Vérifié par témoin sur
 * `GdprDataPanel.svelte` : le même accès erroné passe avec la forme
 * générique et échoue sur `never` avec la forme annotée.
 *
 * Le tort n'est pas le faux message : c'est ce qu'il provoque. Puisque
 * déréférencer dans le script « ne compile pas », on n'écrit plus que dans le
 * gabarit, où la vérification est plus lâche. **Le type cesse alors d'être
 * exercé là où il mordrait.**
 *
 * C'est ainsi que `QuoteComparison` a pu diverger de son DTO sans que rien ne
 * l'annonce : le frontend lisait `quotes`, `recommendation` et
 * `complies_with_belgian_law`, le serveur servait `comparison_items`,
 * `total_quotes` et `recommended_quote_id`. Aucun nom ne correspondait, la
 * table de comparaison n'a jamais affiché une ligne, et `svelte-check` — que
 * la CI décrit comme garant du contrat frontend↔backend — n'avait rien à
 * dire.
 *
 * L'écart est étroit et vérifié : `let x: Meeting[] = $state([])` type
 * correctement. Seule la forme nullable initialisée à `null` ou `undefined`
 * se réduit à `never`.
 *
 * Le cliquet est à zéro : c'est une interdiction. Les seize occurrences
 * relevées le 2026-09-09 ont été converties.
 */

const RACINE = join(process.cwd(), "src");

const FORME_FAUTIVE =
  /let\s+(\w+)\s*:\s*([^=\n]+?)\s*=\s*\$state\(\s*(?:null|undefined)\s*\)/g;

function composants(racine: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) trouves.push(...composants(chemin));
    else if (entree.endsWith(".svelte")) trouves.push(chemin);
  }
  return trouves;
}

function declarationsFautives(): string[] {
  const fautives: string[] = [];
  for (const fichier of composants(RACINE)) {
    const source = readFileSync(fichier, "utf-8").replace(
      /<!--[\s\S]*?-->/g,
      "",
    );
    for (const m of source.matchAll(FORME_FAUTIVE)) {
      const ligne = source.slice(0, m.index).split("\n").length;
      fautives.push(
        `  ${relative(process.cwd(), fichier)}:${ligne}\n` +
          `      let ${m[1]}: ${m[2].trim()} = $state(null)\n` +
          `      → let ${m[1]} = $state<${m[2].trim()}>(null)`,
      );
    }
  }
  return fautives;
}

describe("les états nullables sont typés par la forme générique", () => {
  it("ne déclare aucun état nullable par annotation", () => {
    expect(
      declarationsFautives().join("\n"),
      "Ces déclarations ne typent rien : `svelte-check` réduit la variable à " +
        "`never`, ce qui rend toute lecture dans le bloc script impossible et " +
        "pousse le code vers le gabarit, où le type n'est plus exercé.\n\n" +
        "Écrivez `$state<T | null>(null)`.",
    ).toBe("");
  });

  it("voit encore des états dans les composants", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite.
    const source = composants(RACINE)
      .map((f) => readFileSync(f, "utf-8"))
      .join("\n");
    expect(
      (source.match(/\$state[<(]/g) ?? []).length,
      "plus aucun `$state` trouvé : le motif a changé, ou le répertoire a " +
        "bougé. Vérifiez avant de vous réjouir.",
    ).toBeGreaterThan(100);
  });
});
