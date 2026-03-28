/**
 * SCENARIO: Consultation et vote sur un sondage (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Francois (syndic) se connecte, cree un sondage et le publie
 *   2. Alice (coproprietaire) se connecte, trouve le sondage et vote
 *   3. Francois revient, cloture le sondage et consulte les resultats
 *
 * Cadre legal: Article 577-8/4 §4 du Code Civil Belge
 * Duree video attendue : ~90-120 secondes (rythme humain, multi-role)
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

test.describe("Scenario: Sondage multi-role (Francois lance, Alice vote)", () => {
  test.setTimeout(180_000);

  let seedData: any;
  let pollId: string;

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

    // 3. Create and publish a poll via Francois
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
      const startsAt = new Date();
      const endsAt = new Date();
      endsAt.setDate(endsAt.getDate() + 7);

      const pollResp = await request.post(`${API_BASE}/polls`, {
        data: {
          building_id: building.id,
          poll_type: "yes_no",
          title: "Faut-il repeindre le hall d'entree en bleu ?",
          description:
            "Suite aux remarques de plusieurs coproprietaires lors de la " +
            "derniere AG, nous souhaitons recueillir l'avis de tous " +
            "concernant la couleur du hall d'entree.",
          starts_at: startsAt.toISOString(),
          ends_at: endsAt.toISOString(),
          is_anonymous: false,
          allow_multiple_votes: false,
          options: [
            { option_text: "Oui", display_order: 1 },
            { option_text: "Non", display_order: 2 },
          ],
        },
        headers: syndicHeaders,
      });
      const poll = await pollResp.json();
      pollId = poll.id;

      // Publish the poll (Draft -> Active)
      await request.put(`${API_BASE}/polls/${pollId}/publish`, {
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

  test("Francois consulte, Alice vote, Francois cloture et voit les resultats", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Francois se connecte et consulte le sondage actif
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    await humanClick(page, "nav-link-sondages");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Sondages");
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
    if (await buildingSelect.isVisible({ timeout: 2000 }).catch(() => false)) {
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

    // Verifier que le sondage actif apparait dans la liste
    await expect(page.getByTestId("poll-list")).toBeVisible({
      timeout: 15000,
    });

    const pollCard = page
      .getByTestId("poll-card")
      .filter({ hasText: "repeindre le hall" })
      .first();
    await expect(pollCard).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Alice se connecte et vote sur le sondage
    // ============================================================
    await humanLogin(page, "alice@residence-parc.be", "alice123");
    await stepPause(page);

    // Naviguer vers les sondages (community section)
    await humanClick(page, "nav-link-sondages");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Sondages");

    // Attendre la selection de l'immeuble
    await waitForSpinner(page);
    const buildingReady2 = page
      .locator(
        '[data-testid="building-selector"], [data-testid="building-selected"]',
      )
      .first();
    await expect(buildingReady2).toBeVisible({ timeout: 15000 });

    const buildingSelect2 = page.getByTestId("building-selector");
    if (await buildingSelect2.isVisible({ timeout: 2000 }).catch(() => false)) {
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

    // Trouver le sondage et cliquer pour voir le detail
    await expect(page.getByTestId("poll-list")).toBeVisible({
      timeout: 15000,
    });

    const pollCard2 = page
      .getByTestId("poll-card")
      .filter({ hasText: "repeindre le hall" })
      .first();
    await expect(pollCard2).toBeVisible({ timeout: 15000 });

    await humanClickLocator(page, pollCard2.locator("a").first());
    await page.waitForLoadState("domcontentloaded");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que le detail du sondage est affiche
    await expect(page.getByTestId("poll-detail")).toBeVisible({
      timeout: 15000,
    });
    await expect(page.locator("text=repeindre le hall")).toBeVisible({
      timeout: 10000,
    });

    // Voter "Oui"
    const voteOui = page.locator("button").filter({ hasText: /Oui/i }).first();
    await humanClickLocator(page, voteOui);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Francois revient, cloture le sondage et consulte les resultats
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    // Naviguer vers les sondages
    await humanClick(page, "nav-link-sondages");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Attendre la selection de l'immeuble
    await waitForSpinner(page);
    const buildingReady3 = page
      .locator(
        '[data-testid="building-selector"], [data-testid="building-selected"]',
      )
      .first();
    await expect(buildingReady3).toBeVisible({ timeout: 15000 });

    const buildingSelect3 = page.getByTestId("building-selector");
    if (await buildingSelect3.isVisible({ timeout: 2000 }).catch(() => false)) {
      await buildingSelect3.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const options = await buildingSelect3.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Residence du Parc")) {
          const value = await option.getAttribute("value");
          if (value) {
            await buildingSelect3.selectOption(value);
            break;
          }
        }
      }
      await page.waitForTimeout(PACE.AFTER_SELECT);
    }
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Trouver le sondage et ouvrir le detail
    await expect(page.getByTestId("poll-list")).toBeVisible({
      timeout: 15000,
    });

    const pollCard3 = page
      .getByTestId("poll-card")
      .filter({ hasText: "repeindre le hall" })
      .first();
    await expect(pollCard3).toBeVisible({ timeout: 15000 });

    await humanClickLocator(page, pollCard3.locator("a").first());
    await page.waitForLoadState("domcontentloaded");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("poll-detail")).toBeVisible({
      timeout: 15000,
    });

    // Cloturer le sondage (Active -> Closed)
    page.on("dialog", (dialog) => dialog.accept());

    await humanClick(page, "poll-close-button");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre les resultats
    // ============================================================
    await finalPause(page);
  });
});
