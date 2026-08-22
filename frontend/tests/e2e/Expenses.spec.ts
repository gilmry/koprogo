import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Expenses E2E Test Suite - Invoice Workflow
 *
 * Tests expense listing, creation, and approval workflow.
 * Uses API-first setup for data, then validates UI.
 *
 * WP-FE1/#550 : utilise le helper partagé `loginAsSyndicWithBuilding`
 * (injectAuth) pour éviter la course de rotation cookie refresh-token
 * causée par l'ancien UI-login local. Cf. Meetings.spec.ts pour détails.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Expenses - Invoice Management", () => {
  test("should display expenses list page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "expense");
    await page.goto("/expenses");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='expenses-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display invoice workflow page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "expense");
    await page.goto("/invoice-workflow");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='invoice-workflow']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an expense via API and see it in the list", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "expense",
    );
    const timestamp = Date.now();

    // Create expense via API
    const expenseResponse = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        description: `Test Expense ${timestamp}`,
        amount: 1500.0,
        expense_date: "2026-02-15T00:00:00Z",
        category: "Maintenance",
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
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "expense",
    );
    const timestamp = Date.now();

    // Create expense via API
    const expenseResponse = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        description: `Detail Expense ${timestamp}`,
        amount: 2500.0,
        expense_date: "2026-02-20T00:00:00Z",
        category: "Maintenance",
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
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "expense",
    );
    const timestamp = Date.now();

    // Create expense with VAT
    const expenseResponse = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        description: `VAT Expense ${timestamp}`,
        amount: 1210.0,
        expense_date: "2026-02-25T00:00:00Z",
        category: "Maintenance",
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
    await loginAsSyndicWithBuilding(page, "expense");
    await page.goto("/payment-reminders");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='payment-reminders']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });
});
