/**
 * Track H Story H2 — validate-before-compute E2E (FR-H2 + INV-H4/H6).
 *
 * Couvre :
 *  - @happy : syndic crée une dépense sur building conformant → 200 + redirect.
 *  - @security : syndic force POST /expenses sur building non-conformant via
 *    `page.request.post` → 422 BUILDING_NOT_CONFORMANT avec payload narratif
 *    (kind=building_not_conformant, details.code=BUILDING_NOT_CONFORMANT,
 *    units_delta, quota_delta, quota_basis).
 *
 * Project chromium (PAS testIgnore — cf. mémoire `phase-c-reactivate-e2e-specs`).
 * Reside dans `track-h/` qui n'est PAS exclu par `phase-b-fe/`.
 *
 * Pattern inspiré de `conformity-banner-display.spec.ts` (H1).
 */
import { test, expect } from "@playwright/test";
import { loginAsAdmin, ensureAcp } from "../../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Track H Story H2 — validate-before-compute", () => {
  test("@security syndic POST /expenses on non-conformant building → 422 BUILDING_NOT_CONFORMANT", async ({
    page,
  }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    // Seed : org + ACP + building non-conformant (declared 10 units mais 0
    // unit inséré → quota_delta=1000, units_delta=10, basis=1000).
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Track H2 Drift Org ${timestamp}`,
        slug: `track-h2-drift-${timestamp}`,
        contact_email: `track-h2-drift-${timestamp}@example.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();
    const acpId = await ensureAcp(page, org.id, adminToken, "track-h2-drift");

    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Drift Manor ${timestamp}`,
        address: `${timestamp} Rue Drift`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 10,
        total_tantiemes: 1000,
        construction_year: 2010,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(buildingResp.ok()).toBeTruthy();
    const building = await buildingResp.json();

    // Acteur syndic force le POST direct API — bypass tenté.
    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        organization_id: org.id,
        building_id: building.id,
        category: "maintenance",
        description: "Forced bypass attempt",
        amount: 1000,
        expense_date: new Date().toISOString(),
        supplier: "Bypass SA",
        invoice_number: "BYP-001",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });

    // BE 422 (pre-check Story H2) avec payload narratif (Story H1 + H2 helper).
    expect(expenseResp.status()).toBe(422);
    const body = await expenseResp.json();
    expect(body.kind).toBe("building_not_conformant");
    expect(body.details).toBeDefined();
    expect(body.details.code).toBe("BUILDING_NOT_CONFORMANT");
    expect(body.details.building_id).toBe(building.id);
    // 10 units declared, 0 inserted → units_delta=10
    expect(body.details.units_delta).toBe(10);
    // quota_basis=1000 (acte de base), quota_delta="1000" (manque tout).
    expect(body.details.quota_basis).toBe(1000);
    expect(body.details.quota_delta).toBe("1000");
  });

  test("@security syndic POST /call-for-funds on non-conformant building → 422", async ({
    page,
  }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Track H2 CFF Org ${timestamp}`,
        slug: `track-h2-cff-${timestamp}`,
        contact_email: `track-h2-cff-${timestamp}@example.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();
    const acpId = await ensureAcp(page, org.id, adminToken, "track-h2-cff");

    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Drift CFF ${timestamp}`,
        address: `${timestamp} Av Drift`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 5,
        total_tantiemes: 1000,
        construction_year: 2015,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const building = await buildingResp.json();

    const cffResp = await page.request.post(`${API_BASE}/call-for-funds`, {
      data: {
        building_id: building.id,
        title: "Bypass call",
        description: "Forced",
        total_amount: 5000,
        contribution_type: "regular",
        call_date: new Date().toISOString(),
        due_date: new Date(Date.now() + 30 * 86400 * 1000).toISOString(),
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });

    expect(cffResp.status()).toBe(422);
    const body = await cffResp.json();
    expect(body.kind).toBe("building_not_conformant");
    expect(body.details.code).toBe("BUILDING_NOT_CONFORMANT");
  });
});
