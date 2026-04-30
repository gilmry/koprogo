/**
 * SCENARIO: Consultation d'une convocation d'assemblee generale
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet de Francois (syndic) :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Convocations via le menu lateral
 *   3. Selection d'un immeuble
 *   4. Consultation de la liste des convocations
 *   5. Clic sur une convocation pour voir le detail
 *   6. Verification des informations de delai legal belge
 *
 * Duree video attendue : ~45-60 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanClick,
  humanClickLocator,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Francois consulte une convocation d'AG", () => {
  test.setTimeout(120_000);

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

    // 3. Create a convocation via Francois
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "francois@syndic-leroy.be", password: "francois123" },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // Get buildings to find Residence du Parc
    const buildingsResp = await request.get(`${API_BASE}/buildings`, {
      headers: syndicHeaders,
    });
    const buildings = await buildingsResp.json();
    const building = Array.isArray(buildings)
      ? buildings.find((b: any) => b.name?.includes("Residence du Parc"))
      : null;

    if (building) {
      // Create a meeting 30 days out
      const meetingDate = new Date();
      meetingDate.setDate(meetingDate.getDate() + 30);
      const meetingResp = await request.post(`${API_BASE}/meetings`, {
        data: {
          building_id: building.id,
          organization_id: building.organization_id,
          title: "Assemblee Generale Ordinaire annuelle",
          scheduled_date: meetingDate.toISOString(),
          meeting_type: "Ordinary",
          location: "Salle communale, 15 Rue de la Loi",
        },
        headers: syndicHeaders,
      });
      const meeting = await meetingResp.json();

      // Create a convocation for that meeting
      await request.post(`${API_BASE}/convocations`, {
        data: {
          meeting_id: meeting.id,
          building_id: building.id,
          meeting_type: "Ordinary",
          meeting_date: meetingDate.toISOString(),
          language: "fr",
        },
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

  test("Francois consulte une convocation et verifie le delai legal", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers les Convocations via le menu lateral
    // ============================================================
    await humanClick(page, "nav-link-convocations");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Convocations");
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Selectionner l'immeuble (Residence du Parc)
    // ============================================================
    await waitForSpinner(page);

    const buildingSelect = page.getByTestId("building-selector");
    if (await buildingSelect.isVisible({ timeout: 5000 })) {
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
    // ETAPE 4 : Verifier que la liste de convocations est visible
    // ============================================================
    await expect(page.getByTestId("convocation-list")).toBeVisible({
      timeout: 15000,
    });

    await expect(
      page.getByTestId("convocation-rows").locator("li").first(),
    ).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Cliquer sur la convocation pour voir le detail
    // ============================================================
    const convocationRow = page
      .getByTestId("convocation-rows")
      .locator("li")
      .first();
    await humanClickLocator(page, convocationRow.locator("a"));
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("convocation-detail")).toBeVisible({
      timeout: 15000,
    });
    await stepPause(page);

    // ============================================================
    // ETAPE 6 : Verifier les informations de delai legal
    // ============================================================
    const legalDeadline = page.getByTestId("convocation-detail-legal-deadline");
    await legalDeadline.scrollIntoViewIfNeeded();
    await expect(legalDeadline).toBeVisible({ timeout: 5000 });
    await stepPause(page);

    await expect(page.getByTestId("convocation-detail-status")).toBeVisible({
      timeout: 5000,
    });

    await expect(
      page.getByTestId("convocation-detail-recipients-summary"),
    ).toBeVisible({ timeout: 5000 });
    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
