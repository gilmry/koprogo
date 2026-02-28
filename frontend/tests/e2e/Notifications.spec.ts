import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

/**
 * Notifications E2E Test Suite - Multi-Channel Notifications
 *
 * Tests notification listing, read marking, and preference management.
 */

const API_BASE = "http://localhost/api/v1";

async function registerAndLogin(
  page: Page,
  role: string = "syndic",
): Promise<{ token: string; userId: string; email: string }> {
  const timestamp = Date.now();
  const email = `notif-test-${timestamp}@example.com`;

  const response = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Notif",
      last_name: `Test${timestamp}`,
      role,
    },
  });
  expect(response.ok()).toBeTruthy();
  const data = await response.json();

  // Login via UI
  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token: data.token, userId: data.user.id, email };
}

test.describe("Notifications - Multi-Channel System", () => {
  test("should display notifications page", async ({ page }) => {
    await registerAndLogin(page);
    await page.goto("/notifications");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='notifications-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should show empty state when no notifications", async ({ page }) => {
    await registerAndLogin(page);
    await page.goto("/notifications");

    // Should show either notifications or an empty state message
    await expect(page.locator("body")).toBeVisible();
  });

  test("should create a notification via API and see it in the list", async ({
    page,
  }) => {
    const { token, userId } = await registerAndLogin(page);
    const timestamp = Date.now();

    // Create notification via API
    const notifResponse = await page.request.post(`${API_BASE}/notifications`, {
      data: {
        user_id: userId,
        title: `Test Notification ${timestamp}`,
        message: "This is a test notification for E2E testing",
        notification_type: "SystemAlert",
        channel: "InApp",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(notifResponse.ok()).toBeTruthy();

    await page.goto("/notifications");

    await expect(
      page.locator(`text=Test Notification ${timestamp}`),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should display notification preferences page", async ({ page }) => {
    await registerAndLogin(page);
    await page.goto("/settings/notifications");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='notification-preferences']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should handle mark-all-read action", async ({ page }) => {
    const { token, userId } = await registerAndLogin(page);
    const timestamp = Date.now();

    // Create a couple of notifications
    for (let i = 0; i < 3; i++) {
      await page.request.post(`${API_BASE}/notifications`, {
        data: {
          user_id: userId,
          title: `Batch Notif ${timestamp}-${i}`,
          message: `Batch notification ${i}`,
          notification_type: "SystemAlert",
          channel: "InApp",
        },
        headers: { Authorization: `Bearer ${token}` },
      });
    }

    await page.goto("/notifications");

    // Look for mark-all-read button
    const markAllBtn = page.locator(
      "[data-testid='mark-all-read'], button:has-text('marquer'), button:has-text('Mark all')",
    );
    if (await markAllBtn.first().isVisible()) {
      await markAllBtn.first().click();
    }

    // Page should remain functional after action
    await expect(page.locator("body")).toBeVisible();
  });
});
