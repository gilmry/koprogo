import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

/**
 * Cliquet : le frontend n'invente pas de routes que le serveur ne sert pas.
 *
 * ── Ce qui a été trouvé ────────────────────────────────────────────────────
 *
 * Recette du 2026-09-04, finding R4-3 : « sept modules communautaires sur huit
 * n'ont pas de backend ». J'ai d'abord confirmé ce diagnostic, et **il était
 * faux** : les six modules sont complets côté serveur, avec 111 points d'entrée
 * tous enregistrés. J'avais sondé des chemins portés par l'ACP quand le serveur
 * les sert par l'immeuble (#768, fermée avec la correction du diagnostic).
 *
 * Le vrai défaut est un **désaccord de chemin**, et il subsiste : au relevé du
 * 2026-09-06, trente-quatre appels du frontend ne correspondent à aucune route
 * servie. Deux abstractions inventées par le frontend en font l'essentiel :
 *
 *     /bookable-resources/…     le serveur sert /resource-bookings
 *     /loans/…                  le serveur sert /shared-objects/{id}/borrow
 *
 * ── Pourquoi rien ne le voyait ─────────────────────────────────────────────
 *
 * Un appel vers une route absente rend `404`. Le composant affiche « aucune
 * donnée » ou reste en chargement, et **aucun test ne distingue une liste vide
 * d'une route inexistante**. C'est le motif dominant du produit sous une forme
 * de plus : une capacité écrite des deux côtés, et qui ne se rencontre jamais.
 *
 * Ces 404 en rafale sont aussi ce qui a fait bannir les testeurs par CrowdSec
 * (#766) : le défaut ne coûtait pas seulement une fonctionnalité, il rendait la
 * recette impossible.
 *
 * ── Ce que ce test fait ────────────────────────────────────────────────────
 *
 * Il compare les chemins appelés par `lib/api/*.ts` aux chemins déclarés par
 * les attributs de route du backend, et **borne l'écart**. Le nombre ne peut
 * que baisser.
 *
 * Il ne peut pas dire lequel des deux côtés a raison — c'est un arbitrage
 * produit, suivi en #779. Il peut refuser que l'écart grandisse.
 */

/**
 * Appels du frontend sans route correspondante. **Ne doit que BAISSER.**
 *
 * 46 au relevé du 2026-09-06, tous modules confondus. Réparation suivie en #779.
 */
const ORPHELINS_AU_2026_09_06 = 46;

const API_FRONT = join(process.cwd(), "src/lib/api");
const HANDLERS = join(
  process.cwd(),
  "../backend/src/infrastructure/web/handlers",
);

/** Les paramètres sont neutralisés : seule la forme du chemin compte. */
const neutraliser = (chemin: string) =>
  chemin
    .replace(/\$\{[^}]+\}/g, "{x}")
    .replace(/\{[^}]+\}/g, "{x}")
    .split("?")[0];

function cheminsAppeles(): Set<string> {
  const appels = new Set<string>();
  for (const fichier of readdirSync(API_FRONT)) {
    if (!fichier.endsWith(".ts") || fichier.includes(".test.")) continue;
    const source = readFileSync(join(API_FRONT, fichier), "utf8");
    for (const m of source.matchAll(/api\.\w+(?:<[^>]*>)?\(\s*`([^`]+)`/g)) {
      appels.add(neutraliser(m[1]));
    }
    for (const m of source.matchAll(/api\.\w+(?:<[^>]*>)?\(\s*"(\/[^"]+)"/g)) {
      appels.add(neutraliser(m[1]));
    }
  }
  return appels;
}

function cheminsServis(): Set<string> {
  const servis = new Set<string>();
  for (const fichier of readdirSync(HANDLERS)) {
    if (!fichier.endsWith(".rs")) continue;
    const source = readFileSync(join(HANDLERS, fichier), "utf8");
    for (const m of source.matchAll(
      /#\[(?:get|post|put|patch|delete)\("([^"]+)"\)\]/g,
    )) {
      servis.add(neutraliser(m[1]));
    }
  }
  return servis;
}

describe("le frontend n'invente pas de routes (#779)", () => {
  it("ne laisse pas grandir le nombre d'appels sans route servie", () => {
    const appeles = cheminsAppeles();
    const servis = cheminsServis();
    const orphelins = [...appeles].filter((c) => !servis.has(c)).sort();

    expect(
      orphelins.length,
      `${orphelins.length} appels du frontend ne correspondent à aucune route ` +
        `servie, contre ${ORPHELINS_AU_2026_09_06} au 2026-09-06.\n\n` +
        `Un appel vers une route absente rend 404. Le composant affiche ` +
        `« aucune donnée » ou reste en chargement, et aucun test ne distingue ` +
        `une liste vide d'une route inexistante. Ces 404 en rafale sont aussi ` +
        `ce qui a fait bannir les testeurs par CrowdSec (#766).\n\n` +
        `Soit le chemin du frontend est faux, soit la route manque au serveur. ` +
        `Ce test ne peut pas trancher — c'est #779 — mais il refuse que ` +
        `l'écart grandisse.\n\n` +
        orphelins.join("\n"),
    ).toBeLessThanOrEqual(ORPHELINS_AU_2026_09_06);
  });

  /// Sans ce contrôle, un recensement qui ne trouve plus rien rendrait le
  /// cliquet silencieusement vert — le piège déjà posé dans les cinq autres.
  it("reconnaît encore les deux côtés", () => {
    expect(cheminsAppeles().size, "plus aucun appel reconnu").toBeGreaterThan(
      200,
    );
    expect(cheminsServis().size, "plus aucune route reconnue").toBeGreaterThan(
      400,
    );
  });
});
