import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

/**
 * Meetings E2E Test Suite - General Assembly Management
 *
 * Tests meeting listing, creation, and detail pages.
 * Covers AG convocations and resolution viewing.
 */

const API_BASE = "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
}> {
  const timestamp = Date.now();
  const email = `meeting-test-${timestamp}@example.com`;

  const regResponse = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Meeting",
      last_name: `Test${timestamp}`,
      role: "syndic",
    },
  });
  expect(regResponse.ok()).toBeTruthy();
  const userData = await regResponse.json();
  const token = userData.token;

  const buildingResponse = await page.request.post(
    `${API_BASE}/buildings`,
    {
      data: {
        name: `Meeting Building ${timestamp}`,
        address: `${timestamp} Rue AG`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 10,
        construction_year: 2020,
      },
      headers: { Authorization: `Bearer ${token}` },
    },
  );
  expect(buildingResponse.ok()).toBeTruthy();
  const building = await buildingResponse.json();

  // Login via UI
  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, buildingId: building.id };
}

test.describe("Meetings - General Assembly", () => {
  test("should display meetings list page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/meetings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='meetings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a meeting via API and see it in the list", async ({
    page,
  }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();
    const meetingDate = "2026-06-15T14:00:00Z";

    const meetingResponse = await page.request.post(
      `${API_BASE}/meetings`,
      {
        data: {
          building_id: buildingId,
          title: `AG Ordinaire ${timestamp}`,
          meeting_date: meetingDate,
          location: "Salle communale",
          agenda: "Point 1: Comptes annuels\nPoint 2: Budget",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(meetingResponse.ok()).toBeTruthy();

    await page.goto("/meetings");

    await expect(
      page.locator(`text=AG Ordinaire ${timestamp}`),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should navigate to meeting detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const meetingResponse = await page.request.post(
      `${API_BASE}/meetings`,
      {
        data: {
          building_id: buildingId,
          title: `Detail Meeting ${timestamp}`,
          meeting_date: "2026-07-20T10:00:00Z",
          location: "Bureau syndic",
          agenda: "Point 1: Travaux",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(meetingResponse.ok()).toBeTruthy();
    const meeting = await meetingResponse.json();

    await page.goto(`/meeting-detail?id=${meeting.id}`);

    await expect(
      page.locator(`text=Detail Meeting ${timestamp}`),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display convocations page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/convocations");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='convocations-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display polls page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/polls");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='polls-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});
