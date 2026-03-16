import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndic(page: Page): Promise<{
  token: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `gamification-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Gamification Test Org ${timestamp}`,
      slug: `gamif-test-${timestamp}`,
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
      first_name: "Gamification",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token: userData.token, orgId: org.id };
}

test.describe("Gamification - Achievements & Challenges", () => {
  test("should display gamification page", async ({ page }) => {
    await setupSyndic(page);
    await page.goto("/gamification");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='gamification']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an achievement via API", async ({ page }) => {
    const { token, orgId } = await setupSyndic(page);
    const timestamp = Date.now();

    const achResp = await page.request.post(`${API_BASE}/achievements`, {
      data: {
        organization_id: orgId,
        name: `Premier pas ${timestamp}`,
        description: "Première action dans le système",
        category: "Community",
        tier: "Bronze",
        points_value: 10,
        requirements: { action: "login", count: 1 },
        is_secret: false,
        is_repeatable: false,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(achResp.status())).toBeTruthy();
    const achievement = await achResp.json();
    expect(achievement.organization_id).toBe(orgId);
  });

  test("should list achievements for organization", async ({ page }) => {
    const { token, orgId } = await setupSyndic(page);

    const listResp = await page.request.get(
      `${API_BASE}/organizations/${orgId}/achievements`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const achievements = await listResp.json();
    expect(Array.isArray(achievements)).toBeTruthy();
  });

  test("should access admin gamification page", async ({ page }) => {
    await setupSyndic(page);
    await page.goto("/admin/gamification");

    await expect(page.locator("body")).toBeVisible();
  });
});
