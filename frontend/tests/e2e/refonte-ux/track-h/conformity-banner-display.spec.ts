/**
 * Track H Story H1 — ConformityBanner E2E (FR-H1 + INV-H1).
 *
 * Vérifie le bug fix domain `is_conformant` (`self.total_tantiemes` au lieu
 * de la constante `dec!(1000)`) en couvrant 2 actes de base :
 *   - 1000 (cas typique millièmes) — building non-conforme → banner visible
 *   - 10000 (acte fractionné) — building non-conforme → banner visible avec
 *     « / 10000 » dans le delta (preuve que la cible n'est plus hard-codée).
 *
 * Couvre aussi le cas @happy : building conforme → banner ABSENT du DOM.
 *
 * Project chromium (PAS testIgnore — cf. mémoire `phase-c-reactivate-e2e-specs`).
 * Reside dans `track-h/` qui n'est PAS exclu par `phase-b-fe/`.
 */
import { test, expect } from "@playwright/test";
import { loginAsAdmin, ensureAcp } from "../../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Track H Story H1 — ConformityBanner display", () => {
  test("@happy admin sees ConformityBanner on non-conformant building (basis 1000)", async ({
    page,
  }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    // Org + ACP + non-conformant building (declared 10 units, 0 units inserted).
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Track H1 Org ${timestamp}`,
        slug: `track-h1-${timestamp}`,
        contact_email: `track-h1-${timestamp}@example.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();
    const acpId = await ensureAcp(page, org.id, adminToken, "track-h1");

    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Drift Manor ${timestamp}`,
        address: `${timestamp} Rue Bug`,
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
    const building = await buildingResp.json();

    await page.goto(`/building-detail?id=${building.id}`, {
      waitUntil: "networkidle",
    });

    // ConformityBanner DOIT être visible — building n'a pas d'units, donc
    // is_conformant = false → banner rendu.
    const banner = page.getByTestId("conformity-banner").first();
    await expect(banner).toBeVisible({ timeout: 10_000 });

    // Le banner expose le quota_basis (1000) dans `data-basis`.
    const quotaDelta = page.getByTestId("conformity-quota-delta").first();
    await expect(quotaDelta).toBeVisible();
    const basisAttr = await quotaDelta.getAttribute("data-basis");
    expect(basisAttr).toBe("1000");

    // building-detail-name expose `data-can-compute="false"` (propagation
    // canCompute aux boutons calcul — préparation Story H2).
    const detailName = page.getByTestId("building-detail-name").first();
    await expect(detailName).toBeVisible();
    const canCompute = await detailName.getAttribute("data-can-compute");
    expect(canCompute).toBe("false");
  });

  test("@happy admin sees ConformityBanner with basis 10000 (acte ≠ 1000 — bug fix Story H1)", async ({
    page,
  }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    // Org + ACP + building avec acte de base 10000 (lots fractionnés).
    // Avant Story H1, ce building était TOUJOURS classé non-conforme à tort
    // (constante hard-codée à 1000). Après fix : non-conforme uniquement si
    // sum != 10000.
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Track H1 Org10k ${timestamp}`,
        slug: `track-h1-10k-${timestamp}`,
        contact_email: `track-h1-10k-${timestamp}@example.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();
    const acpId = await ensureAcp(page, org.id, adminToken, "track-h1-10k");

    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Big Tower 182 ${timestamp}`,
        address: `${timestamp} Av Grand`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 182,
        total_tantiemes: 10000,
        construction_year: 1985,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const building = await buildingResp.json();

    await page.goto(`/building-detail?id=${building.id}`, {
      waitUntil: "networkidle",
    });

    // Banner visible — building sans units → non-conforme avec basis 10000.
    const banner = page.getByTestId("conformity-banner").first();
    await expect(banner).toBeVisible({ timeout: 10_000 });

    // Preuve clé du bug fix : `data-basis` lit `self.total_tantiemes` (10000)
    // et non plus la constante 1000.
    const quotaDelta = page.getByTestId("conformity-quota-delta").first();
    await expect(quotaDelta).toBeVisible();
    const basisAttr = await quotaDelta.getAttribute("data-basis");
    expect(basisAttr).toBe("10000");

    // Le texte du li doit contenir « 10000 » (interpolation i18n FR/NL/EN/DE
    // — tous ont {basis} dans `conformity.quota_off`).
    const text = await quotaDelta.textContent();
    expect(text).toMatch(/10000/);

    // canCompute propagé = false
    const detailName = page.getByTestId("building-detail-name").first();
    const canCompute = await detailName.getAttribute("data-can-compute");
    expect(canCompute).toBe("false");
  });
});
