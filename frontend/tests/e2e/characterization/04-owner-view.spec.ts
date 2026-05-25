/**
 * Characterization Spec 04 — Owner view of his units and votes
 *
 * GOAL : Geler la vue owner de ses lots + ses pages portail (units, expenses, profile, tickets, payments).
 *
 * STATUT : Caractérisation (NON TDD red-first). Doit être GREEN sur HEAD pré-refonte.
 *
 * SOURCE : docs/maury/refonte-ux-multi-role-acp/stories.md §2 Story 0.1
 */
import { test, expect } from "@playwright/test";
import { setupContainerApiUrl } from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

interface OwnerLoginCtx {
  ownerEmail: string;
  ownerToken: string;
  orgId: string;
  buildingId: string;
}

async function registerOwnerWithBuilding(
  page: import("@playwright/test").Page,
  prefix: string,
): Promise<OwnerLoginCtx> {
  const timestamp = Date.now();
  const ownerEmail = `${prefix}-${timestamp}@example.com`;
  const password = "test123456";

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const { token: adminToken } = await adminLoginResp.json();

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `${prefix} Org ${timestamp}`,
      slug: `${prefix}-${timestamp}`,
      contact_email: ownerEmail,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `${prefix} Building ${timestamp}`,
      address: `${timestamp} Rue Owner`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 5,
      construction_year: 2018,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email: ownerEmail,
      password,
      first_name: "Owner",
      last_name: `View${timestamp}`,
      role: "owner",
      organization_id: org.id,
    },
  });
  const { token: ownerToken } = await regResp.json();

  // Ensure __ENV__.API_URL is injected for container mode before any UI nav
  await setupContainerApiUrl(page);

  // Login UI
  await page.goto("/login");
  await page.getByTestId("login-email").fill(ownerEmail);
  await page.getByTestId("login-password").fill(password);
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(owner|syndic|admin|accountant)/, {
    timeout: 15000,
  });

  return {
    ownerEmail,
    ownerToken,
    orgId: org.id,
    buildingId: building.id,
  };
}

test.describe("Characterization 04 — Owner view", () => {
  test("owner dashboard renders", async ({ page }) => {
    await registerOwnerWithBuilding(page, "char-owner-dash");
    await page.goto("/owner");
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='owner-dashboard']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("owner units page renders (his lots)", async ({ page }) => {
    await registerOwnerWithBuilding(page, "char-owner-units");
    await page.goto("/owner/units");
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='owner-units']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("owner expenses page renders", async ({ page }) => {
    await registerOwnerWithBuilding(page, "char-owner-exp");
    await page.goto("/owner/expenses");
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='owner-expenses']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("owner payments page renders", async ({ page }) => {
    await registerOwnerWithBuilding(page, "char-owner-pay");
    await page.goto("/owner/payments");
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='owner-payments']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("owner profile page renders", async ({ page }) => {
    await registerOwnerWithBuilding(page, "char-owner-prof");
    await page.goto("/owner/profile");
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='owner-profile']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("owner tickets page renders", async ({ page }) => {
    await registerOwnerWithBuilding(page, "char-owner-tix");
    await page.goto("/owner/tickets");
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='owner-tickets']").first(),
    ).toBeVisible({ timeout: 10000 });
  });
});
