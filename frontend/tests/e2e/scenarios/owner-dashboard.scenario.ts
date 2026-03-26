/**
 * SCENARIO: Un coproprietaire consulte son tableau de bord
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'un coproprietaire :
 *   1. Connexion via le formulaire login
 *   2. Arrivee sur le tableau de bord proprietaire
 *   3. Consultation des widgets (stats, immeubles, lots)
 *   4. Navigation vers la section Paiements
 *   5. Pause finale montrant l'espace proprietaire
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

test.describe("Scenario: Le coproprietaire consulte son tableau de bord", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let ownerEmail: string;
  let ownerPassword: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    ownerEmail = `scenario-owner-${ts}@koprogo.test`;
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
        name: `Copro Bellevue ${ts}`,
        slug: `copro-bellevue-${ts}`,
        contact_email: `syndic-${ts}@koprogo.test`,
        subscription_plan: "professional",
      },
      headers: adminHeaders,
    });
    const org = await orgResp.json();

    // 3. Register syndic (needed for building creation context)
    const syndicEmail = `scenario-syndic-${ts}@koprogo.test`;
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: syndicEmail,
        password: "test123456",
        first_name: "Sophie",
        last_name: "Lambert",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Create building
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Residence Les Tilleuls ${ts}`,
        address: "8 Avenue des Arts",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        total_units: 16,
        construction_year: 2005,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
    const building = await buildingResp.json();

    // 5. Register owner user
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: ownerEmail,
        password: ownerPassword,
        first_name: "Pierre",
        last_name: "Dubois",
        role: "owner",
        organization_id: org.id,
      },
    });

    // 6. Login as syndic to create expenses (context data)
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: "test123456" },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 7. Create a few expenses for the building
    const expenseDescriptions = [
      {
        description: "Entretien ascenseur - Trimestre 1",
        amount: 1250.0,
        category: "Maintenance",
      },
      {
        description: "Nettoyage parties communes - Mars",
        amount: 480.0,
        category: "Cleaning",
      },
      {
        description: "Assurance immeuble 2026",
        amount: 3200.0,
        category: "Insurance",
      },
    ];

    for (const expense of expenseDescriptions) {
      await request.post(`${API_BASE}/expenses`, {
        data: {
          building_id: building.id,
          description: expense.description,
          amount: expense.amount,
          category: expense.category,
          date: new Date().toISOString().split("T")[0],
          organization_id: org.id,
        },
        headers: syndicHeaders,
      });
    }
  });

  test("Un coproprietaire se connecte et explore son tableau de bord", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, ownerEmail, ownerPassword);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Verifier l'arrivee sur le tableau de bord owner
    // ============================================================
    await expect(page.getByTestId("owner-dashboard")).toBeVisible({
      timeout: 15000,
    });

    // Attendre que le dashboard charge completement
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Le dashboard affiche le message de bienvenue
    await expect(
      page.locator("h1").first(),
    ).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Consulter les widgets du dashboard
    // ============================================================
    // Scroller doucement pour montrer les stats et les immeubles
    await page.evaluate(() => {
      window.scrollTo({ top: 300, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Scroller encore pour voir les lots et tickets
    await page.evaluate(() => {
      window.scrollTo({ top: 600, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Scroller pour voir les actions rapides
    await page.evaluate(() => {
      window.scrollTo({ top: 900, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Revenir en haut
    await page.evaluate(() => {
      window.scrollTo({ top: 0, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 4 : Navigation vers la section Paiements
    // ============================================================
    await humanClick(page, "nav-link-paiements");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Paiements est chargee
    await expect(page.locator("main").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Retour au tableau de bord
    // ============================================================
    await humanClick(page, "nav-link-tableau-de-bord");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("owner-dashboard")).toBeVisible({
      timeout: 15000,
    });

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
