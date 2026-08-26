/**
 * Shared authentication helpers for Playwright E2E tests.
 *
 * Replaces UI login (goto /login, fill, click, waitForURL) with direct
 * localStorage injection — saves ~5s per test and keeps videos focused
 * on the actual feature being tested.
 */
import { test } from "@playwright/test";
import type { APIRequestContext, Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

// ---------------------------------------------------------------------------
// Connexion admin mutualisée (anti-429)
// ---------------------------------------------------------------------------
//
// `/api/v1/auth/login` est rate-limité par Traefik en production :
// average=5/minute, burst=10, par IP source (docker-compose.prod.yml,
// middleware `koprogo-login-ratelimit`). Le garde-fou est volontaire : le
// hachage bcrypt est fait côté serveur et un burst saturerait l'unique cœur
// de la VPS.
//
// Or chaque helper d'authentification re-loguait l'admin, soit une connexion
// PAR TEST. Sur la campagne smoke du 2026-08-26 contre koprogo.com, cela a
// produit ~18 connexions/minute contre 5 autorisées : 47 des 51 blocs d'échec
// remontaient à la même ligne, avec `SyntaxError: Unexpected token 'T', "Too
// Many Requests" is not valid JSON` — le corps 429 de Traefik, en texte brut,
// passé à `.json()`.
//
// Le compte admin est le même pour toute la campagne : une seule connexion
// suffit. Le jeton est donc mémorisé au niveau module (partagé par tous les
// fichiers d'un même worker Playwright) et renouvelé 60 s avant son
// expiration réelle, lue dans le JWT plutôt que supposée.
//
// Le retry sur 429 reste nécessaire malgré le cache : plusieurs workers, ou
// une campagne lancée juste après une autre, peuvent encore franchir le
// seuil. Traefik n'émet pas de `Retry-After`, d'où l'attente fixe alignée sur
// la fenêtre du middleware.

let cachedAdminToken: string | null = null;
let cachedAdminExpiry = 0;

/** Alimente le cache partagé depuis une connexion faite ailleurs. */
function primeAdminTokenCache(token: string): void {
  cachedAdminToken = token;
  const exp = jwtExpiryMs(token);
  cachedAdminExpiry = exp > 0 ? exp - 60_000 : Date.now() + 15 * 60_000;
}

/** Expiration réelle du JWT (ms epoch), 0 si illisible. */
function jwtExpiryMs(token: string): number {
  try {
    const payload = token.split(".")[1];
    const json = JSON.parse(
      Buffer.from(payload, "base64url").toString("utf-8"),
    );
    return typeof json.exp === "number" ? json.exp * 1000 : 0;
  } catch {
    return 0;
  }
}

/**
 * Jeton superadmin, mutualisé sur toute la campagne.
 *
 * Une seule requête `/auth/login` par worker au lieu d'une par test.
 */
/**
 * Connexion admin RÉELLE, avec retry sur 429. Pas de cache.
 *
 * Utilisée telle quelle quand l'appelant a besoin de l'effet de bord d'une
 * vraie requête — le `Set-Cookie: koprogo_refresh` déposé dans le contexte —
 * et non seulement d'un jeton porteur.
 */
/**
 * Parse une reponse en exigeant un statut 2xx.
 *
 * Les helpers de seed enchainaient `await resp.json()` SANS jamais regarder
 * le statut. Un echec transitoire sur une seule requete produisait donc un
 * `undefined` silencieux, et le defaut ressortait beaucoup plus loin sous une
 * forme trompeuse — typiquement un 422 « immeuble non conforme » alors que la
 * vraie cause etait un lot jamais cree.
 *
 * Constate sur `ChargeDistribution`, vert en isolation et rouge en campagne.
 * Le produit avait raison a chaque fois ; c'est le harnais qui construisait
 * un etat incomplet sans le dire.
 */
async function expectOk<T = any>(
  resp: {
    status: () => number;
    ok: () => boolean;
    text: () => Promise<string>;
    json: () => Promise<any>;
  },
  label: string,
): Promise<T> {
  if (!resp.ok()) {
    throw new Error(
      `${label}: HTTP ${resp.status()} — ${(await resp.text()).slice(0, 200)}`,
    );
  }
  return (await resp.json()) as T;
}

export async function performAdminLogin(
  target: Page | APIRequestContext,
): Promise<string> {
  // `scenarios/` n'a pas de Page au moment du seed : il travaille avec un
  // APIRequestContext nu. Les deux exposent `.post()`, on normalise ici.
  const api: APIRequestContext =
    "request" in target
      ? (target as Page).request
      : (target as APIRequestContext);

  const MAX_TRIES = 4;
  let lastStatus = 0;
  let lastBody = "";

  for (let attempt = 1; attempt <= MAX_TRIES; attempt++) {
    const resp = await api.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    lastStatus = resp.status();

    if (resp.ok()) {
      const data = await resp.json();
      if (!data.token) {
        throw new Error(
          `performAdminLogin: réponse 200 sans champ token : ${JSON.stringify(data).slice(0, 200)}`,
        );
      }
      return data.token as string;
    }

    lastBody = (await resp.text()).slice(0, 120);

    if (lastStatus !== 429 || attempt === MAX_TRIES) break;

    // Fenêtre du middleware Traefik : 1 minute. On attend un cinquième de
    // fenêtre par tentative, ce qui suffit à reconstituer des jetons du
    // seau sans immobiliser la campagne une minute entière.
    await new Promise((r) => setTimeout(r, 12_000 * attempt));
  }

  throw new Error(
    `performAdminLogin: échec après ${MAX_TRIES} tentatives — HTTP ${lastStatus} : ${lastBody}`,
  );
}

/**
 * Jeton superadmin mutualisé sur toute la campagne (une connexion par worker).
 *
 * À réserver aux appels API porteurs. Pour ouvrir une session NAVIGATEUR,
 * passer par `loginAsAdmin`, qui a besoin du cookie et donc d'une vraie
 * requête (cf. le commentaire qui y est).
 */
export async function adminLogin(
  target: Page | APIRequestContext,
): Promise<string> {
  if (cachedAdminToken && Date.now() < cachedAdminExpiry) {
    return cachedAdminToken;
  }
  const token = await performAdminLogin(target);
  primeAdminTokenCache(token);
  return token;
}

interface AuthContext {
  token: string;
  adminToken: string;
  orgId: string;
  email: string;
  userId: string;
}

interface SyndicContext extends AuthContext {
  buildingId: string;
  /** Post-#602 : units/meetings/etc. requièrent acp_id, pas organization_id. */
  acpId: string;
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
  const adminToken = await adminLogin(page);

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
  const org = await expectOk(orgResp, "seed:org");

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
  const userData = await expectOk(regResp, "seed:reg");

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
  const acp = await expectOk(createResp, "seed:create");
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
    // Verifier CHAQUE creation.
    //
    // Sans ce controle, un echec transitoire sur un seul lot passait
    // inapercu : `unitIds` recevait `undefined`, l'immeuble restait NON
    // CONFORME (somme des quotites != total_tantiemes), et le defaut
    // ressortait bien plus loin sous la forme d'un 422 au calcul de
    // repartition — un message qui ne dit rien de la vraie cause.
    //
    // C'est exactement ce qui faisait echouer `ChargeDistribution` en
    // campagne alors qu'il passe seul : le produit refusait a juste titre de
    // calculer sur un immeuble que le helper avait laisse incomplet.
    if (unitResp.status() !== 201) {
      throw new Error(
        `seedConformantUnits: lot ${i + 1}/${totalUnits} (quota ${quota}) ` +
          `refuse en HTTP ${unitResp.status()} — ` +
          `${(await unitResp.text()).slice(0, 160)}`,
      );
    }
    const unit = await expectOk(unitResp, "seed:unit");
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
  const building = await expectOk(buildingResp, "seed:building");

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

  return { ...auth, buildingId: building.id, acpId };
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
  const unit = await expectOk(unitResp, "seed:unit");

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
  const meeting = await expectOk(meetingResp, "seed:meeting");

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
  const expense = await expectOk(expenseResp, "seed:expense");

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
  const owner = await expectOk(ownerResp, "seed:owner");

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
  const ownerUserData = await expectOk(regResp, "seed:reg");
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
  const owner = await expectOk(ownerResp, "seed:owner");

  return { ...ctx, ownerId: owner.id, ownerToken };
}

/**
 * Login as admin (superadmin) — inject auth into browser.
 */
export async function loginAsAdmin(
  page: Page,
): Promise<{ token: string; adminToken: string }> {
  // NE PAS utiliser le jeton memorise ici.
  //
  // `adminLogin()` rend une CHAINE, reutilisable partout pour un en-tete
  // Authorization. Mais ouvrir une SESSION NAVIGATEUR demande autre chose :
  // le `Set-Cookie: koprogo_refresh` (HttpOnly, SameSite=Strict, scope
  // /api/v1/auth) que seule une vraie requete /auth/login depose dans le
  // pot a cookies DE CE CONTEXTE. Sans lui, `authStore.init()` ne peut pas
  // faire son silent-refresh, l'access token reste vide en memoire et le
  // RouteGuard renvoie sur /login?redirect=...
  //
  // Un cookie est lie au contexte, un bearer ne l'est pas : les deux
  // chemins ne se substituent pas l'un a l'autre. C'est exactement le
  // piege dans lequel la mutualisation du jeton m'a fait tomber
  // (11 echecs sur refonte-ux/fix-admin-buttons-acp, tous en redirection
  // vers /login alors que les tests visaient des boutons Svelte 5).
  //
  // `loginAsSyndic` n'a pas ce probleme : son POST /auth/register passe par
  // `page.request` et depose bien le cookie du syndic dans le contexte.
  const token = await performAdminLogin(page);
  // Une connexion reelle vient d'avoir lieu : autant en faire profiter le
  // cache partage plutot que d'en consommer une de plus juste apres.
  primeAdminTokenCache(token);

  await injectAuth(page, token, {
    email: "admin@koprogo.com",
    first_name: "Admin",
    last_name: "KoproGo",
    role: "superadmin",
  });

  return { token, adminToken: token };
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

  const adminToken = await adminLogin(page);

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `${prefix} Org ${timestamp}`,
      slug: `${prefix}-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await expectOk(orgResp, "seed:org");

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
  const userData = await expectOk(regResp, "seed:reg");

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

// ---------------------------------------------------------------------------
// Connexion par le FORMULAIRE, avec reprise sur rate limit
// ---------------------------------------------------------------------------
//
// Certains parcours doivent passer par l'UI : ils verifient le redirect par
// role, la banniere de contexte, ou tout simplement que le formulaire marche.
// Ils ne peuvent donc pas utiliser `injectAuth`.
//
// Mais chaque soumission declenche un `/api/v1/auth/login`, plafonne a
// 5/minute par IP source chez Traefik en production (`koprogo-login-ratelimit`,
// average=5 period=1m burst=10). Une suite qui se connecte a chaque test
// depasse ce seuil : le back rend 429, le front reste sur /login, et
// `waitForURL` expire au bout de 15 s sur une navigation qui n'aura jamais
// lieu. Le symptome ne dit rien du rate limit — d'ou le temps qu'il a fallu
// pour l'identifier.
//
// On ne peut pas lire le statut HTTP depuis le formulaire : on traite donc le
// timeout comme un signal de throttling probable et on retente apres une
// attente alignee sur la fenetre du middleware.

/** Connexion par le formulaire, avec 2 reprises espacees sur echec. */
export async function uiLoginWithRetry(
  page: Page,
  email: string,
  password: string,
  urlPattern: RegExp = /\/(admin|syndic|owner|accountant)/,
  timeoutMs = 15_000,
): Promise<void> {
  const MAX_TRIES = 3;
  let lastErr: unknown;

  for (let attempt = 1; attempt <= MAX_TRIES; attempt++) {
    try {
      await page.goto("/login", { waitUntil: "networkidle" });
      await page.getByTestId("login-email").fill(email);
      await page.getByTestId("login-password").fill(password);
      await page.getByTestId("login-submit").click();
      await page.waitForURL(urlPattern, { timeout: timeoutMs });
      await page.waitForLoadState("networkidle");
      return;
    } catch (err) {
      lastErr = err;
      if (attempt === MAX_TRIES) break;

      // ETENDRE LE BUDGET DU TEST AVANT D'ATTENDRE.
      //
      // Sans cela, la reprise se retourne contre nous : le delai par defaut
      // d'un test Playwright est de 30 s, et une seule attente de 20 s le
      // consomme presque entierement. Le test expirait alors sur un
      // « Test timeout of 30000ms exceeded » qui ne dit rien du rate limit —
      // j'ai introduit ce defaut en ajoutant la reprise, et il a fait
      // echouer OwnerDashboard, role-delegation et technical-spec-flow.
      //
      // Une reprise ne vaut que si le test a le temps de la voir aboutir.
      test.setTimeout(90_000 + 40_000 * attempt);

      // Fenetre Traefik : 1 minute. Une attente courte d'abord : le seau se
      // recharge en continu (5 jetons/minute), quelques secondes suffisent
      // souvent, et on ne paie la longue attente qu'en cas d'echec repete.
      await new Promise((r) => setTimeout(r, 8_000 * attempt));
    }
  }

  throw new Error(
    `uiLoginWithRetry: echec apres ${MAX_TRIES} tentatives pour ${email} ` +
      `(rate limit /auth/login probable) — ${String(lastErr).slice(0, 200)}`,
  );
}
