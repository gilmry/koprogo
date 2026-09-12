import { describe, expect, it } from "vitest";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Deux piles Compose ne peuvent pas porter le même nom de projet.
 *
 * ── Ce qui est arrivé, et qui n'a tenu qu'au fait qu'on n'a rien lancé ──
 *
 * Sur le VPS ecosolva, la démo tourne depuis `docker-compose.prod.yml` et
 * `docker-compose.ecosolva.yml`. Aucun des trois fichiers ne posait de
 * `name:`, donc Compose déduisait le nom de projet du répertoire — `koprogo` —
 * **pour les trois**.
 *
 * Conséquence mesurée le 2026-09-12, sans rien exécuter :
 *
 *     $ docker compose -f docker-compose.yml ps
 *     koprogo-backend  koprogo-frontend  koprogo-minio  koprogo-postgres
 *
 * La pile de développement revendiquait les conteneurs vivants. Pire, son
 * volume `postgres_data` résolvait vers `koprogo_postgres_data` — **le volume
 * de données de la démo**, créé le 2026-08-25. Un `docker compose up`, que
 * `Makefile:31` et `Makefile:371` lancent, l'aurait monté.
 *
 * ── Pourquoi une garde de fichiers et pas une garde d'exécution ─────────
 *
 * Il n'existe aucun moyen sûr de vérifier ce défaut en le déclenchant : le
 * témoin serait la destruction qu'on cherche à empêcher. Cette garde se lit
 * donc sur les fichiers, et son témoin est de retirer un `name:`.
 *
 * C'est la seule garde de ce dépôt dont le défaut gardé ne peut pas être
 * reproduit pour vérification. Elle le dit plutôt que de laisser croire le
 * contraire.
 */

const RACINE = join(process.cwd(), "..");

/**
 * Les piles suivies par git, et le nom de projet que chacune DOIT porter.
 *
 * `docker-compose.ecosolva.yml` n'y figure pas, et c'est délibéré : il est
 * ÉCRIT PAR LE CRON de déploiement de l'hôte (`/etc/cron.d/ecosolva-auto-deploy`)
 * et `.gitignore` l'exclut. Il n'existe donc ni en CI ni dans un clone neuf.
 *
 * Y poser un `name:` à la main aurait été effacé au déploiement suivant. Ce
 * n'est pas nécessaire : `docker-compose.prod.yml` épingle `name: koprogo`, et
 * la superposition des deux fichiers en hérite — vérifié sur l'hôte, la pile
 * déployée reste reconnue.
 */
const PILES: Array<[string, string]> = [
  ["docker-compose.yml", "koprogo-dev"],
  ["docker-compose.prod.yml", "koprogo"],
  // Ajoutée le 2026-09-12 : elle portait le MÊME défaut, resté armé. Sans
  // `name:`, ses conteneurs héritaient du label `project=koprogo`, et un
  // `docker compose -f docker-compose.mcp.yml down` aurait emporté la démo.
  // Cette liste est la raison pour laquelle il a fini par se voir — une
  // garde qui ne connaît que deux fichiers sur trois ne garde que deux
  // tiers du dépôt (#872).
  ["docker-compose.mcp.yml", "koprogo-mcp"],
];

/** Les surcouches propres à un hôte : contrôlées si présentes, jamais exigées. */
const SURCOUCHES = ["docker-compose.ecosolva.yml"];

function nomDeProjet(fichier: string): string | null {
  const source = readFileSync(join(RACINE, fichier), "utf-8");
  const m = source.match(/^name:\s*(\S+)\s*$/m);
  return m ? m[1] : null;
}

function nomsDeConteneurs(fichier: string): string[] {
  const source = readFileSync(join(RACINE, fichier), "utf-8");
  return [...source.matchAll(/^\s*container_name:\s*(\S+)\s*$/gm)].map(
    (m) => m[1],
  );
}

/**
 * Les ports PUBLIÉS SUR L'HÔTE par une pile.
 *
 * Compose accepte trois formes, et seule la partie hôte nous intéresse :
 *
 *     - "80:80"                 → 80
 *     - "127.0.0.1:15432:5432"  → 15432
 *     - "8090:80"               → 8090
 *
 * Le port CONTENEUR est sans danger : il vit dans le réseau de sa pile, et
 * deux piles peuvent servir sur :80 chacune de leur côté. C'est la
 * publication qui est globale à la machine.
 */
function portsPublies(fichier: string): number[] {
  const source = readFileSync(join(RACINE, fichier), "utf-8");
  const ports: number[] = [];
  for (const m of source.matchAll(/^\s*-\s*"([^"]+)"\s*(?:#.*)?$/gm)) {
    const morceaux = m[1].split(":");
    if (morceaux.length < 2 || morceaux.length > 3) continue;
    const hote = morceaux[morceaux.length - 2];
    const conteneur = morceaux[morceaux.length - 1];
    // Les deux doivent être numériques : sinon ce n'est pas un mappage de
    // port mais une autre liste entre guillemets (une commande, un label).
    if (!/^\d+$/.test(hote) || !/^\d+$/.test(conteneur)) continue;
    ports.push(Number(hote));
  }
  return ports;
}

/**
 * Une pile, et TOUS les fichiers qui la composent.
 *
 * La distinction compte : une surcouche ne forme pas une pile de plus, elle
 * se SUPERPOSE à une pile existante. `docker-compose.ecosolva.yml` fait
 * partie de la pile déployée — exiger qu'elle en soit disjointe aurait rendu
 * la garde rouge sur un montage parfaitement correct.
 */
function fichiersDeChaquePile(): Array<[string, string[]]> {
  return PILES.map(([principal]) => {
    const surcouches =
      principal === "docker-compose.prod.yml"
        ? SURCOUCHES.filter((f) => existsSync(join(RACINE, f)))
        : [];
    return [principal, [principal, ...surcouches]] as [string, string[]];
  });
}

