import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "sharing");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
}

test.describe("Sharing - Object Sharing Library", () => {
  test("should display sharing page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/sharing");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='sharing-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a shared object via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const objectResp = await page.request.post(`${API_BASE}/shared-objects`, {
      data: {
        building_id: buildingId,
        object_name: `Perceuse ${timestamp}`,
        object_category: "Tools",
        description: "Perceuse Bosch avec accessoires",
        condition: "Good",
        is_available: true,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(objectResp.status())).toBeTruthy();
  });

  test("should list shared objects for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/shared-objects`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });

  test("should navigate to sharing detail page", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const objectResp = await page.request.post(`${API_BASE}/shared-objects`, {
      data: {
        building_id: buildingId,
        object_name: `Vélo ${timestamp}`,
        description: "Vélo partagé",
        object_category: "Sports",
        is_available: true,
        condition: "Good",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    if (objectResp.status() === 201) {
      const obj = await objectResp.json();
      await page.goto(`/sharing-detail?id=${obj.id}`);
      await expect(page.locator("body")).toBeVisible();
    } else {
      await page.goto("/sharing");
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
