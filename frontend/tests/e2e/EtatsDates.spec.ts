import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { loginAsSyndicWithUnit } from "./helpers/auth";

/**
 * Etats Dates E2E Test Suite - Belgian Property Sales Document
 *
 * Tests etat date creation, status transitions, and reference number lookup.
 * Belgian legal requirement: Syndic must provide etat daté within 10 working days.
 * Mirrors workflows from backend/tests/e2e_etat_date.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupWithUnitAndOwner(page: Page) {
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

  test("should create an etat date and retrieve it", async ({ page }) => {
    const { token, buildingId, unitId, orgId } =
      await setupWithUnitAndOwner(page);
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

    if (etatResp.ok()) {
      const etat = await etatResp.json();
      expect(etat.id).toBeTruthy();
      expect(etat.unit_id).toBe(unitId);
      expect(etat.status).toBe("Requested");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/etats-dates/${etat.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(etat.id);
    }
  });

  test("should list etats-dates for building", async ({ page }) => {
    const { token, buildingId } = await setupWithUnitAndOwner(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/etats-dates`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const etats = await listResp.json();
    expect(Array.isArray(etats)).toBeTruthy();
  });

  test("should mark etat date as in-progress (Requested → InProgress)", async ({
    page,
  }) => {
    const { token, buildingId, unitId, orgId } =
      await setupWithUnitAndOwner(page);
    const timestamp = Date.now();

    const etatResp = await page.request.post(`${API_BASE}/etats-dates`, {
      data: {
        unit_id: unitId,
        building_id: buildingId,
        organization_id: orgId,
        language: "fr",
        notary_name: `Maître Martin ${timestamp}`,
        notary_email: `martin-${timestamp}@example.com`,
        reference_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (etatResp.ok()) {
      const etat = await etatResp.json();

      const progressResp = await page.request.put(
        `${API_BASE}/etats-dates/${etat.id}/mark-in-progress`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(progressResp.status())).toBeTruthy();

      if (progressResp.ok()) {
        const updated = await progressResp.json();
        expect(updated.status).toBe("InProgress");
      }
    }
  });

  test("should list etats-dates for a unit", async ({ page }) => {
    const { token, unitId } = await setupWithUnitAndOwner(page);

    const listResp = await page.request.get(
      `${API_BASE}/units/${unitId}/etats-dates`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const etats = await listResp.json();
    expect(Array.isArray(etats)).toBeTruthy();
  });

  test("should get overdue etats-dates", async ({ page }) => {
    const { token } = await setupWithUnitAndOwner(page);

    const overdueResp = await page.request.get(
      `${API_BASE}/etats-dates/overdue`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 404].includes(overdueResp.status())).toBeTruthy();
  });

  test("should require auth for etats-dates API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/etats-dates`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
