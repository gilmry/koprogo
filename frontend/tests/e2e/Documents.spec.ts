import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndic(
  page: Page,
): Promise<{ token: string; orgId: string }> {
  const timestamp = Date.now();
  const email = `document-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Document Test Org ${timestamp}`,
      slug: `doc-test-${timestamp}`,
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
      first_name: "Document",
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

test.describe("Documents - File Storage", () => {
  test("should display documents page", async ({ page }) => {
    await setupSyndic(page);
    await page.goto("/documents");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='documents-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner documents page", async ({ page }) => {
    await setupSyndic(page);
    await page.goto("/owner/documents");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should list documents via API", async ({ page }) => {
    const { token } = await setupSyndic(page);

    const listResp = await page.request.get(`${API_BASE}/documents`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.ok()).toBeTruthy();
    const response = await listResp.json();
    // Response can be array or paginated
    expect(
      Array.isArray(response) ||
        response.data !== undefined ||
        response.items !== undefined,
    ).toBeTruthy();
  });

  test("should require auth for documents", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/documents`);
    expect(resp.status()).toBe(401);
  });
});
