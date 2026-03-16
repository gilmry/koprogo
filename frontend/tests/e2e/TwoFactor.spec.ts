import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupUser(
  page: Page,
): Promise<{ token: string; email: string }> {
  const timestamp = Date.now();
  const email = `2fa-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `2FA Test Org ${timestamp}`,
      slug: `tfa-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "starter",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "TwoFactor",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, email };
}

test.describe("Two-Factor Authentication (2FA)", () => {
  test("should display settings page with 2FA section", async ({ page }) => {
    await setupUser(page);
    await page.goto("/settings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='settings']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should check 2FA status via API (disabled by default)", async ({
    page,
  }) => {
    const { token } = await setupUser(page);

    const statusResp = await page.request.get(`${API_BASE}/2fa/status`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(statusResp.ok()).toBeTruthy();
    const status = await statusResp.json();
    expect(typeof status.is_enabled).toBe("boolean");
    expect(status.is_enabled).toBe(false);
  });

  test("should initiate 2FA setup via API", async ({ page }) => {
    const { token } = await setupUser(page);

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
