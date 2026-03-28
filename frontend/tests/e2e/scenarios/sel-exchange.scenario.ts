/**
 * SCENARIO: Marketplace du SEL entre deux coproprietaires (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Alice (coproprietaire) se connecte, navigue vers le SEL, consulte ses offres
 *   2. Bob (coproprietaire) se connecte, parcourt la marketplace, consulte l'offre d'Alice
 *   3. Bob consulte le leaderboard et les statistiques communautaires
 *
 * Cadre legal belge: Les SEL sont legaux et non-taxables si non-commerciaux
 * Duree video attendue : ~70-90 secondes (rythme humain, multi-role)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanFill,
  waitForSpinner,
  stepPause,
  finalPause,
  humanGoto,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: SEL multi-role (Alice offre, Bob parcourt)", () => {
  test.setTimeout(180_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let aliceEmail: string;
  let alicePassword: string;
  let bobEmail: string;
  let bobPassword: string;
  let buildingName: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    aliceEmail = `scenario-sel-alice-${ts}@koprogo.test`;
    alicePassword = "test123456";
    bobEmail = `scenario-sel-bob-${ts}@koprogo.test`;
    bobPassword = "test123456";

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
        contact_email: `org-sel-${ts}@koprogo.test`,
        subscription_plan: "professional",
      },
      headers: adminHeaders,
    });
    const org = await orgResp.json();

    // 3. Register syndic (needed to create building + owners)
    const syndicEmail = `scenario-sel-syndic-${ts}@koprogo.test`;
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: syndicEmail,
        password: "test123456",
        first_name: "Marc",
        last_name: "Lecomte",
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

    // 5. Register Alice (owner user account)
    const aliceRegResp = await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: aliceEmail,
        password: alicePassword,
        first_name: "Alice",
        last_name: `Janssen${ts}`,
        role: "owner",
        organization_id: org.id,
      },
    });
    const aliceUser = await aliceRegResp.json();
    const aliceUserId =
      aliceUser.user?.id || aliceUser.id || aliceUser.user_id || "";

    // Create Alice's owner record linked to user
    const alice = await request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Alice",
        last_name: `Janssen${ts}`,
        email: aliceEmail,
        address: "15 Place Communale, Apt 1",
        city: "Ixelles",
        postal_code: "1050",
        country: "Belgium",
        user_id: aliceUserId,
      },
      headers: syndicHeaders,
    });
    const aliceOwner = await alice.json();

    // 6. Register Bob (owner user account)
    const bobRegResp = await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: bobEmail,
        password: bobPassword,
        first_name: "Bob",
        last_name: `Peeters${ts}`,
        role: "owner",
        organization_id: org.id,
      },
    });
    const bobUser = await bobRegResp.json();
    const bobUserId =
      bobUser.user?.id || bobUser.id || bobUser.user_id || "";

    // Create Bob's owner record linked to user
    const bob = await request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Bob",
        last_name: `Peeters${ts}`,
        email: bobEmail,
        address: "15 Place Communale, Apt 2",
        city: "Ixelles",
        postal_code: "1050",
        country: "Belgium",
        user_id: bobUserId,
      },
      headers: syndicHeaders,
    });
    const bobOwner = await bob.json();

    // 7. Login as Alice and Bob to create exchanges (provider_id resolved from auth)
    const aliceLoginResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: aliceEmail, password: alicePassword },
    });
    const aliceAuth = await aliceLoginResp.json();
    const aliceHeaders = { Authorization: `Bearer ${aliceAuth.token}` };

    const bobLoginResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: bobEmail, password: bobPassword },
    });
    const bobAuth = await bobLoginResp.json();
    const bobHeaders = { Authorization: `Bearer ${bobAuth.token}` };

    // Create exchange offers by Alice via API (provider resolved from Alice's JWT)
    await request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: building.id,
        exchange_type: "Service",
        title: "Cours de jardinage",
        description:
          "Je propose de vous apprendre les bases du jardinage urbain sur balcon. " +
          "Entretien des plantes, plantation de legumes, compostage.",
        credits: 2,
      },
      headers: aliceHeaders,
    });

    await request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: building.id,
        exchange_type: "ObjectLoan",
        title: "Pret de perceuse-visseuse",
        description:
          "Perceuse-visseuse sans fil Bosch disponible pour vos travaux. " +
          "Incluant jeu de meches et embouts.",
        credits: 1,
      },
      headers: aliceHeaders,
    });

    // Create exchange offer by Bob via API (provider resolved from Bob's JWT)
    await request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: building.id,
        exchange_type: "SharedPurchase",
        title: "Achat groupe pellets chauffage",
        description:
          "Organisation d'un achat groupe de pellets pour la saison hivernale. " +
          "Economie estimee de 15% sur le prix individuel.",
        credits: 3,
      },
      headers: bobHeaders,
    });
  });

  test("Alice consulte ses offres, Bob parcourt la marketplace", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Alice se connecte et consulte ses offres SEL
    // ============================================================
    await humanLogin(page, aliceEmail, alicePassword);
    await stepPause(page);

    // Naviguer vers le SEL (community section)
    await humanGoto(page, "/exchanges");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("SEL");
    await stepPause(page);

    // Attendre la selection de l'immeuble
    await waitForSpinner(page);
    const buildingReady = page
      .locator(
        '[data-testid="building-selector"], [data-testid="building-selected"]',
      )
      .first();
    await expect(buildingReady).toBeVisible({ timeout: 15000 });

    const buildingSelect = page.getByTestId("building-selector");
    if (
      await buildingSelect
        .isVisible({ timeout: 2000 })
        .catch(() => false)
    ) {
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

    // Attendre que les echanges chargent
    const exchangeList = page.getByTestId("exchange-list");
    await expect(exchangeList).toBeVisible({ timeout: 15000 });

    await expect(
      page.getByTestId("exchange-list-row").first(),
    ).toBeVisible({ timeout: 20000 });

    // Alice voit ses propres offres
    await expect(
      page.locator("text=Cours de jardinage"),
    ).toBeVisible({ timeout: 15000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Bob se connecte et parcourt la marketplace
    // ============================================================
    await humanLogin(page, bobEmail, bobPassword);
    await stepPause(page);

    // Naviguer vers le SEL
    await humanGoto(page, "/exchanges");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("SEL");

    // Attendre la selection de l'immeuble
    await waitForSpinner(page);
    const buildingReady2 = page
      .locator(
        '[data-testid="building-selector"], [data-testid="building-selected"]',
      )
      .first();
    await expect(buildingReady2).toBeVisible({ timeout: 15000 });

    const buildingSelect2 = page.getByTestId("building-selector");
    if (
      await buildingSelect2
        .isVisible({ timeout: 2000 })
        .catch(() => false)
    ) {
      await buildingSelect2.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const options = await buildingSelect2.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Residence Solidaire")) {
          const value = await option.getAttribute("value");
          if (value) {
            await buildingSelect2.selectOption(value);
            break;
          }
        }
      }
      await page.waitForTimeout(PACE.AFTER_SELECT);
    }
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Attendre que les echanges chargent
    const exchangeList2 = page.getByTestId("exchange-list");
    await expect(exchangeList2).toBeVisible({ timeout: 15000 });

    await expect(
      page.getByTestId("exchange-list-row").first(),
    ).toBeVisible({ timeout: 20000 });

    // Bob voit les offres d'Alice dans la marketplace
    await expect(
      page.locator("text=Cours de jardinage"),
    ).toBeVisible({ timeout: 15000 });
    await expect(
      page.locator("text=Pret de perceuse-visseuse"),
    ).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Bob consulte le leaderboard et les statistiques
    // ============================================================
    // Consulter les statistiques SEL
    const statsPanel = page.getByTestId("sel-statistics");
    try {
      await statsPanel.waitFor({ state: "visible", timeout: 5000 });
      await statsPanel.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BETWEEN_STEPS);
    } catch {
      // Statistics panel may not render if no completed exchanges
    }

    // Consulter le classement (Leaderboard)
    const leaderboard = page.getByTestId("leaderboard");
    try {
      await leaderboard.waitFor({ state: "visible", timeout: 5000 });
      await leaderboard.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BETWEEN_STEPS);
    } catch {
      // Leaderboard may not render if no credit balances yet
    }

    await stepPause(page);

    // Bob utilise la recherche pour trouver l'offre de perceuse
    const searchInput = page.getByTestId("exchange-search-input");
    if (await searchInput.isVisible({ timeout: 3000 }).catch(() => false)) {
      await humanFill(page, "exchange-search-input", "perceuse");
      await page.waitForTimeout(PACE.BETWEEN_STEPS);

      await expect(
        page.locator("text=perceuse-visseuse"),
      ).toBeVisible({ timeout: 10000 });

      // Effacer la recherche
      await humanFill(page, "exchange-search-input", "");
      await page.waitForTimeout(PACE.BETWEEN_STEPS);
    }

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
