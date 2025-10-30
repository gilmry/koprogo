import { test, expect, Page } from "@playwright/test";

/**
 * GDPR E2E Test Suite
 *
 * Tests comprehensive GDPR workflows (Articles 15 & 17):
 * - User self-service data export (Article 15 - Right to Access)
 * - User self-service data erasure (Article 17 - Right to Erasure)
 * - Admin-initiated data export for any user
 * - Admin-initiated data erasure with legal holds validation
 * - Audit logs viewing and verification
 *
 * All tests use data-testid attributes for stable selectors.
 * Videos are recorded for documentation purposes.
 */

// Helper: Login as user
async function loginAsUser(
  page: Page,
  email: string,
  password: string,
): Promise<void> {
  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill(password);
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(owner|syndic|accountant|admin)/);
}

// Helper: Create test user via API
async function createTestUser(
  page: Page,
  role: string = "owner",
): Promise<{ email: string; password: string; userId: string }> {
  const timestamp = Date.now();
  const email = `gdpr-test-${timestamp}@example.com`;
  const password = "test123456";

  const response = await page.request.post("/api/v1/auth/register", {
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
    userId: data.user.id,
  };
}

// Helper: Login as SuperAdmin
async function loginAsSuperAdmin(page: Page): Promise<void> {
  // Default superadmin credentials (from seed data or env)
  await loginAsUser(page, "admin@koprogo.com", "admin123");
}

test.describe("GDPR - User Self-Service (Articles 15 & 17)", () => {
  test("should export user personal data (Article 15)", async ({ page }) => {
    // Arrange: Create test user
    const testUser = await createTestUser(page, "owner");

    // Act: Login and navigate to profile/settings
    await loginAsUser(page, testUser.email, testUser.password);
    await page.goto("/owner");

    // Navigate to GDPR data panel (assuming it's in settings or profile)
    // For now, assuming there's a direct route or link
    await page.goto("/settings/gdpr"); // Adjust route as needed

    // Wait for GDPR panel to load
    await expect(page.getByTestId("gdpr-data-panel")).toBeVisible();

    // Click export button
    await page.getByTestId("gdpr-export-button").click();

    // Wait for export modal
    await expect(page.getByTestId("gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });

    // Verify export data is displayed
    await expect(
      page.getByTestId("gdpr-export-modal").locator("text=export_date"),
    ).toBeVisible();
    await expect(
      page.getByTestId("gdpr-export-modal").locator("text=user"),
    ).toBeVisible();

    // Download JSON
    const downloadPromise = page.waitForEvent("download");
    await page.getByTestId("gdpr-download-export-button").click();
    const download = await downloadPromise;

    // Verify download
    expect(download.suggestedFilename()).toContain("gdpr-export");
    expect(download.suggestedFilename()).toContain(".json");

    // Close modal
    await page.keyboard.press("Escape");
    await expect(page.getByTestId("gdpr-export-modal")).not.toBeVisible();
  });

  test("should check erasure eligibility before erasure", async ({ page }) => {
    // Arrange: Create test user
    const testUser = await createTestUser(page, "owner");

    // Act: Login and navigate to GDPR panel
    await loginAsUser(page, testUser.email, testUser.password);
    await page.goto("/settings/gdpr");

    await expect(page.getByTestId("gdpr-data-panel")).toBeVisible();

    // Click erase button
    await page.getByTestId("gdpr-erase-button").click();

    // System should check legal holds via /api/v1/gdpr/can-erase
    // Wait for confirmation modal
    await expect(page.getByTestId("gdpr-erase-confirm-modal")).toBeVisible({
      timeout: 10000,
    });

    // Verify modal shows erasure details
    await expect(
      page
        .getByTestId("gdpr-erase-confirm-modal")
        .locator("text=/anonymi[sz]e/i"),
    ).toBeVisible();
  });

  test("should erase user data with confirmation (Article 17)", async ({
    page,
  }) => {
    // Arrange: Create test user
    const testUser = await createTestUser(page, "owner");

    // Act: Login and navigate to GDPR panel
    await loginAsUser(page, testUser.email, testUser.password);
    await page.goto("/settings/gdpr");

    await expect(page.getByTestId("gdpr-data-panel")).toBeVisible();

    // Click erase button
    await page.getByTestId("gdpr-erase-button").click();

    // Wait for confirmation modal
    await expect(page.getByTestId("gdpr-erase-confirm-modal")).toBeVisible({
      timeout: 10000,
    });

    // Confirm erasure
    await page.getByTestId("gdpr-erase-confirm-button").click();

    // Wait for success message (toast or banner)
    await expect(
      page.locator("text=/success|anonymi[sz]ed|erased/i"),
    ).toBeVisible({ timeout: 10000 });

    // User should be logged out automatically after 3 seconds
    await page.waitForURL("/login", { timeout: 10000 });

    // Verify cannot login with old credentials
    await page.getByTestId("login-email").fill(testUser.email);
    await page.getByTestId("login-password").fill(testUser.password);
    await page.getByTestId("login-submit").click();

    // Should show error (anonymized users cannot login)
    await expect(page.getByTestId("login-error")).toBeVisible();
  });

  test("should block erasure if legal holds exist", async ({ page }) => {
    // This test requires backend to have legal holds logic
    // For now, we'll test the UI behavior when can_erase=false

    // Note: To properly test this, we'd need to:
    // 1. Create user with pending legal obligations (unpaid expenses, active contracts)
    // 2. Attempt erasure
    // 3. Verify error message about legal holds

    // This would require API setup, so marking as todo for now
    test.skip();
  });
});

test.describe("GDPR - Admin Operations (SuperAdmin)", () => {
  test("should view admin GDPR management panel", async ({ page }) => {
    // Arrange: Login as SuperAdmin
    await loginAsSuperAdmin(page);

    // Act: Navigate to admin GDPR panel
    await page.goto("/admin/gdpr"); // Adjust route as needed

    // Assert: Panel is visible
    await expect(page.getByTestId("admin-gdpr-panel")).toBeVisible();
    await expect(page.getByTestId("admin-gdpr-title")).toBeVisible();
    await expect(page.getByTestId("admin-gdpr-search")).toBeVisible();
  });

  test("should search and export user data as admin", async ({ page }) => {
    // Arrange: Create test user and login as SuperAdmin
    const testUser = await createTestUser(page, "owner");
    await loginAsSuperAdmin(page);

    // Act: Navigate to admin GDPR panel
    await page.goto("/admin/gdpr");
    await expect(page.getByTestId("admin-gdpr-panel")).toBeVisible();

    // Search for test user
    await page.getByTestId("admin-gdpr-search").fill(testUser.email);
    await page.keyboard.press("Enter");

    // Wait for search results
    await page.waitForTimeout(1000); // Wait for debounce/filter

    // Find user in table and click export
    const userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: testUser.email });
    await expect(userRow).toBeVisible();

    await userRow.getByTestId("admin-gdpr-export-user").click();

    // Wait for export modal
    await expect(page.getByTestId("admin-gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });

    // Verify export data
    await expect(
      page.getByTestId("admin-gdpr-export-modal").locator("text=export_date"),
    ).toBeVisible();

    // Download JSON
    const downloadPromise = page.waitForEvent("download");
    await page.getByTestId("admin-gdpr-download-button").click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toContain("gdpr-export");
  });

  test("should erase user data as admin with confirmation", async ({
    page,
  }) => {
    // Arrange: Create test user and login as SuperAdmin
    const testUser = await createTestUser(page, "owner");
    await loginAsSuperAdmin(page);

    // Act: Navigate to admin GDPR panel
    await page.goto("/admin/gdpr");
    await expect(page.getByTestId("admin-gdpr-panel")).toBeVisible();

    // Search for test user
    await page.getByTestId("admin-gdpr-search").fill(testUser.email);
    await page.keyboard.press("Enter");
    await page.waitForTimeout(1000);

    // Find user and click erase
    const userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: testUser.email });
    await expect(userRow).toBeVisible();

    await userRow.getByTestId("admin-gdpr-erase-user").click();

    // Wait for confirmation modal
    await expect(page.getByTestId("admin-gdpr-erase-modal")).toBeVisible({
      timeout: 10000,
    });

    // Verify modal shows user details
    await expect(
      page.getByTestId("admin-gdpr-erase-modal").locator(`text=${testUser.email}`),
    ).toBeVisible();

    // Confirm erasure
    await page.getByTestId("admin-gdpr-erase-confirm").click();

    // Wait for success banner
    await expect(page.getByTestId("admin-gdpr-erasure-result")).toBeVisible({
      timeout: 10000,
    });

    // Verify success message mentions email notification
    await expect(
      page
        .getByTestId("admin-gdpr-erasure-result")
        .locator("text=/email|notified/i"),
    ).toBeVisible();
  });

  test("should view and filter audit logs", async ({ page }) => {
    // Arrange: Login as SuperAdmin
    await loginAsSuperAdmin(page);

    // Act: Navigate to admin GDPR panel
    await page.goto("/admin/gdpr");
    await expect(page.getByTestId("admin-gdpr-panel")).toBeVisible();

    // Toggle audit logs section
    await page.getByTestId("admin-gdpr-audit-toggle").click();

    // Wait for audit logs to load
    await expect(page.getByTestId("admin-gdpr-audit-logs")).toBeVisible();

    // Verify audit log table has entries
    const auditRows = page.getByTestId("admin-gdpr-audit-log-row");
    await expect(auditRows.first()).toBeVisible({ timeout: 5000 });

    // Verify log entries have required fields
    const firstRow = auditRows.first();
    await expect(firstRow.locator("text=/\\d{1,2}\\/\\d{1,2}\\/\\d{4}/")).toBeVisible(); // Date
    await expect(
      firstRow.locator("text=/Export|Erase|GdprData/i"),
    ).toBeVisible(); // Event type
  });

  test("should paginate through audit logs", async ({ page }) => {
    // Arrange: Login as SuperAdmin
    await loginAsSuperAdmin(page);

    // Act: Navigate to admin GDPR panel and show audit logs
    await page.goto("/admin/gdpr");
    await page.getByTestId("admin-gdpr-audit-toggle").click();
    await expect(page.getByTestId("admin-gdpr-audit-logs")).toBeVisible();

    // Check if pagination controls exist
    const paginationNext = page.locator(
      '[data-testid="admin-gdpr-audit-logs"] button:has-text("Next")',
    );
    const paginationPrev = page.locator(
      '[data-testid="admin-gdpr-audit-logs"] button:has-text("Previous")',
    );

    // If there are multiple pages, test pagination
    if (await paginationNext.isEnabled()) {
      // Get first log ID on page 1
      const firstLogPage1 = await page
        .getByTestId("admin-gdpr-audit-log-row")
        .first()
        .innerText();

      // Go to page 2
      await paginationNext.click();
      await page.waitForTimeout(500);

      // Get first log ID on page 2
      const firstLogPage2 = await page
        .getByTestId("admin-gdpr-audit-log-row")
        .first()
        .innerText();

      // Verify logs are different
      expect(firstLogPage1).not.toEqual(firstLogPage2);

      // Go back to page 1
      await paginationPrev.click();
      await page.waitForTimeout(500);

      // Verify we're back to original logs
      const firstLogBackToPage1 = await page
        .getByTestId("admin-gdpr-audit-log-row")
        .first()
        .innerText();
      expect(firstLogBackToPage1).toEqual(firstLogPage1);
    }
  });

  test("should refresh audit logs manually", async ({ page }) => {
    // Arrange: Login as SuperAdmin
    await loginAsSuperAdmin(page);

    // Act: Navigate to admin GDPR panel
    await page.goto("/admin/gdpr");
    await page.getByTestId("admin-gdpr-audit-toggle").click();
    await expect(page.getByTestId("admin-gdpr-audit-logs")).toBeVisible();

    // Get current log count
    const initialLogs = await page
      .getByTestId("admin-gdpr-audit-log-row")
      .count();

    // Click refresh button
    await page.getByTestId("admin-gdpr-refresh-logs").click();

    // Wait for reload
    await page.waitForTimeout(1000);

    // Verify logs are still visible (refresh worked)
    const refreshedLogs = await page
      .getByTestId("admin-gdpr-audit-log-row")
      .count();
    expect(refreshedLogs).toBeGreaterThanOrEqual(initialLogs);
  });
});

test.describe("GDPR - Cross-Organization Access (SuperAdmin)", () => {
  test("should allow SuperAdmin to export data from any organization", async ({
    page,
  }) => {
    // Arrange: Create user in specific organization
    const testUser = await createTestUser(page, "syndic");
    await loginAsSuperAdmin(page);

    // Act: Export user data (cross-org)
    await page.goto("/admin/gdpr");
    await page.getByTestId("admin-gdpr-search").fill(testUser.email);
    await page.keyboard.press("Enter");
    await page.waitForTimeout(1000);

    const userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: testUser.email });
    await expect(userRow).toBeVisible();

    // Click export
    await userRow.getByTestId("admin-gdpr-export-user").click();

    // Assert: Export modal opens (SuperAdmin has access)
    await expect(page.getByTestId("admin-gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });
  });

  test("should allow SuperAdmin to erase data from any organization", async ({
    page,
  }) => {
    // Arrange: Create user in specific organization
    const testUser = await createTestUser(page, "accountant");
    await loginAsSuperAdmin(page);

    // Act: Erase user data (cross-org)
    await page.goto("/admin/gdpr");
    await page.getByTestId("admin-gdpr-search").fill(testUser.email);
    await page.keyboard.press("Enter");
    await page.waitForTimeout(1000);

    const userRow = page
      .getByTestId("admin-gdpr-user-row")
      .filter({ hasText: testUser.email });
    await expect(userRow).toBeVisible();

    // Click erase
    await userRow.getByTestId("admin-gdpr-erase-user").click();

    // Assert: Erase modal opens (SuperAdmin has access)
    await expect(page.getByTestId("admin-gdpr-erase-modal")).toBeVisible({
      timeout: 10000,
    });
  });
});

test.describe("GDPR - Complete End-to-End Journey", () => {
  test("should complete full GDPR lifecycle: register → export → erase", async ({
    page,
  }) => {
    // Step 1: Register new user
    await page.goto("/register");
    await expect(page.getByTestId("register-form")).toBeVisible();

    const timestamp = Date.now();
    const email = `gdpr-e2e-${timestamp}@example.com`;
    const password = "test123456";

    await page.getByTestId("register-first-name").fill("E2E");
    await page.getByTestId("register-last-name").fill(`Test${timestamp}`);
    await page.getByTestId("register-email").fill(email);
    await page.getByTestId("register-password").fill(password);
    await page.getByTestId("register-confirm-password").fill(password);
    await page.getByTestId("register-role").selectOption("owner");
    await page.getByTestId("register-submit").click();

    // Wait for redirect to owner dashboard
    await page.waitForURL("/owner", { timeout: 10000 });

    // Step 2: Export personal data (Article 15)
    await page.goto("/settings/gdpr");
    await expect(page.getByTestId("gdpr-data-panel")).toBeVisible();

    await page.getByTestId("gdpr-export-button").click();
    await expect(page.getByTestId("gdpr-export-modal")).toBeVisible({
      timeout: 10000,
    });

    // Verify export contains user email
    await expect(
      page.getByTestId("gdpr-export-modal").locator(`text=${email}`),
    ).toBeVisible();

    // Download export
    const downloadPromise = page.waitForEvent("download");
    await page.getByTestId("gdpr-download-export-button").click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain("gdpr-export");

    // Close modal
    await page.keyboard.press("Escape");

    // Step 3: Erase personal data (Article 17)
    await page.getByTestId("gdpr-erase-button").click();
    await expect(page.getByTestId("gdpr-erase-confirm-modal")).toBeVisible({
      timeout: 10000,
    });

    await page.getByTestId("gdpr-erase-confirm-button").click();

    // Wait for success and auto-logout
    await expect(
      page.locator("text=/success|anonymi[sz]ed/i"),
    ).toBeVisible({ timeout: 10000 });
    await page.waitForURL("/login", { timeout: 10000 });

    // Step 4: Verify cannot login anymore
    await page.getByTestId("login-email").fill(email);
    await page.getByTestId("login-password").fill(password);
    await page.getByTestId("login-submit").click();

    await expect(page.getByTestId("login-error")).toBeVisible();

    // Step 5: Verify admin can see audit logs
    await loginAsSuperAdmin(page);
    await page.goto("/admin/gdpr");
    await page.getByTestId("admin-gdpr-audit-toggle").click();
    await expect(page.getByTestId("admin-gdpr-audit-logs")).toBeVisible();

    // Search for logs related to this user's email
    // Note: Logs might show user_id instead of email after anonymization
    const auditRows = page.getByTestId("admin-gdpr-audit-log-row");
    await expect(auditRows.first()).toBeVisible();
  });
});
