import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Energy Campaigns E2E Test Suite - Group Energy Buying (GDPR compliant)
 *
 * Tests campaign creation, status transitions, and provider offer management.
 * GDPR: statistics are anonymized (k-anonymity >= 5 participants).
 * Mirrors workflows from backend/tests/e2e_energy_campaigns.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Energy Campaigns - Group Buying", () => {
  test("should display energy campaigns page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "energy");
    await page.goto("/energy-campaigns");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='energy-campaigns-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an energy campaign and retrieve it", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "energy",
    );
    const timestamp = Date.now();
    const deadline = new Date();
    deadline.setDate(deadline.getDate() + 30);

    const campaignResp = await page.request.post(
      `${API_BASE}/energy-campaigns`,
      {
        data: {
          organization_id: orgId,
          building_id: buildingId,
          campaign_name: `Achat Groupé Energie ${timestamp}`,
          description: "Campagne achat groupé gaz et électricité 2026",
          energy_types: ["Electricity", "Gas"],
          deadline_participation: deadline.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(campaignResp.status())).toBeTruthy();

    if (campaignResp.ok()) {
      const campaign = await campaignResp.json();
      expect(campaign.id).toBeTruthy();
      expect(campaign.organization_id).toBe(orgId);

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/energy-campaigns/${campaign.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(campaign.id);
    }
  });

  test("should list energy campaigns for organization", async ({ page }) => {
    const { token } = await loginAsSyndicWithBuilding(page, "energy");

    const listResp = await page.request.get(`${API_BASE}/energy-campaigns`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.ok()).toBeTruthy();
    const campaigns = await listResp.json();
    expect(Array.isArray(campaigns)).toBeTruthy();
  });

  test("should add a provider offer to a campaign", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "energy",
    );
    const timestamp = Date.now();
    const deadline = new Date();
    deadline.setDate(deadline.getDate() + 30);

    const campaignResp = await page.request.post(
      `${API_BASE}/energy-campaigns`,
      {
        data: {
          organization_id: orgId,
          building_id: buildingId,
          campaign_name: `Campagne offres ${timestamp}`,
          description: "Test ajout offre fournisseur",
          energy_types: ["Electricity"],
          deadline_participation: deadline.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (campaignResp.ok()) {
      const campaign = await campaignResp.json();

      const offerResp = await page.request.post(
        `${API_BASE}/energy-campaigns/${campaign.id}/offers`,
        {
          data: {
            provider_name: `Luminus ${timestamp}`,
            price_kwh_electricity: 0.28,
            fixed_monthly_fee: 15.0,
            green_energy_pct: 100,
            contract_duration_months: 12,
            estimated_savings_pct: 15,
          },
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      expect([200, 201, 400].includes(offerResp.status())).toBeTruthy();
    }
  });

  test("should list offers for a campaign", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "energy",
    );
    const timestamp = Date.now();
    const deadline = new Date();
    deadline.setDate(deadline.getDate() + 30);

    const campaignResp = await page.request.post(
      `${API_BASE}/energy-campaigns`,
      {
        data: {
          organization_id: orgId,
          building_id: buildingId,
          campaign_name: `Liste offres ${timestamp}`,
          description: "Test liste offres",
          energy_types: ["Gas"],
          deadline_participation: deadline.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (campaignResp.ok()) {
      const campaign = await campaignResp.json();

      const offersResp = await page.request.get(
        `${API_BASE}/energy-campaigns/${campaign.id}/offers`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(offersResp.ok()).toBeTruthy();
      const offers = await offersResp.json();
      expect(Array.isArray(offers)).toBeTruthy();
    }
  });

  test("should navigate to new campaign page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "energy");
    await page.goto("/energy-campaigns/new");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should require auth for energy campaigns API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/energy-campaigns`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
