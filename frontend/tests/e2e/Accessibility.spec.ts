import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

/**
 * WCAG 2.1 Level AA Accessibility Tests
 *
 * Uses axe-core to automatically detect accessibility violations.
 * Tests cover key pages and interactive components.
 */

const WCAG_AA_TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];

test.describe("Accessibility - WCAG 2.1 AA Compliance", () => {
  test("login page should have no accessibility violations", async ({
    page,
  }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    const results = await new AxeBuilder({ page })
      .withTags(WCAG_AA_TAGS)
      .analyze();

    expect(results.violations).toEqual([]);
  });

  test("login page should have proper heading hierarchy", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    // Check h1 exists (use first() to avoid strict mode with browser extensions)
    const h1 = page.locator("main h1, h1.text-4xl").first();
    await expect(h1).toBeVisible();

    // Check page title
    const title = await page.title();
    expect(title).toContain("KoproGo");
  });

  test("login page should have proper form labels", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    // Email input should have associated label
    const emailLabel = page.locator('label[for="email"]');
    await expect(emailLabel).toBeVisible();

    // Password input should have associated label
    const passwordLabel = page.locator('label[for="password"]');
    await expect(passwordLabel).toBeVisible();
  });

  test("login page should support keyboard navigation", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    // Tab through form elements
    await page.keyboard.press("Tab"); // Skip link
    await page.keyboard.press("Tab"); // Email input
    const emailInput = page.locator("#email");
    await expect(emailInput).toBeFocused();

    await page.keyboard.press("Tab"); // Password input
    const passwordInput = page.locator("#password");
    await expect(passwordInput).toBeFocused();
  });

  test("skip navigation link should be accessible", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    // Skip link should exist
    const skipLink = page.locator('a[href="#main-content"]');
    await expect(skipLink).toBeAttached();

    // Should become visible on focus
    await skipLink.focus();
    await expect(skipLink).toBeVisible();
  });

  test("page should have proper lang attribute", async ({ page }) => {
    await page.goto("/login");
    const lang = await page.locator("html").getAttribute("lang");
    expect(lang).toBe("fr");
  });

  test("main content should have proper landmark", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    const main = page.locator("main#main-content");
    await expect(main).toBeAttached();
  });

  test("images should have alt attributes", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    // Check all images have alt text
    const images = page.locator("img");
    const count = await images.count();
    for (let i = 0; i < count; i++) {
      const alt = await images.nth(i).getAttribute("alt");
      expect(alt).not.toBeNull();
    }
  });

  test("color contrast should meet WCAG AA standards", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2aa"])
      .options({ runOnly: ["color-contrast"] })
      .analyze();

    expect(results.violations).toEqual([]);
  });

  test("focus indicators should be visible", async ({ page }) => {
    await page.goto("/login");
    await page.waitForLoadState("networkidle");

    // Tab to email input and verify focus ring
    const emailInput = page.locator("#email");
    await emailInput.focus();

    // The input should have a visible focus indicator (focus:ring-2)
    const styles = await emailInput.evaluate((el) => {
      const computed = window.getComputedStyle(el);
      return {
        outlineStyle: computed.outlineStyle,
        boxShadow: computed.boxShadow,
      };
    });

    // Should have either outline or box-shadow for focus indicator
    const hasFocusIndicator =
      styles.outlineStyle !== "none" ||
      (styles.boxShadow !== "none" && styles.boxShadow !== "");
    expect(hasFocusIndicator).toBe(true);
  });
});
