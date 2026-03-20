import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Call For Funds - Revenue Management", () => {
  test("should display call-for-funds page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "cff");
    await page.goto("/call-for-funds");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='call-for-funds-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a call for funds via API", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "cff",
    );
    const timestamp = Date.now();
    const dueDate = new Date();
    dueDate.setDate(dueDate.getDate() + 30);

    const cffResp = await page.request.post(`${API_BASE}/call-for-funds`, {
      data: {
        organization_id: orgId,
        building_id: buildingId,
        title: `Appel fonds T1 2026 ${timestamp}`,
        total_amount: 5000.0,
        contribution_type: "Regular",
        call_date: new Date().toISOString(),
        due_date: dueDate.toISOString(),
        description: "Provision charges courantes",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(cffResp.status())).toBeTruthy();
    const cff = await cffResp.json();
    expect(cff.building_id).toBe(buildingId);
  });

  test("should list calls for funds via API", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(page, "cff");

    const listResp = await page.request.get(
      `${API_BASE}/call-for-funds?building_id=${buildingId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const response = await listResp.json();
    expect(Array.isArray(response)).toBeTruthy();
  });
});
