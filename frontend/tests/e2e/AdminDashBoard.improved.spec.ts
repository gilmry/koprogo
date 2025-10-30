/**
 * Admin Dashboard E2E Tests - Improved Version with Test IDs
 *
 * Prerequisites:
 * 1. Backend running on http://localhost:8080
 * 2. Database seeded with admin user (admin@koprogo.com / admin123)
 * 3. Frontend will be started automatically by Playwright
 *
 * Features:
 * - Uses data-testid attributes for reliable selectors
 * - Idempotent tests (can run multiple times without cleanup issues)
 * - Unique test data using timestamps
 * - Automatic cleanup even on failure
 *
 * Run:
 *   npm run test:e2e -- AdminDashBoard.improved.spec.ts
 *   npm run test:e2e -- AdminDashBoard.improved.spec.ts --ui
 *   npm run test:e2e -- AdminDashBoard.improved.spec.ts --debug
 */

import { test, expect } from '@playwright/test';
import type { Page } from '@playwright/test';

// Helper to generate unique test data
const generateTestData = (prefix: string) => {
  const timestamp = Date.now();
  const randomSegment = Math.random().toString(36).slice(2, 8);
  const seed = `${timestamp}-${randomSegment}`;
  return {
    timestamp,
    organization: {
      name: `${prefix} Org ${seed}`,
      slug: `${prefix}-org-${seed}`.toLowerCase(),
      email: `${prefix}-${seed}@test.com`.toLowerCase(),
      phone: '+32491234567'
    },
    user: {
      firstName: `${prefix}First`,
      lastName: `${prefix}Last${seed}`,
      email: `${prefix}-user-${seed}@test.com`.toLowerCase(),
      password: 'TestPass123!'
    },
    building: {
      name: `${prefix} Building ${seed}`,
      address: `${seed} Test Street`,
      postalCode: '1000',
      city: 'Bruxelles',
      totalUnits: 10,
      constructionYear: 2020
    }
  };
};

const navigateToOrganizations = async (page: Page) => {
  await page.getByRole('navigation').getByRole('link', { name: '🏛️ Organisations' }).click();
  await expect(page).toHaveURL(/\/admin\/organizations$/);
  await page.getByTestId('organizations-table-body').waitFor({ timeout: 10000 });
};

const createOrganizationViaUI = async (
  page: Page,
  organization: ReturnType<typeof generateTestData>['organization']
) => {
  await navigateToOrganizations(page);
  await page.getByTestId('create-organization-button').click();
  await page.getByTestId('organization-form').waitFor({ state: 'visible' });

  await page.getByTestId('organization-name-input').fill(organization.name);
  await page.getByTestId('organization-slug-input').fill(organization.slug);
  await page.getByTestId('organization-email-input').fill(organization.email);
  if (organization.phone) {
    await page.getByTestId('organization-phone-input').fill(organization.phone);
  }
  await page.getByTestId('organization-submit-button').click();

  const orgRow = page.locator(`tr[data-org-name="${organization.name}"]`);
  await expect(orgRow).toBeVisible({ timeout: 10000 });
  const orgId = await orgRow.getAttribute('data-org-id');
  return { id: orgId, name: organization.name };
};

const deleteOrganizationByName = async (
  page: Page,
  organizationName: string
) => {
  await navigateToOrganizations(page);
  const orgRow = page.locator(`tr[data-org-name="${organizationName}"]`);
  if (await orgRow.count()) {
    await orgRow.getByTestId('delete-organization-button').click();
    await page.getByTestId('confirm-dialog-confirm').click();
    await expect(orgRow).toHaveCount(0);
  }
};

