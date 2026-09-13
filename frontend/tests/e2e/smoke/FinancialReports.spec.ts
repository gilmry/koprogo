import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "../helpers/auth";

import { API_BASE } from "../helpers/adresses";

async function setupAccountant(page: Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "finreport");

  // Register an accountant user in the same organization
  const timestamp = Date.now();
  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email: `accountant-${timestamp}@test.com`,
      password: "test123456",
      first_name: "Comptable",
      last_name: `Test${timestamp}`,
      role: "accountant",
      organization_id: ctx.orgId,
    },
  });
  const accountantData = await regResp.json();
  const accountantToken = accountantData.token;

  return {
    token: ctx.token,
    accountantToken,
    buildingId: ctx.buildingId,
    orgId: ctx.orgId,
  };
}

test.describe("Financial Reports - Balance Sheet & Income Statement", () => {
  test("should display reports page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "finreport");
    await page.goto("/reports");

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("[data-testid='reports']").first()).toBeVisible({
      timeout: 10000,
    });
  });

  test("should get balance sheet via API", async ({ page }) => {
    const { accountantToken, buildingId } = await setupAccountant(page);

    const bsResp = await page.request.get(
      `${API_BASE}/reports/balance-sheet?building_id=${buildingId}`,
      { headers: { Authorization: `Bearer ${accountantToken}` } },
    );
    expect(
      bsResp.status(),
      `bsResp : ${await bsResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(200);
  });

  test("should get income statement via API", async ({ page }) => {
    const { accountantToken } = await setupAccountant(page);

    const isResp = await page.request.get(
      `${API_BASE}/reports/income-statement?period_start=2026-01-01T00:00:00Z&period_end=2026-12-31T23:59:59Z`,
      { headers: { Authorization: `Bearer ${accountantToken}` } },
    );
    expect(
      isResp.status(),
      `isResp : ${await isResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(200);
  });

  test("should require auth for financial reports", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/reports/balance-sheet`);
    expect(
      resp.status(),
      `resp : ${await resp.text().catch(() => "<corps illisible>")}`,
    ).toBe(401);
  });
});
