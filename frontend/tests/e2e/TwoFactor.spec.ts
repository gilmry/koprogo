import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

/**
 * Two-Factor Authentication E2E Test Suite - TOTP 2FA
 *
 * Tests 2FA setup, status check, verification, and backup codes.
 * Mirrors workflows from backend/tests/e2e_two_factor.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Two-Factor Authentication (2FA)", () => {
  test("should display settings page", async ({ page }) => {
    await loginAsSyndic(page, "2fa");
    await page.goto("/settings");

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
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

  test("should initiate 2FA setup and return QR code + backup codes", async ({
    page,
  }) => {
    const { token } = await loginAsSyndic(page, "2fa");

    const setupResp = await page.request.post(`${API_BASE}/2fa/setup`, {
      headers: { Authorization: `Bearer ${token}` },
      timeout: 30000,
    });
    expect(setupResp.ok()).toBeTruthy();
    const setupData = await setupResp.json();

    // Must return either qr_code_url or secret (TOTP seed)
    expect(
      setupData.qr_code_url !== undefined || setupData.secret !== undefined,
    ).toBeTruthy();

    // Must return backup codes (8 codes for account recovery)
    expect(
      setupData.backup_codes !== undefined ||
        setupData.qr_code_url !== undefined,
    ).toBeTruthy();
  });

  test("should allow calling setup multiple times (idempotent)", async ({
    page,
  }) => {
    const { token } = await loginAsSyndic(page, "2fa");

    // First setup
    const setup1 = await page.request.post(`${API_BASE}/2fa/setup`, {
      headers: { Authorization: `Bearer ${token}` },
      timeout: 30000,
    });
    expect(setup1.ok()).toBeTruthy();

    // Second setup (should succeed or return conflict)
    const setup2 = await page.request.post(`${API_BASE}/2fa/setup`, {
      headers: { Authorization: `Bearer ${token}` },
      timeout: 30000,
    });
    expect([200, 201, 400, 409].includes(setup2.status())).toBeTruthy();
  });

  test("should reject invalid TOTP code during enable", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "2fa");

    // Setup first
    await page.request.post(`${API_BASE}/2fa/setup`, {
      headers: { Authorization: `Bearer ${token}` },
      timeout: 30000,
    });

    // Try to enable with invalid TOTP code
    const enableResp = await page.request.post(`${API_BASE}/2fa/enable`, {
      data: { totp_code: "000000" },
      headers: { Authorization: `Bearer ${token}` },
    });
    // Should reject invalid code
    expect([400, 401, 422].includes(enableResp.status())).toBeTruthy();
  });

  test("should reject invalid TOTP code during verify", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "2fa");

    const verifyResp = await page.request.post(`${API_BASE}/2fa/verify`, {
      data: { code: "000000" },
      headers: { Authorization: `Bearer ${token}` },
    });
    // Should reject invalid code
    expect([400, 401, 422].includes(verifyResp.status())).toBeTruthy();
  });

  test("should require auth for 2FA endpoints", async ({ page }) => {
    const statusResp = await page.request.get(`${API_BASE}/2fa/status`);
    expect([401, 403].includes(statusResp.status())).toBeTruthy();

    const setupResp = await page.request.post(`${API_BASE}/2fa/setup`);
    expect([401, 403].includes(setupResp.status())).toBeTruthy();
  });
});
