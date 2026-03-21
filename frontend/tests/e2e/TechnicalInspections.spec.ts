import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "inspection");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
}

test.describe("Technical Inspections - Mandatory Compliance", () => {
  test("should display inspections page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/inspections");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='inspections-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a technical inspection via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();
    const nextInspection = new Date();
    nextInspection.setFullYear(nextInspection.getFullYear() + 2);

    const inspResp = await page.request.post(
      `${API_BASE}/technical-inspections`,
      {
        data: {
          building_id: buildingId,
          organization_id: orgId,
          title: `Inspection ascenseur ${timestamp}`,
          inspection_type: "elevator",
          inspector_name: `Bureau Véritas ${timestamp}`,
          inspection_date: new Date().toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(inspResp.status())).toBeTruthy();
    const inspection = await inspResp.json();
    expect(inspection.building_id).toBe(buildingId);
  });

  test("should list upcoming inspections", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const upcomingResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/technical-inspections/upcoming`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(upcomingResp.ok()).toBeTruthy();
    const inspections = await upcomingResp.json();
    expect(Array.isArray(inspections)).toBeTruthy();
  });

  test("should list overdue inspections", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const overdueResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/technical-inspections/overdue`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(overdueResp.ok()).toBeTruthy();
  });
});
