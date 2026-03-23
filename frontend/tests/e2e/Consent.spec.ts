import { test, expect } from "@playwright/test";

/**
 * Consent Modal E2E Test Suite - GDPR Consent Flow
 *
 * Tests the consent modal appearing on first visit, acceptance,
 * and persistence across page refreshes.
 * Uses Traefik on http://localhost.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Consent - Privacy Policy Consent Modal", () => {
  test("should display consent modal on first visit when no consent is stored", async ({
    page,
  }) => {
    // Clear any existing consent state
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.removeItem("consent-accepted");
    });

    // Reload the page to trigger consent check
    await page.reload({ waitUntil: "domcontentloaded" });

    // Wait for the consent modal accept button to appear
    // The ConsentModal checks localStorage on mount
    await expect(page.getByTestId("consent-modal-accept-btn")).toBeVisible({
      timeout: 10000,
    });

    // Privacy policy link should also be visible
    await expect(page.getByTestId("consent-modal-privacy-link")).toBeVisible();
  });

  test("should accept consent and hide the modal", async ({ page }) => {
    // Clear consent state
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.removeItem("consent-accepted");
    });

    await page.reload({ waitUntil: "domcontentloaded" });

    // Wait for modal
    await expect(page.getByTestId("consent-modal-accept-btn")).toBeVisible({
      timeout: 10000,
    });

    // Click accept
    await page.getByTestId("consent-modal-accept-btn").click();

    // Modal should disappear
    await expect(
      page.getByTestId("consent-modal-accept-btn"),
    ).not.toBeVisible({ timeout: 5000 });

    // Verify localStorage was set
    const consentValue = await page.evaluate(() =>
      localStorage.getItem("consent-accepted"),
    );
    expect(consentValue).toBe("true");
  });

  test("should persist consent status after page refresh", async ({
    page,
  }) => {
    // Set consent in localStorage
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.setItem("consent-accepted", "true");
    });

    // Reload page
    await page.reload({ waitUntil: "domcontentloaded" });

    // Modal should NOT appear since consent was already accepted
    // Wait a bit to ensure the component has mounted and checked
    await page.waitForTimeout(2000);
    await expect(
      page.getByTestId("consent-modal-accept-btn"),
    ).not.toBeVisible();
  });

  test("should record consent via API when user is authenticated", async ({
    page,
  }) => {
    const timestamp = Date.now();
    const email = `consent-test-${timestamp}@example.com`;

    // Register user
    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email,
        password: "test123456",
        first_name: "Consent",
        last_name: `Test${timestamp}`,
        role: "owner",
      },
    });
    expect(regResp.ok()).toBeTruthy();
    const userData = await regResp.json();
    const token = userData.token;

    // Record consent via API
    const consentResp = await page.request.post(`${API_BASE}/consent`, {
      data: { consent_type: "privacy_policy" },
      headers: { Authorization: `Bearer ${token}` },
    });

    expect(consentResp.ok()).toBeTruthy();
    const consent = await consentResp.json();
    expect(consent.consent_type).toBe("privacy_policy");
  });

  test("should link to privacy policy page from consent modal", async ({
    page,
  }) => {
    // Clear consent state
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.removeItem("consent-accepted");
    });

    await page.reload({ waitUntil: "domcontentloaded" });

    // Wait for modal
    await expect(page.getByTestId("consent-modal-privacy-link")).toBeVisible({
      timeout: 10000,
    });

    // Verify the privacy policy link points to the correct URL
    const href = await page
      .getByTestId("consent-modal-privacy-link")
      .getAttribute("href");
    expect(href).toContain("/privacy-policy");
  });
});
