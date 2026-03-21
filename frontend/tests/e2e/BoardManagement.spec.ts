import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `board-mgmt-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Board Mgmt Test Org ${timestamp}`,
      slug: `board-mgmt-test-${timestamp}`,
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
      first_name: "Board",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Board Building ${timestamp}`,
      address: `${timestamp} Rue Conseil`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 25,
      construction_year: 2003,
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

test.describe("Board Management - Conseil de Copropriété", () => {
  test("should display board dashboard page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/board-dashboard");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='board-dashboard']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display syndic board members page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/syndic/board-members");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should list active board members for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/board-members/active`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const members = await listResp.json();
    expect(Array.isArray(members)).toBeTruthy();
  });

  test("should list board decisions for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/board-decisions`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const decisions = await listResp.json();
    expect(Array.isArray(decisions)).toBeTruthy();
  });

  test("should get board statistics", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const statsResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/board-members/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(statsResp.ok()).toBeTruthy();
  });
});
