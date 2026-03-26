/**
 * SCENARIO: Marketplace du Systeme d'Echange Local (SEL)
 *
 * Documentation Vivante -- video exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page SEL via le menu lateral
 *   3. Selection d'un immeuble
 *   4. Consultation du tableau de bord SEL (statistiques, classement)
 *   5. Consultation de la liste des echanges
 *   6. Utilisation des filtres de recherche
 *
 * Duree video attendue : ~45-55 secondes (rythme humain)
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

test.describe("Scenario: Marketplace du SEL (Systeme d'Echange Local)", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-syndic-sel-${ts}@koprogo.test`;
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
        name: `Scenario SEL Org ${ts}`,
        slug: `scenario-sel-${ts}`,
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
        first_name: "Marc",
        last_name: "Lecomte",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // Login as syndic to create resources
    const syndicLoginResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: syndicPassword },
    });
    const syndic = await syndicLoginResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 4. Create building
    buildingName = `Residence Solidaire ${ts}`;
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "15 Place Communale",
        city: "Ixelles",
        postal_code: "1050",
        country: "Belgium",
        total_units: 16,
        construction_year: 2000,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
    const building = await buildingResp.json();

    // 5. Create two owners for the community
    const owner1Resp = await request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Alice",
        last_name: `Janssen${ts}`,
        email: `alice-sel-${ts}@koprogo.test`,
        address: "15 Place Communale, Apt 1",
        city: "Ixelles",
        postal_code: "1050",
        country: "Belgium",
      },
      headers: syndicHeaders,
    });
    const owner1 = await owner1Resp.json();

    const owner2Resp = await request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Bob",
        last_name: `Peeters${ts}`,
        email: `bob-sel-${ts}@koprogo.test`,
        address: "15 Place Communale, Apt 2",
        city: "Ixelles",
        postal_code: "1050",
        country: "Belgium",
      },
      headers: syndicHeaders,
    });
    const owner2 = await owner2Resp.json();

    // 6. Create some sample exchanges via API (to populate the marketplace)
    await request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: building.id,
        provider_id: owner1.id,
        exchange_type: "Service",
        title: "Cours de jardinage",
        description:
          "Je propose de vous apprendre les bases du jardinage urbain sur balcon. " +
          "Entretien des plantes, plantation de legumes, compostage.",
        credits: 2,
      },
      headers: syndicHeaders,
    });

    await request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: building.id,
        provider_id: owner2.id,
        exchange_type: "ObjectLoan",
        title: "Pret de perceuse-visseuse",
        description:
          "Perceuse-visseuse sans fil Bosch disponible pour vos travaux. " +
          "Incluant jeu de meches et embouts.",
        credits: 1,
      },
      headers: syndicHeaders,
    });

    await request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: building.id,
        provider_id: owner1.id,
        exchange_type: "SharedPurchase",
        title: "Achat groupe pellets chauffage",
        description:
          "Organisation d'un achat groupe de pellets pour la saison hivernale. " +
          "Economie estimee de 15% sur le prix individuel.",
        credits: 3,
      },
      headers: syndicHeaders,
    });
  });

  test("Un syndic consulte la marketplace SEL et ses statistiques", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers le SEL via le menu lateral
    // ============================================================
    await humanClick(page, "nav-link-sel");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page SEL est chargee
    await expect(page.locator("main h1").first()).toContainText("SEL");
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Selectionner l'immeuble (ou attendre auto-selection)
    // ============================================================
    await waitForSpinner(page);

    // Wait for building selection to be ready
    const buildingReady = page.locator('[data-testid="building-selector"], [data-testid="building-selected"]').first();
    await expect(buildingReady).toBeVisible({ timeout: 15000 });

    const buildingSelect = page.getByTestId("building-selector");
    if (await buildingSelect.isVisible({ timeout: 2000 }).catch(() => false)) {
      await buildingSelect.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const options = await buildingSelect.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Residence Solidaire")) {
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
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Attendre que les donnees chargent apres selection
    // ============================================================
    // The ExchangeList component loads data asynchronously after building selection.
    // Wait for the exchange-list container AND for actual rows to appear.
    const exchangeList = page.getByTestId("exchange-list");
    await expect(exchangeList).toBeVisible({ timeout: 15000 });

    // Wait for actual exchange rows to render (not just the container)
    await expect(
      page.getByTestId("exchange-list-row").first(),
    ).toBeVisible({ timeout: 20000 });

    // ============================================================
    // ETAPE 5 : Consulter les statistiques SEL
    // ============================================================
    const statsPanel = page.getByTestId("sel-statistics");
    try {
      await statsPanel.waitFor({ state: "visible", timeout: 5000 });
      await statsPanel.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BETWEEN_STEPS);
    } catch {
      // Statistics panel may not render if no completed exchanges
    }
    await stepPause(page);

    // ============================================================
    // ETAPE 6 : Consulter le classement (Leaderboard)
    // ============================================================
    const leaderboard = page.getByTestId("leaderboard");
    try {
      await leaderboard.waitFor({ state: "visible", timeout: 5000 });
      await leaderboard.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BETWEEN_STEPS);
    } catch {
      // Leaderboard may not render if no credit balances yet
    }
    await stepPause(page);

    // ============================================================
    // ETAPE 7 : Consulter la liste des echanges
    // ============================================================
    await exchangeList.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Verifier que les echanges sont affiches
    await expect(
      page.locator("text=Cours de jardinage"),
    ).toBeVisible({ timeout: 15000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 8 : Utiliser les filtres de recherche
    // ============================================================

    // Filtrer par type "Service"
    const typeFilter = page.getByTestId("exchange-filter-type");
    if (await typeFilter.isVisible({ timeout: 5000 })) {
      await typeFilter.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      await typeFilter.selectOption("Service");
      await page.waitForTimeout(PACE.AFTER_SELECT);
      await page.waitForTimeout(PACE.BETWEEN_STEPS);
    }

    // Remettre "Tous les types"
    if (await typeFilter.isVisible({ timeout: 3000 })) {
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      await typeFilter.selectOption("all");
      await page.waitForTimeout(PACE.AFTER_SELECT);
    }

    // Recherche textuelle
    await humanFill(page, "exchange-search-input", "perceuse");
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Verifier que le resultat filtre est visible
    await expect(
      page.locator("text=perceuse-visseuse"),
    ).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // Effacer la recherche pour montrer tous les echanges
    await humanFill(page, "exchange-search-input", "");
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
