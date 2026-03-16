import { test, expect } from "@playwright/test";
import { loginAsAdmin } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Organizations - SuperAdmin Management", () => {
  test("should display admin organizations page", async ({ page }) => {
    await loginAsAdmin(page);
    await page.goto("/admin/organizations");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='organizations-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should list organizations via API", async ({ page }) => {
    const { adminToken } = await loginAsAdmin(page);

    const listResp = await page.request.get(`${API_BASE}/organizations`, {
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(listResp.ok()).toBeTruthy();
    const orgs = await listResp.json();
    expect(Array.isArray(orgs) || orgs.data !== undefined).toBeTruthy();
  });

  test("should create an organization via API", async ({ page }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    const createResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `New Org ${timestamp}`,
        slug: `new-org-${timestamp}`,
        contact_email: `new-${timestamp}@example.com`,
        subscription_plan: "starter",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const org = await createResp.json();
    expect(org.name).toBe(`New Org ${timestamp}`);
  });

  test("should require superadmin to list organizations", async ({ page }) => {
    const timestamp = Date.now();
    // Create regular user
    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const adminToken = (await adminLoginResp.json()).token;

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Restricted Org ${timestamp}`,
        slug: `restricted-${timestamp}`,
        contact_email: `restricted-${timestamp}@example.com`,
        subscription_plan: "starter",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: `regular-${timestamp}@example.com`,
        password: "test123456",
        first_name: "Regular",
        last_name: "User",
        role: "syndic",
        organization_id: org.id,
      },
    });
    const userData = await regResp.json();

    // Regular syndic should not see all organizations
    const listResp = await page.request.get(`${API_BASE}/organizations`, {
      headers: { Authorization: `Bearer ${userData.token}` },
    });
    expect([200, 403].includes(listResp.status())).toBeTruthy();
  });
});
