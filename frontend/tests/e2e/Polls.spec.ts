import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `poll-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Poll Test Org ${timestamp}`,
      slug: `poll-test-${timestamp}`,
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
      first_name: "Poll",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Poll Building ${timestamp}`,
      address: `${timestamp} Rue Sondage`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 12,
      construction_year: 2008,
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

test.describe("Polls - Board Decision Polling", () => {
  test("should display polls page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/polls");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='polls-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should navigate to new poll page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/polls/new");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should create a poll via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();
    const startDate = new Date();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "yes_no",
        title: `Repeindre le hall d'entrée ? ${timestamp}`,
        description: "Sondage avant AG",
        ends_at: endDate.toISOString(),
        is_anonymous: false,
        options: [],
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(pollResp.status())).toBeTruthy();
    const poll = await pollResp.json();
    expect(poll.building_id).toBe(buildingId);
  });

  test("should list active polls for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/polls/active`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const polls = await listResp.json();
    expect(Array.isArray(polls)).toBeTruthy();
  });
});
