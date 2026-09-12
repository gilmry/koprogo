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

  it("la pile de développement porte un projet distinct de la déployée", () => {
    const dev = nomDeProjet("docker-compose.yml");
    const prod = nomDeProjet("docker-compose.prod.yml");
    expect(
      dev,
      `La pile de développement et la pile déployée portent toutes deux le ` +
        `projet « ${dev} ».\n\n` +
        `Le volume \`postgres_data\` de la pile de dev résoudrait alors vers ` +
        `\`${prod}_postgres_data\`, qui est la base de la démo. Un ` +
        `\`docker compose up\` monterait les données vivantes (#872).`,
    ).not.toBe(prod);
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

  it("aucun nom de conteneur n'est partagé entre dev et déployé", () => {
    const dev = new Set(nomsDeConteneurs("docker-compose.yml"));
    const deploye = new Set([
      ...nomsDeConteneurs("docker-compose.prod.yml"),
      ...SURCOUCHES.filter((f) => existsSync(join(RACINE, f))).flatMap(
        nomsDeConteneurs,
      ),
    ]);
    const partages = [...dev].filter((n) => deploye.has(n));
    expect(
      partages,
      `Noms de conteneurs partagés entre la pile de dev et la déployée :\n\n` +
        partages.map((n) => `  ${n}`).join("\n") +
        `\n\nUn nom de conteneur est GLOBAL à l'hôte, il ignore le nom de ` +
        `projet. Deux piles au même nom ne peuvent pas coexister, et la ` +
        `seconde échoue au lancement (#872).`,
    ).toEqual([]);
  });

  it("lit bien les fichiers, et n'est pas verte par vacuité", () => {
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
