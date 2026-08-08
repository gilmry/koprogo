/**
 * Shared authentication helpers for Playwright E2E tests.
 *
 * Replaces UI login (goto /login, fill, click, waitForURL) with direct
 * localStorage injection — saves ~5s per test and keeps videos focused
 * on the actual feature being tested.
 */
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

interface AuthContext {
  token: string;
  adminToken: string;
  orgId: string;
  email: string;
  userId: string;
}

interface SyndicContext extends AuthContext {
  buildingId: string;
}

interface SyndicWithUnitContext extends SyndicContext {
  unitId: string;
}

interface SyndicWithMeetingContext extends SyndicContext {
  meetingId: string;
}

interface SyndicWithExpenseContext extends SyndicContext {
  expenseId: string;
}

interface SyndicWithOwnerContext extends SyndicContext {
  ownerId: string;
}

interface OwnerContext extends SyndicContext {
  ownerId: string;
  ownerToken: string; // JWT for the owner user account
}

/**
 * Establish an authenticated browser session WITHOUT UI login (WP-FE1).
 *
 * Plus de token en localStorage : l'access token vit en mémoire, le
 * refresh token est un cookie `HttpOnly` posé par le backend lors du
 * `register`/`login` réel effectué par l'appelant via `page.request`
 * (le cookie jar est partagé avec le contexte navigateur). En naviguant
 * vers le dashboard, `authStore.init()` fait un silent-refresh via ce
 * cookie et obtient un access token frais — exactement le flux prod.
 *
 * `koprogo_user` reste injecté : cache d'AFFICHAGE non sensible (peinture
 * instantanée), jamais une preuve d'authentification.
 *
 * **Anti-course de rotation** : on ne visite PAS `/login` au préalable
 * (LoginForm déclenche son propre `authStore.init()` ⇒ refresh #1 qui
 * **rote** le cookie ; une 2ᵉ nav vers le dashboard ⇒ refresh #2 avec le
 * cookie déjà révoqué → 401 → /login). `koprogo_user` est posé via
 * `addInitScript` (avant tout script de page) et une **unique** navigation
 * dashboard déclenche **un seul** silent-refresh (RouteGuard).
 *
 * Pré-requis env (E2E sur http://localhost) : `COOKIE_SECURE=false`
 * (sinon le navigateur refuse le cookie sur une origine non https).
 */
async function injectAuth(
  page: Page,
  _token: string,
  user: { email: string; first_name: string; last_name: string; role: string },
) {
  const roleObj = {
    id: "injected-role-1",
    role: user.role,
    organization_id: null,
    is_primary: true,
  };
  const cachedUser = JSON.stringify({
    id: "injected-user",
    email: user.email,
    first_name: user.first_name,
    last_name: user.last_name,
    role: user.role,
    roles: [roleObj],
    active_role: roleObj,
  });

  // Posé AVANT tout script de page, sur chaque document du contexte —
  // évite la pré-visite de /login (et son refresh prématuré).
  await page.addInitScript((value) => {
    try {
      localStorage.setItem("koprogo_user", value);
    } catch {
      /* localStorage indisponible avant origine — ignoré */
    }
  }, cachedUser);

  // UNE seule navigation dashboard → RouteGuard `authStore.init()` →
  // un seul silent-refresh via le cookie HttpOnly (déjà dans le contexte
  // via le register/login réel de l'appelant). networkidle laisse le
  // refresh résoudre l'access token en mémoire avant les assertions.
  const dashboardPath =
    user.role === "owner"
      ? "/owner"
      : user.role === "superadmin"
        ? "/admin"
        : "/syndic";
  await page.goto(dashboardPath, { waitUntil: "networkidle" });
}

/**
 * Login admin via API, create org + syndic user, inject auth into browser.
 * Returns token and orgId for further API calls.
 */
