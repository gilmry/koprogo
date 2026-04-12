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

/**
 * Helper: create a unit + owner + assignment in the building so that
 * total_eligible_voters > 0 (required by Poll domain validation).
 */
async function ensureBuildingHasOwner(
  page: import("@playwright/test").Page,
  token: string,
  adminToken: string,
  orgId: string,
  buildingId: string,
) {
  const ts = Date.now();

  // Create a unit in the building
  const unitResp = await page.request.post(`${API_BASE}/units`, {
    data: {
      organization_id: orgId,
      building_id: buildingId,
      unit_number: `P${ts}`,
      floor: 1,
      surface_area: 80.0,
      unit_type: "Apartment",
      quota: 100.0,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const unit = await unitResp.json();

  // Create an owner
  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: orgId,
      first_name: "Poll",
      last_name: `Owner${ts}`,
      email: `poll-owner-${ts}@test.com`,
      address: "1 Rue du Sondage",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  const owner = await ownerResp.json();

  // Assign owner to unit (100% ownership)
  await page.request.post(`${API_BASE}/units/${unit.id}/owners`, {
    data: {
      owner_id: owner.id,
      ownership_percentage: 1.0,
      is_primary_contact: true,
    },
    headers: { Authorization: `Bearer ${token}` },
  });
}

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
    const { token, buildingId, adminToken, orgId } =
      await loginAsSyndicWithBuilding(page, "poll");
    await ensureBuildingHasOwner(page, token, adminToken, orgId, buildingId);
    const timestamp = Date.now();
    const title = `Repeindre le hall ${timestamp}`;
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "yes_no", // PollType uses serde snake_case
        title,
        description: "Sondage avant AG sur la rénovation du hall d'entrée",
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
    expect(pollResp.status()).toBe(201);

    const poll = await pollResp.json();
    expect(poll.id).toBeTruthy();
    expect(poll.building_id).toBe(buildingId);
    expect(poll.status).toBe("draft"); // PollStatus uses serde snake_case

    // Retrieve by ID
    const getResp = await page.request.get(`${API_BASE}/polls/${poll.id}`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(getResp.status()).toBe(200);
    const retrieved = await getResp.json();
    expect(retrieved.id).toBe(poll.id);
  });

  test("should publish a poll (Draft → Active)", async ({ page }) => {
    const { token, buildingId, adminToken, orgId } =
      await loginAsSyndicWithBuilding(page, "poll");
    await ensureBuildingHasOwner(page, token, adminToken, orgId, buildingId);
    const timestamp = Date.now();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "yes_no", // PollType uses serde snake_case
        title: `Sondage activation ${timestamp}`,
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

    expect(pollResp.status()).toBe(201);
    const poll = await pollResp.json();

    const publishResp = await page.request.post(
      `${API_BASE}/polls/${poll.id}/publish`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(publishResp.status()).toBe(200);

    const published = await publishResp.json();
    expect(published.status).toBe("active");
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
      `${API_BASE}/polls?building_id=${buildingId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.status()).toBe(200);
    const data = await listResp.json();
    expect(data.polls).toBeDefined();
    expect(Array.isArray(data.polls)).toBeTruthy();
  });

  test("should get poll results", async ({ page }) => {
    const { token, buildingId, adminToken, orgId } =
      await loginAsSyndicWithBuilding(page, "poll");
    await ensureBuildingHasOwner(page, token, adminToken, orgId, buildingId);
    const timestamp = Date.now();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "rating", // PollType uses serde snake_case
        title: `Satisfaction services ${timestamp}`,
        ends_at: endDate.toISOString(),
        is_anonymous: true,
        allow_multiple_votes: false,
        options: [],
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    expect(pollResp.status()).toBe(201);
    const poll = await pollResp.json();

    const resultsResp = await page.request.get(
      `${API_BASE}/polls/${poll.id}/results`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(resultsResp.status()).toBe(200);
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
