import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndic(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndic(page, "gamification");
  return { token: ctx.token, orgId: ctx.orgId };
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
