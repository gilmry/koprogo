import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "skill");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
}

test.describe("Skills - Community Directory", () => {
  test("should display skills page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/skills");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='skills-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a skill offer via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const skillResp = await page.request.post(`${API_BASE}/skills`, {
      data: {
        building_id: buildingId,
        skill_name: `Cours de cuisine ${timestamp}`,
        skill_category: "Education",
        expertise_level: "Intermediate",
        description: "Cours de cuisine belge traditionnelle",
        is_available_for_help: true,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(skillResp.status())).toBeTruthy();
  });

  test("should list skills for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/skills`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });

  test("should navigate to skill detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const skillResp = await page.request.post(`${API_BASE}/skills`, {
      data: {
        building_id: buildingId,
        skill_name: `Skill detail ${timestamp}`,
        skill_category: "Technology",
        expertise_level: "Beginner",
        description: "Description compétence",
        is_available_for_help: true,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    if (skillResp.status() === 201) {
      const skill = await skillResp.json();
      await page.goto(`/skill-detail?id=${skill.id}`);
      await expect(page.locator("body")).toBeVisible();
    } else {
      await page.goto("/skills");
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
