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

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let adminToken: string;

  test.beforeAll(async ({ request }) => {
    // 1. Login admin — only need to verify the admin account exists
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    adminToken = admin.token;

    const adminHeaders = { Authorization: `Bearer ${adminToken}` };

    // 2. Ensure at least one organization exists for the demo
    const ts = Date.now();
    const orgResp = await request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Demo Copropriete ASBL ${ts}`,
        slug: `demo-copro-${ts}`,
        contact_email: `demo-${ts}@koprogo.test`,
        subscription_plan: "professional",
      },
      headers: adminHeaders,
    });
    const org = await orgResp.json();

    // 3. Ensure at least one building exists
    await request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Residence du Parc ${ts}`,
        address: "15 Rue de la Loi",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        total_units: 24,
        construction_year: 1998,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });

    // 4. Ensure at least one extra user exists
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: `demo-syndic-${ts}@koprogo.test`,
        password: "test123456",
        first_name: "Jean",
        last_name: "Martin",
        role: "syndic",
        organization_id: org.id,
      },
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
    await expect(
      page.getByTestId("organizations-table-body"),
    ).toBeVisible({ timeout: 15000 });

    // Verifier qu'au moins une organisation apparait
    await expect(
      page.getByTestId("organization-row").first(),
    ).toBeVisible({ timeout: 10000 });

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
    await expect(
      page.getByTestId("users-table-body"),
    ).toBeVisible({ timeout: 15000 });

    // Verifier qu'au moins un utilisateur apparait
    await expect(
      page.getByTestId("user-row").first(),
    ).toBeVisible({ timeout: 10000 });

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
