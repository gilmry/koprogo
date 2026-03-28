/**
 * SCENARIO: Gestion des moyens de paiement (SINGLE ROLE - owner)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'Alice (coproprietaire) :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Moyens de paiement via le menu lateral
 *   3. Ajout d'un nouveau moyen de paiement (carte bancaire) via le formulaire
 *   4. Verification que le nouveau moyen de paiement apparait dans la liste
 *
 * Duree video attendue : ~40-50 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanFill,
  humanClick,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Gestion des moyens de paiement (Alice)", () => {
  test.setTimeout(120_000);

  let seedData: any;

  test.beforeAll(async ({ request }) => {
    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Seed the world
    const seedResp = await request.post(`${API_BASE}/seed/scenario/world`, {
      headers: adminHeaders,
    });
    if (!seedResp.ok()) {
      console.log("Seed world already exists, continuing...");
    } else {
      seedData = await seedResp.json();
      seedData = seedData.data;
    }
  });

  test.afterAll(async ({ request }) => {
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    await request.delete(`${API_BASE}/seed/scenario/world`, {
      headers: { Authorization: `Bearer ${admin.token}` },
    });
  });

  test("Alice ajoute un moyen de paiement via l'interface", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "alice@residence-parc.be", "alice123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers Moyens de paiement via le menu
    // ============================================================
    await humanClick(page, "nav-link-moyens-paiement");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText(
      "Moyens de Paiement",
    );
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Attendre le chargement de la liste
    // ============================================================
    await waitForSpinner(page);
    await expect(page.getByTestId("payment-method-list")).toBeVisible({
      timeout: 15000,
    });
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Cliquer sur "Ajouter un moyen de paiement"
    // ============================================================
    await humanClick(page, "add-payment-method-btn");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("method-type-select")).toBeVisible({
      timeout: 10000,
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // ============================================================
    // ETAPE 5 : Remplir le formulaire d'ajout
    // ============================================================
    // Display Label
    await humanFill(page, "display-label-input", "Visa Alice ****4242");

    // Stripe Payment Method ID
    await humanFill(page, "stripe-id-input", "pm_test_alice_4242");

    // Card Brand
    await humanFill(page, "brand-input", "Visa");

    // Last 4
    await humanFill(page, "last4-input", "4242");

    await stepPause(page);

    // ============================================================
    // ETAPE 6 : Soumettre le formulaire
    // ============================================================
    await humanClick(page, "submit-btn");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 7 : Verifier que le moyen de paiement apparait
    // ============================================================
    await waitForSpinner(page);
    await page.waitForTimeout(2000);

    await expect(page.getByTestId("payment-method-list")).toBeVisible({
      timeout: 15000,
    });

    await expect(page.locator("text=Visa Alice")).toBeVisible({
      timeout: 15000,
    });

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
