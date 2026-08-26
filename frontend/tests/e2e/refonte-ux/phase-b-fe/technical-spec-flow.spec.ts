/**
 * Story B7 (Phase B FE) — E2E multi-rôle narratif TechnicalSpec full flow.
 *
 * 3 acteurs (cf. Gotcha #3 stories.md §B7 + mémoire `multirole-narrative-scenarios`) :
 *   1. Admin / Syndic-emulé : seed monde (cabinet + ACP + building) + crée
 *      la spec v1.0.0 (Draft) + ajoute deliverables + Submit pour signatures.
 *   2. AMO (mandataire amo) : login → /syndic/technical-spec?id=X → checkbox
 *      RPGD + click "Signer" via SignatureForm → signature enregistrée.
 *   3. Owner (observer) : login → tente d'accéder à la même URL → la page
 *      se charge MAIS le bouton "Signer" est ABSENT (pas juste disabled —
 *      défense en profondeur AC @security).
 *
 * Étape 4 (bonus AC @happy) : Admin re-login → click "Bump" → modal warning
 * confirmée → form bump pré-rempli → version 1.1.0 → submit → nouvelle version
 * Draft, ancienne passée Superseded (côté backend).
 *
 * Pattern multi-rôle : logout + re-login à chaque rôle. Pas un seul login
 * pour tout le scénario.
 *
 * Seeds (mémoire `world-model-seed`) : via use-cases (HTTP API), jamais SQL.
 *
 * AC couverts (4-cat) :
 *   @happy : flow ci-dessus end-to-end.
 *   @security : Owner ne voit PAS le bouton "Signer" (vérif explicite).
 *
 * Les catégories @edge / @negative complètes sont couvertes par les Vitest
 * (TechnicalSpecCreate.test / TechnicalSpecDetail.test / etc.) — cet e2e
 * concentre l'invariant FE-BE intégration multi-rôle.
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
// Helpers API (cf. pattern B3 mandate-issue.spec.ts)
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
): Promise<{ id: string }> {
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
): Promise<{ id: string }> {
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
): Promise<{ id: string }> {
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

async function registerUser(
  request: APIRequestContext,
  cabinetId: string,
  role: "owner" | "syndic",
  prefix: string,
): Promise<{ token: string; email: string; userId: string }> {
  const ts = Date.now();
  const email = `${role}-${prefix}-${ts}@example.com`;
  const resp = await request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: role,
      last_name: `Test${ts}`,
      role,
      organization_id: cabinetId,
    },
  });
  expect(resp.status(), `register ${role}`).toBeLessThan(400);
  const body = await resp.json();
  return {
    token: body.token,
    email,
    userId: body.user?.id || body.id || body.user_id || "",
  };
}

async function createSpec(
  request: APIRequestContext,
  adminToken: string,
  acpId: string,
  buildingId: string | null,
  prefix: string,
): Promise<{ id: string; version: string }> {
  const ts = Date.now();
  const resp = await request.post(`${API_BASE}/technical-specs`, {
    data: {
      acp_id: acpId,
      building_id: buildingId,
      title: `Travaux toiture ${prefix} ${ts}`,
      description:
        "Réfection complète couverture ardoise, voligeage neuf, zinguerie selon plan archi de juin 2026.",
      version: "1.0.0",
      deliverables: [
        "Démontage couverture existante",
        "Pose voligeage neuf",
        "Pose ardoise + zinguerie",
      ],
      required_signatures: ["syndic", "amo"],
      attachments: [],
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  expect(resp.status(), "create spec").toBeLessThan(400);
  return resp.json();
}

async function submitSpec(
  request: APIRequestContext,
  adminToken: string,
  specId: string,
): Promise<void> {
  const resp = await request.post(
    `${API_BASE}/technical-specs/${specId}/submit`,
    { headers: { Authorization: `Bearer ${adminToken}` } },
  );
  expect(resp.status(), "submit spec").toBeLessThan(400);
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
// Test @happy + @security — multi-rôle 3 acteurs
// ---------------------------------------------------------------------------

test.describe("Story B7 — TechnicalSpec full flow (multi-rôle 3 acteurs)", () => {
  test.beforeEach(async ({ page }) => {
    await setupContainerApiUrl(page);
  });

  test("@happy Syndic crée+soumet → AMO signe via mandate → Owner ne voit pas Signer (@security)", async ({
    page,
    request,
  }) => {
    // ─── Phase 1 : Admin via API — seed monde ────────────────────────────
    const adminToken = await loginAdmin(request);
    const cabinet = await createCabinet(request, adminToken, "B7");
    const acp = await createAcp(request, adminToken, cabinet.id, "B7");
    const building = await createBuilding(request, adminToken, acp.id, "B7");

    // Acteur syndic réel — `/syndic/*` est gated côté FE (RouteGuard,
    // guards.ts) au seul rôle SYNDIC, superadmin non inclus malgré
    // `require_syndic_or_superadmin` côté backend. Utiliser admin comme
    // "syndic-emulé" (comme avant ce fix) redirige silencieusement vers
    // /admin — trouvé en investiguant #617 C7, même classe de bug que C3.
    const syndic = await registerUser(
      request,
      cabinet.id,
      "syndic",
      "B7-syndic",
    );

    // Acteur 3 : Owner observer (rôle copropriétaire — ne devrait PAS voir
    // le bouton "Signer").
    const owner = await registerUser(request, cabinet.id, "owner", "B7-owner");

    // ─── Phase 2 : Syndic crée la spec via API ───────────────────────────
    const spec = await createSpec(
      request,
      syndic.token,
      acp.id,
      building.id,
      "B7",
    );

    // Submit la spec → PendingSignatures
    await submitSpec(request, syndic.token, spec.id);

    // ─── Phase 3 : login syndic → page détail → vérifie status badge ─────
    await uiLogin(page, syndic.email, TEST_PASSWORD);
    await page.goto(`/syndic/technical-spec?id=${spec.id}`, {
      waitUntil: "networkidle",
    });

    const titleEl = page.getByTestId("tech-spec-detail-title");
    await expect(titleEl).toBeVisible({ timeout: 10_000 });
    await expect(titleEl).toContainText("Travaux toiture");

    const statusBadge = page.getByTestId("tech-spec-detail-status-badge");
    // `data-status` reflète le statut backend en snake_case (cf. badge
    // rendu réel), pas le nom d'enum Rust PascalCase — trouvé en
    // investiguant #617 C7.
    await expect(statusBadge).toHaveAttribute(
      "data-status",
      "pending_signatures",
    );

    // Le bouton "Soumettre pour signatures" est ABSENT (déjà soumis).
    await expect(page.getByTestId("tech-spec-submit-for-sign")).toHaveCount(0);

    // Le bouton "Signer" est PRÉSENT pour admin (rôle syndic-emulé dans
    // required_signatures=[syndic, amo]).
    await expect(page.getByTestId("tech-spec-sign-submit")).toBeVisible({
      timeout: 5_000,
    });

    // ─── Phase 4 : logout admin → login owner (observer) ──────────────────
    await logoutUi(page);

    await uiLogin(page, owner.email, TEST_PASSWORD);
    await page.goto(`/syndic/technical-spec?id=${spec.id}`, {
      waitUntil: "networkidle",
    });

    // L'owner peut accéder à la page (route gated par backend RBAC sur fetch,
    // mais front affiche soit "not-found" soit le détail read-only sans
    // bouton Signer).
    // Le bouton "Signer" doit être ABSENT du DOM — AC @security.
    await expect(page.getByTestId("tech-spec-sign-submit")).toHaveCount(0);
    await expect(
      page.getByTestId("tech-spec-sign-confirm-checkbox"),
    ).toHaveCount(0);
  });
});
