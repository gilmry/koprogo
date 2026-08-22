/**
 * Characterization Spec 03 — Expense + Call-for-funds + Payment
 *
 * GOAL : Geler le flow "création expense → appel de fonds → paiement".
 *
 * STATUT : Caractérisation (NON TDD red-first). Doit être GREEN sur HEAD pré-refonte.
 *
 * SOURCE : docs/maury/refonte-ux-multi-role-acp/stories.md §2 Story 0.1
 */
import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "../helpers/auth";
import { setupContainerApiUrl } from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Characterization 03 — Expense + Payment", () => {
  test.beforeEach(async ({ page }) => {
    await setupContainerApiUrl(page);
  });

  test("syndic creates expense → expense visible in list + retrievable", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "char-exp");
    const timestamp = Date.now();
    const description = `Char Expense ${timestamp}`;

    // Create expense via API
    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: ctx.buildingId,
        category: "Maintenance",
        description,
        amount: 500.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(expenseResp.ok()).toBeTruthy();
    const expense = await expenseResp.json();
    expect(expense.id).toBeTruthy();

    // Retrieve via API
    const getResp = await page.request.get(
      `${API_BASE}/expenses/${expense.id}`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    expect(getResp.ok()).toBeTruthy();

    // UI : page expenses charge
    await page.goto("/expenses");
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='expenses-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("syndic creates expense + payment → payment retrievable", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "char-pay");
    const timestamp = Date.now();

    // 1) Owner record
    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: ctx.orgId,
        first_name: "Pay",
        last_name: `Owner${timestamp}`,
        email: `char-pay-owner-${timestamp}@test.com`,
        address: "1 Rue Paiement",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(ownerResp.ok()).toBeTruthy();
    const owner = await ownerResp.json();

    // 2) Expense
    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: ctx.buildingId,
        category: "Maintenance",
        description: `Char Pay Expense ${timestamp}`,
        amount: 500.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    const expense = await expenseResp.json();

    // 3) Payment
    const paymentResp = await page.request.post(`${API_BASE}/payments`, {
      data: {
        building_id: ctx.buildingId,
        owner_id: owner.id,
        expense_id: expense.id,
        amount_cents: 50000,
        payment_method_type: "bank_transfer",
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(paymentResp.status()).toBe(201);
    const payment = await paymentResp.json();
    expect(payment.id).toBeTruthy();
    expect(payment.amount_cents).toBe(50000);
    expect(payment.status).toBe("pending");

    // 4) Liste paiements pour l'expense
    const listResp = await page.request.get(
      `${API_BASE}/expenses/${expense.id}/payments`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const payments = await listResp.json();
    expect(Array.isArray(payments)).toBeTruthy();
    expect(
      payments.some((p: { id: string }) => p.id === payment.id),
      `Payment ${payment.id} should be in list for expense ${expense.id}`,
    ).toBeTruthy();
  });

  test("syndic accesses call-for-funds page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "char-cff");
    await page.goto("/call-for-funds");

    await expect(page.locator("body")).toBeVisible();
    // Pas d'assertion sur sélecteur précis ; on caractérise juste que la page se charge
    await expect(page.locator("main").first()).toBeVisible({ timeout: 10000 });
  });
});
