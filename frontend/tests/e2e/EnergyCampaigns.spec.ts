import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `energy-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Energy Test Org ${timestamp}`,
      slug: `energy-test-${timestamp}`,
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
      first_name: "Energy",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Energy Building ${timestamp}`,
      address: `${timestamp} Rue Energie`,
      city: "Liège",
      postal_code: "4000",
      country: "Belgium",
      total_units: 20,
      construction_year: 1995,
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

test.describe("Energy Campaigns - Group Buying", () => {
  test("should display energy campaigns page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/energy-campaigns");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='energy-campaigns-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an energy campaign via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();
    const deadline = new Date();
    deadline.setDate(deadline.getDate() + 30);

    const campaignResp = await page.request.post(
      `${API_BASE}/energy-campaigns`,
      {
        data: {
          organization_id: orgId,
          building_id: buildingId,
          campaign_name: `Achat Groupé Energie ${timestamp}`,
          description: "Campagne achat groupé gaz et électricité 2026",
          energy_types: ["Electricity", "Gas"],
          deadline_participation: deadline.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(campaignResp.status())).toBeTruthy();
    const campaign = await campaignResp.json();
    expect(campaign.organization_id).toBe(orgId);
  });

  test("should list energy campaigns", async ({ page }) => {
    const { token } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(`${API_BASE}/energy-campaigns`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.ok()).toBeTruthy();
    const campaigns = await listResp.json();
    expect(Array.isArray(campaigns)).toBeTruthy();
  });

  test("should navigate to new campaign page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/energy-campaigns/new");

    await expect(page.locator("body")).toBeVisible();
  });
});
