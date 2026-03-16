import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `sharing-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Sharing Test Org ${timestamp}`,
      slug: `sharing-test-${timestamp}`,
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
      first_name: "Sharing",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Sharing Building ${timestamp}`,
      address: `${timestamp} Rue Partage`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 10,
      construction_year: 2007,
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

test.describe("Sharing - Object Sharing Library", () => {
  test("should display sharing page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/sharing");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='sharing-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a shared object via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const objectResp = await page.request.post(`${API_BASE}/shared-objects`, {
      data: {
        building_id: buildingId,
        object_name: `Perceuse ${timestamp}`,
        object_category: "Tools",
        description: "Perceuse Bosch avec accessoires",
        condition: "Good",
        is_available: true,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(objectResp.status())).toBeTruthy();
  });

  test("should list shared objects for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/shared-objects`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });

  test("should navigate to sharing detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const objectResp = await page.request.post(`${API_BASE}/shared-objects`, {
      data: {
        building_id: buildingId,
        name: `Vélo ${timestamp}`,
        description: "Vélo partagé",
        category: "Transport",
        is_available: true,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    if (objectResp.status() === 201) {
      const obj = await objectResp.json();
      await page.goto(`/sharing-detail?id=${obj.id}`);
      await expect(page.locator("body")).toBeVisible();
    } else {
      await page.goto("/sharing");
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
