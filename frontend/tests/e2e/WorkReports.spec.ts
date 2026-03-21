import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "workreport");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
}

test.describe("Work Reports - Digital Maintenance Logbook", () => {
  test("should display work reports page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/work-reports");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='work-reports-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a work report via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const reportResp = await page.request.post(`${API_BASE}/work-reports`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        work_type: "repair",
        title: `Remplacement robinetterie ${timestamp}`,
        description: "Remplacement robinets cuisine bâtiment A",
        contractor_name: "Plomberie Dupont",
        work_date: new Date().toISOString(),
        completion_date: new Date().toISOString(),
        cost: 0.0,
        warranty_type: "standard",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(reportResp.status())).toBeTruthy();
    const report = await reportResp.json();
    expect(report.building_id).toBe(buildingId);
  });

  test("should list work reports for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/work-reports`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const reports = await listResp.json();
    expect(Array.isArray(reports) || reports.data !== undefined).toBeTruthy();
  });

  test("should check active warranties", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const warrantiesResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/work-reports/warranties/active`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(warrantiesResp.ok()).toBeTruthy();
  });
});
