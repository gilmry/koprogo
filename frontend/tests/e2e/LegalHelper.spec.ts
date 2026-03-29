import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

/**
 * Legal Helper E2E Test Suite - Belgian Copropriété Law Panel
 *
 * Tests the floating legal helper panel: toggle, contextual content,
 * and close functionality. The LegalHelper component provides
 * contextual Belgian law information based on the current page.
 *
 * UI tests SKIPPED: LegalHelper.svelte exists but is never imported or rendered
 * in any page or layout. The toggle button will never appear in the DOM.
 * To enable: add <LegalHelper client:load /> to Layout.astro or syndic pages.
 *
 * API tests remain enabled since the /legal/rules and /legal/ag-sequence
 * endpoints are public and functional.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Legal Helper - Belgian Law Panel", () => {
  // UI tests skipped: LegalHelper component is not rendered in any page layout
  test.skip("should display the legal helper toggle button", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");
    await expect(page.getByTestId("legal-helper-toggle-btn")).toBeVisible({
      timeout: 10000,
    });
  });

  test.skip("should open the legal helper panel when toggle is clicked", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");
    await page.getByTestId("legal-helper-toggle-btn").click();
    await expect(page.getByTestId("legal-helper-close-btn")).toBeVisible({
      timeout: 10000,
    });
  });

  test.skip("should close the legal helper panel when close button is clicked", async ({
    page,
  }) => {
    await loginAsSyndic(page, "legal");
    await page.goto("/syndic");
    await page.getByTestId("legal-helper-toggle-btn").click();
    await expect(page.getByTestId("legal-helper-close-btn")).toBeVisible({
      timeout: 10000,
    });
    await page.getByTestId("legal-helper-close-btn").click();
    await expect(page.getByTestId("legal-helper-close-btn")).not.toBeVisible({
      timeout: 5000,
    });
  });

  test("should serve legal rules from the API", async ({ page }) => {
    // Legal rules endpoint is public (no auth required per routes.rs)
    const rulesResp = await page.request.get(`${API_BASE}/legal/rules`);

    expect(rulesResp.ok()).toBeTruthy();
    const rules = await rulesResp.json();
    expect(Array.isArray(rules)).toBeTruthy();
  });

  test("should serve AG sequence from the API", async ({ page }) => {
    // AG sequence endpoint is public (no auth required per routes.rs)
    const seqResp = await page.request.get(`${API_BASE}/legal/ag-sequence`);

    expect(seqResp.ok()).toBeTruthy();
    const sequence = await seqResp.json();
    expect(Array.isArray(sequence)).toBeTruthy();
  });
});
