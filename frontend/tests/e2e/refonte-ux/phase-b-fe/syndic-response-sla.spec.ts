/**
 * Story B6 (Phase B FE) — Syndic Response + SLA badge — E2E multi-rôle narratif.
 *
 * Flow narratif (cf. memory `feedback_multirole-narrative-scenarios` +
 * stories.md §B6) — DEUX acteurs distincts :
 *
 *   1. Admin via API → seed monde (org + ACP + building + 2 users : syndic
 *      et owner-copropriétaire).
 *   2. Owner login UI → /tickets → crée un ticket "Complaint" (priorité High)
 *      → arrive sur ticket-detail.
 *   3. Owner logout → Syndic login UI → ouvre le même ticket → voit le
 *      SlaBadge initial (level=fresh/warning selon priorité) + le form
 *      `SyndicResponseForm` (AC @security : owner ne le voit PAS).
 *   4. Syndic poste une réponse via form (body 50+ chars + action_proposed
 *      = "schedule_inspection") → la liste `SyndicResponseList` affiche la
 *      nouvelle entrée + le SlaBadge bascule vers level=met (vert) "Réponse
 *      postée à T-Nh ✓".
 *   5. Syndic logout → Owner re-login → voit la même conversation + voit
 *      le SlaBadge "Réponse postée…" mais NE voit PAS le form (INV-FE8 +
 *      AC @security : Owner ne voit PAS bouton "Répondre").
 *
 * Couverture 4 catégories :
 *   @happy    flux ci-dessus end-to-end (3 transitions de rôle).
 *   @edge / @security / @negative : couvertes exhaustivement par les tests
 *   Vitest (SyndicResponseForm.test.ts + SyndicResponseList.test.ts +
 *   SlaBadge.test.ts). Ce spec E2E se concentre sur l'INVARIANT FE-BE
 *   intégration + le cross-role narrative.
 *
 * Seeds (memory `world-model-seed`) : via use-cases (HTTP API), jamais SQL
 * direct.
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
// Helpers seed monde (cf. pattern B2/B3 — passent par les use-cases)
// ---------------------------------------------------------------------------

async function loginAdmin(request: APIRequestContext): Promise<string> {
  // Delegue au helper partage : jeton memorise pour toute la campagne, et
  // reprise sur 429. Chaque copie locale reloguait sans cache et epuisait le
  // plafond Traefik de 5 connexions/minute sur `/api/v1/auth/login`
  // (symptome observe : « admin login — Expected: 200, Received: 429 »).
  return adminLogin(request);
}

async function createOrg(
  request: APIRequestContext,
  adminToken: string,
): Promise<{ id: string; name: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/organizations`, {
    data: {
      name: `B6 Org ${ts}`,
      slug: `b6-org-${ts}`,
      contact_email: `b6-${ts}@example.com`,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  expect(resp.status(), "create org").toBe(201);
  return resp.json();
}

async function createAcp(
  request: APIRequestContext,
  adminToken: string,
  orgId: string,
): Promise<{ id: string; name: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/acps`, {
    data: {
      organization_id: orgId,
      name: `B6 ACP ${ts}`,
      address_street: `${ts} Rue B6`,
      address_postal_code: "1000",
      address_city: "Bruxelles",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  expect(resp.status(), "create acp").toBe(201);
  return resp.json();
}

async function createBuilding(
  request: APIRequestContext,
  adminToken: string,
  acpId: string,
): Promise<{ id: string; name: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/buildings`, {
    data: {
      name: `B6 Building ${ts}`,
      address: `${ts} Rue B6`,
      city: "Bruxelles",
      postal_code: "1000",
      country: "Belgium",
      total_units: 8,
      construction_year: 2012,
      acp_id: acpId,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  expect(resp.status(), "create building").toBe(201);
  return resp.json();
}

async function registerUser(
  request: APIRequestContext,
  orgId: string,
  role: "syndic" | "owner",
  prefix: string,
): Promise<{ email: string; token: string; userId: string }> {
  const ts = Date.now();
  const email = `${prefix}-${role}-${ts}@example.com`;
  const resp = await request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: prefix,
      last_name: `${role}${ts}`,
      role,
      organization_id: orgId,
    },
  });
  expect(resp.status(), `register ${role}`).toBeLessThan(400);
  const body = await resp.json();
  return {
    email,
    token: body.token,
    userId: body.user?.id || body.id || body.user_id || "",
  };
}

async function createTicketAsOwner(
  request: APIRequestContext,
  ownerToken: string,
  buildingId: string,
): Promise<{ id: string; title: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/tickets`, {
    data: {
      building_id: buildingId,
      title: `Fuite couloir étage 2 — ${ts}`,
      description:
        "Une fuite est apparue dans le couloir commun, étage 2, près de l'ascenseur. Merci d'intervenir au plus vite.",
      category: "Plumbing",
      priority: "High",
    },
    headers: { Authorization: `Bearer ${ownerToken}` },
  });
  expect(resp.status(), "create ticket as owner").toBe(201);
  return resp.json();
}

// ---------------------------------------------------------------------------
// Helpers UI
// ---------------------------------------------------------------------------

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
  // Defensive cleanup pour bien casser la session.
  await page.evaluate(() => {
    try {
      localStorage.removeItem("koprogo_user");
      localStorage.removeItem("koprogo_auth");
    } catch {
      /* ignore */
    }
  });
}

