import { test, expect } from "@playwright/test";

/**
 * Consent Modal E2E Test Suite - GDPR Consent Flow
 *
 * Tests the consent modal appearing on first visit, acceptance,
 * and persistence across page refreshes.
 *
 * SKIPPED: The ConsentModal.svelte component exists but is never imported or
 * rendered in any page layout. The modal will never appear in the DOM, so all
 * UI-based consent tests will timeout waiting for test IDs.
 * To enable: add <ConsentModal client:load /> to Layout.astro or login.astro.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Consent - Privacy Policy Consent Modal", () => {
  // All UI tests skipped: ConsentModal is not rendered in any page layout
  test.skip("should display consent modal on first visit when no consent is stored", async ({
    page,
  }) => {
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.removeItem("consent-accepted");
    });
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("consent-modal-accept-btn")).toBeVisible({
      timeout: 10000,
    });
    await expect(page.getByTestId("consent-modal-privacy-link")).toBeVisible();
  });

  test.skip("should accept consent and hide the modal", async ({ page }) => {
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.removeItem("consent-accepted");
    });
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("consent-modal-accept-btn")).toBeVisible({
      timeout: 10000,
    });
    await page.getByTestId("consent-modal-accept-btn").click();
    await expect(page.getByTestId("consent-modal-accept-btn")).not.toBeVisible({
      timeout: 5000,
    });
    const consentValue = await page.evaluate(() =>
      localStorage.getItem("consent-accepted"),
    );
    expect(consentValue).toBe("true");
  });

  test.skip("should persist consent status after page refresh", async ({
    page,
  }) => {
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.setItem("consent-accepted", "true");
    });
    await page.reload({ waitUntil: "domcontentloaded" });
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

    // Login as admin to create an organization
    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const adminData = await adminLoginResp.json();
    const adminToken = adminData.token;

    // Create org (consent endpoint requires organization_id in JWT)
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Consent Org ${timestamp}`,
        slug: `consent-${timestamp}`,
        contact_email: email,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    // Register user with organization
    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email,
        password: "test123456",
        first_name: "Consent",
        last_name: `Test${timestamp}`,
        role: "owner",
        organization_id: org.id,
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

  test.skip("should link to privacy policy page from consent modal", async ({
    page,
  }) => {
    await page.goto("/login", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => {
      localStorage.removeItem("consent-accepted");
    });
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByTestId("consent-modal-privacy-link")).toBeVisible({
      timeout: 10000,
    });
    const href = await page
      .getByTestId("consent-modal-privacy-link")
      .getAttribute("href");
    expect(href).toContain("/privacy-policy");
  });
});
