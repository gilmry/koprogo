/**
 * SCENARIO: Creation d'un sondage et publication
 *
 * Documentation Vivante -- video exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Sondages via le menu lateral
 *   3. Selection d'un immeuble
 *   4. Creation d'un sondage Oui/Non via le formulaire
 *   5. Verification du sondage cree (redirection vers le detail)
 *   6. Publication du sondage (Draft -> Active)
 *
 * Duree video attendue : ~50-60 secondes (rythme humain)
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

test.describe("Scenario: Creation et publication d'un sondage", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;

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

    // 4. Create building
    buildingName = `Residence du Parc ${ts}`;

    await request.post(`${API_BASE}/buildings`, {
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
  });

  test("Un syndic cree un sondage Oui/Non et le publie", async ({ page }) => {
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
    const buildingReady = page.locator('[data-testid="building-selector"], [data-testid="building-selected"]').first();
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

    // ============================================================
    // ETAPE 4 : Cliquer sur "Nouveau sondage"
    // ============================================================
    await humanClick(page, "poll-create-button");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page de creation est chargee
    await page.waitForLoadState("domcontentloaded");
    await expect(page.getByTestId("create-poll-form")).toBeVisible({
      timeout: 15000,
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // ============================================================
    // ETAPE 5 : Selectionner l'immeuble dans le formulaire (ou attendre auto-selection)
    // ============================================================
    await waitForSpinner(page);

    // Wait for building selection to be ready in the form
    const formBuildingReady = page.locator('[data-testid="building-selector"], [data-testid="building-selected"]').first();
    await expect(formBuildingReady).toBeVisible({ timeout: 15000 });

    const formBuildingSelect = page.getByTestId("building-selector");
    if (await formBuildingSelect.isVisible({ timeout: 2000 }).catch(() => false)) {
      await formBuildingSelect.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const formOptions = await formBuildingSelect.locator("option").all();
      for (const option of formOptions) {
        const text = await option.textContent();
        if (text && text.includes("Residence du Parc")) {
          const value = await option.getAttribute("value");
          if (value) {
            await formBuildingSelect.selectOption(value);
            break;
          }
        }
      }
      await page.waitForTimeout(PACE.AFTER_SELECT);
    }
    await stepPause(page);

    // ============================================================
    // ETAPE 6 : Remplir le formulaire de creation
    // ============================================================

    // Le type YesNo est selectionne par defaut

    // Question
    await humanFill(
      page,
      "create-poll-question-input",
      "Faut-il repeindre le hall d'entree en bleu ?",
    );

    // Description (optionnelle)
    await humanFill(
      page,
      "create-poll-description-input",
      "Suite aux remarques de plusieurs coproprietaires lors de la " +
        "derniere AG, nous souhaitons recueillir l'avis de tous " +
        "concernant la couleur du hall d'entree.",
    );

    // La date de fin est pre-remplie automatiquement (J+7)

    await stepPause(page);

    // ============================================================
    // ETAPE 7 : Soumettre le sondage
    // ============================================================
    await humanClick(page, "create-poll-submit-btn");
    await waitForSpinner(page);

    // Verifier le message de succes
    await expect(page.getByTestId("create-poll-success")).toBeVisible({
      timeout: 15000,
    });
    await stepPause(page);

    // ============================================================
    // ETAPE 8 : Attendre la redirection vers le detail du sondage
    // ============================================================
    await page.waitForURL(/\/polls\/detail/, { timeout: 15000 });
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
    // ETAPE 9 : Publier le sondage (Draft -> Active)
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
