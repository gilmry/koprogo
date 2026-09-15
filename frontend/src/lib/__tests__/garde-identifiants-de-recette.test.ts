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
 *
 * ── #872 a rouvert ce fichier : le mot de passe juste est le cas dangereux ──
 *
 * Les deux tests `@happy` ci-dessous affirmaient qu'un identifiant SEUL
 * suffisait à laisser passer un hôte distant. C'était vrai, et c'est
 * exactement le trou que #872 a nommé : un identifiant qui FONCTIONNE contre
 * un hôte distant enchaîne la campagne — écritures, `seed-reset`, `reset-db`
 * — sur des données vivantes, sans qu'aucun message ne l'ait jamais demandé.
 * Le garde ne peut pas savoir si le mot de passe est correct sans l'essayer,
 * et l'essayer EST l'action à empêcher.
 *
 * Ces deux tests sont donc ADAPTÉS, pas supprimés : ils exigent désormais
 * AUSSI `KOPROGO_CONFIRME_HOTE_DISTANT`, une variable disjointe de
 * l'identifiant. Leur intention d'origine — « un choix explicite passe » —
 * reste vraie, seul le nombre de choix exigés change.
 */

/** Le cas dangereux : la variable n'est pas posée du tout. */
const AUCUN_CHOIX = undefined;

/** Le consentement explicite requis en plus de l'identifiant (#872). */
const CONSENTEMENT_DONNE = "1";

describe("les identifiants de recette refusent l'hôte distant au repli", () => {
  it("@happy — laisse passer un hôte distant quand un mot de passe ET un consentement sont choisis", () => {
    expect(() =>
      verifieLesIdentifiants(
        "https://koprogo.com",
        "un-vrai-secret",
        CONSENTEMENT_DONNE,
      ),
    ).not.toThrow();
  });

  it("@happy — laisse passer même si le choix EST le repli du seed, consentement donné", () => {
    // C'est le cas de la démo depuis le 2026-09-12 : l'exploitant a posé
    // `admin123` dans l'environnement pour que l'upsert cesse de l'effacer.
    // La valeur est faible, et c'est son affaire ; le serveur l'avertit
    // lui-même au démarrage. Ce garde n'a pas à refuser un choix explicite —
    // à condition que le SECOND choix, le consentement, soit lui aussi posé.
    expect(() =>
      verifieLesIdentifiants(
        "https://koprogo.com",
        "admin123",
        CONSENTEMENT_DONNE,
      ),
    ).not.toThrow();
  });

  it("@security — un identifiant qui semble réel, SANS consentement, s'arrête net", () => {
    // Le cœur de #872 : ce n'est pas le mot de passe faux qui est
    // dangereux, c'est le mot de passe juste. Le garde ne peut pas
    // distinguer les deux sans se connecter — donc il exige le consentement
    // dans tous les cas où un identifiant est posé.
    expect(() =>
      verifieLesIdentifiants(
        "https://koprogo.com",
        "un-vrai-secret",
        AUCUN_CHOIX,
      ),
    ).toThrow(/KOPROGO_CONFIRME_HOTE_DISTANT/);
  });

  it("@negative — un consentement vide ne compte pas comme un choix", () => {
    // `export KOPROGO_CONFIRME_HOTE_DISTANT=` est la même illusion que pour
    // le mot de passe : la variable existe dans l'environnement, vide.
    expect(() =>
      verifieLesIdentifiants("https://koprogo.com", "un-vrai-secret", ""),
    ).toThrow(/KOPROGO_CONFIRME_HOTE_DISTANT/);
  });

  it("@edge — le consentement n'est jamais requis contre un hôte local", () => {
    // Sans ce test, on pourrait croire qu'il faut désormais exporter
    // `KOPROGO_CONFIRME_HOTE_DISTANT` même en local. La CI ne le pose pas.
    expect(() =>
      verifieLesIdentifiants(undefined, AUCUN_CHOIX, AUCUN_CHOIX),
    ).not.toThrow();
    expect(() =>
      verifieLesIdentifiants(
        "http://localhost:8090",
        "peu-importe",
        AUCUN_CHOIX,
      ),
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
    // Un mot de passe SEUL, sans consentement, déclenche désormais le garde
    // de #872 (ci-dessus) — mais son message ne doit toujours citer aucune
    // valeur, seulement le nom des variables.
    let message = "";
    try {
      verifieLesIdentifiants("https://koprogo.com", "S3cret-De-Prod");
    } catch (e) {
      message = (e as Error).message;
    }
    expect(message).not.toContain("S3cret-De-Prod");
    expect(message).toContain("KOPROGO_CONFIRME_HOTE_DISTANT");

    // Avec le consentement AUSSI posé, plus rien à fuiter : ça passe.
    message = "";
    try {
      verifieLesIdentifiants(
        "https://koprogo.com",
        "S3cret-De-Prod",
        CONSENTEMENT_DONNE,
      );
    } catch (e) {
      message = (e as Error).message;
    }
    expect(message).toBe("");

    // …et sur le cas qui DÉCLENCHE au premier palier (pas d'identifiant du
    // tout), le message nomme la variable sans jamais citer une valeur :
    // une erreur atterrit dans les journaux de CI, qui se conservent.
    try {
      verifieLesIdentifiants("https://koprogo.com", AUCUN_CHOIX);
    } catch (e) {
      expect((e as Error).message).not.toContain("admin123");
    }
  });
});
