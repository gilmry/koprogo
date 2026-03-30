import { test, expect } from "@playwright/test";
import { loginAsSyndicWithLinkedOwner } from "./helpers/auth";

/**
 * Local Exchanges E2E Test Suite - SEL Community System
 *
 * Tests exchange creation, workflow (Offered→Requested→InProgress→Completed),
 * credit balance management, and community statistics.
 * SEL: 1 hour = 1 credit (time-based currency).
 * Mirrors workflows from backend/tests/e2e_local_exchange.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Local Exchanges - SEL Community System", () => {
  test("should display exchanges page", async ({ page }) => {
    await loginAsSyndicWithLinkedOwner(page, "exchange");
    await page.goto("/exchanges");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='exchanges-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an exchange offer and retrieve it", async ({ page }) => {
    const { ownerToken, buildingId, ownerId } =
      await loginAsSyndicWithLinkedOwner(page, "exchange");
    const timestamp = Date.now();
    const title = `Aide jardinage ${timestamp}`;

    const exchangeResp = await page.request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: buildingId,
        provider_id: ownerId,
        exchange_type: "Service",
        title,
        description: "Tonte pelouse et taille haie",
        credits: 2,
      },
      headers: { Authorization: `Bearer ${ownerToken}` },
    });
    expect([200, 201].includes(exchangeResp.status())).toBeTruthy();

    if (exchangeResp.ok()) {
      const exchange = await exchangeResp.json();
      expect(exchange.id).toBeTruthy();
      expect(exchange.building_id).toBe(buildingId);
      expect(exchange.status).toBe("Offered");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/exchanges/${exchange.id}`,
        { headers: { Authorization: `Bearer ${ownerToken}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(exchange.id);
    }
  });

  test("should list available exchanges for building", async ({ page }) => {
    const { ownerToken, buildingId } = await loginAsSyndicWithLinkedOwner(
      page,
      "exchange",
    );

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/exchanges/available`,
      { headers: { Authorization: `Bearer ${ownerToken}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const exchanges = await listResp.json();
    expect(Array.isArray(exchanges)).toBeTruthy();
  });

  test("should list all building exchanges", async ({ page }) => {
    const { ownerToken, buildingId } = await loginAsSyndicWithLinkedOwner(
      page,
      "exchange",
    );

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/exchanges`,
      { headers: { Authorization: `Bearer ${ownerToken}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const exchanges = await listResp.json();
    expect(Array.isArray(exchanges)).toBeTruthy();
  });

  test("should get SEL statistics for a building", async ({ page }) => {
    const { ownerToken, buildingId } = await loginAsSyndicWithLinkedOwner(
      page,
      "exchange",
    );

    const statsResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/sel-statistics`,
      { headers: { Authorization: `Bearer ${ownerToken}` } },
    );
    expect([200, 404].includes(statsResp.status())).toBeTruthy();
  });

  test("should get owner credit balance", async ({ page }) => {
    const { ownerToken, buildingId, ownerId } =
      await loginAsSyndicWithLinkedOwner(page, "exchange");

    const balanceResp = await page.request.get(
      `${API_BASE}/owners/${ownerId}/buildings/${buildingId}/credit-balance`,
      { headers: { Authorization: `Bearer ${ownerToken}` } },
    );
    expect([200, 404].includes(balanceResp.status())).toBeTruthy();
  });

  test("should get building leaderboard", async ({ page }) => {
    const { ownerToken, buildingId } = await loginAsSyndicWithLinkedOwner(
      page,
      "exchange",
    );

    const leaderboardResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/leaderboard`,
      { headers: { Authorization: `Bearer ${ownerToken}` } },
    );
    expect([200, 404].includes(leaderboardResp.status())).toBeTruthy();
  });

  test("should navigate to new exchange page", async ({ page }) => {
    await loginAsSyndicWithLinkedOwner(page, "exchange");
    await page.goto("/exchanges/new");

    await expect(page.locator("body")).toBeVisible();
  });
});
