/**
 * SCENARIO: Cycle de vie d'un ticket de maintenance
 *
 * Documentation Vivante — vidéo exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Tickets via le menu latéral
 *   3. Sélection d'un immeuble
 *   4. Création d'un ticket via le formulaire UI
 *   5. Vérification que le ticket apparaît dans la liste
 *   6. Navigation vers le détail du ticket
 *
 * Durée vidéo attendue : ~45-60 secondes (rythme humain)
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

// Scenarios are human-paced — allow 2 minutes per test
test.describe("Scénario: Cycle de vie d'un ticket de maintenance", () => {
  test.setTimeout(120_000);
  // ----- Données de test (créées via API, invisibles en vidéo) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-syndic-${ts}@koprogo.test`;
    syndicPassword = "test123456";

    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Create org
    const orgResp = await request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Scenario Org ${ts}`,
        slug: `scenario-${ts}`,
        contact_email: syndicEmail,
        subscription_plan: "professional",
      },
      headers: adminHeaders,
    });
    const org = await orgResp.json();

    // 3. Register syndic
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: syndicEmail,
        password: syndicPassword,
        first_name: "Marie",
        last_name: "Dupont",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Create building
    buildingName = `Résidence Bellevue ${ts}`;
    await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "42 Avenue Louise",
        city: "Bruxelles",
        postal_code: "1050",
        country: "Belgium",
        total_units: 12,
        construction_year: 1985,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
  });

  test("Un syndic crée un ticket de maintenance via l'interface", async ({
    page,
  }) => {
    // ============================================================
    // ÉTAPE 1 : Connexion (visible dans la vidéo)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ÉTAPE 2 : Navigation vers les Tickets via le menu latéral
    // ============================================================
    await humanClick(page, "nav-link-tickets");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que la page Tickets est chargée
    await expect(page.locator("main h1").first()).toContainText("Tickets");
    await stepPause(page);

    // ============================================================
    // ÉTAPE 3 : Sélectionner l'immeuble
    // ============================================================
    // Attendre que le BuildingSelector charge les données
    await waitForSpinner(page);

    // Sélectionner l'immeuble dans le dropdown
    const buildingSelect = page.getByTestId("building-selector");
    if (await buildingSelect.isVisible({ timeout: 5000 })) {
      await buildingSelect.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      // Sélectionner par texte partiel (le nom contient le timestamp)
      const options = await buildingSelect.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Résidence Bellevue")) {
          const value = await option.getAttribute("value");
          if (value) {
            await buildingSelect.selectOption(value);
            break;
          }
        }
      }
      await page.waitForTimeout(PACE.AFTER_SELECT);
    }
    await waitForSpinner(page);
    await stepPause(page);

    // ============================================================
    // ÉTAPE 4 : Ouvrir le formulaire de création de ticket
    // ============================================================
    await humanClick(page, "tickets-create-btn");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que le modal/formulaire est ouvert
    await expect(page.getByTestId("ticket-create-form")).toBeVisible({
      timeout: 10000,
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // ============================================================
    // ÉTAPE 5 : Remplir le formulaire de création
    // ============================================================
    await humanFill(
      page,
      "ticket-title-input",
      "Fuite d'eau dans le hall du 2ème étage",
    );

    await humanFill(
      page,
      "ticket-description-input",
      "Une fuite d'eau importante a été constatée au plafond du hall " +
        "d'entrée du 2ème étage. L'eau s'infiltre depuis l'appartement " +
        "du 3ème étage. Intervention urgente nécessaire.",
    );

    await humanSelect(page, "ticket-priority-select", "High");

    await humanSelect(page, "ticket-category-select", "Plumbing");

    await stepPause(page);

    // ============================================================
    // ÉTAPE 6 : Soumettre le ticket
    // ============================================================
    await humanClick(page, "ticket-submit-btn");
    await waitForSpinner(page);

    // La page devrait recharger et afficher le ticket dans la liste
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ÉTAPE 7 : Vérifier que le ticket apparaît dans la liste
    // ============================================================
    // Après la soumission, la page recharge (window.location.reload)
    // Le building_id est perdu → on attend puis re-navigue via le menu
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Re-cliquer sur Tickets dans le menu pour forcer le rechargement
    await humanClick(page, "nav-link-tickets");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Attendre que le BuildingSelector auto-sélectionne et recharge la liste
    await waitForSpinner(page);
    await page.waitForTimeout(3000); // Laisser le temps au composant de recharger

    // Chercher le ticket créé
    await expect(
      page.locator("text=Fuite d'eau dans le hall"),
    ).toBeVisible({ timeout: 20000 });

    await stepPause(page);

    // ============================================================
    // ÉTAPE 8 : Cliquer sur le ticket pour voir le détail
    // ============================================================
    const ticketRow = page
      .locator('[data-testid="ticket-row"]')
      .filter({ hasText: "Fuite d'eau" })
      .first();
    await humanClickLocator(page, ticketRow);

    // Vérifier que la page de détail est affichée
    await expect(page.getByTestId("ticket-detail-title")).toContainText(
      "Fuite d'eau",
      { timeout: 10000 },
    );

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la vidéo montre le résultat
    // ============================================================
    await finalPause(page);
  });
});
