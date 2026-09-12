import { describe, it, expect } from "vitest";
import { readFileSync, existsSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Garde-fou : la PWA s'installe.
 *
 * ── Ce qui a été trouvé ────────────────────────────────────────────────────
 *
 * Mesuré au navigateur le 2026-09-06, sur la production :
 *
 *     enregistrements de service worker : 0
 *     page contrôlée par un SW          : false
 *     caches : koprogo-v1 → 0 entrée
 *
 * Le cache **existait et était vide**. `caches.open()` avait réussi, puis
 * `cache.addAll(PRECACHE_ASSETS)` avait échoué — ce qui rejette la promesse
 * du `install` et **annule atomiquement toute l'installation**.
 *
 * Deux causes, cumulées :
 *
 *   1. `public/icons/` ne contenait qu'un README. Les huit icônes que le
 *      manifeste déclare n'avaient jamais été engendrées : 404.
 *   2. `/buildings` et `/documents` répondent 301 — Astro ajoute la barre
 *      oblique finale — et `Cache.put()` refuse toute réponse redirigée.
 *
 * Depuis novembre 2025, **aucun utilisateur n'a jamais eu d'application
 * installée**.
 *
 * ── Ce que ce test refuse ──────────────────────────────────────────────────
 *
 * Trois choses, toutes vérifiables sans navigateur :
 *
 *   — qu'une icône déclarée par le manifeste n'existe pas ;
 *   — qu'une entrée de préchargement soit sujette à redirection ;
 *   — que le préchargement du confort redevienne atomique.
 *
 * Voir #804.
 */

const RACINE = process.cwd();
const SW = join(RACINE, "public/service-worker.js");
const MANIFESTE = join(RACINE, "public/manifest.json");

describe("la PWA s'installe (#804)", () => {
  const source = readFileSync(SW, "utf8");

  it("livre toutes les icônes que le manifeste déclare", () => {
    const manifeste = JSON.parse(readFileSync(MANIFESTE, "utf8"));
    const manquantes: string[] = [];
    const vides: string[] = [];

    for (const icone of manifeste.icons ?? []) {
      const chemin = join(RACINE, "public", icone.src);
      if (!existsSync(chemin)) {
        manquantes.push(icone.src);
      } else if (statSync(chemin).size === 0) {
        vides.push(icone.src);
      }
    }

    expect(
      [...manquantes, ...vides],
      `Le manifeste déclare des icônes qui n'existent pas ou sont vides.\n\n` +
        `Une icône préchargée qui répond 404 fait échouer \`cache.addAll\`, ` +
        `donc l'installation entière du service worker. C'est ce qui a rendu ` +
        `la PWA inopérante de novembre 2025 au 2026-09-07 (#804).\n\n` +
        `Manquantes : ${manquantes.join(", ") || "aucune"}\n` +
        `Vides : ${vides.join(", ") || "aucune"}`,
    ).toEqual([]);
  });

  it("ne précharge aucune page sujette à redirection", () => {
    // Astro sert les pages avec une barre oblique finale et redirige en 301
    // sans elle. `Cache.put()` refuse une réponse redirigée.
    const routes = [...source.matchAll(/"(\/[a-z0-9-]+)"/gi)]
      .map((m) => m[1])
      .filter(
        (r) =>
          r !== "/" &&
          !r.startsWith("/api") &&
          !r.includes(".") &&
          !r.startsWith("/icons"),
      );

    expect(
      routes,
      `Des routes de page sont préchargées sans barre oblique finale : ` +
        `Astro les redirige en 301, et \`Cache.put()\` refuse une réponse ` +
        `obtenue par redirection. C'est l'une des deux causes de #804.\n\n` +
        `Écrivez « /buildings/ » plutôt que « /buildings ».\n\n` +
        routes.join(", "),
    ).toEqual([]);
  });

  it("ne précharge le confort qu'un par un", () => {
    // `addAll` est atomique : une ressource en échec annule tout. L'essentiel
    // peut se le permettre — sans page de repli, mieux vaut échouer — mais le
    // confort ne doit rien empêcher.
    expect(
      source,
      "Le préchargement optionnel est redevenu atomique. Une icône " +
        "manquante annulerait de nouveau l'installation entière (#804).",
    ).toContain("Promise.allSettled");

    expect(
      source,
      "La distinction entre l'essentiel et le confort a disparu du service " +
        "worker.",
    ).toContain("PRECACHE_ESSENTIEL");

    // La page de repli reste stricte : un service worker installé sans elle
    // laisse l'utilisateur devant l'erreur du navigateur.
    const essentiel = source.slice(
      source.indexOf("PRECACHE_ESSENTIEL"),
      source.indexOf("PRECACHE_CONFORT"),
    );
    expect(
      essentiel,
      "La page de repli hors ligne n'est plus dans l'essentiel : un service " +
        "worker installé sans elle est pire que pas de service worker.",
    ).toContain("/offline.html");
  });
});
