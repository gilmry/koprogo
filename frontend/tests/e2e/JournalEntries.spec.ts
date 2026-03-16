import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupAccountant(page: Page): Promise<{
  token: string;
  buildingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `journal-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminToken = (await adminLoginResp.json()).token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Journal Test Org ${timestamp}`,
      slug: `journal-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Journal",
      last_name: `Test${timestamp}`,
      role: "superadmin",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Journal Building ${timestamp}`,
      address: `${timestamp} Rue Comptabilité`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 6,
      construction_year: 2010,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return { token, buildingId: building.id, orgId: org.id };
}

test.describe("Journal Entries - Double-Entry Accounting", () => {
  test("should display journal entries page", async ({ page }) => {
    await setupAccountant(page);
    await page.goto("/journal-entries");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='journal-entries']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a balanced journal entry via API", async ({ page }) => {
    const { token, buildingId, orgId } = await setupAccountant(page);
    const timestamp = Date.now();

    const entryResp = await page.request.post(`${API_BASE}/journal-entries`, {
      data: {
        organization_id: orgId,
        building_id: buildingId,
        journal_type: "ODS",
        entry_date: new Date().toISOString(),
        description: `Écriture test ${timestamp}`,
        lines: [
          {
            account_code: "6120",
            debit: 100.0,
            credit: 0.0,
            description: "Débit frais entretien",
          },
          {
            account_code: "5500",
            debit: 0.0,
            credit: 100.0,
            description: "Crédit caisse",
          },
        ],
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(entryResp.status())).toBeTruthy();
  });

  test("should reject unbalanced journal entry", async ({ page }) => {
    const { token, buildingId, orgId } = await setupAccountant(page);

    const entryResp = await page.request.post(`${API_BASE}/journal-entries`, {
      data: {
        organization_id: orgId,
        building_id: buildingId,
        journal_type: "ODS",
        entry_date: new Date().toISOString(),
        description: "Écriture déséquilibrée",
        lines: [
          { account_code: "6120", debit: 100.0, credit: 0.0 },
          { account_code: "5500", debit: 0.0, credit: 50.0 },
        ],
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([400, 422].includes(entryResp.status())).toBeTruthy();
  });

  test("should list journal entries via API", async ({ page }) => {
    const { token } = await setupAccountant(page);

    const listResp = await page.request.get(`${API_BASE}/journal-entries`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.ok()).toBeTruthy();
  });
});
