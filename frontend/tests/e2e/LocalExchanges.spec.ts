import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupOwnerInBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  ownerId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `exchange-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Exchange Test Org ${timestamp}`,
      slug: `exchange-test-${timestamp}`,
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
      first_name: "Exchange",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Exchange Building ${timestamp}`,
      address: `${timestamp} Rue SEL`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 10,
      construction_year: 2010,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: org.id,
      first_name: "Jean",
      last_name: `Echange${timestamp}`,
      email: `owner-exchange-${timestamp}@test.com`,
      address: "1 Rue SEL",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  const owner = await ownerResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return {
    token,
    buildingId: building.id,
    ownerId: owner.id,
    orgId: org.id,
  };
}

test.describe("Local Exchanges - SEL Community System", () => {
  test("should display exchanges page", async ({ page }) => {
    await setupOwnerInBuilding(page);
    await page.goto("/exchanges");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='exchanges-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an exchange offer via API", async ({ page }) => {
    const { token, buildingId, ownerId } = await setupOwnerInBuilding(page);
    const timestamp = Date.now();

    const exchangeResp = await page.request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: buildingId,
        provider_id: ownerId,
        exchange_type: "Service",
        title: `Aide jardinage ${timestamp}`,
        description: "Tonte pelouse et taille haie",
        credits: 2,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(exchangeResp.status())).toBeTruthy();
    const exchange = await exchangeResp.json();
    expect(exchange.building_id).toBe(buildingId);
  });

  test("should list available exchanges for building", async ({ page }) => {
    const { token, buildingId } = await setupOwnerInBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/exchanges/available`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const exchanges = await listResp.json();
    expect(Array.isArray(exchanges)).toBeTruthy();
  });

  test("should navigate to new exchange page", async ({ page }) => {
    await setupOwnerInBuilding(page);
    await page.goto("/exchanges/new");

    await expect(page.locator("body")).toBeVisible();
  });
});
