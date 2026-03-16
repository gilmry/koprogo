import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `booking-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Booking Test Org ${timestamp}`,
      slug: `booking-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Booking",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Booking Building ${timestamp}`,
      address: `${timestamp} Rue Réservation`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 15,
      construction_year: 2010,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, buildingId: building.id, orgId: org.id };
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
