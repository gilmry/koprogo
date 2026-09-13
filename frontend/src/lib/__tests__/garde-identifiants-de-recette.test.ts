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
 * La CI amorce sa propre base sans la variable : le repli y est légitime.
 * Un garde qui casserait la CI pour protéger la démo serait pire que le
 * défaut qu'il corrige. Il ne mord donc que sur un hôte DISTANT.
 *
 * ── Ce qu'il surveille : le CHOIX, pas la valeur ───────────────────────
 *
 * Sa première version comparait le mot de passe à `admin123`. Le 2026-09-12 a
 * montré l'erreur : la démo a reçu `KOPROGO_SUPERADMIN_PASSWORD=admin123` dans
 * son environnement, pour que l'upsert du seed cesse d'effacer la valeur à
 * chaque redémarrage. Le garde aurait refusé une campagne légitime parce que
 * la bonne valeur ressemblait à la mauvaise.
 *
 * Le danger est de **ne pas avoir choisi**. Choisir `admin123` sciemment est
 * une décision d'exploitation — le serveur la signale déjà de son côté, et ce
 * n'est pas à ce fichier d'en juger.
 */

/** Le cas dangereux : la variable n'est pas posée du tout. */
const AUCUN_CHOIX = undefined;

describe("les identifiants de recette refusent l'hôte distant au repli", () => {
  it("@happy — laisse passer un hôte distant quand un mot de passe est choisi", () => {
    expect(() =>
      verifieLesIdentifiants("https://koprogo.com", "un-vrai-secret"),
    ).not.toThrow();
  });

  it("@happy — laisse passer même si le choix EST le repli du seed", () => {
    // C'est le cas de la démo depuis le 2026-09-12 : l'exploitant a posé
    // `admin123` dans l'environnement pour que l'upsert cesse de l'effacer.
    // La valeur est faible, et c'est son affaire ; le serveur l'avertit
    // lui-même au démarrage. Ce garde n'a pas à refuser un choix explicite.
    expect(() =>
      verifieLesIdentifiants("https://koprogo.com", "admin123"),
    ).not.toThrow();
  });

  it("@negative — s'arrête sur un hôte distant sans aucun choix", () => {
    expect(() =>
      verifieLesIdentifiants("https://koprogo.com", AUCUN_CHOIX),
    ).toThrow(/KOPROGO_SUPERADMIN_PASSWORD/);
  });

  it("@negative — une variable vide ne compte pas comme un choix", () => {
    // `export KOPROGO_SUPERADMIN_PASSWORD=` est la façon la plus courante de
    // croire avoir posé la variable sans l'avoir fait.
    expect(() => verifieLesIdentifiants("https://koprogo.com", "")).toThrow(
      /KOPROGO_SUPERADMIN_PASSWORD/,
    );
  });

  it("@negative — le message nomme AUSSI la variable d'adresse", () => {
    // Sans elle, le lecteur ne sait pas pourquoi le garde s'est déclenché
    // chez lui et pas en CI.
    expect(() =>
      verifieLesIdentifiants("https://koprogo.com", AUCUN_CHOIX),
    ).toThrow(/PLAYWRIGHT_BASE_URL/);
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
        () => verifieLesIdentifiants(local, AUCUN_CHOIX),
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
        () => verifieLesIdentifiants(illisible, AUCUN_CHOIX),
        `${illisible} est illisible : le garde laisse passer`,
      ).not.toThrow();
    }
  });

  it("@edge — un schéma exotique mais lisible compte comme distant", () => {
    // `pas://une/url` porte un hôte, donc la campagne vise bien quelque chose
    // qu'elle n'amorce pas. Le garde mord, et c'est le comportement voulu.
    expect(() => verifieLesIdentifiants("pas://une/url", AUCUN_CHOIX)).toThrow(
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

    // …et sur le cas qui DÉCLENCHE, le message nomme la variable sans jamais
    // citer une valeur : une erreur atterrit dans les journaux de CI, qui se
    // conservent.
    try {
      verifieLesIdentifiants("https://koprogo.com", AUCUN_CHOIX);
    } catch (e) {
      expect((e as Error).message).not.toContain("admin123");
    }
  });
});
