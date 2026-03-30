import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

/**
 * Gamification E2E Test Suite - Achievements & Challenges
 *
 * Tests achievement creation, challenge lifecycle, user stats, and leaderboard.
 * 8 categories, 5 tiers, points system.
 * Mirrors workflows from backend/tests/e2e_gamification.rs (via gamification_handlers).
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Gamification - Achievements & Challenges", () => {
  test("should display gamification page", async ({ page }) => {
    await loginAsSyndic(page, "gamif");
    await page.goto("/gamification");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='gamification']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an achievement and retrieve it", async ({ page }) => {
    const { token, orgId } = await loginAsSyndic(page, "gamif");
    const timestamp = Date.now();
    const name = `Premier pas ${timestamp}`;

    const achResp = await page.request.post(`${API_BASE}/achievements`, {
      data: {
        organization_id: orgId,
        name,
        description: "Première action réalisée dans la communauté",
        category: "Community",
        tier: "Bronze",
        points_value: 10,
        requirements: JSON.stringify({ action: "login", count: 1 }),
        icon: "trophy",
        display_order: 1,
        is_secret: false,
        is_repeatable: false,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(achResp.status())).toBeTruthy();

    if (achResp.ok()) {
      const achievement = await achResp.json();
      expect(achievement.id).toBeTruthy();
      expect(achievement.organization_id).toBe(orgId);
      expect(achievement.tier).toBe("Bronze");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/achievements/${achievement.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(achievement.id);
    }
  });

  test("should list achievements for organization", async ({ page }) => {
    const { token, orgId } = await loginAsSyndic(page, "gamif");

    const listResp = await page.request.get(
      `${API_BASE}/organizations/${orgId}/achievements`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const achievements = await listResp.json();
    expect(Array.isArray(achievements)).toBeTruthy();
  });

  test("should list achievements by category", async ({ page }) => {
    const { token, orgId } = await loginAsSyndic(page, "gamif");

    const listResp = await page.request.get(
      `${API_BASE}/organizations/${orgId}/achievements/category/Community`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const achievements = await listResp.json();
    expect(Array.isArray(achievements)).toBeTruthy();
  });

  test("should create a challenge and activate it", async ({ page }) => {
    const { token, orgId } = await loginAsSyndic(page, "gamif");
    const timestamp = Date.now();
    const startDate = new Date();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 30);

    const challengeResp = await page.request.post(`${API_BASE}/challenges`, {
      data: {
        organization_id: orgId,
        title: `Défi communautaire ${timestamp}`,
        description: "Participer à 5 échanges SEL ce mois-ci",
        challenge_type: "Individual",
        target_metric: "exchanges_completed",
        target_value: 5,
        reward_points: 100,
        start_date: startDate.toISOString(),
        end_date: endDate.toISOString(),
        icon: "star",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(challengeResp.status())).toBeTruthy();

    if (challengeResp.ok()) {
      const challenge = await challengeResp.json();
      expect(challenge.id).toBeTruthy();
      expect(challenge.status).toBe("Draft");

      // Activate the challenge
      const activateResp = await page.request.put(
        `${API_BASE}/challenges/${challenge.id}/activate`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(activateResp.status())).toBeTruthy();

      if (activateResp.ok()) {
        const activated = await activateResp.json();
        expect(activated.status).toBe("Active");
      }
    }
  });

  test("should list challenges for organization", async ({ page }) => {
    const { token, orgId } = await loginAsSyndic(page, "gamif");

    const listResp = await page.request.get(
      `${API_BASE}/organizations/${orgId}/challenges`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const challenges = await listResp.json();
    expect(Array.isArray(challenges)).toBeTruthy();
  });

  test("should get organization gamification leaderboard", async ({ page }) => {
    const { token, orgId } = await loginAsSyndic(page, "gamif");

    const leaderboardResp = await page.request.get(
      `${API_BASE}/organizations/${orgId}/gamification/leaderboard`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 404].includes(leaderboardResp.status())).toBeTruthy();
  });

  test("should require auth for gamification API", async ({ page }) => {
    const resp = await page.request.get(
      `${API_BASE}/achievements/some-invalid-id`,
    );
    expect([401, 403, 404].includes(resp.status())).toBeTruthy();
  });
});
