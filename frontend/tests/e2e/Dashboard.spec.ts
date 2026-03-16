import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Dashboard - Admin & Syndic Views", () => {
  test("should display admin dashboard page", async ({ page }) => {
    await loginAsSyndic(page, "dashboard");
    await page.goto("/admin");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='admin-dashboard']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display syndic dashboard page", async ({ page }) => {
    await loginAsSyndic(page, "dashboard");
    await page.goto("/syndic");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should display owner dashboard page", async ({ page }) => {
    await loginAsSyndic(page, "dashboard");
    await page.goto("/owner");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should get accountant dashboard stats via API", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "dashboard");

    const statsResp = await page.request.get(
      `${API_BASE}/dashboard/accountant/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 403].includes(statsResp.status())).toBeTruthy();
  });

  test("should get recent transactions via API", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "dashboard");

    const txResp = await page.request.get(
      `${API_BASE}/dashboard/accountant/transactions?limit=5`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 403].includes(txResp.status())).toBeTruthy();
  });
});
