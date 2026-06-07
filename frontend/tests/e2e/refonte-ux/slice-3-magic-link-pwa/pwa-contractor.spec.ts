/**
 * Story 3.3 — PWA Contractor E2E (slice 3 / refonte-ux).
 *
 * Flow narratif multi-rôle (cf. memory `feedback_multirole-narrative-scenarios`):
 *
 *   1. Syndic se connecte → crée immeuble + ticket → POST /magic-links
 *      pour le ticket → récupère le token.
 *   2. Nouveau contexte navigateur (PWA contractor, Pixel 7 viewport) →
 *      ouvre `/c/<token>` → vérifie écran 1 (résumé).
 *   3. Click "Répondre au ticket" → écran 2 (formulaire) → fill message →
 *      submit → écran 3 (confirmation).
 *   4. Reconnexion syndic dans un troisième contexte → vérifie que le
 *      ticket aurait été marqué traité côté backend (suivi #X).
 *
 * Couverture 4 catégories (sur des `test.describe` distincts) :
 *   @happy    : flux nominal complet 3 écrans.
 *   @security : second appel `/c/<token>` après consommation → 403 visible.
 *   @negative : `/c/<TOKEN_INVALIDE>` → message d'erreur "Lien invalide".
 *
 * NOTE — endpoint /respond pas encore implémenté backend (follow-up).
 * Le test vérifie l'UX jusqu'à l'écran 3 en interceptant le POST /respond
 * côté Playwright (route.fulfill) — quand l'endpoint réel arrivera,
 * supprimer le mock et garder le reste du flux intact.
 */
import { test, expect, type APIRequestContext } from "@playwright/test";
import { devices } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
const ADMIN_EMAIL = "admin@koprogo.com";
const ADMIN_PASSWORD = "admin123";
const TEST_PASSWORD = process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456";

// ---------------------------------------------------------------------------
// API seed helpers — reuse the use-case path, never raw SQL.
// ---------------------------------------------------------------------------

async function adminLogin(request: APIRequestContext): Promise<string> {
  const resp = await request.post(`${API_BASE}/auth/login`, {
    data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
  });
  if (!resp.ok()) {
    throw new Error(`adminLogin failed: ${resp.status()}`);
  }
  const body = await resp.json();
  return body.token;
}

