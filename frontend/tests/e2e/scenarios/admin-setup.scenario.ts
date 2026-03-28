/**
 * SCENARIO: Administration de la plateforme par le SuperAdmin
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'un administrateur :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la liste des Organisations
 *   3. Consultation de la liste des organisations
 *   4. Navigation vers la liste des Immeubles
 *   5. Consultation de la liste des immeubles
 *   6. Navigation vers la liste des Utilisateurs
 *   7. Consultation de la liste des utilisateurs
 *   8. Pause finale sur le tableau de bord admin
 *
 * Duree video attendue : ~40-50 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanClick,
  humanGoto,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Le SuperAdmin explore la plateforme", () => {
  test.setTimeout(120_000);

  let seedData: any;

  test.beforeAll(async ({ request }) => {
    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Seed the world (creates orgs, buildings, users — rich data for admin to explore)
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

  test("Le SuperAdmin consulte organisations, immeubles et utilisateurs", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "admin@koprogo.com", "admin123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers les Organisations
    // ============================================================
    await humanClick(page, "nav-link-organisations");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Organisations est chargee
    await expect(page.getByTestId("organizations-table-body")).toBeVisible({
      timeout: 15000,
    });

    // Verifier qu'au moins une organisation apparait
    await expect(page.getByTestId("organization-row").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Navigation vers les Immeubles
    // ============================================================
    await humanClick(page, "nav-link-immeubles");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Immeubles est chargee
    await expect(page.locator("main h1").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Navigation vers les Utilisateurs
    // ============================================================
    await humanClick(page, "nav-link-utilisateurs");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Utilisateurs est chargee
    await expect(page.getByTestId("users-table-body")).toBeVisible({
      timeout: 15000,
    });

    // Verifier qu'au moins un utilisateur apparait
    await expect(page.getByTestId("user-row").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Retour au tableau de bord admin
    // ============================================================
    await humanGoto(page, "/admin");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
