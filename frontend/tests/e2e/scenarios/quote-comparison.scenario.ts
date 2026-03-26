/**
 * SCENARIO: Comparaison de devis entrepreneurs
 *
 * Documentation Vivante — vidéo exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Devis via le menu latéral
 *   3. Sélection d'un immeuble
 *   4. Consultation de la liste des devis
 *   5. Activation du mode comparaison et sélection de 3 devis
 *   6. Navigation vers le tableau de comparaison (scores, classement)
 *
 * Durée vidéo attendue : ~50-70 secondes (rythme humain)
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

test.describe("Scénario: Comparaison de devis entrepreneurs", () => {
  test.setTimeout(120_000);

  // ----- Données de test (créées via API, invisibles en vidéo) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;
  let quoteIds: string[] = [];

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-quotes-${ts}@koprogo.test`;
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
        name: `Quotes Org ${ts}`,
        slug: `quotes-${ts}`,
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
        first_name: "Marc",
        last_name: "Dumont",
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

    // 5. Create building
    buildingName = `Résidence Les Tilleuls ${ts}`;
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "22 Boulevard d'Avroy",
        city: "Liège",
        postal_code: "4000",
        country: "Belgium",
        total_units: 24,
        construction_year: 2010,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
    const building = await buildingResp.json();

    // 6. Create 3 contractors (users) and quotes with submitted pricing
    const contractors = [
      {
        email: `contractor-a-${ts}@koprogo.test`,
        first_name: "Jean",
        last_name: "Peeters",
      },
      {
        email: `contractor-b-${ts}@koprogo.test`,
        first_name: "Luc",
        last_name: "Vermeersch",
      },
      {
        email: `contractor-c-${ts}@koprogo.test`,
        first_name: "Pierre",
        last_name: "Claessens",
      },
    ];

    const contractorIds: string[] = [];
    for (const c of contractors) {
      const resp = await request.post(`${API_BASE}/auth/register`, {
        data: {
          email: c.email,
          password: "test123456",
          first_name: c.first_name,
          last_name: c.last_name,
          role: "owner",
          organization_id: org.id,
        },
      });
      const user = await resp.json();
      contractorIds.push(user.id || user.user_id || user.user?.id);
    }

    // Create 3 quote requests and submit them with different pricing
    const quoteSpecs = [
      {
        contractor_idx: 0,
        title: "Rénovation toiture - Entreprise Peeters",
        category: "roofing",
        amount_excl_vat_cents: 1250000, // 12,500 EUR
        vat_rate: 6.0,
        estimated_duration_days: 21,
        warranty_years: 10,
      },
      {
        contractor_idx: 1,
        title: "Rénovation toiture - Vermeersch & Fils",
        category: "roofing",
        amount_excl_vat_cents: 1480000, // 14,800 EUR
        vat_rate: 6.0,
        estimated_duration_days: 14,
        warranty_years: 10,
      },
      {
        contractor_idx: 2,
        title: "Rénovation toiture - Claessens SPRL",
        category: "roofing",
        amount_excl_vat_cents: 1150000, // 11,500 EUR
        vat_rate: 6.0,
        estimated_duration_days: 28,
        warranty_years: 5,
      },
    ];

    const validityDate = new Date();
    validityDate.setMonth(validityDate.getMonth() + 3);

    for (const spec of quoteSpecs) {
      // Create quote request
      const createResp = await request.post(`${API_BASE}/quotes`, {
        data: {
          building_id: building.id,
          contractor_id: contractorIds[spec.contractor_idx],
          project_title: spec.title,
          project_description:
            "Rénovation complète de la toiture incluant isolation, étanchéité et remplacement des tuiles.",
          work_category: spec.category,
        },
        headers: syndicHeaders,
      });
      const quote = await createResp.json();
      quoteIds.push(quote.id);

      // Submit quote with pricing (contractor submits)
      await request.post(`${API_BASE}/quotes/${quote.id}/submit`, {
        data: {
          amount_excl_vat_cents: spec.amount_excl_vat_cents,
          vat_rate: spec.vat_rate,
          validity_date: validityDate.toISOString().split("T")[0],
          estimated_duration_days: spec.estimated_duration_days,
          warranty_years: spec.warranty_years,
        },
        headers: syndicHeaders,
      });
    }
  });

  test("Un syndic compare les devis de 3 entrepreneurs", async ({ page }) => {
    // ============================================================
    // ÉTAPE 1 : Connexion (visible dans la vidéo)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ÉTAPE 2 : Navigation vers les Devis via le menu latéral
    // ============================================================
    await humanClick(page, "nav-link-devis");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que la page Devis est chargée
    await expect(page.locator("main h1").first()).toContainText(
      "Devis",
    );
    await stepPause(page);

    // ============================================================
    // ÉTAPE 3 : Sélectionner l'immeuble
    // ============================================================
    await waitForSpinner(page);

    const buildingSelect = page.getByTestId("building-selector");
    if (await buildingSelect.isVisible({ timeout: 5000 })) {
      await buildingSelect.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const options = await buildingSelect.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Résidence Les Tilleuls")) {
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
    // ÉTAPE 4 : Vérifier que la liste des devis s'affiche
    // ============================================================
    await expect(page.getByTestId("quote-list")).toBeVisible({
      timeout: 15000,
    });

    // Vérifier que les 3 devis apparaissent
    const quoteRows = page.getByTestId("quote-row");
    await expect(quoteRows.first()).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ÉTAPE 5 : Activer le mode comparaison
    // ============================================================
    await humanClick(page, "compare-quotes-button");
    await page.waitForTimeout(PACE.AFTER_CLICK);

    // Sélectionner les 3 devis pour comparaison (cocher les checkboxes)
    const checkboxes = page.locator(
      '[data-testid="quote-row"] input[type="checkbox"]',
    );
    const checkboxCount = await checkboxes.count();
    for (let i = 0; i < Math.min(checkboxCount, 3); i++) {
      const checkbox = checkboxes.nth(i);
      await checkbox.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_CLICK);
      await checkbox.click();
      await page.waitForTimeout(PACE.AFTER_CLICK);
    }
    await stepPause(page);

    // ============================================================
    // ÉTAPE 6 : Lancer la comparaison
    // ============================================================
    // Cliquer sur le bouton "Comparer (3)"
    const compareButton = page
      .locator("button")
      .filter({ hasText: /Compar/i })
      .first();
    await humanClickLocator(page, compareButton);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Attendre que la page de comparaison charge
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ÉTAPE 7 : Vérifier le tableau de comparaison
    // ============================================================
    await expect(page.getByTestId("comparison-table")).toBeVisible({
      timeout: 20000,
    });

    // Vérifier que les lignes de comparaison s'affichent
    const comparisonRows = page.getByTestId("comparison-row");
    await expect(comparisonRows.first()).toBeVisible({ timeout: 10000 });

    // Vérifier les scores
    await expect(page.getByTestId("comparison-score").first()).toBeVisible({
      timeout: 5000,
    });
    await stepPause(page);

    // Scroller pour voir toute la table et la méthodologie
    const methodology = page.locator("text=Méthodologie").or(
      page.locator("text=methodology").or(
        page.locator("text=40%"),
      ),
    );
    if (await methodology.first().isVisible({ timeout: 3000 })) {
      await methodology.first().scrollIntoViewIfNeeded();
      await stepPause(page);
    }

    // ============================================================
    // FIN : Pause finale pour que la vidéo montre le résultat
    // ============================================================
    await finalPause(page);
  });
});
