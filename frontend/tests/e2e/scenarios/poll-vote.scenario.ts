/**
 * SCENARIO: Consultation d'un sondage et publication
 *
 * Documentation Vivante -- video exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Sondages via le menu lateral
 *   3. Selection d'un immeuble
 *   4. Verification que le sondage pre-cree (via API) apparait dans la liste
 *   5. Clic sur le sondage pour voir le detail
 *   6. Publication du sondage (Draft -> Active)
 *
 * Duree video attendue : ~50-60 secondes (rythme humain)
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

test.describe("Scenario: Consultation et publication d'un sondage", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;
  let pollId: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-syndic-poll-${ts}@koprogo.test`;
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
        name: `Scenario Poll Org ${ts}`,
        slug: `scenario-poll-${ts}`,
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
        first_name: "Sophie",
        last_name: "Martin",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Login syndic
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: syndicPassword },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 5. Create building
    buildingName = `Residence du Parc ${ts}`;

    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "25 Avenue du Parc",
        city: "Bruxelles",
        postal_code: "1060",
        country: "Belgium",
        total_units: 20,
        construction_year: 1975,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
    const building = await buildingResp.json();

    // 6. Create poll via API (Draft status)
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
  });

  test("Un syndic consulte un sondage et le publie", async ({ page }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers les Sondages via le menu lateral
    // ============================================================
    await humanClick(page, "nav-link-sondages");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Sondages est chargee
    await expect(page.locator("main h1").first()).toContainText("Sondages");
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Selectionner l'immeuble (ou attendre auto-selection)
    // ============================================================
    await waitForSpinner(page);

    // Wait for building selection to be ready
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
    // ETAPE 4 : Verifier que le sondage pre-cree apparait
    // ============================================================
    await expect(page.getByTestId("poll-list")).toBeVisible({
      timeout: 15000,
    });

    // The poll created via API should appear in the list
    const pollCard = page
      .getByTestId("poll-card")
      .filter({ hasText: "repeindre le hall" })
      .first();
    await expect(pollCard).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Cliquer sur le sondage pour voir le detail
    // ============================================================
    await humanClickLocator(page, pollCard.locator("a").first());
    await page.waitForLoadState("domcontentloaded");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que le detail du sondage est affiche
    await expect(page.getByTestId("poll-detail")).toBeVisible({
      timeout: 15000,
    });

    // Verifier que le titre du sondage est visible
    await expect(page.locator("text=repeindre le hall")).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 6 : Publier le sondage (Draft -> Active)
    // ============================================================
    // Le sondage est en Draft, on clique sur "Publier"
    page.on("dialog", (dialog) => dialog.accept());

    await humanClick(page, "poll-publish-button");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le sondage actif
    // ============================================================
    await finalPause(page);
  });
});
