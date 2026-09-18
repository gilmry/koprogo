import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

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

/**
 * Le même défaut, mais pour un lecteur humain.
 *
 * `docs/HUMAN_REVIEW_PLAN_v0.1.0.md` — le plan que suit le relecteur qui
 * signe G1 — a porté pendant des mois, en en-tête :
 *
 *     **URL** : http://localhost (dev) ou https://staging.koprogo.com (staging)
 *
 * Un relecteur qui l'appliquait créait des assemblées générales, envoyait des
 * convocations et enregistrait des procurations **dans la démo publique**, en
 * croyant travailler sur un bac à sable. `staging.koprogo.com` n'existe pas.
 *
 * Les tests ci-dessus protègent la campagne automatique. Celui-ci protège la
 * campagne manuelle, qui n'a ni `PLAYWRIGHT_API_BASE` ni garde-fou : elle a
 * un humain qui fait ce que le document dit.
 *
 * Il ne lit QUE les blocs de commandes et les en-têtes d'adresse — nommer le
 * port 80 dans une phrase pour expliquer le danger reste permis, et c'est ce
 * que font l'ADR 0050, RELEASE.md et le guide E2E.
 */
const RACINE_DEPOT = join(process.cwd(), "..");

/** Les répertoires dont le contenu décrit un état passé. */
const MATIERE_HISTORIQUE = [
  "archives",
  "archive",
  "github-export",
  "cowork",
  "maury",
  "agent-activity",
  "plans",
  "_build",
  "node_modules",
];

function documentsVivants(racine: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    if (MATIERE_HISTORIQUE.includes(entree)) continue;
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory())
      trouves.push(...documentsVivants(chemin));
    else if (/\.(rst|md)$/.test(entree)) trouves.push(chemin);
  }
  return trouves;
}

function blocsDeCommandes(texte: string, extension: string): string[] {
  if (extension === "md") {
    return [
      ...texte.matchAll(/```(?:bash|sh|shell|console|text)?\n([\s\S]*?)```/g),
    ].map((m) => m[1]);
  }
  return [
    ...texte.matchAll(
      /\.\. code-block:: (?:bash|sh|shell|console|text)\n\n((?:(?:[ \t]+.*)?\n)+)/g,
    ),
  ].map((m) => m[1]);
}

/** `http://localhost` sans port : sur cet hôte, c'est le Traefik de la démo. */
const PORT_80_NU = /http:\/\/localhost(?![:0-9])/;

describe("la documentation n'envoie pas un humain sur la démo", () => {
  it("@security aucun bloc de commandes ne vise http://localhost nu", () => {
    const fautifs: string[] = [];
    const fichiers = [
      ...documentsVivants(join(RACINE_DEPOT, "docs")),
      ...documentsVivants(join(RACINE_DEPOT, ".claude")),
      ...readdirSync(RACINE_DEPOT)
        .filter((f) => /\.md$/.test(f) && f !== "CHANGELOG.md")
        .map((f) => join(RACINE_DEPOT, f)),
    ];

    for (const chemin of fichiers) {
      const texte = readFileSync(chemin, "utf-8");
      const extension = chemin.endsWith(".md") ? "md" : "rst";
      for (const bloc of blocsDeCommandes(texte, extension)) {
        for (const ligne of bloc.split("\n")) {
          if (PORT_80_NU.test(ligne)) {
            fautifs.push(`${relative(RACINE_DEPOT, chemin)} : ${ligne.trim()}`);
          }
        }
      }
    }

    expect(
      fautifs.sort(),
      "Le port 80 de cet hôte est le Traefik de la DÉMO. Une commande " +
        "documentée qui le vise fait écrire un lecteur dans la base " +
        "vivante. La recette est sur 8090 (ADR 0050).",
    ).toEqual([]);
  });

  it("@security le plan de revue humaine désigne la recette, pas la démo", () => {
    const plan = readFileSync(
      join(RACINE_DEPOT, "docs", "HUMAN_REVIEW_PLAN_v0.1.0.md"),
      "utf-8",
    );
    const enTete = plan.split("\n").slice(0, 12).join("\n");
    expect(enTete).toContain("localhost:8090");
    expect(
      PORT_80_NU.test(enTete),
      "L'en-tête du plan de revue ne doit pas donner le port 80 nu comme " +
        "adresse de travail : c'est la démo.",
    ).toBe(false);
  });

  it("@security le plan de revue ne donne aucun mot de passe en clair", () => {
    const plan = readFileSync(
      join(RACINE_DEPOT, "docs", "HUMAN_REVIEW_PLAN_v0.1.0.md"),
      "utf-8",
    );
    // `admin123` a tourné au déploiement : le donner en clair fait perdre
    // une session au relecteur avant qu'il comprenne pourquoi il ne peut
    // pas se connecter.
    expect(plan).not.toContain("admin123");
    expect(plan).toContain("KOPROGO_SUPERADMIN_PASSWORD");
  });
});
