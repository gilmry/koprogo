import { describe, expect, it } from "vitest";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Une recette n'appelle pas une route que le serveur ne sert pas.
 *
 * `poll-vote.scenario.ts` publiait son sondage par `PUT /polls/{id}/publish`.
 * La route est déclarée `#[post("/polls/{id}/publish")]`. Le PUT rendait 404,
 * le sondage restait en brouillon, et le scénario échouait cent lignes plus
 * loin sur `poll-card` introuvable — en accusant l'affichage d'une liste vide.
 *
 * Le défaut est invisible à la lecture : `PUT` et `POST` se ressemblent, et
 * le 404 n'était vérifié nulle part (aucune des 55 réponses d'amorçage ne
 * l'était). Un rapprochement statique le voit en une seconde.
 *
 * Le cliquet est à zéro : c'est une interdiction. Il n'y avait qu'une seule
 * infraction dans toute la suite, et elle a coûté un scénario entier.
 */

const RACINE_E2E = join(process.cwd(), "tests/e2e");
const HANDLERS = join(
  process.cwd(),
  "../backend/src/infrastructure/web/handlers",
);

/** `/polls/{id}/publish` et `/polls/{pollId}/publish` désignent la même route. */
function normalise(chemin: string): string {
  return chemin
    .replace(/\$\{[^}]+\}/g, "{}") // interpolation TypeScript
    .replace(/\{[^}]+\}/g, "{}") // paramètre Actix
    .split("?")[0]
    .replace(/\/+$/, "");
}

function fichiers(racine: string, garder: (f: string) => boolean): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory())
      trouves.push(...fichiers(chemin, garder));
    else if (garder(entree)) trouves.push(chemin);
  }
  return trouves;
}

/** Les routes déclarées par les macros d'Actix, verbe par verbe. */
function routesServies(): Map<string, Set<string>> {
  const routes = new Map<string, Set<string>>();
  for (const f of fichiers(HANDLERS, (e) => e.endsWith(".rs"))) {
    const source = readFileSync(f, "utf-8");
    for (const m of source.matchAll(
      /#\[(get|post|put|delete|patch)\("([^"]+)"\)\]/g,
    )) {
      const cle = normalise(m[2]);
      routes.set(cle, (routes.get(cle) ?? new Set()).add(m[1]));
    }
  }
  return routes;
}

function appelsFautifs(): string[] {
  const routes = routesServies();
  const fautifs: string[] = [];
  for (const f of fichiers(RACINE_E2E, (e) => e.endsWith(".ts"))) {
    const source = readFileSync(f, "utf-8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\/\/[^\n]*/g, "");
    for (const m of source.matchAll(
      /request\.(get|post|put|delete|patch)\(\s*`\$\{API_BASE\}([^`]*)`/g,
    )) {
      const verbe = m[1];
      const url = normalise(m[2]);
      const servis = routes.get(url);
      // Une URL inconnue n'est PAS une infraction : le rapprochement ne
      // couvre pas les routes déclarées ailleurs qu'en macro (scopes,
      // `.service(web::resource(...))`). On ne signale que le cas certain —
      // la route existe, et le verbe employé n'est pas celui qu'elle sert.
      if (servis && !servis.has(verbe)) {
        const ligne = source.slice(0, m.index).split("\n").length;
        fautifs.push(
          `  ${relative(process.cwd(), f)}:${ligne}\n` +
            `      ${verbe.toUpperCase()} ${url} — le serveur sert ` +
            `${[...servis].map((v) => v.toUpperCase()).join(", ")}`,
        );
      }
    }
  }
  return fautifs;
}

describe("les recettes appellent des routes qui existent (#832)", () => {
  it("n'emploie aucun verbe que la route ne sert pas", () => {
    expect(
      appelsFautifs().join("\n"),
      "Ces appels de recette emploient un verbe HTTP que la route ne sert " +
        "pas. Le serveur rend 404, la donnée n'est pas créée, et le test " +
        "échoue bien plus loin en accusant l'affichage.",
    ).toBe("");
  });

  it("lit encore les deux côtés", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite. Il vérifie
    // que le rapprochement a bien deux côtés à rapprocher.
    expect(
      existsSync(HANDLERS),
      "le répertoire des handlers du backend a bougé : la garde ne compare " +
        "plus rien.",
    ).toBe(true);
    expect(
      routesServies().size,
      "plus aucune route déclarée par macro Actix : le motif a changé.",
    ).toBeGreaterThan(100);
    const appels = fichiers(RACINE_E2E, (e) => e.endsWith(".ts"))
      .map((f) => readFileSync(f, "utf-8"))
      .join("\n")
      .match(/request\.(get|post|put|delete|patch)\(/g);
    expect(
      appels?.length ?? 0,
      "plus aucun appel API dans les recettes : le répertoire a bougé.",
    ).toBeGreaterThan(50);
  });
});
