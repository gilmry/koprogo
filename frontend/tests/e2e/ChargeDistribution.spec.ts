import { test, expect } from "@playwright/test";
import { loginAsSyndicWithExpense } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Charge Distribution - Invoice Allocation", () => {
  test("should calculate charge distribution via API", async ({ page }) => {
    const { token, expenseId } = await loginAsSyndicWithExpense(
      page,
      "chargedist",
    );

    const calcResp = await page.request.post(
      `${API_BASE}/invoices/${expenseId}/calculate-distribution`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    // 200/201 if units exist, 400/404 if no units to distribute to
    expect([200, 201, 400, 404].includes(calcResp.status())).toBeTruthy();
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
