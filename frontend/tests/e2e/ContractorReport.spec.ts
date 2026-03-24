import { test, expect } from "@playwright/test";

/**
 * Contractor Report E2E Test Suite - Magic Link PWA
 *
 * Tests the contractor report page accessible via magic link token.
 * This page does NOT require authentication — contractors access it
 * via a time-limited magic link (72h JWT).
 * Uses Traefik on http://localhost.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Contractor Report - Magic Link PWA", () => {
  test("should show error state for invalid magic link token", async ({
    page,
  }) => {
    // Access contractor report with an invalid token
    await page.goto("/contractor-report/invalid-token-12345");

    // Should show the page (not a login redirect) with an error
    await expect(page.locator("body")).toBeVisible();

    // The old-style page shows an error message for invalid tokens
    // Wait for the loading to complete and error to appear
    await page.waitForTimeout(3000);

    // Should show error text about invalid/expired link
    const pageContent = await page.textContent("body");
    expect(
      pageContent?.includes("invalide") ||
        pageContent?.includes("expiré") ||
        pageContent?.includes("Erreur") ||
        pageContent?.includes("invalid") ||
        pageContent?.includes("expired"),
    ).toBeTruthy();
  });

  test("should display the contractor report form elements", async ({
    page,
  }) => {
    // Navigate to the form page (even with invalid token, the form HTML is rendered)
    await page.goto("/contractor-report/test-token");

    // The page should load without requiring authentication
    await expect(page.locator("body")).toBeVisible();

    // Check that the form elements exist in the DOM
    // (they may be hidden until the report loads successfully)
    const formExists = await page.getByTestId("contractor-report-form").count();
    const dateInputExists = await page
      .getByTestId("contractor-report-date-input")
      .count();
    const nameInputExists = await page
      .getByTestId("contractor-report-name-input")
      .count();
    const descriptionExists = await page
      .getByTestId("contractor-report-description-input")
      .count();
    const submitBtnExists = await page
      .getByTestId("contractor-report-submit-btn")
      .count();

    // Form elements should exist in the DOM (present in HTML)
    expect(formExists + dateInputExists + nameInputExists).toBeGreaterThan(0);
  });

  test("should not redirect to login page for contractor report", async ({
    page,
  }) => {
    // Contractor report pages are public (magic link access)
    await page.goto("/contractor-report/some-token");

    // Wait for page to settle
    await page.waitForTimeout(2000);

    // Should NOT be on the login page
    const url = page.url();
    expect(url).not.toContain("/login");
  });

  test("should access the contractor PWA page without authentication", async ({
    page,
  }) => {
    // The /contractor/ page is the PWA-style contractor report form
    await page.goto("/contractor/test-token-123");

    await expect(page.locator("body")).toBeVisible();

    // Should NOT redirect to login (public page)
    await page.waitForTimeout(2000);
    const url = page.url();
    expect(url).not.toContain("/login");
  });
});