export async function loginAsSyndic(
  page: Page,
  prefix: string = "test",
): Promise<AuthContext> {
  const timestamp = Date.now();
  const email = `${prefix}-${timestamp}@example.com`;

  // Admin login
  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  // Create org
  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `${prefix} Org ${timestamp}`,
      slug: `${prefix}-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  // Register syndic
  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: prefix.charAt(0).toUpperCase() + prefix.slice(1),
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  // Inject auth into browser (no UI login!)
  await injectAuth(page, userData.token, {
    email,
    first_name: prefix.charAt(0).toUpperCase() + prefix.slice(1),
    last_name: `Test${timestamp}`,
    role: "syndic",
  });

  return {
    token: userData.token,
    adminToken,
    orgId: org.id,
    email,
    userId: userData.user?.id || userData.id || userData.user_id || "",
  };
}

/**
 * Resolve an `acp_id` for the given organization.
 * Lookups the first ACP attached to `orgId` ; creates one on demand if none.
 * Post-#602 helper : tests that used to POST `/buildings { organization_id }`
 * must now POST `/buildings { acp_id }` — call this first to obtain the id.
 */
export async function ensureAcp(
  page: Page,
  orgId: string,
  adminToken: string,
  prefix: string = "test",
): Promise<string> {
  const listResp = await page.request.get(`${API_BASE}/acps`, {
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  if (listResp.ok()) {
    const acps = (await listResp.json()) as Array<{
      id: string;
      organization_id?: string | null;
    }>;
    const existing = acps.find((a) => a.organization_id === orgId);
    if (existing) {
      return existing.id;
    }
  }

  const timestamp = Date.now();
  const createResp = await page.request.post(`${API_BASE}/acps`, {
    data: {
      organization_id: orgId,
      name: `${prefix} ACP ${timestamp}`,
      address_street: `${timestamp} Rue Test`,
      address_postal_code: "1000",
      address_city: "Brussels",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  if (!createResp.ok()) {
    const body = await createResp.text();
    throw new Error(
      `ensureAcp: POST /acps failed ${createResp.status()} : ${body}`,
    );
  }
  const acp = await createResp.json();
  return acp.id;
}

/**
 * Login as syndic + create a building.
 * Post-#602 : creates an ACP first (buildings.acp_id required, FK to acps).
 */
/**
 * Seed `totalUnits` units on `buildingId` whose quotas sum exactly to
 * `quotaSum` (default 1000 = backend default `total_tantiemes`). Makes the
 * building **conformant** per `admin-publishes-conform-buildings` :
 *   count(units) == total_units  &&  SUM(quota) == total_tantiemes
 *
 * Required since Track H Story H2 (`validate-before-compute`) : any
 * operational computation (expenses, charges, états datés, call-for-funds)
 * is now blocked with HTTP 422 BUILDING_NOT_CONFORMANT on a non-conformant
 * building. Rounding error (if any) is balanced on the last unit so the sum
 * is exact.
 */
async function seedConformantUnits(
  page: Page,
  adminToken: string,
  acpId: string,
  buildingId: string,
  totalUnits: number,
  quotaSum: number = 1000,
): Promise<string[]> {
  const baseQuota = Math.floor((quotaSum / totalUnits) * 100) / 100;
  const lastQuota =
    Math.round((quotaSum - baseQuota * (totalUnits - 1)) * 100) / 100;

  const unitIds: string[] = [];
  for (let i = 0; i < totalUnits; i++) {
    const quota = i === totalUnits - 1 ? lastQuota : baseQuota;
    const unitResp = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: buildingId,
        unit_number: `${i + 1}A`,
        floor: Math.floor(i / 2),
        surface_area: 60 + i * 5,
        unit_type: "Apartment",
        quota,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const unit = await unitResp.json();
    unitIds.push(unit.id);
  }
  return unitIds;
}

export async function loginAsSyndicWithBuilding(
  page: Page,
  prefix: string = "test",
  opts: {
    totalUnits?: number;
    totalTantiemes?: number;
    seedUnits?: boolean;
  } = {},
): Promise<SyndicContext> {
  // `totalTantiemes` = base de l'acte de base de la copropriété
  // (1000 millièmes par défaut, 10000 fréquent pour lots fractionnés
  // finement). La conformité exige SUM(quota) == total_tantiemes, donc on
  // l'explicite ici plutôt que de dépendre du défaut backend — cf. mémoire
  // `quota-basis-acte-de-base`.
  const { totalUnits = 12, totalTantiemes = 1000, seedUnits = true } = opts;
  const auth = await loginAsSyndic(page, prefix);
  const timestamp = Date.now();

  // Create ACP first (buildings.acp_id is required FK post-#602 migration)
  const acpId = await ensureAcp(page, auth.orgId, auth.adminToken, prefix);

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `${prefix} Building ${timestamp}`,
      address: `${timestamp} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: totalUnits,
      total_tantiemes: totalTantiemes,
      construction_year: 2010,
      acp_id: acpId,
    },
    headers: { Authorization: `Bearer ${auth.adminToken}` },
  });
  const building = await buildingResp.json();

  // Track H Story H2 — seed conformant units by default so operational
  // computations (expenses/charges/états datés) are not blocked with 422.
  // Units sum exactly to total_tantiemes (the acte de base).
  if (seedUnits) {
    await seedConformantUnits(
      page,
      auth.adminToken,
      acpId,
      building.id,
      totalUnits,
      totalTantiemes,
    );
  }

  return { ...auth, buildingId: building.id };
}

