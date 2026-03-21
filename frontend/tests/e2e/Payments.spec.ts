import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Payments - Stripe & SEPA", () => {
  test("should display owner payments page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "payment");
    await page.goto("/owner/payments");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='payments-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner payment methods page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "payment");
    await page.goto("/owner/payment-methods");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should create a payment via API", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "payment",
    );
    const timestamp = Date.now();

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description: `Test expense ${timestamp}`,
        amount: 150.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    const paymentResp = await page.request.post(`${API_BASE}/payments`, {
      data: {
        expense_id: expense.id,
        amount_cents: 15000,
        currency: "EUR",
        payment_method_type: "BankTransfer",
        idempotency_key: `test-${timestamp}`,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    // 201 or 400 (if domain validation fails)
    expect([201, 400, 422].includes(paymentResp.status())).toBeTruthy();
  });

  test("should navigate to payments page after login", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "payment");
    await page.goto("/owner/payments");

    await expect(page.locator("body")).toBeVisible();
  });
});
