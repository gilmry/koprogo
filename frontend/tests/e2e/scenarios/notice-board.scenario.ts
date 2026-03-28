/**
 * SCENARIO: Tableau d'affichage communautaire (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Francois (syndic) se connecte, navigue vers les annonces, et cree une nouvelle annonce
 *   2. Alice (coproprietaire) se connecte, consulte la liste des annonces, et lit le detail
 *
 * Duree video attendue : ~70-90 secondes (rythme humain, multi-role)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanFill,
  humanClick,
  humanClickLocator,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Tableau d'affichage communautaire (multi-role)", () => {
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

    // 3. Pre-create and publish some notices via Francois
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "francois@syndic-leroy.be", password: "francois123" },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // Get building ID for Residence du Parc
    const buildingsResp = await request.get(`${API_BASE}/buildings`, {
      headers: syndicHeaders,
    });
    const buildings = await buildingsResp.json();
    const building = Array.isArray(buildings)
      ? buildings.find((b: any) => b.name?.includes("Residence du Parc"))
      : null;

    if (building) {
      const notice1Resp = await request.post(`${API_BASE}/notices`, {
        data: {
          building_id: building.id,
          notice_type: "Announcement",
          category: "General",
          title: "Nettoyage des communs ce samedi",
          content:
            "Un nettoyage complet des parties communes est prevu ce samedi de 9h a 12h. " +
            "Merci de ne pas laisser d'objets dans les couloirs.",
        },
        headers: syndicHeaders,
      });
      const notice1 = await notice1Resp.json();
      await request.post(`${API_BASE}/notices/${notice1.id}/publish`, {
        headers: syndicHeaders,
      });

      const notice2Resp = await request.post(`${API_BASE}/notices`, {
        data: {
          building_id: building.id,
          notice_type: "Event",
          category: "Social",
          title: "Barbecue de quartier - Fete des voisins",
          content:
            "A l'occasion de la fete des voisins, un barbecue est organise dans le " +
            "jardin commun. Chacun apporte un plat a partager.",
          event_date: new Date(
            Date.now() + 14 * 24 * 60 * 60 * 1000,
          ).toISOString(),
          event_location: "Jardin commun, rez-de-chaussee",
          contact_info: "Francois Leroy - francois@syndic-leroy.be",
        },
        headers: syndicHeaders,
      });
      const notice2 = await notice2Resp.json();
      await request.post(`${API_BASE}/notices/${notice2.id}/publish`, {
        headers: syndicHeaders,
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

  test("Francois cree une annonce, Alice la consulte", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Francois se connecte et navigue vers les annonces
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    await humanClick(page, "nav-link-annonces");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Annonces");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Attendre la selection automatique de l'immeuble
    // ============================================================
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
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Francois cree une nouvelle annonce via le formulaire
    // ============================================================
    await humanClick(page, "notices-create-btn");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("notice-create-modal")).toBeVisible({
      timeout: 10000,
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    await humanFill(
      page,
      "notice-title-input",
      "Travaux ascenseur - Interruption prevue",
    );

    await humanFill(
      page,
      "notice-content-input",
      "L'ascenseur sera hors service du lundi au mercredi de la semaine " +
        "prochaine pour une maintenance preventive obligatoire. " +
        "Nous vous prions d'utiliser les escaliers durant cette periode.",
    );

    await stepPause(page);

    // Soumettre l'annonce
    await humanClick(page, "notice-submit-btn");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Alice se connecte et consulte les annonces
    // ============================================================
    await humanLogin(page, "alice@residence-parc.be", "alice123");
    await stepPause(page);

    // Naviguer vers les annonces (community section, shared nav)
    await humanClick(page, "nav-link-annonces");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Annonces");
    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Attendre la selection de l'immeuble et la liste
    // ============================================================
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

    // ============================================================
    // ETAPE 6 : Alice consulte la liste des annonces
    // ============================================================
    await expect(page.getByTestId("notice-list")).toBeVisible({
      timeout: 15000,
    });

    await expect(
      page.getByTestId("notice-list-row").first(),
    ).toBeVisible({ timeout: 15000 });

    // Verifier qu'une annonce pre-creee est visible
    await expect(
      page.locator("text=Nettoyage des communs"),
    ).toBeVisible({ timeout: 10000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 7 : Alice clique sur une annonce pour le detail
    // ============================================================
    const noticeRow = page
      .getByTestId("notice-list-row")
      .filter({ hasText: "Barbecue de quartier" })
      .first();
    await humanClickLocator(page, noticeRow.locator("a"));
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("notice-detail")).toBeVisible({
      timeout: 15000,
    });

    await expect(page.locator("h1")).toContainText("Barbecue de quartier");
    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
