import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithUnit(page: Page): Promise<{
  token: string;
  buildingId: string;
  unitId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `unitowner-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `UnitOwner Test Org ${timestamp}`,
      slug: `unitowner-test-${timestamp}`,
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
      first_name: "UnitOwner",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `UnitOwner Building ${timestamp}`,
      address: `${timestamp} Rue Copropriété`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 5,
      construction_year: 2010,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  const unitResp = await page.request.post(`${API_BASE}/units`, {
    data: {
      organization_id: org.id,
      building_id: building.id,
      unit_number: "A1",
      unit_type: "Apartment",
      floor: 1,
      surface_area: 75.0,
      quota: 100.0,
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  const unit = await unitResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, buildingId: building.id, unitId: unit.id, orgId: org.id };
}

test.describe("Unit Owners - Multi-Owner Support", () => {
  test("should display units page", async ({ page }) => {
    await setupSyndicWithUnit(page);
    await page.goto("/units");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='units-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an owner and assign to unit", async ({ page }) => {
    const { token, unitId, orgId } = await setupSyndicWithUnit(page);
    const timestamp = Date.now();

    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: orgId,
        first_name: "Marie",
        last_name: `Copropriétaire${timestamp}`,
        email: `owner-unit-${timestamp}@test.com`,
        address: "1 Rue Test",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const owner = await ownerResp.json();

    const assignResp = await page.request.post(
      `${API_BASE}/units/${unitId}/owners`,
      {
        data: {
          owner_id: owner.id,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(assignResp.status())).toBeTruthy();
  });

  test("should list owners for a unit", async ({ page }) => {
    const { token, unitId } = await setupSyndicWithUnit(page);

    const listResp = await page.request.get(
      `${API_BASE}/units/${unitId}/owners`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const owners = await listResp.json();
    expect(Array.isArray(owners)).toBeTruthy();
  });

  test("should get total ownership percentage for unit", async ({ page }) => {
    const { token, unitId } = await setupSyndicWithUnit(page);

    const pctResp = await page.request.get(
      `${API_BASE}/units/${unitId}/owners/total-percentage`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(pctResp.ok()).toBeTruthy();
  });
});
