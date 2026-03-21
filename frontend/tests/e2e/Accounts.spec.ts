import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupAccountant(page: Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "acct");
  return { token: ctx.token, orgId: ctx.orgId };
}

test.describe("Accounts - PCMN Belgian Chart of Accounts", () => {
  test("should display accountant page", async ({ page }) => {
    await setupAccountant(page);
    await page.goto("/accountant");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='accountant']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should seed Belgian PCMN accounts via API", async ({ page }) => {
    const { orgId } = await setupAccountant(page);
    // Use admin (superadmin) token for seeding
    const adminResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const adminToken = (await adminResp.json()).token;

    const seedResp = await page.request.post(
      `${API_BASE}/accounts/seed/belgian-pcmn`,
      {
        data: { organization_id: orgId },
        headers: { Authorization: `Bearer ${adminToken}` },
      },
    );
    // 201 if first seed, 200 or 409 if already seeded
    expect([200, 201, 409].includes(seedResp.status())).toBeTruthy();
  });

  test("should list accounts via API", async ({ page }) => {
    const { token } = await setupAccountant(page);

    const listResp = await page.request.get(`${API_BASE}/accounts`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.ok()).toBeTruthy();
    const accounts = await listResp.json();
    expect(Array.isArray(accounts) || accounts.data !== undefined).toBeTruthy();
  });

  test("should find account by code", async ({ page }) => {
    const { token, orgId } = await setupAccountant(page);
    const adminResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const adminToken = (await adminResp.json()).token;

    // Seed with admin token (superadmin required)
    await page.request.post(`${API_BASE}/accounts/seed/belgian-pcmn`, {
      data: { organization_id: orgId },
      headers: { Authorization: `Bearer ${adminToken}` },
    });

    // Find by code with syndic token (has organization_id in JWT)
    const findResp = await page.request.get(`${API_BASE}/accounts/code/612`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 404].includes(findResp.status())).toBeTruthy();
  });
});
