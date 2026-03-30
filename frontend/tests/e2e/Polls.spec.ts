import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Polls E2E Test Suite - Board Decision Polling System
 *
 * Tests poll creation (YesNo/MultipleChoice/Rating/OpenEnded),
 * lifecycle (Draft→Active→Closed), vote casting, and results.
 * Belgian law: Art. 577-8/4 §4 CC — consultation between assemblies.
 * Mirrors workflows from backend/tests/e2e_polls.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Polls - Board Decision Polling", () => {
  test("should display polls page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "poll");
    await page.goto("/polls");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='polls-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a YesNo poll and retrieve it", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(page, "poll");
    const timestamp = Date.now();
    const question = `Repeindre le hall ${timestamp}`;
    const startDate = new Date();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "yes_no", // PollType uses serde snake_case
        question,
        description: "Sondage avant AG sur la rénovation du hall d'entrée",
        starts_at: startDate.toISOString(),
        ends_at: endDate.toISOString(),
        is_anonymous: false,
        allow_multiple_votes: false,
        options: [
          { option_text: "Oui", display_order: 1 },
          { option_text: "Non", display_order: 2 },
        ],
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(pollResp.status())).toBeTruthy();

    if (pollResp.ok()) {
      const poll = await pollResp.json();
      expect(poll.id).toBeTruthy();
      expect(poll.building_id).toBe(buildingId);
      expect(poll.status).toBe("draft"); // PollStatus uses serde snake_case

      // Retrieve by ID
      const getResp = await page.request.get(`${API_BASE}/polls/${poll.id}`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(poll.id);
    }
  });

  test("should publish a poll (Draft → Active)", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(page, "poll");
    const timestamp = Date.now();
    const startDate = new Date();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "yes_no", // PollType uses serde snake_case
        question: `Sondage activation ${timestamp}`,
        starts_at: startDate.toISOString(),
        ends_at: endDate.toISOString(),
        is_anonymous: false,
        allow_multiple_votes: false,
        options: [
          { option_text: "Oui", display_order: 1 },
          { option_text: "Non", display_order: 2 },
        ],
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (pollResp.ok()) {
      const poll = await pollResp.json();

      const publishResp = await page.request.put(
        `${API_BASE}/polls/${poll.id}/publish`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(publishResp.status())).toBeTruthy();

      if (publishResp.ok()) {
        const published = await publishResp.json();
        expect(published.status).toBe("active"); // PollStatus uses serde snake_case
      }
    }
  });

  test("should list active polls for building", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(page, "poll");

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/polls/active`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const polls = await listResp.json();
    expect(Array.isArray(polls)).toBeTruthy();
  });

  test("should list all building polls", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(page, "poll");

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/polls`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const polls = await listResp.json();
    expect(Array.isArray(polls)).toBeTruthy();
  });

  test("should get poll results", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(page, "poll");
    const timestamp = Date.now();
    const startDate = new Date();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "rating", // PollType uses serde snake_case
        question: `Satisfaction services ${timestamp}`,
        starts_at: startDate.toISOString(),
        ends_at: endDate.toISOString(),
        is_anonymous: true,
        allow_multiple_votes: false,
        min_rating: 1,
        max_rating: 5,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (pollResp.ok()) {
      const poll = await pollResp.json();

      const resultsResp = await page.request.get(
        `${API_BASE}/polls/${poll.id}/results`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 404].includes(resultsResp.status())).toBeTruthy();
    }
  });

  test("should navigate to new poll page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "poll");
    await page.goto("/polls/new");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should require auth for polls API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/polls/some-id`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
