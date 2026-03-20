import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "poll");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
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
