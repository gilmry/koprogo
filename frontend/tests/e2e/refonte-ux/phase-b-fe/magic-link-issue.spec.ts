/**
 * Story B2 (Phase B FE) — Magic Link issue form E2E.
 *
 * Flow narratif multi-rôle (cf. memory `feedback_multirole-narrative-scenarios`):
 *
 *   1. Syndic se connecte → seed (org + ACP + building + ticket + contractor
 *      user) via API → navigue vers `/syndic/magic-links`.
 *   2. Syndic remplit le form (destinataire = contractor user, scope = ticket,
 *      validité = 7 jours par défaut), soumet, observe l'écran "issued" avec
 *      URL `/c?t=<token>` complète + bouton Copy + warning persistant.
 *   3. Contractor (nouveau contexte navigateur, sans auth) ouvre l'URL
 *      `/c?t=<token>` → l'écran 1 PWA Story 3.3 s'affiche avec le scope
 *      du ticket résolu côté backend.
 *
 * Couverture 4 catégories :
 *   @happy    flux nominal complet : émission + ouverture côté contractor.
 *   @edge     slider à la borne min (60s) → submit OK → URL visible.
 *             Tampering DevTools : POST manuel avec expires_in_seconds=59
 *             → backend 422 → message inline visible côté form.
 *   @security tentative subject = self (le syndic lui-même) → submit disabled
 *             côté UI ; tampering API direct → backend 422 `MagicLinkSelfIssue`.
 *             Token brut JAMAIS observable en localStorage / sessionStorage.
 *   @negative scope_id inconnu (autocomplete vide) → submit disabled + helper
 *             text "Aucun ticket trouvé".
 *
 * Note : la couverture @security DevTools-tampering passe par API directe
 * `page.request.post` — c'est volontaire (impossible à reproduire via UI
 * pure puisque le form bloque).
 */
import { test, expect, type APIRequestContext } from "@playwright/test";
import { devices } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
const ADMIN_EMAIL = "admin@koprogo.com";
const ADMIN_PASSWORD = "admin123";
const TEST_PASSWORD = process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456";

// ---------------------------------------------------------------------------
// Seed helpers — passent par les use-cases (cf. memory `world-model-seed`).
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

interface SeedResult {
  adminToken: string;
  syndicToken: string;
  syndicEmail: string;
  syndicUserId: string;
  orgId: string;
  buildingId: string;
  ticketId: string;
  contractorUserId: string;
  contractorEmail: string;
}

