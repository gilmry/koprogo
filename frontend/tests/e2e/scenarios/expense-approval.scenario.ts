/**
 * SCENARIO: Workflow d'approbation d'une facture (SINGLE ROLE - syndic)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet du syndic Francois :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Workflow Factures via le menu lateral
 *   3. Visualisation d'une facture en statut Draft
 *   4. Soumission pour approbation (Draft -> PendingApproval)
 *   5. Approbation de la facture (PendingApproval -> Approved)
 *   6. Verification du statut final Approved
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

test.describe("Scenario: Workflow d'approbation d'une facture", () => {
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

    // 3. Create a Draft expense for the scenario via Francois
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
      await request.post(`${API_BASE}/expenses`, {
        data: {
          building_id: building.id,
          category: "Maintenance",
          description: "Reparation toiture - infiltrations eau",
          amount: 1250.0,
          expense_date: new Date().toISOString(),
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

  test("Francois soumet et approuve une facture via l'interface", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers le Workflow Factures via le menu
    // ============================================================
    await humanClick(page, "nav-link-workflow-factures");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Trouver la facture Draft dans la liste
    // ============================================================
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const invoiceCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(invoiceCard).toBeVisible({ timeout: 15000 });

    await invoiceCard.scrollIntoViewIfNeeded();
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Soumettre pour approbation (Draft -> PendingApproval)
    // ============================================================
    const submitButton = invoiceCard.getByTestId("submit-approval-button");
    await expect(submitButton).toBeVisible({ timeout: 5000 });

    page.on("dialog", (dialog) => dialog.accept());

    await submitButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await submitButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const updatedCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(updatedCard).toBeVisible({ timeout: 15000 });

    const approveButton = updatedCard.getByTestId("approve-button");
    await expect(approveButton).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Approuver la facture (PendingApproval -> Approved)
    // ============================================================
    await approveButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await approveButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    const modal = page.locator(".modal").first();
    await expect(modal).toBeVisible({ timeout: 5000 });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    await expect(modal.locator("text=Reparation toiture")).toBeVisible();

    const confirmApproveButton = modal.locator("button.btn-success").last();
    await confirmApproveButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await confirmApproveButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 6 : Verifier que la facture est Approved
    // ============================================================
    const approvedCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(approvedCard).toBeVisible({ timeout: 15000 });

    const markPaidButton = approvedCard.getByTestId("mark-paid-button");
    await expect(markPaidButton).toBeVisible({ timeout: 10000 });

    await expect(
      approvedCard.getByTestId("submit-approval-button"),
    ).not.toBeVisible();
    await expect(approvedCard.getByTestId("approve-button")).not.toBeVisible();

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
