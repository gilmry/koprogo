/**
 * SCENARIO: Cycle de vie d'un ticket de maintenance (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Charlie (coproprietaire) se connecte et signale une fuite
 *   2. Francois (syndic) se connecte, assigne le ticket
 *   3. Verification que le ticket apparait dans la liste
 *
 * Duree video attendue : ~70-90 secondes (rythme humain, multi-role)
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

test.describe("Scenario: Cycle de vie d'un ticket de maintenance", () => {
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

  test("Charlie signale une fuite, Francois assigne le ticket", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Charlie (coproprietaire) se connecte
    // ============================================================
    await humanLogin(page, "charlie@residence-parc.be", "charlie123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers Mes Tickets via le menu lateral
    // ============================================================
    await humanClick(page, "nav-link-mes-tickets");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Tickets");
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Ouvrir le formulaire de creation de ticket
    // ============================================================
    await humanClick(page, "tickets-create-btn");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("ticket-create-form")).toBeVisible({
      timeout: 10000,
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // ============================================================
    // ETAPE 4 : Remplir le formulaire de creation
    // ============================================================
    await humanFill(
      page,
      "ticket-title-input",
      "Fuite d'eau dans le hall du 2eme etage",
    );

    await humanFill(
      page,
      "ticket-description-input",
      "Une fuite d'eau importante a ete constatee au plafond du hall " +
        "d'entree du 2eme etage. L'eau s'infiltre depuis l'appartement " +
        "du 3eme etage. Intervention urgente necessaire.",
    );

    await humanSelect(page, "ticket-priority-select", "High");

    await humanSelect(page, "ticket-category-select", "Plumbing");

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Soumettre le ticket
    // ============================================================
    await humanClick(page, "ticket-submit-btn");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 6 : Verifier que le ticket apparait dans la liste
    // ============================================================
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await humanClick(page, "nav-link-mes-tickets");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await waitForSpinner(page);
    await page.waitForTimeout(3000);

    await expect(page.locator("text=Fuite d'eau dans le hall")).toBeVisible({
      timeout: 20000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 7 : Francois (syndic) se connecte et consulte les tickets
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    await humanClick(page, "nav-link-tickets");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Tickets");
    await stepPause(page);

    // Selectionner l'immeuble Residence du Parc
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

    // Verifier que le ticket de Charlie apparait
    await expect(page.locator("text=Fuite d'eau dans le hall")).toBeVisible({
      timeout: 20000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 8 : Cliquer sur le ticket pour voir le detail
    // ============================================================
    const ticketRow = page
      .locator('[data-testid="ticket-row"]')
      .filter({ hasText: "Fuite d'eau" })
      .first();
    await humanClickLocator(page, ticketRow);

    await expect(page.getByTestId("ticket-detail-title")).toContainText(
      "Fuite d'eau",
      { timeout: 10000 },
    );

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
