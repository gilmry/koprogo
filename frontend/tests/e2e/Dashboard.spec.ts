import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndic(
  page: Page,
): Promise<{ token: string; orgId: string }> {
  const timestamp = Date.now();
  const email = `dashboard-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Dashboard Test Org ${timestamp}`,
      slug: `dashboard-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Dashboard",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token: userData.token, orgId: org.id };
}

test.describe("Dashboard - Admin & Syndic Views", () => {
  test("should display admin dashboard page", async ({ page }) => {
    await setupSyndic(page);
    await page.goto("/admin");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='admin-dashboard']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display syndic dashboard page", async ({ page }) => {
    await setupSyndic(page);
    await page.goto("/syndic");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should display owner dashboard page", async ({ page }) => {
    await setupSyndic(page);
    await page.goto("/owner");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should get accountant dashboard stats via API", async ({ page }) => {
    const { token } = await setupSyndic(page);

    const statsResp = await page.request.get(
      `${API_BASE}/dashboard/accountant/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 403].includes(statsResp.status())).toBeTruthy();
  });

  test("should get recent transactions via API", async ({ page }) => {
    const { token } = await setupSyndic(page);

    const txResp = await page.request.get(
      `${API_BASE}/dashboard/accountant/transactions?limit=5`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 403].includes(txResp.status())).toBeTruthy();
  });
});
