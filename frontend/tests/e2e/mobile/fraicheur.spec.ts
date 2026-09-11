import { test, expect } from "@playwright/test";
import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Le banc sert `dist/`. Encore faut-il que `dist/` soit à jour.
 *
 * ── Ce qui s'est passé ─────────────────────────────────────────────────────
 *
 * Le serveur du banc sert des fichiers, il ne compile pas. Un `npm run build`
 * qui échoue laisse donc en place le `dist/` PRÉCÉDENT, et le banc continue
 * de tourner, vert ou rouge, sur du code qui n'est plus le vôtre.
 *
 * C'est arrivé pendant l'écriture de ce banc : une erreur de type dans une
 * spec faisait échouer `astro check`, donc le build entier. Trois exécutions
 * de suite ont rapporté un défaut déjà corrigé — le catalogue contenait la
 * clé, le bundle servi ne la contenait pas.
 *
 * En CI le risque n'existe pas : l'étape de build précède et fait échouer le
 * job. En local, rien ne le disait.
 *
 * ── Pourquoi un test plutôt qu'un `&&` dans la commande du serveur ────────
 *
 * Parce qu'un `npm run build &&` avant chaque lancement coûterait une minute à
 * chaque fois, y compris quand rien n'a bougé. Ce contrôle-ci coûte quelques
 * millisecondes et dit exactement ce qu'il faut faire.
 */

function plusRecent(racine: string, apres = 0): number {
  let max = apres;
  for (const entree of readdirSync(racine)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(racine, entree);
    const infos = statSync(chemin);
    max = infos.isDirectory()
      ? plusRecent(chemin, max)
      : Math.max(max, infos.mtimeMs);
  }
  return max;
}

test("@edge `dist/` est plus récent que `src/`", () => {
  const racine = process.cwd();
  const source = plusRecent(join(racine, "src"));
  const construit = plusRecent(join(racine, "dist"));

  const retard = Math.round((source - construit) / 1000);
  expect(
    construit,
    `\`dist/\` a ${retard} s de retard sur \`src/\`.\n\n` +
      `Le banc sert des fichiers, il ne compile pas : il éprouve donc une ` +
      `version périmée du produit, et tout ce qu'il rapporte — vert comme ` +
      `rouge — parle d'un autre code que le vôtre.\n\n` +
      `Lancez \`npm run build\` et regardez son code de sortie : \`astro check\` ` +
      `analyse aussi \`tests/\`, si bien qu'une erreur de type dans une spec ` +
      `suffit à laisser l'ancien \`dist/\` en place.`,
  ).toBeGreaterThanOrEqual(source);
});
