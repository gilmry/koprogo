/**
 * Le registre des règles légales — la page tient-elle son contrat ?
 *
 * ── Pourquoi cette recette existe ─────────────────────────────────────────
 *
 * Le balayage du 2026-09-19 a relevé sur `/legal-rules` un
 * `Svelte error: each_key_duplicate` et 478 caractères rendus (#968). En
 * remontant, le défaut était plus large qu'une clé : le composant déclarait
 * SEPT champs — `code`, `category`, `roles`, `article`, `content`,
 * `keywords`, `title` — là où `GET /legal/rules` en sert CINQ, dont un seul
 * en commun, `title`.
 *
 * Trois conséquences, dont une que le balayage ne pouvait pas voir :
 *
 *   1. `{#each … (regle.code)}` indexait sur un champ absent : vingt règles
 *      partageaient la clé `undefined` ;
 *   2. `article` et `content` rendaient vide — vingt cartes réduites à leur
 *      titre ;
 *   3. **taper dans le champ de recherche levait une exception**, le filtre
 *      appelant `.toLowerCase()` sur `undefined`. Un balayage n'écrit nulle
 *      part : il ne pouvait pas l'atteindre.
 *
 * ── Ce que cette recette vérifie, et ce qu'elle refuse de vérifier ────────
 *
 * Elle vérifie le CONTRAT tel qu'il se voit à l'écran : des cartes peuplées,
 * une clé par règle, une recherche qui filtre sans jeter. Elle ne fige
 * aucune règle de droit en particulier — le registre est amené à grossir,
 * et un test qui compterait vingt entrées deviendrait faux à la
 * vingt-et-unième.
 */
import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

const ROUTE = "/legal-rules";

/**
 * La page exige une session, et c'est discutable.
 *
 * Sa docstring dit tenir ses données de `/legal/rules`, « route publique et
 * déjà servie » — or `RouteGuard` renvoie un visiteur anonyme sur
 * `/login?redirect=/legal-rules`. Une citation d'article que personne ne peut
 * lire sans compte affaiblit précisément ce que ces encarts promettent : ne
 * pas demander de croire sur parole.
 *
 * Ce n'est pas tranché ici — une recette ne décide pas d'une politique
 * d'accès. Elle se connecte, et la question part en issue.
 */
test.beforeEach(async ({ page }) => {
  await loginAsSyndic(page, "regles");
});

test.describe("Registre des règles légales (#968)", () => {
  test("@happy les règles s'affichent avec leur substance, sans erreur de console", async ({
    page,
  }) => {
    const erreurs: string[] = [];
    page.on("console", (m) => {
      if (m.type() === "error") erreurs.push(m.text());
    });
    page.on("pageerror", (e) => erreurs.push(String(e)));

    await page.goto(ROUTE);
    await expect(page.getByTestId("legal-rules-loading")).toBeHidden({
      timeout: 15000,
    });

    const cartes = page.locator("[data-rule-code]");
    await expect(cartes.first()).toBeVisible({ timeout: 15000 });
    const nombre = await cartes.count();
    expect(nombre, "Le registre ne sert aucune règle.").toBeGreaterThan(0);

    // Chaque règle porte une ancre DISTINCTE. C'est exactement ce que
    // `each_key_duplicate` disait : vingt cartes, une seule clé.
    const ancres = await cartes.evaluateAll((els) =>
      els.map((e) => e.getAttribute("data-rule-code")),
    );
    expect(
      ancres.filter((a) => !a).length,
      "Des règles n'ont pas d'ancre : le composant indexe sur un champ que " +
        "l'API ne sert pas.",
    ).toBe(0);
    expect(
      new Set(ancres).size,
      `Ancres dupliquées : ${ancres.length} règles pour ` +
        `${new Set(ancres).size} ancres distinctes.`,
    ).toBe(ancres.length);

    // Une carte qui ne porte que son titre n'explique rien. C'est ce que la
    // page servait : 478 caractères pour vingt règles.
    const texte = (await cartes.first().innerText()).trim();
    expect(
      texte.length,
      `La première règle ne rend que ${texte.length} caractères : le contrat ` +
        `du composant ne correspond pas à ce que le serveur envoie.`,
    ).toBeGreaterThan(80);

    expect(
      erreurs,
      `La console proteste : ${erreurs.slice(0, 3).join(" | ")}`,
    ).toEqual([]);
  });

  test("@edge la recherche filtre sans jeter d'exception", async ({ page }) => {
    // Le cas que le balayage ne pouvait pas atteindre : il ouvre des écrans,
    // il n'écrit dans aucun champ.
    const erreurs: string[] = [];
    page.on("pageerror", (e) => erreurs.push(String(e)));
    page.on("console", (m) => {
      if (m.type() === "error") erreurs.push(m.text());
    });

    await page.goto(ROUTE);
    await expect(page.getByTestId("legal-rules-loading")).toBeHidden({
      timeout: 15000,
    });
    const avant = await page.locator("[data-rule-code]").count();

    await page.getByTestId("legal-rules-search").fill("copropriété");
    await expect
      .poll(async () => page.locator("[data-rule-code]").count(), {
        timeout: 5000,
      })
      .toBeLessThanOrEqual(avant);

    expect(
      erreurs,
      `Taper dans la recherche a jeté : ${erreurs.slice(0, 2).join(" | ")}`,
    ).toEqual([]);

    // Un terme qui ne correspond à rien doit DIRE qu'il ne correspond à
    // rien, pas rendre une liste vide sans explication.
    await page.getByTestId("legal-rules-search").fill("zzzzzzzz");
    await expect(page.getByTestId("legal-rules-empty")).toBeVisible({
      timeout: 5000,
    });
  });

  test("@edge le lien profond met la règle visée en avant", async ({
    page,
  }) => {
    await page.goto(ROUTE);
    await expect(page.getByTestId("legal-rules-loading")).toBeHidden({
      timeout: 15000,
    });
    const premiere = await page
      .locator("[data-rule-code]")
      .first()
      .getAttribute("data-rule-code");
    expect(premiere).toBeTruthy();

    await page.goto(`${ROUTE}?code=${encodeURIComponent(premiere!)}`);
    await expect(page.getByTestId("legal-rules-loading")).toBeHidden({
      timeout: 15000,
    });
    await expect(
      page.locator(`[data-rule-code="${premiere}"]`),
      "L'ancre `?code=` ne désigne plus aucune règle : c'est le lien que " +
        "cette page existe pour servir.",
    ).toHaveAttribute("data-targeted", "true");
  });
});
