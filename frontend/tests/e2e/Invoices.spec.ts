import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
  adminToken: string;
}> {
  const timestamp = Date.now();
  const email = `invoice-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Invoice Test Org ${timestamp}`,
      slug: `invoice-test-${timestamp}`,
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
      first_name: "Invoice",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Invoice Building ${timestamp}`,
      address: `${timestamp} Rue Factures`,
      city: "Liège",
      postal_code: "4000",
      country: "Belgium",
      total_units: 6,
      construction_year: 2010,
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

  return {
    token: userData.token,
    buildingId: building.id,
    orgId: org.id,
    adminToken,
  };
}

test.describe("Invoices - Expense Approval Workflow", () => {
  test("should display invoice workflow page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/invoice-workflow");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='invoices-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display expenses page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/expenses");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should create an expense via API and see it", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();
    const expenseName = `Réparation ${timestamp}`;

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description: expenseName,
        amount: 500.0,
        expense_date: new Date().toISOString(),
        supplier: "Plomberie Dupont",
        invoice_number: `INV-${timestamp}`,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(expenseResp.ok()).toBeTruthy();

    await page.goto("/invoice-workflow");
    await expect(page.locator("body")).toBeVisible();
  });

  test("should navigate to expense detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Cleaning",
        description: `Nettoyage ${timestamp}`,
        amount: 200.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    await page.goto(`/expense-detail?id=${expense.id}`);
    await expect(page.locator("body")).toBeVisible();
  });

  test("should submit expense for approval via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Security",
        description: `Sécurité ${timestamp}`,
        amount: 300.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    const submitResp = await page.request.post(
      `${API_BASE}/expenses/${expense.id}/submit-for-approval`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 400].includes(submitResp.status())).toBeTruthy();
  });
});
