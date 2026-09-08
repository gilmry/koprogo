import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Une ancre ne se construit jamais depuis un texte traduit.
 *
 * C'est le pari sur la langue, mais DÉGUISÉ. Un lecteur qui voit
 * `getByTestId("nav-link-immeubles")` se croit à l'abri : l'ancre a la forme
 * d'un identifiant stable. Elle ne l'est pas.
 *
 * `RoleSubmenu.svelte` construisait `nav-link-{slugify(item.label)}` à partir
 * du libellé traduit. La même entrée de menu rendait donc :
 *
 *     fr   nav-link-immeubles
 *     en   nav-link-buildings
 *     nl   nav-link-gebouwen
 *
 * Dix-huit sélecteurs de recette en dépendaient, à travers dix fichiers de
 * scénario. Ils ne tenaient que parce que les quatre projets Playwright sont
 * épinglés à `fr-BE`. L'en-tête du composant promettait pourtant un
 * `stableSlug`, et une spec avait déjà diagnostiqué le problème dans un
 * commentaire, en le contournant à un seul endroit.
 *
 * Le libellé produisait en prime des collisions, parce que trois clés i18n
 * servent deux écrans chacune : `nav-link-lots` désignait `/units` ET
 * `/owner/units`. C'est le piège de #832, où une ancre posée sur le bon nom
 * mais le mauvais écran a fait échouer quarante fois de suite.
 *
 * Corrigé en dérivant l'ancre de l'`href`, qui ne dépend d'aucune locale et
 * distingue les écrans par construction.
 *
 * Le cliquet est à zéro : c'est une interdiction. Une ancre traduite ne se
 * découvre jamais à l'usage tant qu'on teste dans une seule langue.
 */

const RACINE = join(process.cwd(), "src");

/**
 * Ce qui, dans une interpolation d'ancre, trahit un texte destiné à l'écran.
 * `$_(` et `t(` sont les deux appels de traduction du dépôt ; `label`,
 * `title`, `name` et `libelle` sont les champs qui les portent.
 */
const TEXTE_AFFICHE = /\$_\(|\bt\(|\.label\b|\.title\b|\.name\b|\.libelle\b/;

function gabarits(racine: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) trouves.push(...gabarits(chemin));
    else if (entree.endsWith(".svelte") || entree.endsWith(".astro"))
      trouves.push(chemin);
  }
  return trouves;
}

function ancresInterpolees(): { ou: string; ancre: string }[] {
  const toutes: { ou: string; ancre: string }[] = [];
  for (const fichier of gabarits(RACINE)) {
    const source = readFileSync(fichier, "utf-8")
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\*[\s\S]*?\*\//g, "");
    for (const m of source.matchAll(/data-testid="([^"]*\{[^"]*\})"/g)) {
      const ligne = source.slice(0, m.index).split("\n").length;
      toutes.push({
        ou: `${relative(process.cwd(), fichier)}:${ligne}`,
        ancre: m[1],
      });
    }
  }
  return toutes;
}

describe("les ancres ne dépendent pas de la langue (#803, #834)", () => {
  it("ne construit aucune ancre depuis un texte affiché", () => {
    const fautives = ancresInterpolees().filter((a) =>
      TEXTE_AFFICHE.test(a.ancre),
    );
    expect(
      fautives.map((a) => `  ${a.ou}\n      ${a.ancre}`).join("\n"),
      "Ces ancres se construisent depuis un texte destiné à l'écran. Elles " +
        "changeront avec la langue, et les recettes qui les visent ne " +
        "tiendront que tant qu'un seul projet Playwright existe.\n\n" +
        "Dérivez l'ancre d'une valeur stable : l'`href`, un identifiant, une " +
        "clé i18n — jamais sa traduction.",
    ).toBe("");
  });

  it("voit encore des ancres interpolées", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite. Le dépôt
    // construit 26 préfixes dynamiques (`data-testid.contrat.json`) ; si le
    // détecteur n'en voyait plus aucun, il aurait cessé de lire.
    expect(
      ancresInterpolees().length,
      "plus aucune ancre interpolée : le motif a changé, ou le répertoire a " +
        "bougé. Vérifiez avant de vous réjouir.",
    ).toBeGreaterThan(20);
  });
});
