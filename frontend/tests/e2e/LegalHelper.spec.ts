import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

/**
 * Legal Helper E2E Test Suite - Belgian Copropriété Law Panel
 *
 * Tests the floating legal helper panel: toggle, contextual content,
 * and close functionality. The LegalHelper component provides
 * contextual Belgian law information based on the current page.
 * Uses Traefik on http://localhost.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Legal Helper - Belgian Law Panel", () => {
  test("should display the legal helper toggle button", async ({ page }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");

    // The floating help button should be visible
    await expect(page.getByTestId("legal-helper-toggle-btn")).toBeVisible({
      timeout: 10000,
    });
  });

  test("should open the legal helper panel when toggle is clicked", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");

    // Click the toggle button
    await page.getByTestId("legal-helper-toggle-btn").click();

    // Close button should appear (indicating panel is open)
    await expect(page.getByTestId("legal-helper-close-btn")).toBeVisible({
      timeout: 10000,
    });
  });

  test("should close the legal helper panel when close button is clicked", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");

    // Open panel
    await page.getByTestId("legal-helper-toggle-btn").click();
    await expect(page.getByTestId("legal-helper-close-btn")).toBeVisible({
      timeout: 10000,
    });

    // Close panel
    await page.getByTestId("legal-helper-close-btn").click();

    // Close button should no longer be visible
    await expect(
      page.getByTestId("legal-helper-close-btn"),
    ).not.toBeVisible({ timeout: 5000 });
  });

  test("should serve legal rules from the API", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "legal");

    // Verify the legal rules endpoint is accessible
    const rulesResp = await page.request.get(`${API_BASE}/legal/rules`, {
      headers: { Authorization: `Bearer ${token}` },
    });

    expect(rulesResp.ok()).toBeTruthy();
    const rules = await rulesResp.json();
    expect(Array.isArray(rules)).toBeTruthy();
  });

  test("should serve AG sequence from the API", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "legal");

    // Verify the AG sequence endpoint is accessible
    const seqResp = await page.request.get(`${API_BASE}/legal/ag-sequence`, {
      headers: { Authorization: `Bearer ${token}` },
    });

    expect(seqResp.ok()).toBeTruthy();
    const sequence = await seqResp.json();
    expect(Array.isArray(sequence)).toBeTruthy();
  });
});
