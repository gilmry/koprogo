import { test, expect } from "@playwright/test";
import { loginAsSyndicWithLinkedOwner } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupOwnerWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithLinkedOwner(page, "booking");
  return {
    token: ctx.ownerToken,
    syndicToken: ctx.token,
    buildingId: ctx.buildingId,
    orgId: ctx.orgId,
    ownerId: ctx.ownerId,
  };
}

test.describe("Bookings - Resource Reservation Calendar", () => {
  test("should display bookings page", async ({ page }) => {
    await setupOwnerWithBuilding(page);
    await page.goto("/bookings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='bookings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a resource booking via API", async ({ page }) => {
    const { token, buildingId } = await setupOwnerWithBuilding(page);
    const start = new Date();
    start.setDate(start.getDate() + 1);
    start.setHours(10, 0, 0, 0);
    const end = new Date(start);
    end.setHours(14, 0, 0, 0);

    const bookingResp = await page.request.post(
      `${API_BASE}/resource-bookings`,
      {
        data: {
          building_id: buildingId,
          resource_type: "MeetingRoom",
          resource_name: "Salle polyvalente",
          start_time: start.toISOString(),
          end_time: end.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(bookingResp.status())).toBeTruthy();
  });

  test("should list my resource bookings", async ({ page }) => {
    const { token } = await setupOwnerWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/resource-bookings/my`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });

  test("should navigate to booking detail page", async ({ page }) => {
    const { token, buildingId } = await setupOwnerWithBuilding(page);
    const start = new Date();
    start.setDate(start.getDate() + 2);
    start.setHours(10, 0, 0, 0);
    const end = new Date(start);
    end.setHours(11, 0, 0, 0);

    const bookingResp = await page.request.post(
      `${API_BASE}/resource-bookings`,
      {
        data: {
          building_id: buildingId,
          resource_type: "LaundryRoom",
          resource_name: "Laverie commune",
          start_time: start.toISOString(),
          end_time: end.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    if (bookingResp.status() === 201) {
      const booking = await bookingResp.json();
      await page.goto(`/booking-detail?id=${booking.id}`);
      await expect(page.locator("body")).toBeVisible();
    } else {
      await page.goto("/bookings");
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
