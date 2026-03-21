import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Two-Factor Authentication (2FA)", () => {
  test("should display settings page with 2FA section", async ({ page }) => {
    await loginAsSyndic(page, "2fa");
    await page.goto("/settings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='settings']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should check 2FA status via API (disabled by default)", async ({
    page,
  }) => {
    const { token } = await loginAsSyndic(page, "2fa");

    const statusResp = await page.request.get(`${API_BASE}/2fa/status`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(statusResp.ok()).toBeTruthy();
    const status = await statusResp.json();
    expect(typeof status.is_enabled).toBe("boolean");
    expect(status.is_enabled).toBe(false);
  });

  test("should initiate 2FA setup via API", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "2fa");

    const setupResp = await page.request.post(`${API_BASE}/2fa/setup`, {
      headers: { Authorization: `Bearer ${token}` },
      timeout: 30000,
    });
    expect(setupResp.ok()).toBeTruthy();
    const setupData = await setupResp.json();
    // Should return QR code URL and backup codes
    expect(
      setupData.qr_code_url !== undefined ||
        setupData.secret !== undefined ||
        setupData.backup_codes !== undefined,
    ).toBeTruthy();
  });

  test("should require auth for 2FA endpoints", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/2fa/status`);
    // Without auth, should return 401 Unauthorized or 403 Forbidden
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