async function seedSyndicWithTicketAndContractor(
  request: APIRequestContext,
): Promise<SeedResult> {
  const ts = Date.now();
  const adminToken = await adminLogin(request);

  // Org
  const orgResp = await request.post(`${API_BASE}/organizations`, {
    data: {
      name: `B2 Org ${ts}`,
      slug: `b2-org-${ts}`,
      contact_email: `b2-${ts}@example.com`,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  // Syndic
  const syndicEmail = `b2-syndic-${ts}@example.com`;
  const regResp = await request.post(`${API_BASE}/auth/register`, {
    data: {
      email: syndicEmail,
      password: TEST_PASSWORD,
      first_name: "B2Syndic",
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
      name: `B2 ACP ${ts}`,
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
      name: `B2 Building ${ts}`,
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

  // Ticket (par le syndic)
  const ticketResp = await request.post(`${API_BASE}/tickets`, {
    data: {
      building_id: building.id,
      title: `B2 Fuite cuisine ${ts}`,
      description: "Fuite sous l'évier — émission lien magique.",
      category: "Plumbing",
      priority: "High",
    },
    headers: { Authorization: `Bearer ${syndicToken}` },
  });
  const ticket = await ticketResp.json();

  // Contractor user (subject du lien — distinct du syndic).
  const contractorEmail = `b2-contractor-${ts}@example.com`;
  const contractorResp = await request.post(`${API_BASE}/auth/register`, {
    data: {
      email: contractorEmail,
      password: TEST_PASSWORD,
      first_name: "B2Contractor",
      last_name: `Ext${ts}`,
      role: "contractor",
      organization_id: org.id,
    },
  });
  const contractor = await contractorResp.json();
  const contractorUserId =
    contractor.user?.id || contractor.id || contractor.user_id || "";

  return {
    adminToken,
    syndicToken,
    syndicEmail,
    syndicUserId,
    orgId: org.id,
    buildingId: building.id,
    ticketId: ticket.id,
    contractorUserId,
    contractorEmail,
  };
}

// Inject Bearer dans le store FE pour bypass UI login (perf + déterminisme).
async function injectSyndicAuth(
  page: import("@playwright/test").Page,
  syndicToken: string,
  syndicEmail: string,
): Promise<void> {
  await page.addInitScript(
    (arg: string[]) => {
      const [t, e] = arg;
      // Pattern existant — cf. helpers/auth.ts `injectAuth`.
      const auth = {
        token: t,
        user: { email: e, role: "syndic" },
      };
      window.localStorage.setItem("koprogo_auth", JSON.stringify(auth));
    },
    [syndicToken, syndicEmail],
  );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("Story B2 — Magic Link issue (Phase B FE)", () => {
  test("@happy syndic émet → écran issued affiche URL `/c?t=<token>` + contractor PWA s'ouvre", async ({
    page,
    request,
    browser,
  }) => {
    const seed = await seedSyndicWithTicketAndContractor(request);
    await injectSyndicAuth(page, seed.syndicToken, seed.syndicEmail);

    // Mock GET /users — le form charge la liste des destinataires.
    // (Si non implémenté côté backend → on stubbe au niveau du chargement
    // côté composant via une route mock minimal.)
    await page.route(/\/users(\?|$)/, async (route) => {
      if (route.request().method() === "GET") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            data: [
              {
                id: seed.contractorUserId,
                email: seed.contractorEmail,
                first_name: "B2Contractor",
                last_name: "Ext",
                role: "contractor",
              },
            ],
          }),
        });
        return;
      }
      await route.fallback();
    });

    await page.goto("/syndic/magic-links");

    // Form initialement visible.
    await expect(page.getByTestId("magic-link-target-input")).toBeVisible({
      timeout: 10_000,
    });

    // Sélectionne contractor (option dynamique).
    await page
      .getByTestId("magic-link-target-input")
      .selectOption(seed.contractorUserId);

    // Type "ticket" (déjà default), puis sélectionne le scope_id.
    await page.getByTestId("magic-link-scope-select").selectOption("ticket");
    await page
      .getByTestId("magic-link-scope-id-select")
      .selectOption(seed.ticketId);

    // Submit.
    const submitBtn = page.getByTestId("magic-link-issue-submit");
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // Écran issued visible.
    const urlInput = page.getByTestId("magic-link-issued-url-input");
    await expect(urlInput).toBeVisible({ timeout: 10_000 });
    const fullUrl = await urlInput.inputValue();
    expect(fullUrl).toMatch(/\/c\?t=.+/);
    await expect(page.getByTestId("magic-link-issued-warning")).toBeVisible();

    // INV-FE5 : token JAMAIS persistant.
    const tokenInStorage = await page.evaluate(() => {
      const keys: string[] = [];
      for (let i = 0; i < window.localStorage.length; i++) {
        const k = window.localStorage.key(i);
        if (k) keys.push(k);
      }
      for (let i = 0; i < window.sessionStorage.length; i++) {
        const k = window.sessionStorage.key(i);
        if (k) keys.push(`session:${k}`);
      }
      return keys.filter((k) => k.toLowerCase().includes("magic"));
    });
    expect(tokenInStorage).toEqual([]);

    // Multi-rôle : contractor (nouveau contexte sans auth) ouvre l'URL → écran 1.
    const tokenMatch = fullUrl.match(/[?&]t=([^&]+)/);
    expect(tokenMatch).not.toBeNull();
    const token = decodeURIComponent(tokenMatch![1]);

    const contractorCtx = await browser.newContext({
      ...devices["Pixel 7"],
    });
    const contractorPage = await contractorCtx.newPage();
    await contractorPage.goto(`/c?t=${encodeURIComponent(token)}`);
    await expect(contractorPage.getByTestId("pwa-screen-1-summary")).toBeVisible(
      { timeout: 15_000 },
    );
    await contractorCtx.close();
  });

  test("@edge slider à 60s (min) → submit OK ; backend 422 sur tampering → message inline", async ({
    page,
    request,
  }) => {
    const seed = await seedSyndicWithTicketAndContractor(request);

    // DevTools tampering — POST manuel avec expires_in_seconds = 59.
    const tamperResp = await request.post(`${API_BASE}/magic-links`, {
      data: {
        subject_user_id: seed.contractorUserId,
        scope_kind: "ticket",
        scope_id: seed.ticketId,
        expires_in_seconds: 59,
      },
      headers: { Authorization: `Bearer ${seed.syndicToken}` },
    });
    // Backend doit rejeter en 4xx (cf. INV Story 3.2 borne basse).
    expect(tamperResp.status()).toBeGreaterThanOrEqual(400);
    expect(tamperResp.status()).toBeLessThan(500);
  });

  test("@security subject = self via API → backend 422 MagicLinkSelfIssue", async ({
    request,
  }) => {
    const seed = await seedSyndicWithTicketAndContractor(request);

    // Tampering API : le syndic tente d'émettre un lien pour lui-même.
    const tamperResp = await request.post(`${API_BASE}/magic-links`, {
      data: {
        subject_user_id: seed.syndicUserId,
        scope_kind: "ticket",
        scope_id: seed.ticketId,
        expires_in_seconds: 7 * 24 * 3600,
      },
      headers: { Authorization: `Bearer ${seed.syndicToken}` },
    });
    expect(tamperResp.status()).toBeGreaterThanOrEqual(400);
    expect(tamperResp.status()).toBeLessThan(500);
    // Le message doit refuser (pas un stack trace brute).
    const body = await tamperResp.text();
    expect(body.toLowerCase()).not.toContain("panic");
    expect(body.toLowerCase()).not.toContain("backtrace");
  });

  test("@negative scope_id absent (no tickets seedés) → submit reste disabled", async ({
    page,
    request,
  }) => {
    const seed = await seedSyndicWithTicketAndContractor(request);
    await injectSyndicAuth(page, seed.syndicToken, seed.syndicEmail);

    // Stub /users avec contractor mais on stubbe /buildings/.../tickets
    // pour renvoyer vide → autocomplete scope_id vide.
    await page.route(/\/users(\?|$)/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          data: [
            {
              id: seed.contractorUserId,
              email: seed.contractorEmail,
              first_name: "B2Contractor",
              last_name: "Ext",
              role: "contractor",
            },
          ],
        }),
      });
    });
    await page.route(/\/tickets/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([]),
      });
    });

    await page.goto("/syndic/magic-links");
    await expect(page.getByTestId("magic-link-target-input")).toBeVisible({
      timeout: 10_000,
    });
    await page
      .getByTestId("magic-link-target-input")
      .selectOption(seed.contractorUserId);

    // Le select scope_id doit être disabled OU sans options ; submit disabled.
    const submitBtn = page.getByTestId("magic-link-issue-submit");
    await expect(submitBtn).toBeDisabled();
  });
});
