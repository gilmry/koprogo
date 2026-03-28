/**
 * SCENARIO: Marketplace du SEL entre deux coproprietaires (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Alice (coproprietaire) se connecte, navigue vers le SEL, consulte les offres
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

    // 3. Create exchange offers via Alice and Bob
    const aliceResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "alice@residence-parc.be", password: "alice123" },
    });
    const alice = await aliceResp.json();
    const aliceHeaders = { Authorization: `Bearer ${alice.token}` };

    const bobResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "bob@residence-parc.be", password: "bob123" },
    });
    const bob = await bobResp.json();
    const bobHeaders = { Authorization: `Bearer ${bob.token}` };

    // Get building ID for Residence du Parc
    const buildingsResp = await request.get(`${API_BASE}/buildings`, {
      headers: aliceHeaders,
    });
    const buildings = await buildingsResp.json();
    const building = Array.isArray(buildings)
      ? buildings.find((b: any) => b.name?.includes("Residence du Parc"))
      : null;

    if (building) {
      // Alice creates exchange offers
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

      // Bob creates an exchange offer
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

  test("Alice consulte ses offres, Bob parcourt la marketplace", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Alice se connecte et consulte ses offres SEL
    // ============================================================
    await humanLogin(page, "alice@residence-parc.be", "alice123");
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
        if (text && text.includes("Residence du Parc")) {
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
    await humanLogin(page, "bob@residence-parc.be", "bob123");
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
        if (text && text.includes("Residence du Parc")) {
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
    const statsPanel = page.getByTestId("sel-statistics");
    try {
      await statsPanel.waitFor({ state: "visible", timeout: 5000 });
      await statsPanel.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BETWEEN_STEPS);
    } catch {
      // Statistics panel may not render if no completed exchanges
    }

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
