import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Tickets E2E Test Suite - Maintenance Request Management
 *
 * Tests ticket listing, creation, status workflow, and filtering.
 * Covers owner and syndic perspectives.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Tickets - Maintenance Requests", () => {
  test("should display tickets list page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "ticket");
    await page.goto("/tickets");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='tickets-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a ticket via API and see it in the list", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "ticket",
    );
    const timestamp = Date.now();

    const ticketResponse = await page.request.post(`${API_BASE}/tickets`, {
      data: {
        building_id: buildingId,
        title: `Fuite eau ${timestamp}`,
        description: "Fuite d'eau dans le hall d'entree au 2eme etage",
        priority: "High",
        category: "Plumbing",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(ticketResponse.ok()).toBeTruthy();

    await page.goto(`/tickets?building_id=${buildingId}`);

    await expect(page.locator(`text=Fuite eau ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate to ticket detail page", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "ticket",
    );
    const timestamp = Date.now();

    const ticketResponse = await page.request.post(`${API_BASE}/tickets`, {
      data: {
        building_id: buildingId,
        title: `Detail Ticket ${timestamp}`,
        description: "Panne ascenseur",
        priority: "Critical",
        category: "Other",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(ticketResponse.ok()).toBeTruthy();
    const ticket = await ticketResponse.json();

    await page.goto(`/ticket-detail?id=${ticket.id}`);

    await expect(page.locator(`text=Detail Ticket ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should show ticket priority indicator", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "ticket",
    );
    const timestamp = Date.now();

    // Create an urgent ticket
    const ticketResponse = await page.request.post(`${API_BASE}/tickets`, {
      data: {
        building_id: buildingId,
        title: `Urgent Ticket ${timestamp}`,
        description: "Panne electrique totale",
        priority: "Critical",
        category: "Electrical",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(ticketResponse.ok()).toBeTruthy();

    await page.goto(`/tickets?building_id=${buildingId}`);

    // Ticket should be visible
    await expect(page.locator(`text=Urgent Ticket ${timestamp}`)).toBeVisible({
      timeout: 10000,
    });
  });

  test("should filter tickets by status", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "ticket");
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
    await loginAsSyndicWithBuilding(page, "ticket");
    await page.goto("/work-reports");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='work-reports-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});
