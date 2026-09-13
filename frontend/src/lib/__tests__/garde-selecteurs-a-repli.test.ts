import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Cliquet : un test e2e ne se rabat pas sur autre chose quand son ancrage
 * manque.
 *
 * ── Ce qui a été trouvé ────────────────────────────────────────────────────
 *
 * Relevé du 2026-09-07, en vérifiant que les ancrages posés avant la refonte
 * servent à quelque chose. Cinquante-neuf sélections, dans 38 specs sur 100,
 * avaient cette forme :
 *
 *     page.locator("main h1, main h2, [data-testid='bookings-list']").first()
 *
 * Si `bookings-list` disparaît ou change de cible, **`main h1` prend le
 * relais** — n'importe quel titre de la page. Le test passe.
 *
 * Il affirmait « la liste des réservations s'affiche ». Il vérifiait « la page
 * a un titre ».
 *
 * **Trente-cinq des quarante-quatre ancrages cités n'existaient pas.** C'est
 * le repli qui faisait passer ces tests, et qui masquait leur absence.
 *
 * ── Pourquoi cela visait précisément la refonte ────────────────────────────
 *
 * Le contrat `data-testid` (#802) fige 923 identifiants : aucun ne peut
 * disparaître sans qu'on modifie `data-testid.contrat.json` en connaissance de
 * cause. Cette garde tient.
 *
 * Mais elle ne dit rien de ce que l'ancrage **désigne**. Un identifiant
 * conservé, déplacé sur un autre élément, passe le contrat — et ces 38 specs
 * ne s'en apercevaient pas non plus.
 *
 * Les neuf lots de maquette vont déplacer la barre latérale, les quatre
 * tableaux de bord, le motif de liste et le périmètre applicatif. C'est
 * exactement le moment où un ancrage change de cible sans disparaître.
 *
 * Voir #830.
 */

const RACINE = join(process.cwd(), "tests");

/**
 * Un sélecteur qui contient un `data-testid` **et** une alternative.
 *
 * La virgule est le séparateur de CSS : tout ce qui suit est un repli.
 */
const A_REPLI = /locator\(\s*[`"']([^`"']*data-testid[^`"']*)[`"']/g;

function specs(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...specs(chemin));
      // `.scenario.ts` autant que `.spec.ts` : les douze fichiers du projet
      // `scenarios` échappaient au relevé.
    } else if (entree.endsWith(".spec.ts") || entree.endsWith(".scenario.ts")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

/**
 * Retire les commentaires avant de chercher.
 *
 * Celui qui explique le défaut cite le sélecteur fautif ; le compter
 * reviendrait à signaler la documentation du correctif comme le défaut.
 * `garde-secrets-console` et `piege-de-focus` ont le même réglage.
 */
const sansCommentaires = (s: string) =>
  s.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/[^\n]*/g, "");

describe("aucun test e2e ne se rabat sur un titre (#830)", () => {
  it("n'emploie pas de sélecteur à repli contenant un data-testid", () => {
    const fautes: string[] = [];

    for (const chemin of specs(RACINE)) {
      const source = sansCommentaires(readFileSync(chemin, "utf8"));
      for (const m of source.matchAll(A_REPLI)) {
        if (m[1].includes(",")) {
          fautes.push(
            `${chemin.replace(process.cwd() + "/", "")} — ${m[1].slice(0, 70)}`,
          );
        }
      }
    }

    expect(
      fautes,
      `${fautes.length} sélecteur(s) e2e se rabattent sur autre chose quand ` +
        `leur ancrage manque.\n\n` +
        `Le test passe et ne vérifie pas ce qu'il annonce. C'est le motif ` +
        `dominant de ce produit appliqué à sa propre vérification (#830).\n\n` +
        `Deux issues seulement : poser l'ancrage manquant, ou dire que le ` +
        `test ne peut pas vérifier ce qu'il prétend. Pas de troisième voie.\n\n` +
        fautes.join("\n"),
    ).toEqual([]);
  });

  /// Sans ce contrôle, un répertoire renommé rendrait le cliquet
  /// silencieusement vert — le piège déjà posé dans les douze autres.
  it("trouve encore les specs qu'il surveille", () => {
    expect(
      specs(RACINE).length,
      "plus aucune spec e2e recensée",
    ).toBeGreaterThan(80);
  });
});
