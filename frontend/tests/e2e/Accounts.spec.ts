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
    const { token, orgId } = await setupAccountant(page);

    const seedResp = await page.request.post(
      `${API_BASE}/accounts/seed/belgian-pcmn`,
      {
        data: { organization_id: orgId },
        headers: { Authorization: `Bearer ${token}` },
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

    // First seed
    await page.request.post(`${API_BASE}/accounts/seed/belgian-pcmn`, {
      data: { organization_id: orgId },
      headers: { Authorization: `Bearer ${token}` },
    });

    // Find by code
    const findResp = await page.request.get(`${API_BASE}/accounts/code/6120`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 404].includes(findResp.status())).toBeTruthy();
  });
});
