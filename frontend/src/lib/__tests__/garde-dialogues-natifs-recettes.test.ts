import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Une recette n'installe pas de gestionnaire pour un dialogue qui n'existe
 * plus.
 *
 * #844 a remplacé les soixante `confirm()` et `alert()` natifs du produit par
 * de vraies modales. `garde-dialogues-natifs` borne leur nombre à zéro dans
 * `src`. Mais les recettes, elles, ont gardé leurs
 * `page.on("dialog", d => d.accept())` — onze au total.
 *
 * Un gestionnaire pour un événement qui ne survient plus est pire que du code
 * mort : **il donne l'illusion que le geste est traité.** Le clic ouvre la
 * modale, personne ne confirme, et l'échec survient bien plus loin. Mesuré
 * sur le run du 2026-09-09 :
 *
 *   - `story1-admin-buttons` attendait « annulé » et lisait « En retard » ;
 *   - `AccountantInvoiceWorkflowJourney` et `SyndicDocumentsJourney`
 *     expiraient sur `waitForResponse`, la requête ne partant jamais.
 *
 * Le remplaçant est `confirmerSiDemande(page)`, appelé APRÈS le clic. Quand
 * l'attente est armée avant le clic (`Promise.all`), il faut défaire le
 * `Promise.all` : la confirmation vient entre le clic et la réponse, et le
 * couple ne laisse aucune place à ce geste.
 *
 * Le cliquet borne la dette : onze au relevé, sept après correction des
 * quatre recettes en échec. Les quatre restants sont dans des scénarios qui
 * PASSENT — les toucher risquerait une régression sans rien gagner, mais ils
 * restent du code mort à retirer.
 */

const RACINE = join(process.cwd(), "tests/e2e");

/** Le décompte relevé le 2026-09-09, après correction des quatre en échec. */
const DETTE_AU_2026_09_09 = 6;

const GESTIONNAIRE = /page\.(?:on|once)\(\s*["']dialog["']/g;

function recettes(racine: string): string[] {
  const trouvees: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) trouvees.push(...recettes(chemin));
    else if (entree.endsWith(".spec.ts") || entree.endsWith(".scenario.ts"))
      trouvees.push(chemin);
  }
  return trouvees;
}

function gestionnaires(): string[] {
  const trouves: string[] = [];
  for (const fichier of recettes(RACINE)) {
    const code = readFileSync(fichier, "utf-8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\/\/[^\n]*/g, "");
    for (const m of code.matchAll(GESTIONNAIRE)) {
      const ligne = code.slice(0, m.index).split("\n").length;
      trouves.push(`  ${relative(process.cwd(), fichier)}:${ligne}`);
    }
  }
  return trouves;
}

describe("les recettes n'attendent plus de dialogue natif (#844)", () => {
  it("n'ajoute pas de gestionnaire pour un dialogue supprimé", () => {
    const trouves = gestionnaires();
    expect(
      trouves.length,
      `${trouves.length} gestionnaires de dialogue natif, contre ` +
        `${DETTE_AU_2026_09_09} au 2026-09-09.\n\n` +
        `#844 a remplacé les dialogues natifs par des modales : ces ` +
        `gestionnaires ne se déclenchent plus, et laissent croire que le ` +
        `geste est traité. Le clic ouvre la modale, personne ne confirme, et ` +
        `l'échec survient ailleurs.\n\n` +
        `Appelez \`confirmerSiDemande(page)\` après le clic.\n\n` +
        trouves.join("\n"),
    ).toBeLessThanOrEqual(DETTE_AU_2026_09_09);
  });

  it("lit encore les recettes", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite.
    const fichiers = recettes(RACINE);
    expect(fichiers.length).toBeGreaterThan(50);
    const clics = fichiers
      .map((f) => readFileSync(f, "utf-8"))
      .join("\n")
      .match(/\.click\(/g);
    expect(
      clics?.length ?? 0,
      "plus aucun clic dans les recettes : le répertoire a bougé.",
    ).toBeGreaterThan(100);
  });
});
