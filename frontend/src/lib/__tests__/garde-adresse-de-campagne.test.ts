import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * La campagne e2e ne doit jamais retomber sur le port 80.
 *
 * ── Ce qui est arrivé ─────────────────────────────────────────────────────
 *
 * `const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1"`
 * était recopié dans **93 fichiers**. Sur l'hôte qui porte la démo, ce défaut
 * ne désigne pas le backend local : le port 80 y est tenu par le Traefik de la
 * démo, qui route vers `Host(api.koprogo.com)`.
 *
 * Le 2026-09-12, au premier lancement de la pile de recette, `make test-e2e`
 * n'exportait que `PLAYWRIGHT_BASE_URL`. Les 93 fichiers sont retombés sur le
 * 80 et **57 specs sur 106 ont échoué**, toutes à l'amorçage.
 *
 * Ce qui a protégé la démo n'est pas une garde, c'est un hasard : le port 80
 * rend `301 → https://localhost`, qui n'aboutit pas. Si ce Traefik avait servi
 * la requête, la campagne aurait écrit ses organisations, ses immeubles et ses
 * comptes dans la base vivante.
 *
 * Cette garde existe pour que « une chance » devienne « une règle » (#872).
 */

const E2E = join(process.cwd(), "tests", "e2e");
const SOURCE_UNIQUE = join(E2E, "helpers", "adresses.ts");

/** Tous les `.ts` sous `tests/e2e`, récursivement. */
function fichiersTs(racine: string): string[] {
  const sortie: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) {
      if (entree === "node_modules") continue;
      sortie.push(...fichiersTs(chemin));
    } else if (entree.endsWith(".ts")) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

describe("la campagne e2e vise la recette, jamais la démo", () => {
  it("@security aucun fichier ne retombe sur le port 80 pour l'API", () => {
    const coupables = fichiersTs(E2E)
      .filter((f) => f !== SOURCE_UNIQUE)
      .filter((f) => readFileSync(f, "utf-8").includes("http://localhost/api"));

    expect(
      coupables.map((f) => f.replace(process.cwd() + "/", "")),
      `Ces fichiers désignent l'API sur le port 80 :\n\n` +
        coupables.map((f) => `  ${f}`).join("\n") +
        `\n\nSur un hôte qui porte la démo, le 80 est tenu par SON Traefik. ` +
        `La campagne y amorcerait son monde — organisations, immeubles, ` +
        `comptes — dans les données vivantes.\n\n` +
        `Importez plutôt la constante partagée :\n` +
        `    import { API_BASE } from "…/helpers/adresses";\n`,
    ).toEqual([]);
  });

  it("@edge la source unique existe et porte bien le port de la recette", () => {
    const source = readFileSync(SOURCE_UNIQUE, "utf-8");
    expect(source).toContain("PLAYWRIGHT_API_BASE");
    expect(
      source,
      "La source unique doit désigner la pile de recette (8090), sans quoi " +
        "centraliser n'aurait fait que déplacer le défaut.",
    ).toContain("http://localhost:8090/api/v1");
  });

  it("@edge lit bien les fichiers, et n'est pas verte par vacuité", () => {
    const tous = fichiersTs(E2E);
    expect(
      tous.length,
      "aucun fichier .ts lu sous tests/e2e — le chemin a dû changer",
    ).toBeGreaterThan(50);
    // Le motif cherché doit exister quelque part, sans quoi la recherche
    // passerait sur une chaîne que plus personne n'écrit.
    expect(
      tous.some((f) =>
        readFileSync(f, "utf-8").includes("PLAYWRIGHT_API_BASE"),
      ),
    ).toBe(true);
  });
});
