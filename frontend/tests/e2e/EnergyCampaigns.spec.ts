import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Energy Campaigns - Group Buying", () => {
  test("should display energy campaigns page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "energy");
    await page.goto("/energy-campaigns");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='energy-campaigns-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an energy campaign via API", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "energy",
    );
    const timestamp = Date.now();
    const deadline = new Date();
    deadline.setDate(deadline.getDate() + 30);

    const campaignResp = await page.request.post(
      `${API_BASE}/energy-campaigns`,
      {
        data: {
          organization_id: orgId,
          building_id: buildingId,
          campaign_name: `Achat Groupé Energie ${timestamp}`,
          description: "Campagne achat groupé gaz et électricité 2026",
          energy_types: ["Electricity", "Gas"],
          deadline_participation: deadline.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(campaignResp.status())).toBeTruthy();
    const campaign = await campaignResp.json();
    expect(campaign.organization_id).toBe(orgId);
  });

  test("should list energy campaigns", async ({ page }) => {
    const { token } = await loginAsSyndicWithBuilding(page, "energy");

    const listResp = await page.request.get(`${API_BASE}/energy-campaigns`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.ok()).toBeTruthy();
    const campaigns = await listResp.json();
    expect(Array.isArray(campaigns)).toBeTruthy();
  });

  test("should navigate to new campaign page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "energy");
    await page.goto("/energy-campaigns/new");

    await expect(page.locator("body")).toBeVisible();
  });
});
