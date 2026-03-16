import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `notice-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Notice Test Org ${timestamp}`,
      slug: `notice-test-${timestamp}`,
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
      first_name: "Notice",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Notice Building ${timestamp}`,
      address: `${timestamp} Rue Annonces`,
      city: "Brussels",
      postal_code: "1000",
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

test.describe("Notices - Community Board", () => {
  test("should display notices page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/notices");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='notices-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a notice via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const noticeResp = await page.request.post(`${API_BASE}/notices`, {
      data: {
        building_id: buildingId,
        title: `Travaux ascenseur ${timestamp}`,
        content: "L'ascenseur sera en maintenance le 20 mars de 8h à 12h.",
        notice_type: "Maintenance",
        priority: "Normal",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(noticeResp.status())).toBeTruthy();
    const notice = await noticeResp.json();
    expect(notice.building_id).toBe(buildingId);
  });

  test("should navigate to notice detail", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const noticeResp = await page.request.post(`${API_BASE}/notices`, {
      data: {
        building_id: buildingId,
        title: `Notice detail ${timestamp}`,
        content: "Contenu de l'annonce.",
        notice_type: "General",
        priority: "Low",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const notice = await noticeResp.json();

    await page.goto(`/notice-detail?id=${notice.id}`);
    await expect(page.locator("body")).toBeVisible();
  });

  test("should list building notices via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/notices`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });
});