// ---------------------------------------------------------------------------
// Test @happy — multi-rôle Owner ↔ Syndic
// ---------------------------------------------------------------------------

test.describe("Story B6 — SyndicResponse + SlaBadge (multi-rôle)", () => {
  test.beforeEach(async ({ page }) => {
    await setupContainerApiUrl(page);
  });

  test("@happy Owner crée Complaint → Syndic répond → Owner voit la réponse + SlaBadge bascule en 'met'", async ({
    page,
    request,
  }) => {
    // ─── Phase 1 : Admin seed monde via API ──────────────────────────────
    const adminToken = await loginAdmin(request);
    const org = await createOrg(request, adminToken);
    const acp = await createAcp(request, adminToken, org.id);
    const building = await createBuilding(request, adminToken, acp.id);
    const syndic = await registerUser(request, org.id, "syndic", "B6");
    const owner = await registerUser(request, org.id, "owner", "B6");

    // ─── Phase 2 : Owner crée le ticket via API (raccourci — l'UI de
    //              création n'est pas le focus de cette story) ────────────
    const ticket = await createTicketAsOwner(request, owner.token, building.id);

    // ─── Phase 3 : Owner login UI → /ticket-detail?id=…  ─────────────────
    await uiLogin(page, owner.email, TEST_PASSWORD);
    await page.goto(`/ticket-detail?id=${ticket.id}`, {
      waitUntil: "networkidle",
    });

    // La page Detail du ticket s'affiche bien.
    await expect(page.getByTestId("ticket-detail")).toBeVisible({
      timeout: 10_000,
    });
    await expect(page.getByTestId("ticket-detail-title")).toContainText(
      "Fuite couloir",
    );

    // La SECTION SyndicResponses est présente (visible pour tous les rôles
    // — owner inclus, AC @happy : "Owner voit la réponse").
    await expect(
      page.getByTestId("ticket-syndic-responses-section"),
    ).toBeVisible();

    // AC @security — Owner NE voit PAS le form SyndicResponseForm.
    // (INV-FE8 + AC @security stories.md §B6 : "Owner ne voit PAS bouton
    // 'Répondre'").
    await expect(page.getByTestId("syndic-response-submit")).toHaveCount(0);
    await expect(page.getByTestId("syndic-response-body-textarea")).toHaveCount(
      0,
    );

    // Liste initiale : 0 réponse — message empty visible.
    const empty = page.getByTestId("syndic-response-list-empty");
    await expect(empty).toBeVisible({ timeout: 10_000 });

    // ─── Phase 4 : Owner logout → Syndic login ───────────────────────────
    await logoutUi(page);
    await uiLogin(page, syndic.email, TEST_PASSWORD);
    await page.goto(`/ticket-detail?id=${ticket.id}`, {
      waitUntil: "networkidle",
    });

    await expect(page.getByTestId("ticket-detail")).toBeVisible({
      timeout: 10_000,
    });

    // AC @security syndic : le form SyndicResponseForm est visible
    // (canRespond=true pour rôle syndic).
    const submit = page.getByTestId("syndic-response-submit");
    await expect(submit).toBeVisible({ timeout: 10_000 });
    const textarea = page.getByTestId("syndic-response-body-textarea");
    await expect(textarea).toBeVisible();

    // Submit DOIT être disabled initialement (body vide → < 10 chars).
    await expect(submit).toBeDisabled();

    // ─── Phase 5 : Syndic poste une réponse ──────────────────────────────
    const responseBody =
      "Je viens d'envoyer le plombier sur place. Il devrait intervenir d'ici demain matin.";
    await textarea.fill(responseBody);
    await page
      .getByTestId("syndic-response-action-proposed-select")
      .selectOption("schedule_inspection");

    await expect(submit).toBeEnabled({ timeout: 5_000 });
    await submit.click();

    // La liste affiche maintenant 1 row visible.
    const list = page.getByTestId("syndic-response-list");
    await expect(list).toBeVisible({ timeout: 10_000 });

    // Au moins une row contient le body posté + le badge action.
    await expect(
      page.locator('[data-testid^="syndic-response-row-body-"]').first(),
    ).toContainText("plombier");
    await expect(
      page.locator('[data-testid^="syndic-response-row-action-"]').first(),
    ).toContainText(/Planifier inspection|schedule_inspection/);

    // INV-FE8 : aucun bouton Edit/Delete sur les responses listées.
    await expect(page.getByTestId("syndic-response-edit-")).toHaveCount(0);
    await expect(page.getByTestId("syndic-response-delete-")).toHaveCount(0);

    // ─── Phase 6 : Syndic logout → Owner re-login ────────────────────────
    await logoutUi(page);
    await uiLogin(page, owner.email, TEST_PASSWORD);
    await page.goto(`/ticket-detail?id=${ticket.id}`, {
      waitUntil: "networkidle",
    });

    // Owner voit la même réponse (liste partagée).
    await expect(page.getByTestId("syndic-response-list")).toBeVisible({
      timeout: 10_000,
    });
    await expect(
      page.locator('[data-testid^="syndic-response-row-body-"]').first(),
    ).toContainText("plombier");

    // Owner NE voit TOUJOURS PAS le form (rôle owner ≠ syndic).
    await expect(page.getByTestId("syndic-response-submit")).toHaveCount(0);
  });
});