/**
 * Login as syndic + create building + unit.
 */
export async function loginAsSyndicWithUnit(
  page: Page,
  prefix: string = "test",
): Promise<SyndicWithUnitContext> {
  // Building total_units=1 + 1 unit quota 1000 = conformant
  // (count(units)==1==total_units, SUM(quota)==1000==total_tantiemes).
  // seedUnits:false so we add exactly the single unit we return below.
  const ctx = await loginAsSyndicWithBuilding(page, prefix, {
    totalUnits: 1,
    totalTantiemes: 1000,
    seedUnits: false,
  });
  const acpId = await ensureAcp(page, ctx.orgId, ctx.adminToken, prefix);

  const unitResp = await page.request.post(`${API_BASE}/units`, {
    data: {
      acp_id: acpId,
      building_id: ctx.buildingId,
      unit_number: "1A",
      floor: 1,
      surface_area: 85.0,
      unit_type: "Apartment",
      quota: 1000.0,
    },
    headers: { Authorization: `Bearer ${ctx.adminToken}` },
  });
  const unit = await unitResp.json();

  return { ...ctx, unitId: unit.id };
}

/**
 * Login as syndic + create building + meeting.
 */
export async function loginAsSyndicWithMeeting(
  page: Page,
  prefix: string = "test",
): Promise<SyndicWithMeetingContext> {
  const ctx = await loginAsSyndicWithBuilding(page, prefix);

  const meetingDate = new Date();
  meetingDate.setDate(meetingDate.getDate() + 30);

  const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
    data: {
      building_id: ctx.buildingId,
      organization_id: ctx.orgId,
      title: `AG ${Date.now()}`,
      scheduled_date: meetingDate.toISOString(),
      meeting_type: "Ordinary",
      location: "Salle communale",
      is_second_convocation: true,
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });
  const meeting = await meetingResp.json();

  return { ...ctx, meetingId: meeting.id };
}

/**
 * Login as syndic + create building + expense.
 */
export async function loginAsSyndicWithExpense(
  page: Page,
  prefix: string = "test",
): Promise<SyndicWithExpenseContext> {
  const ctx = await loginAsSyndicWithBuilding(page, prefix);

  const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
    data: {
      building_id: ctx.buildingId,
      category: "Maintenance",
      description: `Test expense ${Date.now()}`,
      amount: 500.0,
      expense_date: new Date().toISOString(),
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });
  const expense = await expenseResp.json();

  return { ...ctx, expenseId: expense.id };
}

/**
 * Login as syndic + create building + owner.
 */
