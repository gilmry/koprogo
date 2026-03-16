import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `recovery-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Recovery Test Org ${timestamp}`,
      slug: `recovery-test-${timestamp}`,
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
      first_name: "Recovery",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Recovery Building ${timestamp}`,
      address: `${timestamp} Rue Recouvrement`,
      city: "Charleroi",
      postal_code: "6000",
      country: "Belgium",
      total_units: 8,
      construction_year: 1998,
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

test.describe("Payment Recovery - Reminder Workflow", () => {
  test("should display payment reminders page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/payment-reminders");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='reminders-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should list payment reminder stats via API", async ({ page }) => {
    const { token } = await setupSyndicWithBuilding(page);

    const statsResp = await page.request.get(
      `${API_BASE}/payment-reminders/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(statsResp.ok()).toBeTruthy();
  });

  test("should create a payment reminder via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);
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
          expense_id: expense.id,
          owner_id: owner.id,
          reminder_level: "Gentle",
          due_date: new Date().toISOString(),
          amount_due: 250.0,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(reminderResp.status())).toBeTruthy();
  });
});
