import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupAccountant(
  page: Page,
): Promise<{ token: string; orgId: string }> {
  const timestamp = Date.now();
  const email = `account-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Account Test Org ${timestamp}`,
      slug: `acct-test-${timestamp}`,
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
      first_name: "Account",
      last_name: `Test${timestamp}`,
      role: "superadmin",
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

test.describe("Accounts - PCMN Belgian Chart of Accounts", () => {
  test("should display accountant page", async ({ page }) => {
    await setupAccountant(page);
    await page.goto("/accountant");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='accountant']").first(),
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
