import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `cff-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `CFF Test Org ${timestamp}`,
      slug: `cff-test-${timestamp}`,
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
      first_name: "CallFunds",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `CFF Building ${timestamp}`,
      address: `${timestamp} Rue Appel Fonds`,
      city: "Ghent",
      postal_code: "9000",
      country: "Belgium",
      total_units: 8,
      construction_year: 2005,
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

  return { token, buildingId: building.id, orgId: org.id };
}

test.describe("Call For Funds - Revenue Management", () => {
  test("should display call-for-funds page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/call-for-funds");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='call-for-funds-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a call for funds via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();
    const dueDate = new Date();
    dueDate.setDate(dueDate.getDate() + 30);

    const cffResp = await page.request.post(`${API_BASE}/call-for-funds`, {
      data: {
        organization_id: orgId,
        building_id: buildingId,
        title: `Appel fonds T1 2026 ${timestamp}`,
        total_amount: 5000.0,
        contribution_type: "Regular",
        call_date: new Date().toISOString(),
        due_date: dueDate.toISOString(),
        description: "Provision charges courantes",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(cffResp.status())).toBeTruthy();
    const cff = await cffResp.json();
    expect(cff.building_id).toBe(buildingId);
  });

  test("should list calls for funds via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/call-for-funds?building_id=${buildingId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const response = await listResp.json();
    expect(Array.isArray(response)).toBeTruthy();
  });
});
