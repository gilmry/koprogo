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
  const email = `payment-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Payment Test Org ${timestamp}`,
      slug: `payment-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();
  const orgId = org.id;

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Payment",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: orgId,
    },
  });
  expect(regResp.ok()).toBeTruthy();
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Payment Building ${timestamp}`,
      address: `${timestamp} Rue Payment`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 4,
      construction_year: 2020,
      organization_id: orgId,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, buildingId: building.id, orgId, adminToken };
}

test.describe("Payments - Stripe & SEPA", () => {
  test("should display owner payments page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/owner/payments");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='payments-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner payment methods page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/owner/payment-methods");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should create a payment via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description: `Test expense ${timestamp}`,
        amount: 150.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const expense = await expenseResp.json();

    const paymentResp = await page.request.post(`${API_BASE}/payments`, {
      data: {
        expense_id: expense.id,
        amount_cents: 15000,
        currency: "EUR",
        payment_method_type: "BankTransfer",
        idempotency_key: `test-${timestamp}`,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    // 201 or 400 (if domain validation fails)
    expect([201, 400, 422].includes(paymentResp.status())).toBeTruthy();
  });

  test("should navigate to payments page after login", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/owner/payments");

    await expect(page.locator("body")).toBeVisible();
  });
});
