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
  const email = `etat-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `EtatDate Test Org ${timestamp}`,
      slug: `etat-test-${timestamp}`,
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
      first_name: "EtatDate",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `EtatDate Building ${timestamp}`,
      address: `${timestamp} Rue Vente`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 5,
      construction_year: 2000,
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
      surface_area: 80.0,
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

test.describe("Etats Dates - Belgian Property Sales Document", () => {
  test("should display etats-dates page", async ({ page }) => {
    await setupSyndicWithUnit(page);
    await page.goto("/etats-dates");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='etats-dates-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an etat date via API", async ({ page }) => {
    const { token, buildingId, unitId, orgId } =
      await setupSyndicWithUnit(page);
    const timestamp = Date.now();

    const etatResp = await page.request.post(`${API_BASE}/etats-dates`, {
      data: {
        unit_id: unitId,
        building_id: buildingId,
        organization_id: orgId,
        language: "FR",
        notary_name: `Maître Dupont ${timestamp}`,
        notary_email: `notaire-${timestamp}@example.com`,
        reference_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(etatResp.status())).toBeTruthy();
    const etat = await etatResp.json();
    expect(etat.unit_id).toBe(unitId);
  });

  test("should list etats-dates for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithUnit(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/etats-dates`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const etats = await listResp.json();
    expect(Array.isArray(etats)).toBeTruthy();
  });

  test("should require auth for etats-dates", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/etats-dates`);
    expect(resp.status()).toBe(401);
  });
});
