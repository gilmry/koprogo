import { test, expect } from "@playwright/test";
import { loginAsSyndicWithOwner } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupOwnerInBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithOwner(page, "exchange");
  return {
    token: ctx.token,
    buildingId: ctx.buildingId,
    ownerId: ctx.ownerId,
    orgId: ctx.orgId,
  };
}

test.describe("Local Exchanges - SEL Community System", () => {
  test("should display exchanges page", async ({ page }) => {
    await setupOwnerInBuilding(page);
    await page.goto("/exchanges");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='exchanges-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an exchange offer via API", async ({ page }) => {
    const { token, buildingId, ownerId } = await setupOwnerInBuilding(page);
    const timestamp = Date.now();

    const exchangeResp = await page.request.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: buildingId,
        provider_id: ownerId,
        exchange_type: "Service",
        title: `Aide jardinage ${timestamp}`,
        description: "Tonte pelouse et taille haie",
        credits: 2,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(exchangeResp.status())).toBeTruthy();
    const exchange = await exchangeResp.json();
    expect(exchange.building_id).toBe(buildingId);
  });

  test("should list available exchanges for building", async ({ page }) => {
    const { token, buildingId } = await setupOwnerInBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/exchanges/available`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const exchanges = await listResp.json();
    expect(Array.isArray(exchanges)).toBeTruthy();
  });

  test("should navigate to new exchange page", async ({ page }) => {
    await setupOwnerInBuilding(page);
    await page.goto("/exchanges/new");

    await expect(page.locator("body")).toBeVisible();
  });
});
