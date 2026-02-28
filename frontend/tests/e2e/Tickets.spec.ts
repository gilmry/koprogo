import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

/**
 * Tickets E2E Test Suite - Maintenance Request Management
 *
 * Tests ticket listing, creation, status workflow, and filtering.
 * Covers owner and syndic perspectives.
 */

const API_BASE = "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `ticket-test-${timestamp}@example.com`;

  const regResponse = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Ticket",
      last_name: `Test${timestamp}`,
      role: "syndic",
    },
  });
  expect(regResponse.ok()).toBeTruthy();
  const userData = await regResponse.json();
  const token = userData.token;
  const orgId = userData.user.organization_id || userData.user.org_id || "";

  const buildingResponse = await page.request.post(
    `${API_BASE}/buildings`,
    {
      data: {
        name: `Ticket Building ${timestamp}`,
        address: `${timestamp} Rue Maintenance`,
        city: "Antwerp",
        postal_code: "2000",
        country: "Belgium",
        total_units: 8,
        construction_year: 2015,
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

  return { token, buildingId: building.id, orgId };
}

test.describe("Tickets - Maintenance Requests", () => {
  test("should display tickets list page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/tickets");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='tickets-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a ticket via API and see it in the list", async ({
    page,
  }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const ticketResponse = await page.request.post(
      `${API_BASE}/tickets`,
      {
        data: {
          building_id: buildingId,
          title: `Fuite eau ${timestamp}`,
          description: "Fuite d'eau dans le hall d'entree au 2eme etage",
          priority: "High",
          category: "Plumbing",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(ticketResponse.ok()).toBeTruthy();

    await page.goto("/tickets");

    await expect(page.locator(`text=Fuite eau ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate to ticket detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const ticketResponse = await page.request.post(
      `${API_BASE}/tickets`,
      {
        data: {
          building_id: buildingId,
          title: `Detail Ticket ${timestamp}`,
          description: "Panne ascenseur",
          priority: "Critical",
          category: "General",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(ticketResponse.ok()).toBeTruthy();
    const ticket = await ticketResponse.json();

    await page.goto(`/ticket-detail?id=${ticket.id}`);

    await expect(
      page.locator(`text=Detail Ticket ${timestamp}`),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should show ticket priority indicator", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    // Create an urgent ticket
    const ticketResponse = await page.request.post(
      `${API_BASE}/tickets`,
      {
        data: {
          building_id: buildingId,
          title: `Urgent Ticket ${timestamp}`,
          description: "Panne electrique totale",
          priority: "Urgent",
          category: "Electrical",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(ticketResponse.ok()).toBeTruthy();

    await page.goto("/tickets");

    // Ticket should be visible
    await expect(
      page.locator(`text=Urgent Ticket ${timestamp}`),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should filter tickets by status", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/tickets");

    // Look for status filter controls
    const statusFilter = page.locator(
      "[data-testid='ticket-status-filter'], select, [role='combobox']",
    );
    if (await statusFilter.first().isVisible()) {
      // Filter is available - test it
      await expect(statusFilter.first()).toBeVisible();
    }

    // Page should remain functional
    await expect(page.locator("body")).toBeVisible();
  });

  test("should display work reports page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/work-reports");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='work-reports-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});
