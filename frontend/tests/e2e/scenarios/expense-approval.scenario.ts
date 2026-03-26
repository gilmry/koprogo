/**
 * SCENARIO: Workflow d'approbation d'une facture
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
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

// Scenarios are human-paced — allow 2 minutes per test
test.describe("Scenario: Workflow d'approbation d'une facture", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingId: string;
  let expenseId: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-syndic-${ts}@koprogo.test`;
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
        name: `Scenario Org ${ts}`,
        slug: `scenario-${ts}`,
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
        first_name: "Marie",
        last_name: "Dupont",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Login as syndic to get token
    const syndicLoginResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: syndicPassword },
    });
    const syndic = await syndicLoginResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 5. Create building
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Residence Bellevue ${ts}`,
        address: "42 Avenue Louise",
        city: "Bruxelles",
        postal_code: "1050",
        country: "Belgium",
        total_units: 12,
        construction_year: 1985,
        organization_id: org.id,
      },
      headers: syndicHeaders,
    });
    const building = await buildingResp.json();
    buildingId = building.id;

    // 6. Create expense (Draft status, amount 1250.00, Maintenance)
    const today = new Date().toISOString().split("T")[0];
    const expenseResp = await request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        organization_id: org.id,
        category: "Maintenance",
        description: "Reparation toiture - infiltrations eau",
        amount: 1250.0,
        expense_date: today,
        supplier: "Toitures Belges SPRL",
        invoice_number: `INV-${ts}`,
      },
      headers: syndicHeaders,
    });
    const expense = await expenseResp.json();
    expenseId = expense.id;
  });

  test("Un syndic soumet et approuve une facture via l'interface", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers le Workflow Factures via le menu
    // ============================================================
    await humanClick(page, "nav-link-workflow-factures");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Workflow Factures est chargee
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Trouver la facture Draft dans la liste
    // ============================================================
    // Attendre que les factures soient chargees
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Chercher la carte de la facture creee
    const invoiceCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(invoiceCard).toBeVisible({ timeout: 15000 });

    // Faire defiler vers la carte si necessaire
    await invoiceCard.scrollIntoViewIfNeeded();
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Soumettre pour approbation (Draft -> PendingApproval)
    // ============================================================
    // Trouver le bouton "Soumettre pour approbation" dans la carte
    const submitButton = invoiceCard.getByTestId("submit-approval-button");
    await expect(submitButton).toBeVisible({ timeout: 5000 });

    // Le clic declenche un confirm() du navigateur — on l'accepte automatiquement
    page.on("dialog", (dialog) => dialog.accept());

    await submitButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await submitButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    // Attendre le rechargement de la liste
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que le statut est passe a PendingApproval
    // La carte doit maintenant afficher le badge "En attente" et le bouton "Approuver"
    const updatedCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(updatedCard).toBeVisible({ timeout: 15000 });

    // Le bouton Approuver doit apparaitre maintenant
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

    // Le modal d'approbation doit apparaitre
    const modal = page.locator(".modal").first();
    await expect(modal).toBeVisible({ timeout: 5000 });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Verifier que le resume de la facture est affiche dans le modal
    await expect(modal.locator("text=Reparation toiture")).toBeVisible();

    // Cliquer sur le bouton "Approuver" dans le modal
    const confirmApproveButton = modal.locator("button.btn-success").last();
    await confirmApproveButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await confirmApproveButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    // Attendre le rechargement de la liste
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 6 : Verifier que la facture est Approved
    // ============================================================
    // Chercher la carte mise a jour
    const approvedCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(approvedCard).toBeVisible({ timeout: 15000 });

    // Le bouton "Marquer comme paye" doit maintenant apparaitre (approve -> can mark paid)
    const markPaidButton = approvedCard.getByTestId("mark-paid-button");
    await expect(markPaidButton).toBeVisible({ timeout: 10000 });

    // Le bouton submit-approval et approve ne doivent plus etre presents
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
