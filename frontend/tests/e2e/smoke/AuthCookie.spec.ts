import { test, expect } from "@playwright/test";

/**
 * WP-FE1 — JWT hors localStorage (vol de session XSS).
 *
 * Vérifie de bout en bout que :
 *  - l'access token n'est JAMAIS en localStorage (mémoire seule) ;
 *  - le refresh token est un cookie HttpOnly (illisible par `document.cookie`),
 *    scopé `/api/v1/auth` ;
 *  - le silent-refresh via ce cookie restaure la session après reload ;
 *  - sans cookie valide, pas de session (redirection /login).
 *
 * Taxonomie 4 catégories (CRITICAL.md #3). Stack live http://localhost
 * (Traefik) ; `COOKIE_SECURE=false` en dev/E2E (cf. docker-compose.yml).
 */

const ADMIN_EMAIL = "admin@koprogo.com";
const ADMIN_PASSWORD = "admin123";

async function uiLogin(page: import("@playwright/test").Page) {
  await page.goto("/login", { waitUntil: "domcontentloaded" });
  await page.fill('input[type="email"]', ADMIN_EMAIL);
  await page.fill('input[type="password"]', ADMIN_PASSWORD);
  await page.click('button[type="submit"]');
  // LoginForm redirige hors /login après authStore.login()
  await page.waitForURL((url) => !url.pathname.startsWith("/login"), {
    timeout: 15000,
  });
}

test.describe("WP-FE1 — refresh token cookie HttpOnly", () => {
  test("@security access token absent de localStorage, refresh illisible JS", async ({
    page,
  }) => {
    await uiLogin(page);

    const lsToken = await page.evaluate(() =>
      localStorage.getItem("koprogo_token"),
    );
    const lsRefresh = await page.evaluate(() =>
      localStorage.getItem("koprogo_refresh_token"),
    );
    expect(lsToken, "access token MUST NOT be in localStorage").toBeNull();
    expect(lsRefresh, "refresh token MUST NOT be in localStorage").toBeNull();

    // Le cookie refresh est HttpOnly → invisible depuis document.cookie.
    const jsCookies = await page.evaluate(() => document.cookie);
    expect(jsCookies).not.toContain("koprogo_refresh");
  });

  test("@edge cookie refresh HttpOnly + scopé /api/v1/auth dans le contexte", async ({
    page,
  }) => {
    await uiLogin(page);

    const cookies = await page.context().cookies();
    const refresh = cookies.find((c) => c.name === "koprogo_refresh");
    expect(refresh, "refresh cookie must be set").toBeTruthy();
    expect(refresh!.httpOnly, "refresh cookie must be HttpOnly").toBe(true);
    expect(refresh!.sameSite).toBe("Strict");
    expect(refresh!.path).toBe("/api/v1/auth");
    expect(refresh!.value.length).toBeGreaterThan(0);
  });

  test("@happy reload conserve la session via silent-refresh cookie", async ({
    page,
  }) => {
    await uiLogin(page);
    const afterLogin = page.url();

    // Reload : l'access token mémoire est perdu ; init() doit le restaurer
    // via le cookie HttpOnly (silent-refresh) sans repasser par /login.
    await page.reload({ waitUntil: "networkidle" });
    await expect
      .poll(() => new URL(page.url()).pathname, { timeout: 15000 })
      .not.toMatch(/^\/login/);
    expect(new URL(page.url()).pathname).toBe(new URL(afterLogin).pathname);
  });

  test("@negative sans cookie refresh → pas de session (redir /login)", async ({
    page,
  }) => {
    await uiLogin(page);

    // Supprime le cookie refresh : le silent-refresh doit échouer (401)
    // et la session être nettoyée.
    await page.context().clearCookies();
    await page.goto("/syndic", { waitUntil: "networkidle" });
    await expect
      .poll(() => new URL(page.url()).pathname, { timeout: 15000 })
      .toMatch(/^\/login/);
  });
});