describe("les piles Compose ne peuvent pas se confondre", () => {
  it("chaque pile déclare son nom de projet explicitement", () => {
    const sansNom = PILES.filter(([f]) => nomDeProjet(f) === null).map(
      ([f]) => f,
    );
    expect(
      sansNom,
      `Ces fichiers ne posent pas de \`name:\` :\n\n` +
        sansNom.map((f) => `  ${f}`).join("\n") +
        `\n\nSans lui, Compose déduit le nom de projet du RÉPERTOIRE. Deux ` +
        `piles dans le même dépôt se retrouvent alors sous le même projet, ` +
        `partagent leurs conteneurs, leurs réseaux et surtout leurs VOLUMES ` +
        `— donc leurs données (#872).`,
    ).toEqual([]);
  });

  it("deux piles ne portent jamais le même nom de projet", () => {
    const vus = new Map<string, string>();
    for (const [fichier] of PILES) {
      const nom = nomDeProjet(fichier);
      if (nom === null) continue; // déjà signalé par le test précédent
      const premier = vus.get(nom);
      expect(
        premier,
        `${fichier} et ${premier} portent tous deux le projet « ${nom} ».\n\n` +
          `Leurs volumes résolvent alors vers les MÊMES noms : le ` +
          `\`postgres_data\` de l'un devient celui de l'autre, et un ` +
          `\`docker compose up\` monte les données de son voisin. C'est ce ` +
          `qui a failli emporter la base de la démo (#872).`,
      ).toBeUndefined();
      vus.set(nom, fichier);
    }
  });

  it("une surcouche d'hôte, si elle existe, n'impose pas un autre projet", () => {
    // `prod` et une surcouche se superposent sur la MÊME pile : si la
    // surcouche posait son propre `name:`, elle prendrait le dessus et un
    // déploiement orphelinerait les conteneurs en cours.
    const attendu = nomDeProjet("docker-compose.prod.yml");
    for (const f of SURCOUCHES) {
      if (!existsSync(join(RACINE, f))) continue;
      const sien = nomDeProjet(f);
      expect(
        sien === null || sien === attendu,
        `${f} impose « ${sien} » là où la pile déployée porte « ${attendu} ». ` +
          `Un déploiement orphelinerait les conteneurs en cours (#872).`,
      ).toBe(true);
    }
  });

  it("aucun nom de conteneur n'est partagé par deux piles", () => {
    // Un nom de conteneur est GLOBAL à l'hôte : il ignore le nom de projet.
    // Deux piles qui en partagent un ne peuvent pas coexister, et la seconde
    // échoue au lancement (#872).
    const proprietaire = new Map<string, string>();
    for (const [pile, fichiers] of fichiersDeChaquePile()) {
      const noms = new Set(fichiers.flatMap(nomsDeConteneurs));
      for (const nom of noms) {
        const premier = proprietaire.get(nom);
        expect(
          premier,
          `Le conteneur « ${nom} » est déclaré par la pile ${pile} ET par ` +
            `${premier}. Un nom de conteneur est global à l'hôte : la ` +
            `seconde pile lancée échouera (#872).`,
        ).toBeUndefined();
        proprietaire.set(nom, pile);
      }
    }
  });

  it("aucun port publié n'est réclamé par deux piles à la fois", () => {
    // Le `name:` sépare les conteneurs, les réseaux et les volumes. Il ne
    // sépare PAS les ports : une publication est globale à la machine.
    //
    // Tant que la pile de dev réclamait le 80, deux issues seulement, toutes
    // deux mauvaises — elle refusait de démarrer, ou elle prenait le port du
    // Traefik de la démo. Et `make test-e2e` visait `http://localhost`, donc
    // `Host(api.koprogo.com)` : la recette écrivait dans la démo.
    //
    // C'est la seconde moitié de #872, tranchée par l'ADR 0050.
    const proprietaire = new Map<number, string>();
    for (const [pile, fichiers] of fichiersDeChaquePile()) {
      const ports = new Set(fichiers.flatMap(portsPublies));
      for (const port of ports) {
        const premier = proprietaire.get(port);
        expect(
          premier,
          `Le port ${port} est publié par la pile ${pile} ET par ` +
            `${premier}.\n\n` +
            `Un port publié est GLOBAL à l'hôte : il ignore le nom de ` +
            `projet. La pile de recette doit donc rester sur ses ports ` +
            `décalés — 8090, 8091, 15432, 19000, 19001 — et surtout PAS sur ` +
            `le 80, qui route vers la démo (ADR 0050, #872).`,
        ).toBeUndefined();
        proprietaire.set(port, pile);
      }
    }
  });

  it("lit bien les fichiers, et n'est pas verte par vacuité", () => {
    // Sans ports lus de part et d'autre, le test de collision ci-dessus
    // comparerait deux ensembles vides et passerait sur rien.
    expect(
      portsPublies("docker-compose.yml").length,
      "aucun port publié lu dans docker-compose.yml",
    ).toBeGreaterThan(0);
    expect(
      portsPublies("docker-compose.prod.yml").length,
      "aucun port publié lu dans docker-compose.prod.yml",
    ).toBeGreaterThan(0);

    for (const [f] of PILES) {
      expect(
        readFileSync(join(RACINE, f), "utf-8").length,
        `${f} est vide ou illisible`,
      ).toBeGreaterThan(200);
    }
    // La pile de dev DOIT nommer des conteneurs, sans quoi le test de
    // partage ci-dessus comparerait deux ensembles vides et passerait sur
    // rien.
    expect(nomsDeConteneurs("docker-compose.yml").length).toBeGreaterThan(0);
  });
});
