/**
 * Story 1.4 — Building Conformity E2E (#553 Bugs 1/3/4 + FR11/FR12/FR23).
 *
 * Validation end-to-end :
 * - Admin crée un building avec units → fiche immeuble affiche count réel +
 *   somme quotas réelle + badge conformité (vert/orange/rouge) + delta.
 * - L'attribut `data-testid="building-conformity-badge"` est présent (ADR-0012).
 * - Le badge est NEVER précédé par parseFloat sur quota_sum (ce test vérifie
 *   uniquement l'affichage, le test Vitest couvre le contrat parseFloat).
 *
 * Helpers shared : `loginAsAdmin`, `loginAsSyndicWithBuilding` (cf.
 * `frontend/tests/e2e/helpers/auth.ts`).
 */
import { test, expect } from "@playwright/test";
import {
  loginAsAdmin,
  loginAsSyndicWithBuilding,
  ensureAcp,
} from "../../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Building Conformity (Story 1.4)", () => {
  test("admin sees non-conformant badge (orange/red) on a building with no units", async ({
    page,
  }) => {
    // Admin login + create a building via API (admin path).
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    // Create an org first (admin role).
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Conformity Org ${timestamp}`,
        slug: `conformity-${timestamp}`,
        contact_email: `conformity-${timestamp}@example.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    // Hotfix #602 — buildings.acp_id (FK acps.id) replaced organization_id.
    const acpId = await ensureAcp(page, org.id, adminToken, "conformity");

    // Create the building (declared 50 units, NO units inserted = non-conformant).
    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Empty Tower ${timestamp}`,
        address: `${timestamp} Rue Test`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 50,
        construction_year: 2010,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const building = await buildingResp.json();

    // Navigate to building detail page (admin shares the route with syndic).
    await page.goto(`/building-detail?id=${building.id}`, {
      waitUntil: "networkidle",
    });

    // Badge MUST be present (FR11/FR12 + ADR-0012 data-testid).
    const badge = page.getByTestId("building-conformity-badge").first();
    await expect(badge).toBeVisible({ timeout: 10_000 });

    // Non-conformant: red (delta = -1000 < 0). The badge class includes "red".
    const badgeClass = await badge.getAttribute("class");
    expect(badgeClass).toMatch(/red/);

    // The units_count display must show "0 / 50" (real value, not derived).
    const unitsCount = page.getByTestId("building-units-count").first();
    await expect(unitsCount).toBeVisible();
    const unitsCountText = await unitsCount.textContent();
    expect(unitsCountText).toContain("0");
    expect(unitsCountText).toContain("50");

    // Quota sum must be "0" (Decimal-as-string, never NaN).
    const quotaSum = page.getByTestId("building-quota-sum").first();
    await expect(quotaSum).toBeVisible();
    const quotaSumText = await quotaSum.textContent();
    expect(quotaSumText).not.toContain("NaN");
    expect(quotaSumText).toMatch(/0/);

    // Quota delta must include "-1000" (Decimal strict).
    const quotaDelta = page.getByTestId("building-quota-delta").first();
    await expect(quotaDelta).toBeVisible();
    const quotaDeltaText = await quotaDelta.textContent();
    expect(quotaDeltaText).toMatch(/-1000|−1000/);
  });

  test("syndic sees conformity metrics on building detail (count + quotas reflect reality)", async ({
    page,
  }) => {
    // Helper créé building via admin token, login syndic page → /buildings detail.
    const ctx = await loginAsSyndicWithBuilding(page, "conf-syndic");

    await page.goto(`/building-detail?id=${ctx.buildingId}`, {
      waitUntil: "networkidle",
    });

    // Badge ConformityBadge présent même sans units (non-conformant).
    const badge = page.getByTestId("building-conformity-badge").first();
    await expect(badge).toBeVisible({ timeout: 10_000 });

    // is_conformant = false → la couleur de fond est red OU orange.
    const badgeClass = await badge.getAttribute("class");
    expect(badgeClass).toMatch(/red|orange/);
  });
});
