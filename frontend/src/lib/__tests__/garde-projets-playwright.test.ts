import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

/**
 * Cliquet : un répertoire de recettes n'appartient qu'à UN projet.
 *
 * ── Le défaut ──────────────────────────────────────────────────────────────
 *
 * `playwright.config.ts` déclare `testDir: "./tests/e2e"` pour son projet de
 * bureau, puis quatre projets qui se taillent chacun un sous-répertoire. Le
 * projet de bureau ramasse donc TOUT, sauf ce que son `testIgnore` écarte à la
 * main.
 *
 * Cette liste est tenue de mémoire. Le banc mobile (#869) est arrivé dans
 * `tests/e2e/mobile/` avec sa propre configuration, et le projet de bureau l'a
 * ramassé : ses sept tests ont tourné à 1280×720 contre la démo, où ils n'ont
 * aucun sens. La CI a rougi d'un défaut inexistant, et le vrai signal — le
 * banc lui-même, vert — était noyé dedans.
 *
 * ── Ce que cette garde recalcule ───────────────────────────────────────────
 *
 * Elle relit les `testDir` déclarés dans TOUTES les configurations Playwright
 * du dossier, et exige que chacun de ceux qui vivent sous `tests/e2e/` soit
 * écarté par le projet de bureau. Elle ne fait pas confiance à la liste : elle
 * la reconstruit.
 *
 * `refonte-ux/` n'est revendiqué par aucun projet : il appartient au projet de
 * bureau, et la garde le laisse tranquille. Ce n'est pas une liste blanche de
 * répertoires, c'est une règle sur ceux qui ont un propriétaire ailleurs.
 */

const RACINE = process.cwd();
const PRINCIPALE = "playwright.config.ts";

/** `testDir: "./tests/e2e/x"` → `x`, dans toutes les configurations. */
function repertoiresRevendiques(): string[] {
  const configs = readdirSync(RACINE).filter(
    (f) => f.startsWith("playwright") && f.endsWith(".config.ts"),
  );
  const trouves = new Set<string>();
  for (const config of configs) {
    const source = readFileSync(join(RACINE, config), "utf8");
    for (const m of source.matchAll(
      /testDir:\s*["'`]\.\/tests\/e2e\/([a-z0-9-]+)["'`]/g,
    )) {
      trouves.add(m[1]);
    }
  }
  return [...trouves].sort();
}

/** Le bloc `testIgnore` du projet de bureau, tel quel. */
function ignoresDuBureau(): string {
  const source = readFileSync(join(RACINE, PRINCIPALE), "utf8");
  const bloc = source.match(/testIgnore:\s*\[([\s\S]*?)\]/);
  return bloc ? bloc[1] : "";
}

describe("les projets Playwright ne se marchent pas dessus", () => {
  it("trouve bien des répertoires revendiqués, et ne passe pas faute d'en trouver", () => {
    // Vérification d'aveuglement : si le motif de lecture des `testDir` cesse
    // de correspondre — guillemets différents, chemin réécrit — la règle
    // suivante devient vraie sur une liste vide.
    expect(
      repertoiresRevendiques().length,
      "Aucun `testDir` sous `tests/e2e/` n'a été lu dans les configurations " +
        "Playwright. Le motif de lecture ne correspond plus : corrigez-le, " +
        "un vert obtenu sur zéro répertoire ne dit rien.",
    ).toBeGreaterThan(0);
  });

  it("écarte du projet de bureau tout répertoire revendiqué ailleurs", () => {
    const ignores = ignoresDuBureau();

    // Deuxième aveuglement : sans bloc trouvé, la comparaison porterait sur
    // une chaîne vide et échouerait pour la mauvaise raison.
    expect(
      ignores,
      `Aucun bloc \`testIgnore\` trouvé dans ${PRINCIPALE}. Il a été déplacé ` +
        "ou réécrit : pointez cette garde dessus.",
    ).not.toEqual("");

    const oublies = repertoiresRevendiques().filter(
      (r) => !ignores.includes(r),
    );

    expect(
      oublies,
      `Ces répertoires ont leur propre projet ou leur propre configuration, ` +
        `et le projet de bureau les ramasse quand même :\n  ${oublies.join("\n  ")}\n\n` +
        `Leurs specs tourneront à 1280×720 contre la démo, c'est-à-dire dans ` +
        `des conditions qu'elles ne visent pas. Ajoutez \`/${oublies[0] ?? "x"}\\//\` ` +
        `au \`testIgnore\` du projet \`chromium\`.`,
    ).toEqual([]);
  });
});
