import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `inspection-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Inspection Test Org ${timestamp}`,
      slug: `insp-test-${timestamp}`,
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
      first_name: "Inspection",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Inspection Building ${timestamp}`,
      address: `${timestamp} Rue Contrôle`,
      city: "Liège",
      postal_code: "4000",
      country: "Belgium",
      total_units: 10,
      construction_year: 1990,
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

test.describe("Technical Inspections - Mandatory Compliance", () => {
  test("should display inspections page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/inspections");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='inspections-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a technical inspection via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();
    const nextInspection = new Date();
    nextInspection.setFullYear(nextInspection.getFullYear() + 2);

    const inspResp = await page.request.post(
      `${API_BASE}/technical-inspections`,
      {
        data: {
          building_id: buildingId,
          organization_id: orgId,
          title: `Inspection ascenseur ${timestamp}`,
          inspection_type: "elevator",
          inspector_name: `Bureau Véritas ${timestamp}`,
          inspection_date: new Date().toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(inspResp.status())).toBeTruthy();
    const inspection = await inspResp.json();
    expect(inspection.building_id).toBe(buildingId);
  });

  test("should list upcoming inspections", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const upcomingResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/technical-inspections/upcoming`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(upcomingResp.ok()).toBeTruthy();
    const inspections = await upcomingResp.json();
    expect(Array.isArray(inspections)).toBeTruthy();
  });

  test("should list overdue inspections", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const overdueResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/technical-inspections/overdue`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(overdueResp.ok()).toBeTruthy();
  });
});
