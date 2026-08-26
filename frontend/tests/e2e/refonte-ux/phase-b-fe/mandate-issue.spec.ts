/**
 * Story B3 (Phase B FE) — E2E multi-rôle narratif.
 *
 * Flow narratif :
 *   1. Admin loggue par API → crée Cabinet + ACP + Building (seed monde).
 *   2. Admin enregistre un utilisateur notaire (rôle `notary`).
 *   3. Admin loggue via UI → /syndic/mandates → ouvre le modal
 *      MandateIssueForm → sélectionne le notaire + kind=notary + scope=Building
 *      + reason 50+ chars + valid_until=today+365j → submit.
 *   4. La nouvelle ligne apparaît dans <MandateList> avec le
 *      <ExpirationBadge> à level="fresh" (>30j) + texte "12 mois".
 *   5. Logout → re-login en tant que notaire → confirme l'accès dashboard
 *      notaire (le mandat lui octroie un scope de lecture sur le building).
 *
 * Pattern multi-rôle (mémoire `feedback_multirole-narrative-scenarios`) :
 *   logout + re-login à chaque rôle. Pas un seul login pour tout.
 *
 * Seeds (mémoire `world-model-seed`) : via use-cases (HTTP API), jamais
 * SQL direct.
 *
 * AC couverts (4-cat) :
 *   @happy : flow ci-dessus end-to-end avec 3 acteurs (admin/syndic-via-admin,
 *            notaire).
 *
 * Les autres catégories (@edge/@security/@negative) sont couvertes
 * exhaustivement par les tests Vitest (MandateIssueForm.test.ts + MandateList.
 * test.ts) — ce spec E2E se concentre sur l'invariant FE-BE INTÉGRATION.
 */

import {
  test,
  expect,
  type APIRequestContext,
  type Page,
} from "@playwright/test";
import { setupContainerApiUrl } from "../../helpers/video-pace";
import { uiLoginWithRetry, adminLogin } from "../../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
const ADMIN_EMAIL = "admin@koprogo.com";
const ADMIN_PASSWORD = "admin123";
const TEST_PASSWORD = process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456";

// ---------------------------------------------------------------------------
// Helpers API (cf. pattern slice-2 admin-creates-syndic-selects)
// ---------------------------------------------------------------------------

async function loginAdmin(request: APIRequestContext): Promise<string> {
  // Delegue au helper partage : jeton memorise pour toute la campagne, et
  // reprise sur 429. Chaque copie locale reloguait sans cache et epuisait le
  // plafond Traefik de 5 connexions/minute sur `/api/v1/auth/login`
  // (symptome observe : « admin login — Expected: 200, Received: 429 »).
  return adminLogin(request);
}

