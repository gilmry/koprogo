import { test, expect, Page } from "@playwright/test";

/**
 * GDPR E2E Test Suite - Idempotent & Self-Contained
 *
 * Tests use Traefik on http://localhost with full CRUD operations.
 * Each test creates its own data, performs operations, and cleans up.
 * Tests mix user and admin actors for realistic scenarios.
 */

const API_BASE = "http://localhost/api/v1";

// Helper: Register and login a new user
async function registerAndLogin(
  page: Page,
  role: string = "owner",
): Promise<{ email: string; password: string; token: string; userId: string }> {
  const timestamp = Date.now();
  const email = `gdpr-test-${timestamp}@example.com`;
  const password = "test123456";

  // Register via API
  const response = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password,
      first_name: "GDPR",
      last_name: `Test${timestamp}`,
      role,
    },
  });

  expect(response.ok()).toBeTruthy();
  const data = await response.json();

  return {
    email,
    password,
    token: data.token,
    userId: data.user.id,
  };
}

// Helper: Login via UI
async function loginViaUI(page: Page, email: string, password: string) {
  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill(password);
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(owner|syndic|accountant|admin)/);
}

// Helper: Login as SuperAdmin
async function loginAsSuperAdmin(page: Page) {
  await loginViaUI(page, "admin@koprogo.com", "admin123");
}

