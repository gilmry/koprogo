import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupAccountant(page: Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "journal");

  // Register an accountant user in the same organization
  const timestamp = Date.now();
  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email: `accountant-${timestamp}@test.com`,
      password: "test123456",
      first_name: "Comptable",
      last_name: `Test${timestamp}`,
      role: "accountant",
      organization_id: ctx.orgId,
    },
  });
  const accountantData = await regResp.json();
  const accountantToken = accountantData.token;

  // Seed PCMN accounts (required for journal entry FK constraints)
  await page.request.post(`${API_BASE}/accounts/seed/belgian-pcmn`, {
    data: { organization_id: ctx.orgId },
    headers: { Authorization: `Bearer ${accountantToken}` },
  });

  return {
    token: ctx.token,
    accountantToken,
    buildingId: ctx.buildingId,
    orgId: ctx.orgId,
  };
}

test.describe("Journal Entries - Double-Entry Accounting", () => {
  test("should display journal entries page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "journal");
    await page.goto("/journal-entries");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='journal-entries']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a balanced journal entry via API", async ({ page }) => {
    const { accountantToken, buildingId, orgId } = await setupAccountant(page);
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
            account_code: "612",
            debit: 100.0,
            credit: 0.0,
            description: "Débit frais entretien",
          },
          {
            account_code: "550",
            debit: 0.0,
            credit: 100.0,
            description: "Crédit caisse",
          },
        ],
      },
      headers: { Authorization: `Bearer ${accountantToken}` },
    });
    expect([200, 201].includes(entryResp.status())).toBeTruthy();
  });

  test("should reject unbalanced journal entry", async ({ page }) => {
    const { accountantToken, buildingId, orgId } = await setupAccountant(page);

    const entryResp = await page.request.post(`${API_BASE}/journal-entries`, {
      data: {
        organization_id: orgId,
        building_id: buildingId,
        journal_type: "ODS",
        entry_date: new Date().toISOString(),
        description: "Écriture déséquilibrée",
        lines: [
          {
            account_code: "612",
            debit: 100.0,
            credit: 0.0,
            description: "Débit",
          },
          {
            account_code: "550",
            debit: 0.0,
            credit: 50.0,
            description: "Crédit insuffisant",
          },
        ],
      },
      headers: { Authorization: `Bearer ${accountantToken}` },
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
