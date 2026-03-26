/**
 * SCENARIO: Tableau d'affichage communautaire (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Le syndic se connecte, navigue vers les annonces, et cree une nouvelle annonce
 *   2. Un coproprietaire se connecte, consulte la liste des annonces, et lit le detail
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

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let ownerEmail: string;
  let ownerPassword: string;
  let buildingName: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-notice-syndic-${ts}@koprogo.test`;
    syndicPassword = "test123456";
    ownerEmail = `scenario-notice-owner-${ts}@koprogo.test`;
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
        name: `Notice Org ${ts}`,
        slug: `notice-${ts}`,
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
        first_name: "Claire",
        last_name: "Janssens",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Login syndic for authed requests
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: syndicPassword },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 5. Register owner user account
    const ownerRegResp = await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: ownerEmail,
        password: ownerPassword,
        first_name: "Thomas",
        last_name: "Leclercq",
        role: "owner",
        organization_id: org.id,
      },
    });
    const ownerUser = await ownerRegResp.json();
    const ownerUserId =
      ownerUser.user?.id || ownerUser.id || ownerUser.user_id || "";

    // 6. Create building
    buildingName = `Residence des Erables ${ts}`;
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "8 Avenue des Arts",
        city: "Liege",
        postal_code: "4000",
        country: "Belgium",
        total_units: 16,
        construction_year: 2005,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
    const building = await buildingResp.json();

    // 7. Create owner record linked to user
    await request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Thomas",
        last_name: "Leclercq",
        email: ownerEmail,
        address: "8 Avenue des Arts, Apt 5B",
        city: "Liege",
        postal_code: "4000",
        country: "Belgium",
        user_id: ownerUserId,
      },
      headers: syndicHeaders,
    });

    // 8. Pre-create and publish some notices via API
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
        contact_info: "Claire Janssens - syndic@residence-erables.be",
      },
      headers: syndicHeaders,
    });
    const notice2 = await notice2Resp.json();
    await request.post(`${API_BASE}/notices/${notice2.id}/publish`, {
      headers: syndicHeaders,
    });
  });

  test("Syndic cree une annonce, coproprietaire la consulte", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Le syndic se connecte et navigue vers les annonces
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
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

    // With a single building in the org, BuildingSelector auto-selects
    const buildingReady = page
      .locator(
        '[data-testid="building-selector"], [data-testid="building-selected"]',
      )
      .first();
    await expect(buildingReady).toBeVisible({ timeout: 15000 });

    // If there is a dropdown (multiple buildings case), select the right one
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
        if (text && text.includes("Residence des Erables")) {
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
    // ETAPE 3 : Le syndic cree une nouvelle annonce via le formulaire
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

    // La page recharge apres creation reussie
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Le coproprietaire se connecte et consulte les annonces
    // ============================================================
    await humanLogin(page, ownerEmail, ownerPassword);
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
        if (text && text.includes("Residence des Erables")) {
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
    // ETAPE 6 : Le coproprietaire consulte la liste des annonces
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
    // ETAPE 7 : Le coproprietaire clique sur une annonce pour le detail
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
