/**
 * SCENARIO: Comparaison de devis entrepreneurs (SINGLE ROLE - syndic)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet de Francois (syndic) :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Devis via le menu lateral
 *   3. Selection d'un immeuble
 *   4. Consultation de la liste des devis (3 devis pre-crees)
 *   5. Navigation vers le tableau de comparaison (scores, classement)
 *   6. Verification de la methodologie de scoring belge
 *
 * Conformite belge: 3 devis obligatoires pour travaux > 5000 EUR
 * Duree video attendue : ~50-70 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanClick,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Comparaison de devis entrepreneurs (Francois)", () => {
  test.setTimeout(120_000);

  let seedData: any;
  let quoteIds: string[] = [];

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

    // 3. Create 3 quotes via Francois for comparison
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
      // Create 3 contractors and their quotes
      const ts = Date.now();
      const contractorIds: string[] = [];
      const contractors = [
        { email: `contractor-a-${ts}@koprogo.test`, first_name: "Jean", last_name: "Peeters" },
        { email: `contractor-b-${ts}@koprogo.test`, first_name: "Luc", last_name: "Vermeersch" },
        { email: `contractor-c-${ts}@koprogo.test`, first_name: "Pierre", last_name: "Claessens" },
      ];

      for (const c of contractors) {
        const resp = await request.post(`${API_BASE}/auth/register`, {
          data: {
            email: c.email,
            password: "test123456",
            first_name: c.first_name,
            last_name: c.last_name,
            role: "owner",
            organization_id: building.organization_id,
          },
        });
        const user = await resp.json();
        contractorIds.push(user.id || user.user_id || user.user?.id);
      }

      const validityDate = new Date();
      validityDate.setMonth(validityDate.getMonth() + 3);

      const quoteSpecs = [
        {
          contractor_idx: 0,
          title: "Renovation toiture - Entreprise Peeters",
          amount_excl_vat: 12500.0,
          vat_rate: 0.06,
          estimated_duration_days: 21,
          warranty_years: 10,
        },
        {
          contractor_idx: 1,
          title: "Renovation toiture - Vermeersch & Fils",
          amount_excl_vat: 14800.0,
          vat_rate: 0.06,
          estimated_duration_days: 14,
          warranty_years: 10,
        },
        {
          contractor_idx: 2,
          title: "Renovation toiture - Claessens SPRL",
          amount_excl_vat: 11500.0,
          vat_rate: 0.06,
          estimated_duration_days: 28,
          warranty_years: 5,
        },
      ];

      for (const spec of quoteSpecs) {
        const createResp = await request.post(`${API_BASE}/quotes`, {
          data: {
            building_id: building.id,
            contractor_id: contractorIds[spec.contractor_idx],
            project_title: spec.title,
            project_description:
              "Renovation complete de la toiture incluant isolation, etancheite et remplacement des tuiles.",
            amount_excl_vat: spec.amount_excl_vat,
            vat_rate: spec.vat_rate,
            validity_date: validityDate.toISOString(),
            estimated_duration_days: spec.estimated_duration_days,
            warranty_years: spec.warranty_years,
          },
          headers: syndicHeaders,
        });
        const quote = await createResp.json();
        quoteIds.push(quote.id);

        // Submit quote (Requested -> Received)
        await request.post(`${API_BASE}/quotes/${quote.id}/submit`, {
          headers: syndicHeaders,
        });
      }
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

  test("Francois compare les devis de 3 entrepreneurs", async ({ page }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers les Devis via le menu lateral
    // ============================================================
    await humanClick(page, "nav-link-devis");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Devis");
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Selectionner l'immeuble (Residence du Parc)
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
    // ETAPE 4 : Verifier que la liste des devis s'affiche
    // ============================================================
    await expect(page.getByTestId("quote-list")).toBeVisible({
      timeout: 15000,
    });

    const quoteRows = page.getByTestId("quote-row");
    await expect(quoteRows.first()).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Naviguer vers la page de comparaison
    // ============================================================
    const compareUrl = `/quotes/compare?ids=${quoteIds.join(",")}`;
    await page.goto(compareUrl, { waitUntil: "domcontentloaded" });
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 6 : Verifier le tableau de comparaison
    // ============================================================
    await expect(page.getByTestId("comparison-table")).toBeVisible({
      timeout: 20000,
    });

    const comparisonRows = page.getByTestId("comparison-row");
    await expect(comparisonRows.first()).toBeVisible({ timeout: 10000 });

    await expect(
      page.getByTestId("comparison-score").first(),
    ).toBeVisible({ timeout: 5000 });
    await stepPause(page);

    // Scroller pour voir la methodologie
    const methodology = page
      .locator("text=Methodologie")
      .or(page.locator("text=methodology").or(page.locator("text=40%")));
    if (await methodology.first().isVisible({ timeout: 3000 })) {
      await methodology.first().scrollIntoViewIfNeeded();
      await stepPause(page);
    }

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
