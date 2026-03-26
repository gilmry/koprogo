/**
 * SCENARIO: Gestion des moyens de paiement
 *
 * Documentation Vivante -- video exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Moyens de paiement via le menu lateral
 *   3. Ajout d'un moyen de paiement (carte bancaire) via le formulaire
 *   4. Verification que le moyen de paiement apparait dans la liste
 *
 * Duree video attendue : ~40-50 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanFill,
  humanClick,
  humanSelect,
  humanClickLocator,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Gestion des moyens de paiement", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let ownerEmail: string;
  let ownerPassword: string;
  let ownerId: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    ownerEmail = `scenario-owner-pm-${ts}@koprogo.test`;
    ownerPassword = "test123456";

    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Create org
    const orgResp = await request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Scenario PM Org ${ts}`,
        slug: `scenario-pm-${ts}`,
        contact_email: `org-pm-${ts}@koprogo.test`,
        subscription_plan: "professional",
      },
      headers: adminHeaders,
    });
    const org = await orgResp.json();

    // 3. Register syndic (to create building + owner)
    const syndicEmail = `scenario-syndic-pm-${ts}@koprogo.test`;
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: syndicEmail,
        password: "test123456",
        first_name: "Syndic",
        last_name: "Paiement",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // Login as syndic
    const syndicLoginResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: "test123456" },
    });
    const syndic = await syndicLoginResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 4. Create building
    await request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Residence Paiement ${ts}`,
        address: "10 Rue du Commerce",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        total_units: 8,
        construction_year: 1990,
        organization_id: org.id,
      },
      headers: syndicHeaders,
    });

    // 5. Create owner
    const ownerResp = await request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Pierre",
        last_name: `Dupont${ts}`,
        email: ownerEmail,
        address: "10 Rue du Commerce",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: syndicHeaders,
    });
    const owner = await ownerResp.json();
    ownerId = owner.id;

    // 6. Register owner user account
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: ownerEmail,
        password: ownerPassword,
        first_name: "Pierre",
        last_name: `Dupont${ts}`,
        role: "owner",
        organization_id: org.id,
      },
    });
  });

  test("Un proprietaire ajoute un moyen de paiement via l'interface", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, ownerEmail, ownerPassword);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers Moyens de paiement via le menu
    // ============================================================
    await humanClick(page, "nav-link-moyens-paiement");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Moyens de paiement est chargee
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

    // Verifier que le modal est ouvert
    await expect(page.getByTestId("method-type-select")).toBeVisible({
      timeout: 10000,
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // ============================================================
    // ETAPE 5 : Remplir le formulaire d'ajout
    // ============================================================

    // Le type "Card" est selectionne par defaut, pas besoin de changer

    // Display Label
    await humanFill(page, "display-label-input", "Visa Pierre ****4242");

    // Stripe Payment Method ID
    await humanFill(page, "stripe-id-input", "pm_test_scenario_4242");

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

    // La liste devrait maintenant contenir le moyen de paiement ajoute
    await expect(page.getByTestId("payment-method-list")).toBeVisible({
      timeout: 15000,
    });

    await expect(
      page.locator("text=Visa Pierre"),
    ).toBeVisible({ timeout: 15000 });

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
