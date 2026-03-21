import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupAccountant(page: Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "journal");
  return {
    token: ctx.token,
    buildingId: ctx.buildingId,
    orgId: ctx.orgId,
  };
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
