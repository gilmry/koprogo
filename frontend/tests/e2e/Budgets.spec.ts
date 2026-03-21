import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "budget");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
}

test.describe("Budgets - Annual Budget Management", () => {
  test("should display budgets page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/budgets");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='budgets-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a budget via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);

    const budgetResp = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        fiscal_year: 2026,
        ordinary_budget: 50000.0,
        extraordinary_budget: 0.0,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(budgetResp.status())).toBeTruthy();
    const budget = await budgetResp.json();
    expect(budget.fiscal_year).toBe(2026);
  });

  test("should list budgets for a building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/budgets`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const budgets = await listResp.json();
    expect(Array.isArray(budgets)).toBeTruthy();
  });

  test("should navigate to budget detail page", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);

    const budgetResp = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        fiscal_year: 2027,
        ordinary_budget: 60000.0,
        extraordinary_budget: 0.0,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const budget = await budgetResp.json();

    await page.goto(`/budget-detail?id=${budget.id}`);
    await expect(page.locator("body")).toBeVisible();
  });
});
