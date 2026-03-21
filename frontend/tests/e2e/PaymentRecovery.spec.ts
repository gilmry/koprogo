import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Payment Recovery - Reminder Workflow", () => {
  test("should display payment reminders page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "recovery");
    await page.goto("/payment-reminders");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='reminders-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should list payment reminder stats via API", async ({ page }) => {
    const { token } = await loginAsSyndicWithBuilding(page, "recovery");

    const statsResp = await page.request.get(
      `${API_BASE}/payment-reminders/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(statsResp.ok()).toBeTruthy();
  });

  test("should create a payment reminder via API", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "recovery",
    );
    const timestamp = Date.now();

    // Create expense first
    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description: `Charge impayée ${timestamp}`,
        amount: 250.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    // Create owner
    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: orgId,
        first_name: "Débiteur",
        last_name: `Test${timestamp}`,
        email: `debiteur-${timestamp}@test.com`,
        address: "1 Rue Test",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const owner = await ownerResp.json();

    const reminderResp = await page.request.post(
      `${API_BASE}/payment-reminders`,
      {
        data: {
          organization_id: orgId,
          expense_id: expense.id,
          owner_id: owner.id,
          level: "FirstReminder",
          due_date: new Date(Date.now() - 15 * 86400000).toISOString(),
          amount_owed: 250.0,
          days_overdue: 15,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(reminderResp.status())).toBeTruthy();
  });
});
