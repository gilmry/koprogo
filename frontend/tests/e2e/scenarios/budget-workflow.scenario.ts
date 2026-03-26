/**
 * SCENARIO: Creation et soumission d'un budget par le syndic
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Budgets via le menu lateral
 *   3. Ouverture du formulaire de creation de budget
 *   4. Selection de l'immeuble
 *   5. Saisie de l'annee fiscale et des montants
 *   6. Soumission du budget
 *   7. Verification du budget dans la liste
 *   8. Navigation vers le detail et soumission pour approbation
 *
 * Duree video attendue : ~50-70 secondes (rythme humain)
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

test.describe("Scenario: Le syndic cree et soumet un budget annuel", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-syndic-budget-${ts}@koprogo.test`;
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
        name: `Copro Budget Demo ${ts}`,
        slug: `copro-budget-${ts}`,
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
        first_name: "Catherine",
        last_name: "Lecomte",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Create building
    buildingName = `Residence Europalia ${ts}`;
    await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "25 Boulevard du Souverain",
        city: "Bruxelles",
        postal_code: "1170",
        country: "Belgium",
        total_units: 30,
        construction_year: 2010,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
  });

  test("Le syndic cree un budget annuel et le soumet pour approbation", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers la page Budgets
    // ============================================================
    await humanClick(page, "nav-link-budgets");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Budgets est chargee
    await expect(page.locator("main").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Ouvrir le formulaire de creation
    // ============================================================
    await humanClick(page, "create-budget-button");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que le formulaire est visible
    await expect(page.getByTestId("budget-building-select")).toBeVisible({
      timeout: 10000,
    });

    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // ============================================================
    // ETAPE 4 : Selectionner l'immeuble
    // ============================================================
    const buildingSelect = page.getByTestId("budget-building-select");
    await buildingSelect.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_SELECT);

    // Selectionner l'immeuble par texte partiel
    const options = await buildingSelect.locator("option").all();
    for (const option of options) {
      const text = await option.textContent();
      if (text && text.includes("Residence Europalia")) {
        const value = await option.getAttribute("value");
        if (value) {
          await buildingSelect.selectOption(value);
          break;
        }
      }
    }
    await page.waitForTimeout(PACE.AFTER_SELECT);

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Remplir le formulaire de budget
    // ============================================================
    // Annee fiscale
    const fiscalYearInput = page.getByTestId("budget-fiscal-year");
    await fiscalYearInput.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_TYPE);
    await fiscalYearInput.clear();
    await fiscalYearInput.fill("2026");
    await page.waitForTimeout(PACE.AFTER_TYPE);

    // Budget ordinaire (charges courantes)
    const ordinaryInput = page.getByTestId("budget-ordinary-amount");
    await ordinaryInput.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_TYPE);
    await ordinaryInput.clear();
    await ordinaryInput.fill("45000");
    await page.waitForTimeout(PACE.AFTER_TYPE);

    // Budget extraordinaire (travaux)
    const extraordinaryInput = page.getByTestId("budget-extraordinary-amount");
    await extraordinaryInput.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_TYPE);
    await extraordinaryInput.clear();
    await extraordinaryInput.fill("15000");
    await page.waitForTimeout(PACE.AFTER_TYPE);

    // Verifier le resume (total + provision mensuelle)
    await expect(page.getByTestId("budget-summary")).toBeVisible({
      timeout: 5000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 6 : Soumettre le formulaire de creation
    // ============================================================
    await humanClick(page, "budget-submit-button");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 7 : Verifier que le budget apparait dans la liste
    // ============================================================
    // Attendre que la liste se recharge
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(
      page.getByTestId("budget-row").first(),
    ).toBeVisible({ timeout: 15000 });

    // Verifier que le budget 2026 apparait
    await expect(
      page.locator("text=2026"),
    ).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 8 : Naviguer vers le detail du budget
    // ============================================================
    // Cliquer sur "Details" du premier budget
    const detailLink = page
      .getByTestId("budget-row")
      .first()
      .locator('a:has-text("Details"), a:has-text("Détails")')
      .first();

    // Si pas de lien Details visible, essayer le lien direct
    if (await detailLink.isVisible({ timeout: 3000 })) {
      await humanClickLocator(page, detailLink);
    } else {
      // Fallback: cliquer sur la ligne elle-meme
      await humanClickLocator(page, page.getByTestId("budget-row").first());
    }

    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page de detail est chargee
    const budgetDetail = page.getByTestId("budget-detail");
    const budgetInfo = page.getByTestId("budget-info");

    if (await budgetDetail.isVisible({ timeout: 10000 })) {
      // On est sur la page de detail
      await expect(budgetInfo).toBeVisible({ timeout: 5000 });

      await stepPause(page);

      // ============================================================
      // ETAPE 9 : Soumettre le budget pour approbation
      // ============================================================
      const submitButton = page.getByTestId("submit-budget-button");
      if (await submitButton.isVisible({ timeout: 5000 })) {
        // Le navigateur va afficher un confirm() — on l'accepte automatiquement
        page.on("dialog", (dialog) => dialog.accept());

        await humanClickLocator(page, submitButton);
        await waitForSpinner(page);
        await page.waitForTimeout(PACE.AFTER_NAVIGATION);

        await stepPause(page);
      }
    }

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