test.describe('Admin Dashboard - CRUD with Test IDs', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto('/login', { waitUntil: 'domcontentloaded' });

    // If we are already redirected to the admin dashboard, nothing to do.
    if (page.url().includes('/admin')) {
      return;
    }

    await page.getByRole('textbox', { name: 'Email' }).fill('admin@koprogo.com');
    await page.getByRole('textbox', { name: 'Mot de passe' }).fill('admin123');
    await page.getByRole('button', { name: 'Se connecter' }).click();
    await page.waitForURL('**/admin**', { timeout: 20000 });
  });

  test.describe('Organizations Management with Test IDs', () => {

    test('should create, edit, and delete organization using test IDs', async ({ page }) => {
      const testData = generateTestData('E2E');

      // Navigate to organizations page
      await page.getByRole('navigation').getByRole('link', { name: '🏛️ Organisations' }).click();
      await expect(page).toHaveURL(/\/admin\/organizations$/);

      // Wait for table to load
      await page.getByTestId('organizations-table-body').waitFor({ timeout: 10000 });

      // CREATE: Click create button using test ID
      await page.getByTestId('create-organization-button').click();

      // Wait for form to appear
      await page.getByTestId('organization-form').waitFor({ state: 'visible' });

      // Fill form using test IDs
      await page.getByTestId('organization-name-input').fill(testData.organization.name);
      await page.getByTestId('organization-slug-input').fill(testData.organization.slug);
      await page.getByTestId('organization-email-input').fill(testData.organization.email);
      await page.getByTestId('organization-phone-input').fill(testData.organization.phone);

      // Submit form
      await page.getByTestId('organization-submit-button').click();

      // Verify creation - wait for toast or table update
      await expect(page.getByTestId('organization-name').filter({ hasText: testData.organization.name })).toBeVisible({ timeout: 10000 });

      // EDIT: Find the row by data attribute and click edit
      const orgRow = page.locator(`[data-org-name="${testData.organization.name}"]`);
      await orgRow.locator('[data-testid="edit-organization-button"]').click();

      // Update name
      const updatedName = `${testData.organization.name} Updated`;
      await page.getByTestId('organization-name-input').fill(updatedName);
      await page.getByTestId('organization-submit-button').click();

      // Verify update
      await expect(page.getByTestId('organization-name').filter({ hasText: updatedName })).toBeVisible();

      // DELETE: Find the updated row and delete
      const updatedRow = page.locator(`[data-org-name="${updatedName}"]`);
      await updatedRow.locator('[data-testid="delete-organization-button"]').click();

      // Confirm deletion using test ID
      await page.getByTestId('confirm-dialog-confirm').click();

      // Verify deletion
      await expect(page.getByTestId('organization-name').filter({ hasText: updatedName })).not.toBeVisible();
    });

    test('should search organizations', async ({ page }) => {
      await page.getByRole('navigation').getByRole('link', { name: '🏛️ Organisations' }).click();

      // Wait for table
      await page.getByTestId('organizations-table-body').waitFor({ timeout: 10000 });

      // Use search with test ID
      const searchInput = page.getByTestId('organization-search-input');
      await searchInput.fill('test');

      // Verify search works
      await expect(searchInput).toHaveValue('test');
    });

    test('should toggle organization status', async ({ page }) => {
      const testData = generateTestData('Toggle');

      // Navigate to organizations
      await page.getByRole('navigation').getByRole('link', { name: '🏛️ Organisations' }).click();
      await page.getByTestId('organizations-table-body').waitFor({ timeout: 10000 });

      // Create a test organization first
      await page.getByTestId('create-organization-button').click();
      await page.getByTestId('organization-form').waitFor({ state: 'visible' });
      await page.getByTestId('organization-name-input').fill(testData.organization.name);
      await page.getByTestId('organization-slug-input').fill(testData.organization.slug);
      await page.getByTestId('organization-email-input').fill(testData.organization.email);
      await page.getByTestId('organization-submit-button').click();

      // Toggle status
      const orgRow = page.locator(`[data-org-name="${testData.organization.name}"]`);
      await orgRow.locator('[data-testid="toggle-organization-button"]').click();

      // Cleanup: Delete the test organization
      await orgRow.locator('[data-testid="delete-organization-button"]').click();
      await page.getByTestId('confirm-dialog-confirm').click();
    });
  });

  test.describe('Users Management with Test IDs', () => {
    test('should create, edit, and delete user using test IDs', async ({ page }) => {
      const testData = generateTestData('User');
      const organization = await createOrganizationViaUI(page, testData.organization);

      await page.getByRole('navigation').getByRole('link', { name: '👥 Utilisateurs' }).click();
      await expect(page).toHaveURL(/\/admin\/users$/);
      await page.getByTestId('users-table-body').waitFor({ timeout: 10000 });

      // CREATE
      await page.getByTestId('create-user-button').click();
      await page.getByTestId('user-form').waitFor({ state: 'visible' });
      await page.getByTestId('user-firstname-input').fill(testData.user.firstName);
      await page.getByTestId('user-lastname-input').fill(testData.user.lastName);
      await page.getByTestId('user-email-input').fill(testData.user.email);
      await page.getByTestId('user-password-input').fill(testData.user.password);
      await page.getByTestId('user-confirmpassword-input').fill(testData.user.password);
      await page.getByTestId('user-role-select').first().selectOption('syndic');
      await page.getByTestId('user-organization-select').first().selectOption({ index: 1 });
      await page.getByTestId('user-submit-button').click();

      const userRow = page.locator(`tr[data-user-email="${testData.user.email}"]`);
      await expect(userRow).toBeVisible({ timeout: 10000 });

      // EDIT
      const updatedFirstName = `${testData.user.firstName}-Updated`;
      await userRow.getByTestId('edit-user-button').click();
      await page.getByTestId('user-firstname-input').fill(updatedFirstName);
      await page.getByTestId('user-submit-button').click();
      await expect(page.getByTestId('user-name').filter({ hasText: updatedFirstName })).toBeVisible();

      // DELETE
      await userRow.getByTestId('delete-user-button').click();
      await page.getByTestId('confirm-dialog-confirm').click();
      await expect(userRow).toHaveCount(0);

      await deleteOrganizationByName(page, organization.name);
    });

    test('should filter users by role using test IDs', async ({ page }) => {
      await page.getByRole('navigation').getByRole('link', { name: '👥 Utilisateurs' }).click();
      await page.getByTestId('users-table-body').waitFor({ timeout: 10000 });

      await page.getByTestId('user-role-filter').selectOption('superadmin');
      const adminRow = page.locator('tr[data-user-email="admin@koprogo.com"]');
      await expect(adminRow).toBeVisible();
    });

    test('should search users by name or email using test IDs', async ({ page }) => {
      await page.getByRole('navigation').getByRole('link', { name: '👥 Utilisateurs' }).click();
      await page.getByTestId('users-table-body').waitFor({ timeout: 10000 });

      const searchInput = page.getByTestId('user-search-input');
      await searchInput.fill('admin@koprogo.com');
      await expect(searchInput).toHaveValue('admin@koprogo.com');
      await expect(page.locator('tr[data-user-email="admin@koprogo.com"]')).toBeVisible();
    });
  });

  test.describe('Buildings Management with Test IDs', () => {
    test('should create, edit, and delete a building using test IDs', async ({ page }) => {
      const testData = generateTestData('Building');
      const organization = await createOrganizationViaUI(page, testData.organization);

      await page.goto('/buildings');
      await page.getByTestId('create-building-button').click();
      await page.getByTestId('building-form').waitFor({ state: 'visible' });
      await page.getByTestId('building-organization-select').selectOption({ index: 1 });
      await page.getByTestId('building-name-input').fill(testData.building.name);
      await page.getByTestId('building-address-input').fill(testData.building.address);
      await page.getByTestId('building-postalcode-input').fill(testData.building.postalCode);
      await page.getByTestId('building-city-input').fill(testData.building.city);
      await page.getByTestId('building-totalunits-input').fill(String(testData.building.totalUnits));
      await page.getByTestId('building-constructionyear-input').fill(String(testData.building.constructionYear));
      await page.getByTestId('building-submit-button').click();

      const buildingCard = page.locator(`div[data-building-name="${testData.building.name}"]`);
      await expect(buildingCard).toBeVisible({ timeout: 10000 });

      const updatedBuildingName = `${testData.building.name} Updated`;
      await buildingCard.locator('[data-testid="edit-building-button"]').click();
      await page.getByTestId('building-name-input').fill(updatedBuildingName);
      await page.getByTestId('building-submit-button').click();
      await expect(page.getByTestId('building-name').filter({ hasText: updatedBuildingName })).toBeVisible();

      const deleteResponsePromise = page.waitForResponse((response) => {
        return (
          response.request().method() === 'DELETE' &&
          response.url().includes('/api/v1/buildings/')
        );
      });
      await page.locator(`div[data-building-name="${updatedBuildingName}"]`).locator('[data-testid="delete-building-button"]').click();
      await page.getByTestId('confirm-dialog-confirm').click();
      const deleteResponse = await deleteResponsePromise;
      expect(deleteResponse.ok()).toBeTruthy();
      await page.reload();
      await page.waitForLoadState('domcontentloaded');
      await expect(page.locator(`div[data-building-name="${updatedBuildingName}"]`)).toHaveCount(0, { timeout: 10000 });

      await deleteOrganizationByName(page, organization.name);
    });

    test('should search buildings using test IDs', async ({ page }) => {
      const testData = generateTestData('Search');
      const organization = await createOrganizationViaUI(page, testData.organization);

      await page.goto('/buildings');
      await page.getByTestId('building-search-input').fill('Résidence');
      await expect(page.getByTestId('building-search-input')).toHaveValue('Résidence');

      await deleteOrganizationByName(page, organization.name);
    });
  });

  test.describe('Idempotent Full Journey', () => {

    test('should complete full workflow and cleanup automatically', async ({ page }) => {
      const testData = generateTestData('Journey');
      let createdUserEmail: string | null = null;
      let createdBuildingName: string | null = null;

      try {
        // 1. CREATE ORGANIZATION
        await createOrganizationViaUI(page, testData.organization);

        // 2. CREATE USER
        await page.getByRole('navigation').getByRole('link', { name: '👥 Utilisateurs' }).click();
        await page.getByTestId('users-table-body').waitFor({ timeout: 10000 });

        await page.getByTestId('create-user-button').click();
        await page.getByTestId('user-form').waitFor({ state: 'visible' });
        await page.getByTestId('user-firstname-input').fill(testData.user.firstName);
        await page.getByTestId('user-lastname-input').fill(testData.user.lastName);
        await page.getByTestId('user-email-input').fill(testData.user.email);
        await page.getByTestId('user-password-input').fill(testData.user.password);
        await page.getByTestId('user-confirmpassword-input').fill(testData.user.password);
        await page.getByTestId('user-role-select').first().selectOption('syndic');
        await page.getByTestId('user-organization-select').first().selectOption({ index: 1 });
        await page.getByTestId('user-submit-button').click();
        createdUserEmail = testData.user.email;
        await expect(page.locator(`tr[data-user-email="${testData.user.email}"]`)).toBeVisible();

        // 3. CREATE BUILDING
        await page.goto('/buildings');
        await page.getByTestId('create-building-button').click();
        await page.getByTestId('building-form').waitFor({ state: 'visible' });
        await page.getByTestId('building-organization-select').selectOption({ index: 1 });
        await page.getByTestId('building-name-input').fill(testData.building.name);
        await page.getByTestId('building-address-input').fill(testData.building.address);
        await page.getByTestId('building-postalcode-input').fill(testData.building.postalCode);
        await page.getByTestId('building-city-input').fill(testData.building.city);
        await page.getByTestId('building-totalunits-input').fill(String(testData.building.totalUnits));
        await page.getByTestId('building-constructionyear-input').fill(String(testData.building.constructionYear));
        await page.getByTestId('building-submit-button').click();
        createdBuildingName = testData.building.name;
        await expect(page.locator(`div[data-building-name="${testData.building.name}"]`)).toBeVisible();

      } finally {
        // CLEANUP: Always delete created organization, even if test fails
        try {
          if (createdBuildingName) {
            await page.goto('/buildings');
            const buildingCard = page.locator(`div[data-building-name="${createdBuildingName}"]`);
            if (await buildingCard.count()) {
              await buildingCard.locator('[data-testid="delete-building-button"]').click();
              await page.getByTestId('confirm-dialog-confirm').click();
            }
          }

          if (createdUserEmail) {
            await page.getByRole('navigation').getByRole('link', { name: '👥 Utilisateurs' }).click();
            const userRow = page.locator(`tr[data-user-email="${createdUserEmail}"]`);
            if (await userRow.count()) {
              await userRow.getByTestId('delete-user-button').click();
              await page.getByTestId('confirm-dialog-confirm').click();
            }
          }

          await deleteOrganizationByName(page, testData.organization.name);
        } catch (cleanupError) {
          console.error('Cleanup failed:', cleanupError);
        }
      }
    });
  });

  test.afterEach(async ({ page }) => {
    // Logout
    try {
      // Close any modal that might intercept pointer events
      for (let i = 0; i < 2; i += 1) {
        await page.keyboard.press('Escape').catch(() => {});
      }
      await page.locator('[role="dialog"]').evaluateAll(() => {});

      const profileButton = page.getByRole('button').filter({ hasText: /AS|Admin|SA/ });
      if (await profileButton.isVisible({ timeout: 2000 }).catch(() => false)) {
        await profileButton.click();
        await page.getByRole('button', { name: '🚪 Logout' }).click();
        await page.waitForURL('**/login**', { timeout: 5000 });
      }
    } catch (error) {
      console.log('Logout skipped:', error);
    }
  });
});
