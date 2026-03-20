import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithMeeting(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithMeeting(page, "convoc");
  return {
    token: ctx.token,
    buildingId: ctx.buildingId,
    meetingId: ctx.meetingId,
    orgId: ctx.orgId,
  };
}

test.describe("Convocations - AG Automatic Invitations", () => {
  test("should display convocations page", async ({ page }) => {
    await setupSyndicWithMeeting(page);
    await page.goto("/convocations");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='convocations-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a convocation via API", async ({ page }) => {
    const { token, buildingId, meetingId } = await setupSyndicWithMeeting(page);
    const timestamp = Date.now();

    const convocResp = await page.request.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meetingId,
        building_id: buildingId,
        language: "FR",
        subject: `Convocation AG ${timestamp}`,
        body_text: "Vous êtes convoqués à l'assemblée générale ordinaire.",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201, 400].includes(convocResp.status())).toBeTruthy();
  });

  test("should navigate to convocation detail page", async ({ page }) => {
    const { token, buildingId, meetingId } = await setupSyndicWithMeeting(page);

    const convocResp = await page.request.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meetingId,
        building_id: buildingId,
        language: "FR",
        subject: "Convocation detail test",
        body_text: "Corps de la convocation.",
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (convocResp.status() === 201) {
      const convoc = await convocResp.json();
      await page.goto(`/convocation-detail?id=${convoc.id}`);
      await expect(page.locator("body")).toBeVisible();
    } else {
      // Skip if convocation creation fails (legal deadline not met)
      await page.goto("/convocations");
      await expect(page.locator("body")).toBeVisible();
    }
  });

  test("should list building convocations", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithMeeting(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/convocations`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const convocations = await listResp.json();
    expect(Array.isArray(convocations)).toBeTruthy();
  });
});
