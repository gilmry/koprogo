import { test, expect } from "@playwright/test";
import { loginAsSyndicWithExpense } from "../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Charge Distribution - Invoice Allocation", () => {
  test("should calculate charge distribution via API", async ({ page }) => {
    const { token, expenseId, buildingId, orgId, adminToken } =
      await loginAsSyndicWithExpense(page, "chargedist");
    const timestamp = Date.now();

    // On REUTILISE un lot deja seede, on n'en cree pas un de plus.
    //
    // `loginAsSyndicWithBuilding` construit un immeuble conforme : 12 lots
    // dont la somme des quotites vaut exactement `total_tantiemes` (1000).
    // Ajouter un 13e lot a 100 quotites portait la somme a 1100 et violait
    // l'invariant de l'acte de base. Le calcul de repartition applique la
    // regle « validate before compute » (Track H Story H2) et repondait 422,
    // avec la liste des invariants manquants — comportement correct du
    // produit, que ce test contredisait en cassant lui-meme la conformite
    // qu'il venait d'etablir.
    const unitsResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/units`,
      { headers: { Authorization: `Bearer ${adminToken}` } },
    );
    expect(unitsResp.status()).toBe(200);
    const unitsBody = await unitsResp.json();
    const units = Array.isArray(unitsBody) ? unitsBody : (unitsBody.data ?? []);
    expect(units.length).toBeGreaterThan(0);
    const unit = units[0];

    // Create an owner
    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: orgId,
        first_name: "ChargeOwner",
        last_name: `Test${timestamp}`,
        email: `chargeowner-${timestamp}@test.com`,
        address: "1 Rue Test",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(ownerResp.status()).toBe(201);
    const owner = await ownerResp.json();

    // Assign owner to unit (100% ownership)
    const assignResp = await page.request.post(
      `${API_BASE}/units/${unit.id}/owners`,
      {
        data: {
          owner_id: owner.id,
          ownership_percentage: 1.0,
          start_date: new Date().toISOString(),
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(assignResp.status()).toBe(201);

    // Expense must be Approved before calculating distribution
    await page.request.put(`${API_BASE}/invoices/${expenseId}/submit`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    await page.request.put(`${API_BASE}/invoices/${expenseId}/approve`, {
      headers: { Authorization: `Bearer ${token}` },
    });

    const calcResp = await page.request.post(
      `${API_BASE}/invoices/${expenseId}/calculate-distribution`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(calcResp.status()).toBe(200);
  });

  test("should get charge distribution for invoice", async ({ page }) => {
    const { token, expenseId } = await loginAsSyndicWithExpense(
      page,
      "chargedist",
    );

    const getResp = await page.request.get(
      `${API_BASE}/invoices/${expenseId}/distribution`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(getResp.ok()).toBeTruthy();
    const distributions = await getResp.json();
    expect(Array.isArray(distributions)).toBeTruthy();
  });

  test("should require auth for charge distribution", async ({ page }) => {
    const resp = await page.request.get(
      `${API_BASE}/invoices/some-id/distribution`,
    );
    expect(resp.status()).toBe(401);
  });

  test("should display accountant page where distributions are shown", async ({
    page,
  }) => {
    await loginAsSyndicWithExpense(page, "chargedist");
    await page.goto("/accountant");

    await expect(page.locator("body")).toBeVisible();
  });
});
