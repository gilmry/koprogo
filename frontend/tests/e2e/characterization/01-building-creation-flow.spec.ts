/**
 * Characterization Spec 01 — Building Creation Flow
 *
 * GOAL : Geler le flow "admin crée building -> assignation organization -> visible côté syndic".
 *
 * STATUT : Caractérisation (NON TDD red-first). Doit être GREEN sur HEAD pré-refonte.
 *
 * SOURCE : docs/maury/refonte-ux-multi-role-acp/stories.md §2 Story 0.1 (@edge)
 */
import { test, expect } from "@playwright/test";
import { setupContainerApiUrl } from "../helpers/video-pace";
import { ensureAcp } from "../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Characterization 01 — Building Creation Flow", () => {
  test.beforeEach(async ({ page }) => {
    await setupContainerApiUrl(page);
  });

  test("admin creates building assigned to org → syndic of that org sees it", async ({
    page,
  }) => {
    const timestamp = Date.now();
    const syndicEmail = `char-bld-syndic-${timestamp}@example.com`;
    const syndicPwd = "test123456";

    // 1) Admin login (API)
    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    expect(adminLoginResp.ok()).toBeTruthy();
    const { token: adminToken } = await adminLoginResp.json();

    // 2) Admin creates organization
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Char Bld Org ${timestamp}`,
        slug: `char-bld-${timestamp}`,
        contact_email: syndicEmail,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(orgResp.status()).toBe(201);
    const org = await orgResp.json();

    // 3) Admin creates building assigned to org (via ACP post-#602)
    const acpId = await ensureAcp(page, org.id, adminToken, "char-bld");
    const buildingName = `Char Building ${timestamp}`;
    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: `${timestamp} Rue Caractérisation`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 10,
        construction_year: 2020,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(buildingResp.ok()).toBeTruthy();
    const building = await buildingResp.json();
    expect(building.id).toBeTruthy();
    expect(building.acp_id).toBe(acpId);

    // 4) Register syndic for that organization
    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: syndicEmail,
        password: syndicPwd,
        first_name: "Bld",
        last_name: `Syndic${timestamp}`,
        role: "syndic",
        organization_id: org.id,
      },
    });
    expect(regResp.ok()).toBeTruthy();
    const { token: syndicToken } = await regResp.json();

    // 5) Syndic login via UI
    await page.goto("/login");
    await page.getByTestId("login-email").fill(syndicEmail);
    await page.getByTestId("login-password").fill(syndicPwd);
    await page.getByTestId("login-submit").click();
    await page.waitForURL(/\/(syndic|admin|owner|accountant)/, {
      timeout: 15000,
    });

    // 6) Syndic GET /buildings (API) — building visible
    const apiListResp = await page.request.get(`${API_BASE}/buildings`, {
      headers: { Authorization: `Bearer ${syndicToken}` },
    });
    expect(apiListResp.ok()).toBeTruthy();
    const apiBody = await apiListResp.json();
    // API peut renvoyer soit array soit {data: [...], pagination}
    const list: Array<{ id: string }> = Array.isArray(apiBody)
      ? apiBody
      : apiBody.data || [];
    const found = list.some((b) => b.id === building.id);
    expect(
      found,
      `Building ${building.id} should be visible to syndic of org ${org.id} via API`,
    ).toBeTruthy();

    // 7) Syndic navigates to /buildings — UI affiche la page sans crash
    // (le text=<name> sur la liste est observé flaky sur HEAD — cf Buildings.spec.ts:30,
    //  rouge en local. On caractérise ici seulement le chargement de la page.)
    await page.goto("/buildings");
    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("main").first()).toBeVisible({ timeout: 10000 });
  });

  test("admin GET /buildings/<id> retrieves the building (API characterization)", async ({
    page,
  }) => {
    // Caractérise le contrat API building retrieve (pas la page detail UI car
    // /building-detail?id=... est observé flaky sur HEAD : redirect vers /login
    // sans le query param — comportement existant Buildings.spec.ts:62 aussi
    // rouge sur HEAD. On fige ici l'API contract, pas le UI page detail.
    const timestamp = Date.now();
    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const { token: adminToken } = await adminLoginResp.json();

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Char Bld Det Org ${timestamp}`,
        slug: `char-bld-det-${timestamp}`,
        contact_email: `char-bld-det-${timestamp}@example.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    const acpId = await ensureAcp(page, org.id, adminToken, "char-bld-det");
    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Char Detail Building ${timestamp}`,
        address: `${timestamp} Rue Detail`,
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 5,
        construction_year: 2018,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(buildingResp.ok()).toBeTruthy();
    const building = await buildingResp.json();

    // GET /buildings/<id>
    const getResp = await page.request.get(
      `${API_BASE}/buildings/${building.id}`,
      { headers: { Authorization: `Bearer ${adminToken}` } },
    );
    expect(getResp.ok()).toBeTruthy();
    const fetched = await getResp.json();
    expect(fetched.id).toBe(building.id);
    expect(fetched.name).toBe(`Char Detail Building ${timestamp}`);
  });
});
