import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Documents - File Storage", () => {
  test("should display documents page", async ({ page }) => {
    await loginAsSyndic(page, "doc");
    await page.goto("/documents");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='documents-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display owner documents page", async ({ page }) => {
    await loginAsSyndic(page, "doc");
    await page.goto("/owner/documents");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should list documents via API", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "doc");

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