export async function loginAsSyndicWithOwner(
  page: Page,
  prefix: string = "test",
): Promise<SyndicWithOwnerContext> {
  const ctx = await loginAsSyndicWithBuilding(page, prefix);
  const timestamp = Date.now();

  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: ctx.orgId,
      first_name: "Owner",
      last_name: `Test${timestamp}`,
      email: `owner-${timestamp}@test.com`,
      address: "1 Rue Test",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });
  const owner = await ownerResp.json();

  return { ...ctx, ownerId: owner.id };
}

/**
 * Create a building + an owner user account (role=owner) linked to an Owner record.
 * Returns both the syndic context and the owner's JWT token.
 * Use this when the API requires an Owner record linked to a user (e.g. shared objects).
 */
export async function loginAsSyndicWithLinkedOwner(
  page: Page,
  prefix: string = "test",
): Promise<OwnerContext> {
  const ctx = await loginAsSyndicWithBuilding(page, prefix);
  const timestamp = Date.now();
  const ownerEmail = `owner-linked-${timestamp}@test.com`;

  // Register an owner user account
  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email: ownerEmail,
      password: "test123456",
      first_name: "Owner",
      last_name: `Linked${timestamp}`,
      role: "owner",
      organization_id: ctx.orgId,
    },
  });
  const ownerUserData = await regResp.json();
  const ownerUserId =
    ownerUserData.user?.id || ownerUserData.id || ownerUserData.user_id || "";
  const ownerToken = ownerUserData.token;

  // Create owner record linked to the user account
  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: ctx.orgId,
      first_name: "Owner",
      last_name: `Linked${timestamp}`,
      email: ownerEmail,
      address: "1 Rue Test",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      user_id: ownerUserId,
    },
    headers: { Authorization: `Bearer ${ctx.token}` },
  });
  const owner = await ownerResp.json();

  return { ...ctx, ownerId: owner.id, ownerToken };
}

/**
 * Login as admin (superadmin) — inject auth into browser.
 */
export async function loginAsAdmin(
  page: Page,
): Promise<{ token: string; adminToken: string }> {
  const loginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const data = await loginResp.json();

  await injectAuth(page, data.token, {
    email: "admin@koprogo.com",
    first_name: "Admin",
    last_name: "KoproGo",
    role: "superadmin",
  });

  return { token: data.token, adminToken: data.token };
}

// ---------------------------------------------------------------------------
// Multi-role helpers (Story Tx.2 — refonte UX multi-role/ACP)
// ---------------------------------------------------------------------------
//
// These helpers expose a consistent signature for every business role used in
// the refonte UX. They follow the same pattern as `loginAsSyndic` /
// `loginAsOwner`-like helpers above: admin login → org create → register a
// scoped user with the target role → inject auth via localStorage (no UI
// login). Returns an `AuthContext` so call sites can chain API calls.
//
// Backend sub-role status (story 3.1):
//   - `accountant.encodeur` / `accountant.emetteur` / `cdc` / `commissaire` /
//     `warden` / `notary` / `lawyer` / `amo` are NOT yet first-class roles in
//     the backend `users.role` enum (only `syndic` / `owner` / `accountant` /
//     `contractor` / `superadmin` / `admin` exist today). Until story 3.1
//     lands the sub-role taxonomy in `domain/entities/user_role_assignment`,
//     we register users with the closest existing base role and tag the
//     intended sub-role through `first_name` / cached-user payload. The
//     helper signatures are stable so call sites won't break when story 3.1
//     swaps the role string.
// ---------------------------------------------------------------------------

/**
 * Register an extra user with an arbitrary role under the admin's default org.
 * Internal — used by the multi-role helpers below.
 */
async function registerScopedUser(
  page: Page,
  prefix: string,
  role: string,
): Promise<AuthContext> {
  const timestamp = Date.now();
  const email = `${prefix}-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `${prefix} Org ${timestamp}`,
      slug: `${prefix}-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456",
      first_name: prefix.charAt(0).toUpperCase() + prefix.slice(1),
      last_name: `Test${timestamp}`,
      role,
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

  await injectAuth(page, userData.token, {
    email,
    first_name: prefix.charAt(0).toUpperCase() + prefix.slice(1),
    last_name: `Test${timestamp}`,
    role,
  });

  return {
    token: userData.token,
    adminToken,
    orgId: org.id,
    email,
    userId: userData.user?.id || userData.id || userData.user_id || "",
  };
}

