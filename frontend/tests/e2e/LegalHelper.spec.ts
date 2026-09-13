import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

/**
 * Legal Helper E2E Test Suite - Belgian Copropriété Law Panel
 *
 * Tests the floating legal helper panel: toggle, contextual content,
 * and close functionality. The LegalHelper component provides
 * contextual Belgian law information based on the current page.
 *
 * Les tests d'interface étaient SAUTÉS, et leur en-tête portait le remède :
 * « To enable: add <LegalHelper client:load /> to Layout.astro or syndic
 * pages ». Le composant existait, ses 322 lignes étaient écrites, ses tests
 * étaient écrits, et rien ne le montait. Personne ne pouvait l'atteindre.
 *
 * C'est fait : `Layout.astro` le monte derrière `showNav`, donc sur les pages
 * authentifiées et pas sur l'écran de connexion. Le chargement des règles a
 * été rendu paresseux au passage — il partait `onMount`, ce qui aurait coûté
 * trois requêtes sur CHAQUE page pour un panneau rarement ouvert.
 *
 * Les tests d'API restent ce qu'ils étaient : `/legal/rules` et
 * `/legal/ag-sequence` sont publics et fonctionnels.
 */

import { API_BASE } from "./helpers/adresses";

test.describe("Legal Helper - Belgian Law Panel", () => {
  // Les trois tests d'interface étaient sautés parce que le composant n'était
  // monté nulle part. Il l'est désormais dans `Layout.astro`, derrière
  // `showNav` — donc sur les pages authentifiées, pas sur la connexion.
  test("@happy should display the legal helper toggle button", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");
    await expect(page.getByTestId("legal-helper-toggle-btn")).toBeVisible({
      timeout: 10000,
    });
  });

  test("@happy should open the legal helper panel when toggle is clicked", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");
    await page.getByTestId("legal-helper-toggle-btn").click();
    await expect(page.getByTestId("legal-helper-close-btn")).toBeVisible({
      timeout: 10000,
    });
  });

  test("@happy should close the legal helper panel when close button is clicked", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");
    await page.getByTestId("legal-helper-toggle-btn").click();
    await expect(page.getByTestId("legal-helper-close-btn")).toBeVisible({
      timeout: 10000,
    });
    await page.getByTestId("legal-helper-close-btn").click();
    await expect(page.getByTestId("legal-helper-close-btn")).not.toBeVisible({
      timeout: 5000,
    });
  });

  // Skip: /legal/rules endpoint not implemented yet
  /**
   * Le panneau ne s'affiche pas sur l'écran de connexion.
   *
   * Je l'ai monté derrière `showNav` dans `Layout.astro`, et rien ne le
   * vérifiait. Ce n'est pas une préférence d'ergonomie : le panneau appelle
   * `/legal/rules`, `/legal/ag-sequence` et `/legal/majority-for` dès qu'on
   * l'ouvre. L'exposer avant l'authentification donnerait à un visiteur
   * anonyme un bouton qui déclenche trois appels, sur une page dont tout
   * l'enjeu est de ne rien faire tant qu'on ne sait pas qui frappe.
   *
   * Les cinq autres tests de ce fichier sont tous `@happy`. Celui-ci est le
   * seul qui éprouve un refus — la taxonomie l'a rendu visible en comptant
   * mes réactivations, et c'est exactement ce à quoi elle sert : dire si les
   * chemins d'erreur et les refus sont éprouvés, ou si tout est nominal.
   */
  test("@security n'apparaît pas avant l'authentification", async ({
    page,
  }) => {
    await page.goto("/login");
    await expect(page.getByTestId("legal-helper-toggle-btn")).toHaveCount(0);
  });

  test("@happy should serve legal rules from the API", async ({ page }) => {
    // Legal rules endpoint is public (no auth required per routes.rs)
    const rulesResp = await page.request.get(`${API_BASE}/legal/rules`);

    expect(rulesResp.ok()).toBeTruthy();
    const rules = await rulesResp.json();
    expect(Array.isArray(rules)).toBeTruthy();
  });

  // Skip: /legal/ag-sequence endpoint not implemented yet
  test("@happy should serve AG sequence from the API", async ({ page }) => {
    // AG sequence endpoint is public (no auth required per routes.rs)
    const seqResp = await page.request.get(`${API_BASE}/legal/ag-sequence`);

    expect(seqResp.ok()).toBeTruthy();
    const sequence = await seqResp.json();
    expect(Array.isArray(sequence)).toBeTruthy();
  });
});
