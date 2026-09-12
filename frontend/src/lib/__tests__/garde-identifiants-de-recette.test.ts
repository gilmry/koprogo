import { describe, expect, it } from "vitest";
import { verifieLesIdentifiants } from "../../../tests/e2e/helpers/identifiants";

/**
 * Viser un hôte distant avec le mot de passe de repli doit s'arrêter net.
 *
 * ── Le fait qui a coûté une demi-journée ─────────────────────────────────
 *
 * `seed_superadmin` fait un **upsert à chaque démarrage** depuis
 * `KOPROGO_SUPERADMIN_PASSWORD`. Le jour où la démo a reçu cette variable, le
 * repli `admin123` a cessé de valoir, et toute campagne Playwright contre
 * `api.koprogo.com` est tombée à la première connexion — sur un `401 Invalid
 * credentials` qui parle d'identifiants, pas de configuration.
 *
 * Le dépôt avait prévu le jour, mot pour mot, dans le fichier gardé ici :
 *
 * > « le jour où elle le fera, toute la suite e2e tombera d'un coup, et aucun
 * > message d'erreur ne parlera du mot de passe. »
 *
 * La prévision s'est vérifiée le 2026-09-10 (#870).
 *
 * ── Pourquoi rétablir le mot de passe ne suffit PAS ─────────────────────
 *
 * L'upsert repart de l'environnement à chaque démarrage. Corriger la ligne en
 * base laisse le prochain déploiement la reprendre, et le piège se réarme sans
 * que personne ne l'ait touché. Le garde est la seule parade qui survit à un
 * redéploiement.
 *
 * ── Pourquoi ce garde ne doit PAS se déclencher en CI ──────────────────
 *
 * La CI amorce sa propre base sans la variable : `admin123` y est légitime.
 * Un garde qui casserait la CI pour protéger la démo serait pire que le
 * défaut qu'il corrige. Il ne mord donc que sur un hôte DISTANT.
 */

const REPLI = "admin123";

describe("les identifiants de recette refusent l'hôte distant au repli", () => {
  it("@happy — laisse passer un hôte distant avec un vrai mot de passe", () => {
    expect(() =>
      verifieLesIdentifiants("https://koprogo.com", "un-vrai-secret"),
    ).not.toThrow();
  });

  it("@negative — s'arrête sur un hôte distant resté au repli", () => {
    expect(() =>
      verifieLesIdentifiants("https://koprogo.com", REPLI),
    ).toThrow(/KOPROGO_SUPERADMIN_PASSWORD/);
  });

  it("@negative — le message nomme AUSSI la variable d'adresse", () => {
    // Sans elle, le lecteur ne sait pas pourquoi le garde s'est déclenché
    // chez lui et pas en CI.
    expect(() => verifieLesIdentifiants("https://koprogo.com", REPLI)).toThrow(
      /PLAYWRIGHT_BASE_URL/,
    );
  });

  it("@edge — ne mord sur aucune forme d'hôte local", () => {
    for (const local of [
      undefined,
      "",
      "http://localhost",
      "http://localhost:3000",
      "http://127.0.0.1:4321",
      "http://0.0.0.0:80",
      "http://[::1]:3000",
    ]) {
      expect(
        () => verifieLesIdentifiants(local, REPLI),
        `${local ?? "(non défini)"} est local : la CI doit continuer`,
      ).not.toThrow();
    }
  });

  it("@edge — une adresse illisible ne fait pas tomber la suite", () => {
    // Un garde qui plante sur une entrée qu'il ne comprend pas remplace une
    // panne par une autre. Dans le doute il laisse passer : ce n'est pas lui
    // qui authentifie.
    //
    // `pas://une/url` n'est PAS un bon exemple, contrairement à ce que ce test
    // affirmait d'abord : `new URL` l'analyse très bien et en tire l'hôte
    // « une ». Les vraies entrées illisibles sont celles sans autorité.
    for (const illisible of ["pas-une-url", "://", "http://"]) {
      expect(
        () => verifieLesIdentifiants(illisible, REPLI),
        `${illisible} est illisible : le garde laisse passer`,
      ).not.toThrow();
    }
  });

  it("@edge — un schéma exotique mais lisible compte comme distant", () => {
    // `pas://une/url` porte un hôte, donc la campagne vise bien quelque chose
    // qu'elle n'amorce pas. Le garde mord, et c'est le comportement voulu.
    expect(() => verifieLesIdentifiants("pas://une/url", REPLI)).toThrow(
      /KOPROGO_SUPERADMIN_PASSWORD/,
    );
  });

  it("@security — n'imprime jamais la valeur du mot de passe", () => {
    let message = "";
    try {
      verifieLesIdentifiants("https://koprogo.com", "S3cret-De-Prod");
    } catch (e) {
      message = (e as Error).message;
    }
    // Un vrai mot de passe ne déclenche rien, donc rien à fuiter ici…
    expect(message).toBe("");

    // …et sur le cas qui DÉCLENCHE, le repli ne doit pas non plus être
    // recopié : un message d'erreur atterrit dans les journaux de CI.
    try {
      verifieLesIdentifiants("https://koprogo.com", REPLI);
    } catch (e) {
      expect((e as Error).message).not.toContain(REPLI);
    }
  });
});
