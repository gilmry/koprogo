import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { loginAsSyndicWithUnit } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithUnit(page: Page) {
  const ctx = await loginAsSyndicWithUnit(page, "etat");
  const timestamp = Date.now();

  // Create an owner and assign to unit (required for etat date)
  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: ctx.orgId,
      first_name: "Etat",
      last_name: `Owner${timestamp}`,
      email: `etat-owner-${timestamp}@test.com`,
      address: "1 Rue Test",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });
  const owner = await ownerResp.json();

  await page.request.post(`${API_BASE}/units/${ctx.unitId}/owners`, {
    data: {
      owner_id: owner.id,
      ownership_percentage: 1.0,
      is_primary_contact: true,
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });

  return {
    token: ctx.token,
    buildingId: ctx.buildingId,
    unitId: ctx.unitId,
    orgId: ctx.orgId,
  };
}

test.describe("Etats Dates - Belgian Property Sales Document", () => {
  test("should display etats-dates page", async ({ page }) => {
    await loginAsSyndicWithUnit(page, "etat");
    await page.goto("/etats-dates");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='etats-dates-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an etat date via API", async ({ page }) => {
    const { token, buildingId, unitId, orgId } =
      await setupSyndicWithUnit(page);
    const timestamp = Date.now();

    const etatResp = await page.request.post(`${API_BASE}/etats-dates`, {
      data: {
        unit_id: unitId,
        building_id: buildingId,
        organization_id: orgId,
        language: "fr",
        notary_name: `Maître Dupont ${timestamp}`,
        notary_email: `notaire-${timestamp}@example.com`,
        reference_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(etatResp.status())).toBeTruthy();
    const etat = await etatResp.json();
    expect(etat.unit_id).toBe(unitId);
  });

  test("should list etats-dates for building", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithUnit(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/etats-dates`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const etats = await listResp.json();
    expect(Array.isArray(etats)).toBeTruthy();
  });

  test("should require auth for etats-dates", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/etats-dates`);
    expect(resp.status()).toBe(401);
  });
});
