import { test, expect } from "@playwright/test";
import { loginAsSyndicWithLinkedOwner, ensureAcp } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

// OwnerDashboard.spec.ts (préexistant) ne teste que l'état vide (owner sans
// lot lié). Ce test vérifie les écrans en lecture seule du portail
// copropriétaire avec de vraies données liées (lot + dépense + document).
test.describe("Copropriétaire — écrans en lecture seule, données réelles", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("dashboard, mes lots, mes dépenses, mes documents : rendu peuplé pour un owner réellement lié", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithLinkedOwner(page, "journey-owner");

    // Réutilise un lot déjà seedé (conformant) par loginAsSyndicWithBuilding
    // plutôt que d'en créer un nouveau : créer un lot supplémentaire casse
    // la conformité (SUM(quota) != total_tantiemes) et POST /expenses est
    // gated 422 sur bâtiment non-conforme (Track H Story H2).
    await ensureAcp(page, ctx.orgId, ctx.adminToken, "journey-owner");
    const unitsResp = await page.request.get(
      `${API_BASE}/buildings/${ctx.buildingId}/units`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    const seededUnits = await unitsResp.json();
    const unit = seededUnits[0];
    const linkResp = await page.request.post(
      `${API_BASE}/units/${unit.id}/owners`,
      {
        data: {
          owner_id: ctx.ownerId,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${ctx.token}` },
      },
    );
    expect(linkResp.status()).toBe(201);

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: ctx.buildingId,
        category: "Maintenance",
        description: `Charge visible owner ${Date.now()}`,
        amount: 250.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(expenseResp.status()).toBe(201);
    const expense = await expenseResp.json();

    const docTitle = `Reglement ${Date.now()}`;
    const uploadResp = await page.request.post(`${API_BASE}/documents`, {
      multipart: {
        file: {
          name: "reglement.pdf",
          mimeType: "application/pdf",
          buffer: Buffer.from("%PDF-1.4 test owner document"),
        },
        building_id: ctx.buildingId,
        document_type: "Regulation",
        title: docTitle,
        uploaded_by: ctx.userId,
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    // L'assertion porte le CORPS de la reponse : un « Expected 201, Received
    // 500 » nu ne dit rien de la cause, et c'est exactement ce qui a fait
    // perdre du temps sur ce defaut. Le handler renvoie `{"error": ...}`, on
    // le remonte donc dans le message d'echec.
    expect(
      uploadResp.status(),
      `upload document: ${await uploadResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(201);

    // --- Dashboard : vérifie qu'on navigue bien en tant qu'owner lié
    // (dernier register/login à avoir posé son cookie dans ce contexte
    // partagé, cf. findings.md) et pas resté sur la session syndic.
    await page.goto("/owner", { waitUntil: "networkidle" });
    await expect(page.getByTestId("owner-dashboard")).toBeVisible();
    await expect(page.locator("body")).toContainText("Owner");

    // --- Mes lots : le lot créé/lié doit apparaître.
    await page.goto("/owner/units", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toContainText(unit.unit_number);

    // --- Mes dépenses : la dépense créée doit apparaître, et le bouton
    // "Créer une facture" (réservé syndic/comptable, backend
    // check_owner_readonly) ne doit pas être visible pour un owner.
    await page.goto("/owner/expenses", { waitUntil: "networkidle" });
    // Un seul bâtiment lié à cet owner : BuildingSelector auto-sélectionne
    // (pas de <select>, juste un résumé "building-selected").
    await expect(page.getByTestId("building-selected")).toBeVisible();
    await expect(
      page.locator('[data-testid="expense-card"]', {
        hasText: expense.description,
      }),
    ).toBeVisible();
    await expect(page.getByTestId("create-button")).toHaveCount(0);

    // --- Mes documents : le document uploadé (par le syndic) doit être
    // listé et téléchargeable pour l'owner.
    await page.goto("/owner/documents", { waitUntil: "networkidle" });
    const docRow = page.locator("tr", { hasText: docTitle });
    await expect(docRow).toBeVisible();
    const [downloadReq] = await Promise.all([
      page.waitForRequest(
        (r) => r.url().includes("/download") && r.method() === "GET",
      ),
      docRow.getByRole("button", { name: "Télécharger" }).click(),
    ]);
    expect(await downloadReq.headerValue("authorization")).toMatch(
      /^Bearer ey/,
    );

    // --- Mes paiements : écran en lecture seule, doit au moins se charger
    // sans erreur (pas de fixture de paiement construite ici).
    await page.goto("/owner/payments", { waitUntil: "networkidle" });
    await expect(page.locator("body")).toBeVisible();
  });
});
