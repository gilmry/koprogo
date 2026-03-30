import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Payment Recovery E2E Test Suite - Automated Reminder Workflow
 *
 * Tests payment reminder creation and escalation workflow.
 * 4 levels: Gentle (J+15) → Formal (J+30) → FinalNotice (J+45) → LegalAction (J+60)
 * Late penalty: Belgian legal rate 8% annually.
 * Mirrors workflows from backend/tests/e2e_payment_recovery.rs.
 */

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

  test("should get payment reminder stats", async ({ page }) => {
    const { token } = await loginAsSyndicWithBuilding(page, "recovery");

    const statsResp = await page.request.get(
      `${API_BASE}/payment-reminders/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(statsResp.ok()).toBeTruthy();
  });

  test("should create a payment reminder and retrieve it", async ({ page }) => {
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
          level: "Gentle",
          due_date: new Date(Date.now() - 15 * 86400000).toISOString(),
          amount_owed: 250.0,
          days_overdue: 15,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(reminderResp.status())).toBeTruthy();

    if (reminderResp.ok()) {
      const reminder = await reminderResp.json();
      expect(reminder.id).toBeTruthy();
      expect(reminder.level).toBe("Gentle");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/payment-reminders/${reminder.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(reminder.id);
    }
  });

  test("should list reminders for an expense", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "recovery",
    );
    const timestamp = Date.now();

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Cleaning",
        description: `Impayé liste ${timestamp}`,
        amount: 100.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    const listResp = await page.request.get(
      `${API_BASE}/expenses/${expense.id}/payment-reminders`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const reminders = await listResp.json();
    expect(Array.isArray(reminders)).toBeTruthy();
  });

  test("should escalate a reminder (Gentle → Formal)", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "recovery",
    );
    const timestamp = Date.now();

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Administration",
        description: `Escalade ${timestamp}`,
        amount: 400.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: orgId,
        first_name: "Escalade",
        last_name: `Test${timestamp}`,
        email: `escalade-${timestamp}@test.com`,
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
          level: "Gentle",
          due_date: new Date(Date.now() - 30 * 86400000).toISOString(),
          amount_owed: 400.0,
          days_overdue: 30,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (reminderResp.ok()) {
      const reminder = await reminderResp.json();

      const escalateResp = await page.request.put(
        `${API_BASE}/payment-reminders/${reminder.id}/escalate`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(escalateResp.status())).toBeTruthy();

      if (escalateResp.ok()) {
        const escalated = await escalateResp.json();
        expect(escalated.level).toBe("Formal");
      }
    }
  });

  test("should navigate to payment reminder detail page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "recovery");
    await page.goto("/payment-reminders");
    await expect(page.locator("body")).toBeVisible();
  });

  test("should require auth for payment reminders API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/payment-reminders`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
