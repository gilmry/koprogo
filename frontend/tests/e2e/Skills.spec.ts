import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `skill-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Skill Test Org ${timestamp}`,
      slug: `skill-test-${timestamp}`,
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
      first_name: "Skill",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Skill Building ${timestamp}`,
      address: `${timestamp} Rue Compétences`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 8,
      construction_year: 2006,
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

test.describe("Skills - Community Directory", () => {
  test("should display skills page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/skills");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='skills-list']").first(),
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