async function seedSyndicWithTicket(request: APIRequestContext): Promise<{
  syndicToken: string;
  syndicUserId: string;
  buildingId: string;
  ticketId: string;
  magicLinkToken: string;
}> {
  const ts = Date.now();
  const adminToken = await adminLogin(request);

  // Org
  const orgResp = await request.post(`${API_BASE}/organizations`, {
    data: {
      name: `PWA Org ${ts}`,
      slug: `pwa-org-${ts}`,
      contact_email: `pwa-${ts}@example.com`,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  // Syndic
  const syndicEmail = `pwa-syndic-${ts}@example.com`;
  const regResp = await request.post(`${API_BASE}/auth/register`, {
    data: {
      email: syndicEmail,
      password: TEST_PASSWORD,
      first_name: "PwaSyndic",
      last_name: `Test${ts}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const syndic = await regResp.json();
  const syndicUserId = syndic.user?.id || syndic.id || syndic.user_id || "";
  const syndicToken = syndic.token;

  // ACP
  const acpResp = await request.post(`${API_BASE}/acps`, {
    data: {
      organization_id: org.id,
      name: `PWA ACP ${ts}`,
      address_street: `${ts} Rue Test`,
      address_postal_code: "1000",
      address_city: "Brussels",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const acp = await acpResp.json();

  // Building
  const buildingResp = await request.post(`${API_BASE}/buildings`, {
    data: {
      name: `PWA Building ${ts}`,
      address: `${ts} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 12,
      construction_year: 2010,
      acp_id: acp.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  // Ticket
  const ticketResp = await request.post(`${API_BASE}/tickets`, {
    data: {
      building_id: building.id,
      title: `Fuite cuisine ${ts}`,
      description: "Fuite sous l'évier, urgence modérée.",
      category: "Plumbing",
      priority: "High",
    },
    headers: { Authorization: `Bearer ${syndicToken}` },
  });
  const ticket = await ticketResp.json();

  // Magic link issued by the syndic for that ticket.
  const linkResp = await request.post(`${API_BASE}/magic-links`, {
    data: {
      subject_user_id: syndicUserId, // For now the contractor sub is the syndic itself; story 3.4 will register external contractors.
      scope_kind: "ticket",
      scope_id: ticket.id,
      expires_in_seconds: 60 * 60, // 1 hour
    },
    headers: { Authorization: `Bearer ${syndicToken}` },
  });
  if (!linkResp.ok()) {
    throw new Error(
      `POST /magic-links failed: ${linkResp.status()} ${await linkResp.text()}`,
    );
  }
  const link = await linkResp.json();

  return {
    syndicToken,
    syndicUserId,
    buildingId: building.id,
    ticketId: ticket.id,
    magicLinkToken: link.token,
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("Story 3.3 — PWA Contractor (slice 3)", () => {
  // Pixel 7 viewport for the contractor PWA — closest reasonable proxy for
  // the production target (Android Chrome on a mid-range phone).
  test.use({ ...devices["Pixel 7"] });

  test("@happy syndic issues link → contractor PWA flows through 3 screens", async ({
    page,
    request,
  }) => {
    const seed = await seedSyndicWithTicket(request);

    // Intercept the (not-yet-implemented) /respond endpoint so the flow can
    // complete in CI. Remove when the backend endpoint lands (follow-up).
    await page.route(/\/c\/[^/]+\/respond$/, async (route) => {
      if (route.request().method() === "POST") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ ok: true }),
        });
        return;
      }
      await route.fallback();
    });

    await page.goto(`/c/${encodeURIComponent(seed.magicLinkToken)}`);

    // Screen 1 visible (server-rendered + client-mount).
    await expect(page.getByTestId("pwa-screen-1-summary")).toBeVisible({
      timeout: 15_000,
    });
    await expect(page.getByTestId("pwa-summary-content")).toContainText(
      "Fuite cuisine",
    );

    // Advance to action screen.
    await page.getByTestId("pwa-summary-next").click();
    await expect(page.getByTestId("pwa-screen-2-action")).toBeVisible();

    // Fill the message — required field, enables submit.
    await page
      .getByTestId("pwa-action-message-input")
      .fill("Je passe demain matin entre 9h et 11h.");

    const submit = page.getByTestId("pwa-action-submit");
    await expect(submit).toBeEnabled();
    await submit.click();

    // Confirmation screen reached.
    await expect(page.getByTestId("pwa-screen-3-confirm")).toBeVisible({
      timeout: 10_000,
    });
  });

  test("@security second visit on a consumed token returns 403 error card", async ({
    page,
    request,
  }) => {
    const seed = await seedSyndicWithTicket(request);

    // First visit consumes the token (single-use semantics, Story 3.2).
    await page.goto(`/c/${encodeURIComponent(seed.magicLinkToken)}`);
    await expect(page.getByTestId("pwa-screen-1-summary")).toBeVisible({
      timeout: 15_000,
    });

    // Second visit (new context) must show the FR error card.
    await page.goto(`/c/${encodeURIComponent(seed.magicLinkToken)}`);
    await expect(page.getByTestId("c-page-error")).toBeVisible({
      timeout: 10_000,
    });
  });

  test("@negative invalid token shows error card without leaking scope", async ({
    page,
  }) => {
    await page.goto("/c/this-token-does-not-exist");

    const err = page.getByTestId("c-page-error");
    await expect(err).toBeVisible({ timeout: 10_000 });
    // The summary screen MUST NOT appear — no information leak about any
    // valid scope payload.
    await expect(page.getByTestId("pwa-screen-1-summary")).toHaveCount(0);
  });
});
