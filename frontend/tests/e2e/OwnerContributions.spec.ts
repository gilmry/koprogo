import { test, expect } from "@playwright/test";
import { loginAsSyndicWithOwner } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Owner Contributions - Payment Tracking", () => {
  test("should display owner contributions page", async ({ page }) => {
    await loginAsSyndicWithOwner(page, "contrib");
    await page.goto("/owner-contributions");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='contributions-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a contribution via API", async ({ page }) => {
    const { token, ownerId, orgId } = await loginAsSyndicWithOwner(
      page,
      "contrib",
    );
    const timestamp = Date.now();

    const contribResp = await page.request.post(
      `${API_BASE}/owner-contributions`,
      {
        data: {
          organization_id: orgId,
          owner_id: ownerId,
          description: `Provision T2 2026 ${timestamp}`,
          amount: 800.0,
          contribution_type: "Regular",
          contribution_date: new Date().toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(contribResp.status())).toBeTruthy();
  });

  test("should list contributions for owner", async ({ page }) => {
    const { token, ownerId } = await loginAsSyndicWithOwner(page, "contrib");

    const listResp = await page.request.get(
      `${API_BASE}/owner-contributions?owner_id=${ownerId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });
});
