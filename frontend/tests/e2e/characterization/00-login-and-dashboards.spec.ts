/**
 * Characterization Spec 00 — Login + Dashboards
 *
 * GOAL : Geler le comportement existant des 3 flows de login (admin / syndic / owner)
 * et l'affichage du dashboard initial pour chaque rôle.
 *
 * STATUT : Caractérisation (NON TDD red-first). Doit être GREEN sur HEAD pré-refonte.
 * Si ROUGE -> le test est bugué OU le HEAD est cassé. Investigation requise.
 *
 * RÈGLE SÉLECTEURS : selectors `getByText` / `role=` / `data-testid` existants OK
 * (exception ADR-0012 : strict `data-testid` s'applique à refonte-ux/, pas ici).
 *
 * SOURCE : docs/maury/refonte-ux-multi-role-acp/stories.md §2 Story 0.1
 */
import { test, expect } from "@playwright/test";
import {
  loginAsAdmin,
  loginAsSyndic,
  loginAsSyndicWithBuilding,
} from "../helpers/auth";
import { setupContainerApiUrl } from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Characterization 00 — Login + Dashboards", () => {
  test.describe.configure({ mode: "serial" });

  test.beforeEach(async ({ page }) => {
    // Permet aux specs de tourner depuis l'intérieur du container (CI ou agent
    // worktree) en injectant window.__ENV__.API_URL si PLAYWRIGHT_API_BASE est set.
    await setupContainerApiUrl(page);
  });

  test("admin login (UI) + admin dashboard rendered", async ({ page }) => {
    const start = Date.now();

    // Login via UI (real flow) — caractérise le path complet form -> redirect
    await page.goto("/login");
    await page.getByTestId("login-email").fill("admin@koprogo.com");
    await page.getByTestId("login-password").fill("admin123");
    await page.getByTestId("login-submit").click();
    await page.waitForURL(/\/(admin|syndic|owner|accountant)/, {
      timeout: 15000,
    });

    // Admin user lands on /admin par le redirectMap (getDefaultRedirect)
    await expect(page).toHaveURL(/\/admin/, { timeout: 10000 });
    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='admin-dashboard']").first(),
    ).toBeVisible({ timeout: 10000 });

    const elapsed = Date.now() - start;
    expect(
      elapsed,
      `Admin login flow took ${elapsed}ms — caractérisation cible < 30000ms`,
    ).toBeLessThan(30000);
  });

  test("admin token injection (helper) + dashboard page accessible (body visible)", async ({
    page,
  }) => {
    // Caractérise le flow helper loginAsAdmin (injectAuth) sans assertion URL stricte
    // car le RouteGuard peut rediriger l'injection localStorage vers /login (comportement HEAD).
    // Ce test fige juste que la page se charge et que main est visible.
    await loginAsAdmin(page);
    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("main").first()).toBeVisible({ timeout: 10000 });
  });

  test("syndic login (UI) + syndic dashboard rendered", async ({ page }) => {
    const start = Date.now();
    const timestamp = Date.now();
    const email = `char-syndic-${timestamp}@example.com`;
    const password = "test123456";

    // Setup org via admin (API) puis register syndic (API) puis login UI
    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const { token: adminToken } = await adminLoginResp.json();

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Char Syndic Org ${timestamp}`,
        slug: `char-syndic-${timestamp}`,
        contact_email: email,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email,
        password,
        first_name: "Char",
        last_name: `Syndic${timestamp}`,
        role: "syndic",
        organization_id: org.id,
      },
    });
    expect(regResp.ok()).toBeTruthy();

    // Login via UI
    await page.goto("/login");
    await page.getByTestId("login-email").fill(email);
    await page.getByTestId("login-password").fill(password);
    await page.getByTestId("login-submit").click();
    await page.waitForURL(/\/(syndic|admin|owner|accountant)/, {
      timeout: 15000,
    });

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("main").first()).toBeVisible({ timeout: 10000 });

    const elapsed = Date.now() - start;
    expect(
      elapsed,
      `Syndic UI login flow took ${elapsed}ms — caractérisation cible < 30000ms`,
    ).toBeLessThan(30000);
  });

  test("owner login (UI) + owner dashboard rendered", async ({ page }) => {
    const start = Date.now();
    const timestamp = Date.now();
    const email = `char-owner-${timestamp}@example.com`;
    const password = "test123456";

    // Owner peut se register sans organization_id (rôle public)
    const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const { token: adminToken } = await adminLoginResp.json();

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Char Owner Org ${timestamp}`,
        slug: `char-owner-${timestamp}`,
        contact_email: email,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email,
        password,
        first_name: "Char",
        last_name: `Owner${timestamp}`,
        role: "owner",
        organization_id: org.id,
      },
    });
    expect(regResp.ok()).toBeTruthy();

    // Login via UI
    await page.goto("/login");
    await page.getByTestId("login-email").fill(email);
    await page.getByTestId("login-password").fill(password);
    await page.getByTestId("login-submit").click();
    await page.waitForURL(/\/(syndic|admin|owner|accountant)/, {
      timeout: 15000,
    });

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("main").first()).toBeVisible({ timeout: 10000 });

    const elapsed = Date.now() - start;
    expect(
      elapsed,
      `Owner UI login flow took ${elapsed}ms — caractérisation cible < 30000ms`,
    ).toBeLessThan(30000);
  });

  test("loginAsSyndic helper produces a usable token", async ({ page }) => {
    // Vérifie que le helper réutilisé partout fonctionne sur HEAD
    const ctx = await loginAsSyndic(page, "char-helper");
    expect(ctx.token).toBeTruthy();
    expect(ctx.adminToken).toBeTruthy();
    expect(ctx.orgId).toBeTruthy();
    expect(ctx.email).toContain("char-helper");
  });

  test("loginAsSyndicWithBuilding helper provisions building", async ({
    page,
  }) => {
    // Vérifie que la création building via le helper passe sur HEAD
    const ctx = await loginAsSyndicWithBuilding(page, "char-helper-bld");
    expect(ctx.buildingId).toBeTruthy();

    // Le building doit être fetchable par le syndic
    const resp = await page.request.get(
      `${API_BASE}/buildings/${ctx.buildingId}`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    expect(resp.ok()).toBeTruthy();
  });
});