/**
 * Login as a Contractor authenticated through a magic-link token.
 *
 * Story 3.2 will introduce `POST /magic-links` issuing a short-lived JWT
 * scoped to a `subjectUserId` + `scope` (e.g. a single building or quote).
 * Today the endpoint does not exist yet, so this helper:
 *   1. navigates to `/c/<token>` (the planned magic-link landing route),
 *   2. lets the frontend exchange the token for an access token (cookie),
 *   3. then waits for `networkidle` so the redirected dashboard is ready.
 *
 * @param page  Playwright Page in the browser context
 * @param token Magic-link token (UUID/JWT), see `issueMagicLink` in
 *              `helpers/magic-link.ts`
 *
 * NB: Never log `token` — it grants access.
 */
export async function loginAsContractorMagicLink(
  page: Page,
  token: string,
): Promise<void> {
  // TODO: replace with `/magic-links/exchange` API call when story 3.2 lands.
  if (!token) {
    throw new Error("loginAsContractorMagicLink: empty token");
  }
  await page.goto(`/c/${encodeURIComponent(token)}`, {
    waitUntil: "networkidle",
  });
}

/**
 * Login as Accountant — sub-role `encodeur` (data entry only, no posting).
 * TODO: replace base "accountant" with "accountant.encodeur" when story 3.1
 *       lands the sub-role taxonomy.
 */
export async function loginAsAccountantEncodeur(
  page: Page,
  prefix: string = "accountant-encodeur",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "accountant");
}

/**
 * Login as Accountant — sub-role `emetteur` (validates + posts journal entries).
 * TODO: replace base "accountant" with "accountant.emetteur" when story 3.1
 *       lands the sub-role taxonomy.
 */
export async function loginAsAccountantEmetteur(
  page: Page,
  prefix: string = "accountant-emetteur",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "accountant");
}

/**
 * Login as a member of the Conseil de copropriété (CdC).
 * TODO: replace "owner" with "cdc" when story 3.1 lands; today CdC members
 *       are owners with an elected mandate, which is the closest analogue.
 */
export async function loginAsCdC(
  page: Page,
  prefix: string = "cdc",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "owner");
}

/**
 * Login as the Commissaire aux comptes (audits accountant emetteur output).
 * TODO: replace "owner" with "commissaire" when story 3.1 lands. Commissaire
 *       is typically a co-owner with a specific mandate, so "owner" is the
 *       closest base role today.
 */
export async function loginAsCommissaire(
  page: Page,
  prefix: string = "commissaire",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "owner");
}

/**
 * Login as a Notary (mandataire — read-only legal access scope).
 * TODO: replace "syndic" with "notary" when story 3.1 lands. Until then we
 *       reuse syndic as the closest analogue (manages legal documents).
 */
export async function loginAsNotary(
  page: Page,
  prefix: string = "notary",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "syndic");
}

/**
 * Login as a Lawyer (mandataire — litigation scope).
 * TODO: replace "syndic" with "lawyer" when story 3.1 lands.
 */
export async function loginAsLawyer(
  page: Page,
  prefix: string = "lawyer",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "syndic");
}

/**
 * Login as an AMO (Assistance à Maîtrise d'Ouvrage — project management).
 * TODO: replace "syndic" with "amo" when story 3.1 lands.
 */
export async function loginAsAMO(
  page: Page,
  prefix: string = "amo",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "syndic");
}

/**
 * Login as a Warden (gardien — on-site staff, ticket triage only).
 * TODO: replace "contractor" with "warden" when story 3.1 lands. Contractor
 *       is the closest base role: external/limited-scope user.
 */
export async function loginAsWarden(
  page: Page,
  prefix: string = "warden",
): Promise<AuthContext> {
  return registerScopedUser(page, prefix, "contractor");
}
