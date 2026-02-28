import { test, expect } from "@playwright/test";

/**
 * Login E2E Test Suite - Authentication Flows
 *
 * Tests the login page, error handling, and session management.
 * Uses Traefik on http://localhost.
 */

const API_BASE = "http://localhost/api/v1";

test.describe("Login - Authentication Flow", () => {
  test("should display login form with email and password fields", async ({
    page,
  }) => {
    await page.goto("/login");

    await expect(page.getByTestId("login-email")).toBeVisible();
    await expect(page.getByTestId("login-password")).toBeVisible();
    await expect(page.getByTestId("login-submit")).toBeVisible();
  });

  test("should show error on invalid credentials", async ({ page }) => {
    await page.goto("/login");

    await page.getByTestId("login-email").fill("invalid@example.com");
    await page.getByTestId("login-password").fill("wrongpassword");
    await page.getByTestId("login-submit").click();

    await expect(page.getByTestId("login-error")).toBeVisible({
      timeout: 10000,
    });
  });

  test("should login successfully and redirect to dashboard", async ({
    page,
  }) => {
    const timestamp = Date.now();
    const email = `login-test-${timestamp}@example.com`;
    const password = "test123456";

    // Register via API
    const regResponse = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email,
        password,
        first_name: "Login",
        last_name: `Test${timestamp}`,
        role: "owner",
      },
    });
    expect(regResponse.ok()).toBeTruthy();

    // Login via UI
    await page.goto("/login");
    await page.getByTestId("login-email").fill(email);
    await page.getByTestId("login-password").fill(password);
    await page.getByTestId("login-submit").click();

    await page.waitForURL(/\/(owner|syndic|accountant|admin)/, {
      timeout: 15000,
    });

    // Verify we're on a dashboard page
    await expect(page.locator("body")).toBeVisible();
  });

  test("should show validation for empty fields", async ({ page }) => {
    await page.goto("/login");

    // Click submit without filling fields
    await page.getByTestId("login-submit").click();

    // Browser native validation or custom validation should prevent submission
    // The form should still be on the login page
    await expect(page).toHaveURL(/\/login/);
  });

  test("should navigate to register page from login", async ({ page }) => {
    await page.goto("/login");

    // Look for a register link
    const registerLink = page.locator('a[href*="register"]');
    if (await registerLink.isVisible()) {
      await registerLink.click();
      await page.waitForURL(/\/register/, { timeout: 10000 });
      await expect(page).toHaveURL(/\/register/);
    }
  });
});
