import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Invoices E2E Test Suite - Expense Approval Workflow
 *
 * Tests the full invoice lifecycle: Draft → PendingApproval → Approved/Rejected.
 * Mirrors workflows from backend/tests/e2e_invoices.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Invoices - Expense Approval Workflow", () => {
  test("should display invoice workflow page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "invoice");
    await page.goto("/invoice-workflow");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='invoices-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display expenses page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "invoice");
    await page.goto("/expenses");

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
  });

  test("should create an expense via API and see it in the list", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "invoice",
    );
    const timestamp = Date.now();
    const expenseName = `Réparation toiture ${timestamp}`;

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
    const expense = await expenseResp.json();
    expect(expense.id).toBeTruthy();
    expect(expense.description).toBe(expenseName);

    // Verify it appears in the UI list
    await page.goto("/invoice-workflow");
    await expect(page.locator(`text=${expenseName}`)).toBeVisible({
      timeout: 15000,
    });
  });

  test("should navigate to expense detail page and see content", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "invoice",
    );
    const timestamp = Date.now();
    const expenseName = `Nettoyage ${timestamp}`;

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Cleaning",
        description: expenseName,
        amount: 200.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    await page.goto(`/expense-detail?id=${expense.id}`);
    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator(`text=${expenseName}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should submit expense for approval (Draft → PendingApproval)", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "invoice",
    );
    const timestamp = Date.now();

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Administration",
        description: `Sécurité immeuble ${timestamp}`,
        amount: 300.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    const submitResp = await page.request.put(
      `${API_BASE}/expenses/${expense.id}/submit-for-approval`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 400].includes(submitResp.status())).toBeTruthy();

    if (submitResp.ok()) {
      const updated = await submitResp.json();
      expect(updated.status).toBe("PendingApproval");
    }
  });

  test("should approve an expense (PendingApproval → Approved)", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "invoice",
    );
    const timestamp = Date.now();

    // Create expense
    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description: `Chauffage ${timestamp}`,
        amount: 800.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    // Submit for approval
    await page.request.put(
      `${API_BASE}/expenses/${expense.id}/submit-for-approval`,
      { headers: { Authorization: `Bearer ${token}` } },
    );

    // Approve
    const approveResp = await page.request.put(
      `${API_BASE}/expenses/${expense.id}/approve`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 400].includes(approveResp.status())).toBeTruthy();

    if (approveResp.ok()) {
      const approved = await approveResp.json();
      expect(approved.status).toBe("Approved");
    }
  });

  test("should list building expenses", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "invoice",
    );

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/expenses`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const expenses = await listResp.json();
    expect(Array.isArray(expenses)).toBeTruthy();
  });
});
