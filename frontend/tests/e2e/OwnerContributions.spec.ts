import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithOwner(page: Page): Promise<{
  token: string;
  buildingId: string;
  ownerId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `contribution-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Contribution Test Org ${timestamp}`,
      slug: `contrib-test-${timestamp}`,
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
      first_name: "Contribution",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Contribution Building ${timestamp}`,
      address: `${timestamp} Rue Versements`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 6,
      construction_year: 2008,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: org.id,
      first_name: "Propriétaire",
      last_name: `Test${timestamp}`,
      email: `owner-contrib-${timestamp}@test.com`,
      address: "1 Rue Versements",
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

  return { token, buildingId: building.id, ownerId: owner.id, orgId: org.id };
}

test.describe("Owner Contributions - Payment Tracking", () => {
  test("should display owner contributions page", async ({ page }) => {
    await setupSyndicWithOwner(page);
    await page.goto("/owner-contributions");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='contributions-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a contribution via API", async ({ page }) => {
    const { token, ownerId, orgId } = await setupSyndicWithOwner(page);
    const timestamp = Date.now();

    const contribResp = await page.request.post(
      `${API_BASE}/owner-contributions`,
      {
        data: {
          organization_id: orgId,
          owner_id: ownerId,
          description: `Provision T2 2026 ${timestamp}`,
          amount: 800.0,
          contribution_type: "Regular",
          due_date: new Date().toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(contribResp.status())).toBeTruthy();
  });

  test("should list contributions for owner", async ({ page }) => {
    const { token, ownerId } = await setupSyndicWithOwner(page);

    const listResp = await page.request.get(
      `${API_BASE}/owner-contributions?owner_id=${ownerId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });
});
