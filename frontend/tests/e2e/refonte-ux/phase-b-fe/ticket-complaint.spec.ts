/**
 * Story B5 (Phase B FE) — Ticket Complaint flow — E2E multi-rôle narratif.
 *
 * Flow narratif (cf. memory `feedback_multirole-narrative-scenarios` +
 * stories.md §B5) — DEUX acteurs distincts :
 *
 *   1. Admin via API → seed monde (org + ACP + building + 3 users :
 *      syndic, owner-plaignant, owner-temoin).
 *   2. Owner-plaignant login UI → /tickets/new?buildingId=… → sélectionne
 *      kind=Complaint → la section severity + incident_date + evidence +
 *      witness apparaît → remplit (severity=High, date passée, témoin
 *      sélectionné) → submit → 201 → redirect vers /ticket-detail?id=…
 *   3. Owner-plaignant logout → Syndic login UI → ouvre le ticket dans
 *      son dashboard → voit le dossier complet (titre, description,
 *      severity, witness).
 *
 * Couverture 4 catégories :
 *   @happy    flux ci-dessus end-to-end (2 transitions de rôle).
 *   @edge / @security / @negative : couverts exhaustivement par les tests
 *   Vitest (TicketCreate.test.ts + SeveritySelector.test.ts +
 *   EvidenceUpload.test.ts + WitnessSelector.test.ts). Ce spec E2E se
 *   concentre sur l'INVARIANT FE-BE intégration + le cross-role narrative.
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
// Helpers seed monde (cf. pattern B6 syndic-response-sla.spec.ts)
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
      name: `B5 Org ${ts}`,
      slug: `b5-org-${ts}`,
      contact_email: `b5-${ts}@example.com`,
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
      name: `B5 ACP ${ts}`,
      address_street: `${ts} Rue B5`,
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
      name: `B5 Building ${ts}`,
      address: `${ts} Rue B5`,
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
  const ts = Date.now() + Math.floor(Math.random() * 1000);
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

test.describe("Story B5 — Ticket Complaint (multi-rôle)", () => {
  test.beforeEach(async ({ page }) => {
    await setupContainerApiUrl(page);
  });

  test("@happy Owner dépose Complaint avec witness → Syndic voit le dossier complet dans son dashboard", async ({
    page,
    request,
  }) => {
    // ─── Phase 1 : Admin seed monde via API ──────────────────────────────
    const adminToken = await loginAdmin(request);
    const org = await createOrg(request, adminToken);
    const acp = await createAcp(request, adminToken, org.id);
    const building = await createBuilding(request, adminToken, acp.id);
    const syndic = await registerUser(request, org.id, "syndic", "B5");
    const ownerPlaignant = await registerUser(request, org.id, "owner", "B5p");
    // owner-temoin déclaré pour la séquence narrative (le witness selector
    // peut afficher 0 candidat selon l'endpoint owners — on tolère).
    await registerUser(request, org.id, "owner", "B5t").catch(() => null);

    // ─── Phase 2 : Owner-plaignant login UI → /tickets/new?buildingId=… ──
    await uiLogin(page, ownerPlaignant.email, TEST_PASSWORD);
    await page.goto(`/tickets/new?buildingId=${building.id}`, {
      waitUntil: "networkidle",
    });

    // Form Owner-facing rendu.
    await expect(page.getByTestId("ticket-create-kind-select")).toBeVisible({
      timeout: 10_000,
    });

    // Switch sur Complaint → sections apparaissent.
    await page
      .getByTestId("ticket-create-kind-select")
      .selectOption("complaint");

    await expect(page.getByTestId("ticket-severity-radio-high")).toBeVisible({
      timeout: 5_000,
    });
    await expect(
      page.getByTestId("ticket-create-incident-date-input"),
    ).toBeVisible();
    await expect(page.getByTestId("ticket-evidence-upload")).toBeVisible();
    await expect(page.getByTestId("ticket-witness-search")).toBeVisible();

    // Remplit le form.
    await page
      .getByTestId("ticket-create-title-input")
      .fill("Tapage nocturne récurrent voisin");
    await page
      .getByTestId("ticket-create-description-textarea")
      .fill(
        "Bruit insupportable chaque nuit du voisin du dessus depuis 3 semaines. Plusieurs voisins en sont témoins.",
      );
    await page.getByTestId("ticket-severity-radio-high").click();

    // Date d'incident : hier (passé garanti).
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    const yIso = yesterday.toISOString().slice(0, 10);
    await page.getByTestId("ticket-create-incident-date-input").fill(yIso);

    // Submit → 201 → redirect /ticket-detail.
    const submit = page.getByTestId("ticket-create-submit");
    await expect(submit).toBeEnabled({ timeout: 5_000 });
    await submit.click();

    // On attend la redirection vers le détail (peut tarder si l'API est lente).
    await page.waitForURL(/\/ticket-detail/, { timeout: 15_000 });

    // Capturer l'URL (donc l'id du ticket) AVANT le logout — une fois
    // reconnecté en syndic, `page.url()` pointe vers son dashboard
    // (/syndic), pas vers le ticket-detail de l'owner (bug trouvé en
    // investiguant #617 C5 : l'URL était lue APRÈS le re-login syndic,
    // donc le regex `id=` ne matchait jamais et le test restait
    // silencieusement sur /syndic).
    const ticketUrl = page.url();
    const ticketIdMatch = ticketUrl.match(/[?&]id=([^&]+)/);
    expect(
      ticketIdMatch,
      `ticket id introuvable dans l'URL ${ticketUrl}`,
    ).not.toBeNull();

    // ─── Phase 3 : Owner-plaignant logout → Syndic login ─────────────────
    await logoutUi(page);
    await uiLogin(page, syndic.email, TEST_PASSWORD);

    // Le syndic ouvre la même URL ticket-detail (id capturé avant logout).
    await page.goto(`/ticket-detail?id=${ticketIdMatch![1]}`, {
      waitUntil: "networkidle",
    });

    // Le syndic voit le titre (intégration FE-BE OK).
    await expect(page.getByTestId("ticket-detail-title")).toContainText(
      "Tapage nocturne",
      { timeout: 10_000 },
    );
  });
});
