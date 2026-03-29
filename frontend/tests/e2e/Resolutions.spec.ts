import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithMeeting(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithMeeting(page, "resolution");

  // Create a 2nd convocation meeting (no quorum required per Art. 3.87 §5 CC)
  // The default meeting from helper has no quorum validated, so resolution
  // creation would be blocked. Using is_second_convocation bypasses this.
  const meetingDate = new Date();
  meetingDate.setDate(meetingDate.getDate() + 30);
  const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
    data: {
      building_id: ctx.buildingId,
      organization_id: ctx.orgId,
      title: `AG 2e convocation ${Date.now()}`,
      scheduled_date: meetingDate.toISOString(),
      meeting_type: "Ordinary",
      location: "Salle communale",
      is_second_convocation: true,
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });
  if (!meetingResp.ok()) {
    // Fallback: if 2nd convocation creation fails, use original meeting
    // (may fail on quorum check for resolution creation)
    console.log(
      `2nd convocation creation failed (${meetingResp.status()}), using original meeting`,
    );
    return {
      token: ctx.token,
      buildingId: ctx.buildingId,
      meetingId: ctx.meetingId,
      orgId: ctx.orgId,
    };
  }
  const meeting = await meetingResp.json();

  return {
    token: ctx.token,
    buildingId: ctx.buildingId,
    meetingId: meeting.id,
    orgId: ctx.orgId,
  };
}

test.describe("Resolutions - AG Voting System", () => {
  test("should display meetings page with resolutions", async ({ page }) => {
    await setupSyndicWithMeeting(page);
    await page.goto("/meetings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='meetings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a resolution via API", async ({ page }) => {
    const { token, meetingId } = await setupSyndicWithMeeting(page);
    const timestamp = Date.now();

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `Résolution ${timestamp}`,
          description: "Approbation du budget annuel 2026",
          resolution_type: "ordinary",
          majority_required: "absolute",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(resolutionResp.status())).toBeTruthy();
  });

  test("should list meeting resolutions", async ({ page }) => {
    const { token, meetingId } = await setupSyndicWithMeeting(page);

    const listResp = await page.request.get(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const resolutions = await listResp.json();
    expect(Array.isArray(resolutions)).toBeTruthy();
  });

  test("should navigate to meeting detail with resolutions tab", async ({
    page,
  }) => {
    const { meetingId } = await setupSyndicWithMeeting(page);

    await page.goto(`/meeting-detail?id=${meetingId}`);
    await expect(page.locator("body")).toBeVisible();
  });
});