test.describe("GDPR - Complete User Journey (Idempotent)", () => {
  test("should allow user to register, export data, and erase account", async ({
    page,
  }) => {
    // Debug: Listen to console logs
    page.on("console", (msg) => console.log("BROWSER LOG:", msg.text()));
    page.on("pageerror", (err) => console.error("PAGE ERROR:", err.message));

    // Step 1: Register new user
    const user = await registerAndLogin(page, "owner");

    // Step 2: Login via UI
    await loginViaUI(page, user.email, user.password);

    // Step 3: Navigate to GDPR panel
    await page.goto("/settings/gdpr");
    await expect(page.getByTestId("gdpr-data-panel")).toBeVisible();
    await page.waitForTimeout(1000); // Wait for Svelte hydration

    // Step 4: Export personal data (Article 15)
    await page.getByTestId("gdpr-export-button").click();
    await expect(page.getByTestId("gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });

    // Verify export contains user email
    await expect(
      page.getByTestId("gdpr-export-modal").locator(`text=${user.email}`),
    ).toBeVisible();

    // Download export
    const downloadPromise = page.waitForEvent("download");
    await page.getByTestId("gdpr-download-export-button").click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain("gdpr-export");

    await page.getByTestId("gdpr-export-modal-close").click();
    await expect(page.getByTestId("gdpr-export-modal")).not.toBeVisible();

    // Step 5: Erase personal data (Article 17)
    await page.getByTestId("gdpr-erase-button").click();
    await expect(page.getByTestId("gdpr-erase-confirm-modal")).toBeVisible({
      timeout: 10000,
    });

    await page.getByTestId("gdpr-erase-confirm-button").click();

    // Wait for success and auto-logout
    await expect(page.locator("text=/success|anonymi[sz]ed/i")).toBeVisible({
      timeout: 10000,
    });
    await page.waitForURL("/login", { timeout: 10000 });

    // Step 6: Verify cannot login anymore
    await page.getByTestId("login-email").fill(user.email);
    await page.getByTestId("login-password").fill(user.password);
    await page.getByTestId("login-submit").click();

    await expect(page.getByTestId("login-error")).toBeVisible();
  });
});

// TODO: Requires database cleanup before tests (see issue #66)
test.describe.skip("GDPR - Admin Operations (Idempotent)", () => {
  test("should allow admin to export and erase user data", async ({ page }) => {
    // Step 1: Create test user
    const user = await registerAndLogin(page, "owner");

    // Step 2: Login as SuperAdmin
    await loginAsSuperAdmin(page);

    // Step 3: Navigate to admin GDPR panel
    await page.goto("/admin/gdpr");
    await expect(page.getByTestId("admin-gdpr-panel")).toBeVisible();

    // Wait for users table to load (at least one row visible)
    await expect(page.getByTestId("admin-gdpr-user-row").first()).toBeVisible({ timeout: 10000 });

    // Step 4: Search for test user
    await page.getByTestId("admin-gdpr-search").fill(user.email);
    await page.waitForTimeout(500); // Wait for reactive filter to apply

    // Step 5: Export user data as admin
    const userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: user.email });
    await expect(userRow).toBeVisible();

    await userRow.getByTestId("admin-gdpr-export-user").click();
    await expect(page.getByTestId("admin-gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });

    // Download export
    const downloadPromise = page.waitForEvent("download");
    await page.getByTestId("admin-gdpr-download-button").click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain("gdpr-export");

    await page.getByTestId("admin-gdpr-modal-close").click();
    await expect(page.getByTestId("admin-gdpr-export-modal")).not.toBeVisible();

    // Step 6: Erase user data as admin
    await userRow.getByTestId("admin-gdpr-erase-user").click();
    await expect(page.getByTestId("admin-gdpr-erase-modal")).toBeVisible({
      timeout: 10000,
    });

    await page.getByTestId("admin-gdpr-erase-confirm").click();
    await expect(page.getByTestId("admin-gdpr-erasure-result")).toBeVisible({
      timeout: 10000,
    });

    // Step 7: Verify erasure in audit logs
    await page.getByTestId("admin-gdpr-audit-toggle").click();
    await expect(page.getByTestId("admin-gdpr-audit-logs")).toBeVisible();

    const auditRows = page.getByTestId("admin-gdpr-audit-log-row");
    await expect(auditRows.first()).toBeVisible({ timeout: 5000 });
  });
});

// TODO: Requires database cleanup before tests (see issue #66)
test.describe.skip("GDPR - Mixed Scenario: User Creates Data, Admin Exports", () => {
  test("should handle user creating data then admin exporting it", async ({
    page,
  }) => {
    // Step 1: User registers and creates some activity
    const user = await registerAndLogin(page, "syndic");

    // Step 2: User logs in and navigates around (creates activity)
    await loginViaUI(page, user.email, user.password);
    await page.goto("/syndic");
    await page.waitForTimeout(500);

    // Step 3: User logs out
    await page.getByTestId("user-menu-button").click();
    await page.getByTestId("user-menu-logout").click();
    await page.waitForURL("/login");

    // Step 4: Admin logs in
    await loginAsSuperAdmin(page);

    // Step 5: Admin exports user's data
    await page.goto("/admin/gdpr");
    await expect(page.getByTestId("admin-gdpr-user-row").first()).toBeVisible({ timeout: 10000 });
    await page.getByTestId("admin-gdpr-search").fill(user.email);
    await page.waitForTimeout(500);

    const userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: user.email });
    await userRow.getByTestId("admin-gdpr-export-user").click();

    await expect(page.getByTestId("admin-gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });

    // Verify export data contains user info
    await expect(
      page.getByTestId("admin-gdpr-export-modal").locator(`text=${user.email}`),
    ).toBeVisible();

    await page.getByTestId("admin-gdpr-modal-close").click();
    await expect(page.getByTestId("admin-gdpr-export-modal")).not.toBeVisible();

    // Step 6: User logs back in and exports own data
    await page.getByTestId("user-menu-button").click();
    await page.getByTestId("user-menu-logout").click();

    await loginViaUI(page, user.email, user.password);
    await page.goto("/settings/gdpr");
    await expect(page.getByTestId("gdpr-data-panel")).toBeVisible();
    await page.waitForTimeout(1000);

    await page.getByTestId("gdpr-export-button").click();
    await expect(page.getByTestId("gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });

    // User sees their own data
    await expect(
      page.getByTestId("gdpr-export-modal").locator(`text=${user.email}`),
    ).toBeVisible();

    // Cleanup: Erase account
    await page.getByTestId("gdpr-export-modal-close").click();
    await expect(page.getByTestId("gdpr-export-modal")).not.toBeVisible();
    await page.getByTestId("gdpr-erase-button").click();
    await page.getByTestId("gdpr-erase-confirm-button").click();
    await page.waitForURL("/login", { timeout: 10000 });
  });
});

// TODO: Requires database cleanup before tests (see issue #66)
test.describe.skip("GDPR - Audit Logs Verification", () => {
  test("should record all GDPR operations in audit logs", async ({ page }) => {
    // Step 1: Create user and perform export
    const user = await registerAndLogin(page, "owner");
    await loginViaUI(page, user.email, user.password);

    await page.goto("/settings/gdpr");
    await expect(page.getByTestId("gdpr-data-panel")).toBeVisible();
    await page.waitForTimeout(1000);

    // Trigger export (creates audit log)
    await page.getByTestId("gdpr-export-button").click();
    await expect(page.getByTestId("gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });
    await page.getByTestId("gdpr-export-modal-close").click();
    await expect(page.getByTestId("gdpr-export-modal")).not.toBeVisible();

    // Logout
    await page.getByTestId("user-menu-button").click();
    await page.getByTestId("user-menu-logout").click();

    // Step 2: Admin checks audit logs
    await loginAsSuperAdmin(page);
    await page.goto("/admin/gdpr");

    await page.getByTestId("admin-gdpr-audit-toggle").click();
    await expect(page.getByTestId("admin-gdpr-audit-logs")).toBeVisible();

    // Verify audit log entry exists
    const auditRows = page.getByTestId("admin-gdpr-audit-log-row");
    await expect(auditRows.first()).toBeVisible({ timeout: 5000 });

    // Verify contains "Export" event
    await expect(
      page.getByTestId("admin-gdpr-audit-logs").locator("text=/Export/i"),
    ).toBeVisible();

    // Cleanup
    await expect(page.getByTestId("admin-gdpr-user-row").first()).toBeVisible({ timeout: 10000 });
    await page.getByTestId("admin-gdpr-search").fill(user.email);
    await page.waitForTimeout(500);

    const userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: user.email });
    await userRow.getByTestId("admin-gdpr-erase-user").click();
    await page.getByTestId("admin-gdpr-erase-confirm").click();
    await expect(page.getByTestId("admin-gdpr-erasure-result")).toBeVisible({
      timeout: 10000,
    });
  });
});

// TODO: Requires database cleanup before tests (see issue #66)
test.describe.skip("GDPR - Cross-Organization Access", () => {
  test("should allow SuperAdmin to access any user regardless of organization", async ({
    page,
  }) => {
    // Step 1: Create users in different contexts
    const user1 = await registerAndLogin(page, "owner");
    const user2 = await registerAndLogin(page, "syndic");

    // Step 2: Admin accesses both users
    await loginAsSuperAdmin(page);
    await page.goto("/admin/gdpr");
    await expect(page.getByTestId("admin-gdpr-user-row").first()).toBeVisible({ timeout: 10000 });

    // Search user1
    await page.getByTestId("admin-gdpr-search").fill(user1.email);
    await page.waitForTimeout(500);

    let userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: user1.email });
    await expect(userRow).toBeVisible();

    // Search user2
    await page.getByTestId("admin-gdpr-search").fill(user2.email);
    await page.waitForTimeout(500);

    userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: user2.email });
    await expect(userRow).toBeVisible();

    // Cleanup both users
    for (const user of [user1, user2]) {
      await page.getByTestId("admin-gdpr-search").fill(user.email);
      await page.waitForTimeout(500);

      userRow = page
        .getByTestId("admin-gdpr-user-row")
        .filter({ hasText: user.email });
      await userRow.getByTestId("admin-gdpr-erase-user").click();
      await page.getByTestId("admin-gdpr-erase-confirm").click();
      await expect(page.getByTestId("admin-gdpr-erasure-result")).toBeVisible({
        timeout: 10000,
      });
      await page.waitForTimeout(1000);
    }
  });
});
