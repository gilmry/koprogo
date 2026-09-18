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
 *
 * ── Ce que ce nombre N'EST PAS (#938) ──────────────────────────────────────
 *
 * **C'est une borne INFÉRIEURE, pas une mesure.** Deux angles morts connus,
 * tous deux mesurés le 2026-09-18 :
 *
 * 1. **Les composants appellent l'API en direct.** Ce test ne lit que
 *    `src/lib/api/*.ts`. Or **79 appels `api.get/post/put/patch/delete`
 *    vivent dans 39 fichiers `.svelte` / `.astro`**, et aucun n'est examiné.
 *    C'est là qu'était `OnboardingWizard.svelte:465`, qui appelle une route
 *    `/acps/{id}/modules/{m}/enable` que le serveur ne sert pas.
 *
 * 2. **Les gabarits à deux interpolations échappent au détecteur.** Il ne
 *    relève que `/acps/{x}/modules`, à une interpolation.
 *
 * Un chiffre qui sous-compte se lit comme un accord sur ce qu'il n'a pas vu.
 * D'où le second cliquet ci-dessous : il ne répare pas l'angle mort, il
 * **refuse qu'il grandisse**. Le réparer vraiment demande que les routes
 * manquantes existent d'abord — #585, puis #938.
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
        `⚠️ Ce nombre est une BORNE INFÉRIEURE : seuls les appels de ` +
        `src/lib/api/*.ts sont examinés. Les appels directs depuis les ` +
        `composants ne le sont pas (#938).\n\n` +
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

/**
 * Le second cliquet : l'angle mort ne grandit pas.
 *
 * Il ne répare rien — un appel depuis un composant reste invisible au
 * recensement ci-dessus. Mais il transforme un angle mort **inconnu** en
 * angle mort **mesuré**, et refuse qu'on en ajoute.
 *
 * Pourquoi c'est la bonne borne à poser aujourd'hui : relever le détecteur
 * pour qu'il lise les composants ferait bondir le premier cliquet sans
 * qu'une seule route manquante ait été écrite. #938 le dit — « relever le
 * détecteur sans traiter les routes manquantes ne ferait qu'agrandir le même
 * rouge ». Ce qu'on peut faire sans rien agrandir, c'est empêcher la dette
 * de croître pendant qu'on attend #585.
 *
 * La bonne façon d'appeler l'API depuis un composant est de passer par un
 * module de `src/lib/api/` : le chemin y devient visible à la garde, et
 * l'appel devient testable sans monter le composant.
 */
const APPELS_DEPUIS_LES_COMPOSANTS_AU_2026_09_18 = 79;

function appelsDirectsDepuisLesComposants(): string[] {
  const racines = [
    join(process.cwd(), "src/components"),
    join(process.cwd(), "src/pages"),
  ];
  const trouves: string[] = [];

  const parcourir = (dossier: string): void => {
    for (const entree of readdirSync(dossier, { withFileTypes: true })) {
      const chemin = join(dossier, entree.name);
      if (entree.isDirectory()) {
        parcourir(chemin);
      } else if (/\.(svelte|astro)$/.test(entree.name)) {
        const contenu = readFileSync(chemin, "utf8");
        for (const m of contenu.matchAll(
          /\bapi\.(get|post|put|patch|delete)\s*\(/g,
        )) {
          trouves.push(
            `${chemin.replace(process.cwd() + "/", "")} — api.${m[1]}`,
          );
        }
      }
    }
  };

  for (const racine of racines) parcourir(racine);
  return trouves;
}

describe("l'angle mort de la garde ne grandit pas (#938)", () => {
  it("@edge le nombre d'appels API directs depuis les composants ne monte pas", () => {
    const directs = appelsDirectsDepuisLesComposants();

    expect(
      directs.length,
      `${directs.length} appels à l'API vivent directement dans des ` +
        `composants, contre ${APPELS_DEPUIS_LES_COMPOSANTS_AU_2026_09_18} ` +
        `au 2026-09-18.\n\n` +
        `Ces appels sont INVISIBLES au recensement de routes orphelines ` +
        `ci-dessus, qui ne lit que src/lib/api/*.ts. En ajouter un revient ` +
        `à ajouter une route potentiellement absente sans qu'aucune garde ` +
        `ne puisse le voir — c'est ainsi que ` +
        `OnboardingWizard.svelte:465 a pu appeler pendant des mois une ` +
        `route que le serveur ne sert pas.\n\n` +
        `Passez par un module de src/lib/api/ : le chemin y redevient ` +
        `visible, et l'appel testable sans monter le composant.\n\n` +
        directs.slice(0, 20).join("\n"),
    ).toBeLessThanOrEqual(APPELS_DEPUIS_LES_COMPOSANTS_AU_2026_09_18);
  });

  it("@edge le recensement lit encore des composants", () => {
    // Un chemin qui change rendrait ce cliquet vert par vacuité.
    expect(appelsDirectsDepuisLesComposants().length).toBeGreaterThan(50);
  });
});
