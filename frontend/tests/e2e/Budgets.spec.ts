import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `budget-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Budget Test Org ${timestamp}`,
      slug: `budget-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Budget",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Budget Building ${timestamp}`,
      address: `${timestamp} Rue Budget`,
      city: "Namur",
      postal_code: "5000",
      country: "Belgium",
      total_units: 8,
      construction_year: 2012,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token: userData.token, buildingId: building.id, orgId: org.id };
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
        title: "Budget 2026",
        total_budget_amount: 50000.0,
        description: "Budget prévisionnel annuel",
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
        title: "Budget 2027",
        total_budget_amount: 60000.0,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const budget = await budgetResp.json();

    await page.goto(`/budget-detail?id=${budget.id}`);
    await expect(page.locator("body")).toBeVisible();
  });
});
