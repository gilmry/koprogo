import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Aucun alias DNS générique dans une chaîne de connexion d'un compose de prod.
 *
 * ── Ce qui est arrivé ──────────────────────────────────────────────────
 *
 * `koprogo-backend` est attaché à deux réseaux Docker : `koprogo_koprogo-network`
 * et le réseau partagé `ecosolva-web`, où quatre projets de l'hôte (koprogo,
 * taginy, elevia, derniere-chance) déclarent chacun un service `backend`,
 * `frontend`, ou `minio`. Compose EXPOSE TOUJOURS le nom du service comme
 * alias DNS sur chaque réseau auquel le conteneur est attaché — il n'existe
 * aucun réglage pour l'en empêcher.
 *
 * Du 2026-08-25 21:47 UTC au 2026-08-29 11:30 UTC (2538 redémarrages),
 * `S3_ENDPOINT` pointait sur l'alias générique `minio`. Sur `ecosolva-web`,
 * ce nom résolvait vers `derniere-chance-minio` (172.18.0.7) plutôt que
 * `koprogo-minio` — la résolution DNS de Docker sur un conteneur
 * multi-réseaux ne garantit aucun ordre. Les identifiants koprogo n'étaient
 * pas valides chez le voisin : le bucket check échouait, le process sortait
 * en code 1, Docker relançait en boucle. Corrigé par `c8f5789b` en pointant
 * sur `koprogo-minio` — le `container_name`, unique sur l'hôte.
 *
 * ── Pourquoi une garde de fichiers (piste #2 de #731) ───────────────────
 *
 * La piste « propre » — renommer tout service attaché à `ecosolva-web` avec
 * un préfixe projet — dépend de trois AUTRES dépôts (taginy, elevia,
 * derniere-chance) et du réseau partagé lui-même : hors de portée d'un agent
 * sur CE dépôt, et de toute façon une décision Tier 1 côté exploitant VPS.
 *
 * Ce que ce dépôt contrôle : que SES PROPRES composes de prod n'adressent
 * jamais un pair par son alias de service générique, mais par son
 * `container_name` préfixé — la seule chose qui reste unique sur l'hôte
 * quel que soit le réseau. C'est la piste #2 de #731, telle qu'énoncée dans
 * l'issue : « un grep sur `://(backend|frontend|minio|postgres|redis):` dans
 * les composes de prod ».
 *
 * Cette garde ne peut pas exécuter Docker pour observer une collision — le
 * témoin serait la panne de production qu'on cherche à empêcher. Elle lit
 * donc les fichiers, comme `garde-piles-compose-distinctes.test.ts` (#872).
 */

const RACINE = join(process.cwd(), "..");

const NOMS_GENERIQUES = ["backend", "frontend", "minio", "postgres", "redis"];

/**
 * Les composes « de prod » au sens de la piste #2 de #731 : ceux qui
 * décrivent le runtime réellement destiné à un hôte, seul ou partagé.
 *
 * Volontairement absents, et pourquoi aucun n'a besoin de cette garde :
 *
 * - `docker-compose.yml` (recette, ADR 0050) : tourne sur l'hôte partagé
 *   `ecosolva`, mais sur son PROPRE réseau bridge `koprogo-network` — jamais
 *   `ecosolva-web`. C'est le choix du DÉTACHEMENT que l'@edge ci-dessous
 *   vérifie explicitement, pas une omission.
 * - `docker-compose.mcp.yml` : outil de dev (edge-node Raspberry Pi
 *   simulé), aucun label Traefik, réseau privé `koprogo_mcp_network`.
 * - `koprogo-grid/docker-compose.yml`, `load-tests/docker-compose.loadtest.yml` :
 *   sous-projets isolés, jamais déployés sur `ecosolva`.
 *
 * Aucun des exclus ne parle à un conteneur d'un AUTRE projet : le risque que
 * cette garde couvre — la résolution DNS partagée entre locataires — ne les
 * concerne pas.
 */
const COMPOSES_DE_PROD = [
  "docker-compose.prod.yml",
  "infrastructure/_shared/docker-compose/docker-compose.base.yml",
];

/**
 * Les occurrences d'un alias générique dans une chaîne de connexion :
 * `scheme://nom:port` (S3_ENDPOINT) ou `user:pass@nom:port` (DATABASE_URL).
 *
 * `://` ou `@` immédiatement avant le nom exclut les formes préfixées comme
 * `koprogo-minio:9000` — précédé de `-`, jamais de `://` ni `@`.
 */
function alertesAlias(source: string): string[] {
  const motif = new RegExp(
    `(?:://|@)(${NOMS_GENERIQUES.join("|")})(?=:\\d)`,
    "g",
  );
  return [...source.matchAll(motif)].map((m) => m[1]);
}

describe("aucun alias DNS générique dans les composes de prod (#731)", () => {
  it("@happy — koprogo-backend adresse koprogo-minio, le container_name unique sur l'hôte", () => {
    const source = readFileSync(
      join(RACINE, "docker-compose.prod.yml"),
      "utf-8",
    );
    expect(
      source,
      "S3_ENDPOINT doit rester sur koprogo-minio (fix c8f5789b) : c'est le " +
        "container_name, unique sur l'hôte quel que soit le réseau.",
    ).toContain("http://koprogo-minio:9000");
  });

  it("@negative — un alias générique reproduit la config qui a coûté l'indisponibilité", () => {
    // La configuration EXACTE qui a coûté 2538 redémarrages entre le
    // 2026-08-25 21:47 UTC et le 2026-08-29 11:30 UTC (#731).
    const configurationIncriminee = "S3_ENDPOINT: http://minio:9000";
    expect(alertesAlias(configurationIncriminee)).toEqual(["minio"]);

    for (const fichier of COMPOSES_DE_PROD) {
      const source = readFileSync(join(RACINE, fichier), "utf-8");
      expect(
        alertesAlias(source),
        `${fichier} référence un alias générique (${NOMS_GENERIQUES.join(", ")}) ` +
          "dans une chaîne de connexion. Sur ecosolva-web, quatre projets " +
          "revendiquent ce nom : la résolution DNS peut désigner le " +
          "conteneur d'un AUTRE projet, comme le 2026-08-25 (#731).",
      ).toEqual([]);
    }
  });

  it("@edge — aucune tentative de `aliases:`, et la recette choisit le détachement", () => {
    // Compose EXPOSE TOUJOURS le nom du service comme alias, et `aliases:`
    // AJOUTE un alias, il ne retire jamais celui-là. Un fichier qui s'appuie
    // dessus pour « retirer » l'homonyme se trompe de doc Compose.
    for (const fichier of COMPOSES_DE_PROD) {
      const source = readFileSync(join(RACINE, fichier), "utf-8");
      expect(
        source,
        `${fichier} utilise \`aliases:\` : ça n'enlève jamais l'alias de ` +
          "service par défaut, ça en ajoute un (#731). La seule voie réelle " +
          "est le renommage du service ou le détachement du réseau partagé.",
      ).not.toMatch(/^\s*aliases:/m);
    }

    // La pile de recette (ADR 0050) tranche : détachement, pas alias.
    const recette = readFileSync(join(RACINE, "docker-compose.yml"), "utf-8");
    expect(
      recette,
      "docker-compose.yml (recette, ADR 0050) ne doit pas rejoindre le " +
        "réseau partagé ecosolva-web : conforme dès son arrivée signifie " +
        "rester détachée, pas s'en remettre à un alias (#731).",
    ).not.toMatch(/ecosolva-web/);
  });

  it("@security — koprogo n'adresse jamais un pair par `backend`/`frontend` générique", () => {
    // C'est le risque résiduel documenté par #731 : `backend` et `frontend`
    // sont ambigus quatre fois sur ecosolva-web, et rien ne les résout
    // aujourd'hui — un chemin de fuite entre locataires du même hôte, pas
    // seulement une panne. Cette garde ne peut pas corriger les trois autres
    // projets (taginy, elevia, derniere-chance) ni renommer le réseau
    // partagé — décision Tier 1 côté exploitant VPS (#731 piste 1) — mais
    // elle vérifie qu'AUCUN fichier de CE dépôt n'ajoute un cinquième
    // prétendant à ces alias.
    for (const fichier of COMPOSES_DE_PROD) {
      const source = readFileSync(join(RACINE, fichier), "utf-8");
      const cibles = alertesAlias(source).filter((n) =>
        ["backend", "frontend"].includes(n),
      );
      expect(
        cibles,
        `${fichier} adresse un pair par \`backend\`/\`frontend\` générique : ` +
          "c'est exactement le chemin de fuite entre locataires décrit en #731.",
      ).toEqual([]);
    }
  });

  it("lit bien les fichiers, et n'est pas verte par vacuité", () => {
    for (const fichier of COMPOSES_DE_PROD) {
      expect(
        readFileSync(join(RACINE, fichier), "utf-8").length,
        `${fichier} est vide ou illisible`,
      ).toBeGreaterThan(200);
    }
    // Le détecteur discrimine vraiment : il alerte sur le générique...
    expect(alertesAlias("http://minio:9000")).toEqual(["minio"]);
    expect(alertesAlias("postgresql://u:p@postgres:5432/db")).toEqual([
      "postgres",
    ]);
    // ... et se tait sur le préfixé, qui est le correctif attendu.
    expect(alertesAlias("http://koprogo-minio:9000")).toEqual([]);
    expect(alertesAlias("postgresql://u:p@koprogo-postgres:5432/db")).toEqual(
      [],
    );
  });
});
