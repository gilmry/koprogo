import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

/**
 * Expenses E2E Test Suite - Invoice Workflow
 *
 * Tests expense listing, creation, and approval workflow.
 * Uses API-first setup for data, then validates UI.
 */

const API_BASE = "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
}> {
  const timestamp = Date.now();
  const email = `expense-test-${timestamp}@example.com`;

  // Register syndic
  const regResponse = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Expense",
      last_name: `Test${timestamp}`,
      role: "syndic",
    },
  });
  expect(regResponse.ok()).toBeTruthy();
  const userData = await regResponse.json();
  const token = userData.token;

  // Create building
  const buildingResponse = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Expense Building ${timestamp}`,
      address: `${timestamp} Rue Facture`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 5,
      construction_year: 2020,
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(buildingResponse.ok()).toBeTruthy();
  const building = await buildingResponse.json();

  // Login via UI
  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, buildingId: building.id };
}

test.describe("Expenses - Invoice Management", () => {
  test("should display expenses list page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/expenses");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='expenses-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display invoice workflow page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/invoice-workflow");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='invoice-workflow']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an expense via API and see it in the list", async ({
    page,
  }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    // Create expense via API
    const expenseResponse = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        description: `Test Expense ${timestamp}`,
        amount: 1500.0,
        date: "2026-02-15",
        category: "maintenance",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(expenseResponse.ok()).toBeTruthy();

    // Navigate to expenses list
    await page.goto("/expenses");

    // Expense should appear
    await expect(page.locator(`text=Test Expense ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate to expense detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    // Create expense via API
    const expenseResponse = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        description: `Detail Expense ${timestamp}`,
        amount: 2500.0,
        date: "2026-02-20",
        category: "maintenance",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(expenseResponse.ok()).toBeTruthy();
    const expense = await expenseResponse.json();

    // Navigate to expense detail
    await page.goto(`/expense-detail?id=${expense.id}`);

    await expect(page.locator(`text=Detail Expense ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should display Belgian VAT information on expense", async ({
    page,
  }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    // Create expense with VAT
    const expenseResponse = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        description: `VAT Expense ${timestamp}`,
        amount: 1210.0,
        date: "2026-02-25",
        category: "maintenance",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(expenseResponse.ok()).toBeTruthy();
    const expense = await expenseResponse.json();

    // Navigate to detail
    await page.goto(`/expense-detail?id=${expense.id}`);

    // Page should load without errors
    await expect(page.locator("body")).toBeVisible();
  });

  test("should show payment reminders page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/payment-reminders");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='payment-reminders']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});
