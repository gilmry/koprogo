import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupBuildingWithOwner(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "poll");
  const timestamp = Date.now();

  // Create a unit (requires admin token)
  const unitResp = await page.request.post(`${API_BASE}/units`, {
    data: {
      organization_id: ctx.orgId,
      building_id: ctx.buildingId,
      unit_number: "1A",
      floor: 1,
      surface_area: 85.0,
      unit_type: "Apartment",
      quota: 100.0,
    },
    headers: { Authorization: `Bearer ${ctx.adminToken}` },
  });
  const unit = await unitResp.json();

  // Create an owner
  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: ctx.orgId,
      first_name: "Poll",
      last_name: `Voter${timestamp}`,
      email: `voter-${timestamp}@test.com`,
      address: "1 Rue Test",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });
  const owner = await ownerResp.json();

  // Assign owner to unit
  await page.request.post(`${API_BASE}/units/${unit.id}/owners`, {
    data: {
      owner_id: owner.id,
      ownership_percentage: 1.0,
      is_primary_contact: true,
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });

  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
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

  test("should navigate to new poll page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "poll");
    await page.goto("/polls/new");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should create a poll via API", async ({ page }) => {
    const { token, buildingId } = await setupBuildingWithOwner(page);
    const timestamp = Date.now();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7);

    const pollResp = await page.request.post(`${API_BASE}/polls`, {
      data: {
        building_id: buildingId,
        poll_type: "yes_no",
        title: `Repeindre le hall ${timestamp}`,
        description: "Sondage avant AG",
        ends_at: endDate.toISOString(),
        is_anonymous: false,
        options: [
          { option_text: "Oui", display_order: 1 },
          { option_text: "Non", display_order: 2 },
        ],
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(pollResp.status())).toBeTruthy();
    const poll = await pollResp.json();
    expect(poll.building_id).toBe(buildingId);
  });

  test("should list active polls for building", async ({ page }) => {
    const { token, buildingId } = await setupBuildingWithOwner(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/polls/active`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const polls = await listResp.json();
    expect(Array.isArray(polls)).toBeTruthy();
  });
});
