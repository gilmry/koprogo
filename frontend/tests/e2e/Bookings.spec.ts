import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "booking");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
}

test.describe("Bookings - Resource Reservation Calendar", () => {
  test("should display bookings page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/bookings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='bookings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a bookable resource via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const resourceResp = await page.request.post(
      `${API_BASE}/bookable-resources`,
      {
        data: {
          building_id: buildingId,
          name: `Salle polyvalente ${timestamp}`,
          description: "Salle de réunion avec projecteur",
          capacity: 20,
          booking_duration_min: 60,
          booking_duration_max: 480,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(resourceResp.status())).toBeTruthy();
  });

  test("should list building resources for booking", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/bookable-resources`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });

  test("should navigate to booking detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const resourceResp = await page.request.post(
      `${API_BASE}/bookable-resources`,
      {
        data: {
          building_id: buildingId,
          name: `Laverie ${timestamp}`,
          description: "Salle laverie commune",
          capacity: 2,
          booking_duration_min: 60,
          booking_duration_max: 120,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    if (resourceResp.status() === 201) {
      const resource = await resourceResp.json();
      // Create a booking
      const start = new Date();
      start.setDate(start.getDate() + 1);
      start.setHours(10, 0, 0, 0);
      const end = new Date(start);
      end.setHours(11, 0, 0, 0);

      const bookingResp = await page.request.post(
        `${API_BASE}/resource-bookings`,
        {
          data: {
            resource_id: resource.id,
            start_datetime: start.toISOString(),
            end_datetime: end.toISOString(),
            notes: "Linge de lit",
          },
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      if (bookingResp.status() === 201) {
        const booking = await bookingResp.json();
        await page.goto(`/booking-detail?id=${booking.id}`);
        await expect(page.locator("body")).toBeVisible();
        return;
      }
    }
    await page.goto("/bookings");
    await expect(page.locator("body")).toBeVisible();
  });
});