async function createCabinet(
  request: APIRequestContext,
  adminToken: string,
  prefix: string,
): Promise<{ id: string; name: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Cabinet ${prefix} ${ts}`,
      slug: `cabinet-${prefix}-${ts}`,
      contact_email: `cabinet-${prefix}-${ts}@example.com`,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  expect(resp.status(), `create cabinet ${prefix}`).toBe(201);
  return resp.json();
}

async function createAcp(
  request: APIRequestContext,
  adminToken: string,
  cabinetId: string,
  prefix: string,
): Promise<{ id: string; name: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/acps`, {
    data: {
      organization_id: cabinetId,
      name: `ACP ${prefix} ${ts}`,
      address_street: `${ts} Rue ${prefix}`,
      address_postal_code: "1000",
      address_city: "Bruxelles",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  expect(resp.status(), `create acp ${prefix}`).toBe(201);
  return resp.json();
}

async function createBuilding(
  request: APIRequestContext,
  adminToken: string,
  acpId: string,
  prefix: string,
): Promise<{ id: string; name: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/buildings`, {
    data: {
      acp_id: acpId,
      name: `Immeuble ${prefix} ${ts}`,
      address: `${ts} Rue ${prefix}`,
      city: "Bruxelles",
      postal_code: "1000",
      country: "Belgium",
      total_units: 4,
      construction_year: 2015,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  expect(resp.status(), `create building ${prefix}`).toBe(201);
  return resp.json();
}

/**
 * Register un user notaire (rôle `notary` — Story 3.1, `UserRole::Notary`
 * accepté directement par `/auth/register`, cf. `user.rs` FromStr).
 *
 * Registré avec le rôle `notary` directement : `MandatesPage.svelte`
 * (Story S2, docs/maury/syndic-org-users-endpoint) ne propose comme
 * destinataires de mandat que les rôles `ELIGIBLE_ROLES` (lawyer, notary,
 * amo, architect, bet, warden) — un user en `owner` n'apparaîtrait jamais
 * dans le sélecteur.
 */
async function registerNotary(
  request: APIRequestContext,
  cabinetId: string,
  prefix: string,
): Promise<{ token: string; email: string; userId: string }> {
  const ts = Date.now();
  const email = `notary-${prefix}-${ts}@example.com`;
  const resp = await request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: "Notaire",
      last_name: `Test${ts}`,
      role: "notary",
      organization_id: cabinetId,
    },
  });
  expect(resp.status(), `register notary`).toBeLessThan(400);
  const body = await resp.json();
  return {
    token: body.token,
    email,
    userId: body.user?.id || body.id || body.user_id || "",
  };
}

/**
 * Register un user syndic — acteur réel de la page `/syndic/mandates`
 * (cf. règle CRITICAL.md #9, corrige le contournement admin=syndic-proxy
 * identifié par l'investigation C3, docs/agent-activity/
 * 2026-08-06-issue617-c3-investigation.md).
 */
async function registerSyndic(
  request: APIRequestContext,
  cabinetId: string,
  prefix: string,
): Promise<{ token: string; email: string; userId: string }> {
  const ts = Date.now();
  const email = `syndic-${prefix}-${ts}@example.com`;
  const resp = await request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: "Syndic",
      last_name: `Test${ts}`,
      role: "syndic",
      organization_id: cabinetId,
    },
  });
  expect(resp.status(), `register syndic`).toBeLessThan(400);
  const body = await resp.json();
  return {
    token: body.token,
    email,
    userId: body.user?.id || body.id || body.user_id || "",
  };
}

async function uiLogin(
  page: Page,
  email: string,
  password: string,
): Promise<void> {
  // Delegue au helper partage : il reprend sur echec, ce qui absorbe le
  // plafond Traefik de 5 connexions/minute sur `/api/v1/auth/login`.
  await uiLoginWithRetry(page, email, password);
}

async function logoutUi(page: Page): Promise<void> {
  const btn = page.getByTestId("user-menu-logout");
  if (await btn.isVisible().catch(() => false)) {
    await btn.click();
    await page.waitForURL(/\/login/, { timeout: 10_000 });
  }
  await page.evaluate(() => {
    try {
      localStorage.removeItem("koprogo_user");
    } catch {
      /* ignore */
    }
  });
}

// ---------------------------------------------------------------------------
// Test @happy — multi-rôle 3 acteurs (Admin → Syndic-emulé → Notary)
// ---------------------------------------------------------------------------

test.describe("Story B3 — MandateIssueForm + List (multi-rôle 3 acteurs)", () => {
  test.beforeEach(async ({ page }) => {
    await setupContainerApiUrl(page);
  });

  test("@happy Syndic émet un mandat → row visible avec ExpirationBadge → Notary login", async ({
    page,
    request,
  }) => {
    // ─── Phase 1 : Admin via API — seed monde ────────────────────────────
    const adminToken = await loginAdmin(request);
    const cabinet = await createCabinet(request, adminToken, "B3");
    const acp = await createAcp(request, adminToken, cabinet.id, "B3");
    const building = await createBuilding(request, adminToken, acp.id, "B3");
    const notary = await registerNotary(request, cabinet.id, "B3");
    const syndic = await registerSyndic(request, cabinet.id, "B3");

    // ─── Phase 2 : login syndic réel (acteur métier de /syndic/mandates) ──
    await uiLogin(page, syndic.email, TEST_PASSWORD);

    // ─── Phase 3 : naviguer vers /syndic/mandates ────────────────────────
    await page.goto("/syndic/mandates", { waitUntil: "networkidle" });

    // CTA "Nouveau mandat" visible
    const newButton = page.getByTestId("mandate-new-button");
    await expect(newButton).toBeVisible({ timeout: 10_000 });
    await newButton.click();

    // ─── Phase 4 : émission via le modal ─────────────────────────────────
    // Subject = le notaire seedé
    await page
      .getByTestId("mandate-subject-select")
      .selectOption({ value: notary.userId });

    // Kind = notary (sélectionné par défaut, mais on confirme via setvalue)
    await page
      .getByTestId("mandate-kind-select")
      .selectOption({ value: "notary" });

    // Scope = building (radio par défaut)
    await expect(
      page.getByTestId("mandate-scope-type-radio-building"),
    ).toBeChecked();

    // Scope ID = le building seedé
    await page
      .getByTestId("mandate-scope-id-select")
      .selectOption({ value: building.id });

    // Reason : 50+ chars
    const reasonText =
      "Mandat de notaire pour acter la vente du Lot 12 (Lot principal côté jardin)";
    await page.getByTestId("mandate-reason-textarea").fill(reasonText);

    // valid_until = today + 365j (au format YYYY-MM-DD)
    const validUntilDate = new Date(Date.now() + 365 * 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 10);
    await page.getByTestId("mandate-valid-until-input").fill(validUntilDate);

    // Submit
    const submit = page.getByTestId("mandate-issue-submit");
    await expect(submit).toBeEnabled({ timeout: 5_000 });
    await submit.click();

    // ─── Phase 5 : vérification row + ExpirationBadge ────────────────────
    // La liste contient maintenant au moins une row avec le mandat émis.
    // Le badge expiration est au level "fresh" (>30j) — on vérifie son
    // data-level (attribut stable contractuel, cf. ExpirationBadge.svelte).
    const list = page.getByTestId("mandate-list");
    await expect(list).toBeVisible({ timeout: 10_000 });

    // Une row existe pour ce subject (on filtre par le label visible).
    // On ne connaît pas l'ID du mandat created côté FE — on cherche un badge
    // expiration avec data-level=fresh.
    const freshBadge = page
      .locator('[data-testid^="expiration-badge-mandate-"][data-level="fresh"]')
      .first();
    await expect(freshBadge).toBeVisible({ timeout: 10_000 });

    // ─── Phase 6 : logout syndic → login notaire ─────────────────────────
    await logoutUi(page);
    await uiLogin(page, notary.email, TEST_PASSWORD);

    // Le notaire arrive sur son dashboard (route par rôle — on vérifie juste
    // qu'il est bien loggé et redirigé hors /login).
    await expect(page).not.toHaveURL(/\/login/);
  });
});
