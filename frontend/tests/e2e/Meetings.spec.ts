import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Meetings E2E Test Suite - General Assembly Management
 *
 * Tests meeting listing, creation, and detail pages.
 * Covers AG convocations and resolution viewing.
 *
 * WP-FE1/#550 : utilise le helper partagé `loginAsSyndicWithBuilding` qui
 * passe par injectAuth (set localStorage cache + UNE seule nav dashboard +
 * UN seul silent-refresh via cookie HttpOnly). Évite la course de rotation
 * cookie refresh-token causée par l'ancien UI-login (visite /login →
 * refresh #1 rote le cookie → 2ᵉ nav dashboard → refresh #2 sur cookie
 * déjà révoqué → 401 → cascade page rouge / h1 caché).
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Meetings - General Assembly", () => {
  test("should display meetings list page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "meeting");
    await page.goto("/meetings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='meetings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a meeting via API and see it in the list", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "meeting",
    );
    const timestamp = Date.now();
    const meetingDate = "2026-06-15T14:00:00Z";

    const meetingResponse = await page.request.post(`${API_BASE}/meetings`, {
      data: {
        building_id: buildingId,
        title: `AG Ordinaire ${timestamp}`,
        meeting_type: "Ordinary",
        scheduled_date: meetingDate,
        location: "Salle communale",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(meetingResponse.ok()).toBeTruthy();

    await page.goto("/meetings");

    await expect(page.locator(`text=AG Ordinaire ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate to meeting detail page", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "meeting",
    );
    const timestamp = Date.now();

    const meetingResponse = await page.request.post(`${API_BASE}/meetings`, {
      data: {
        building_id: buildingId,
        title: `Detail Meeting ${timestamp}`,
        meeting_type: "Ordinary",
        scheduled_date: "2026-07-20T10:00:00Z",
        location: "Bureau syndic",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(meetingResponse.ok()).toBeTruthy();
    const meeting = await meetingResponse.json();

    await page.goto(`/meeting-detail?id=${meeting.id}`);

    await expect(page.locator(`text=Detail Meeting ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should display convocations page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "meeting");
    await page.goto("/convocations");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='convocations-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display polls page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "meeting");
    await page.goto("/polls");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='polls-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});
