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
 * Inject auth token into browser localStorage without UI login.
 * Navigates to a page first (needed for localStorage domain), then injects.
 */
async function injectAuth(
  page: Page,
  token: string,
  user: { email: string; first_name: string; last_name: string; role: string },
) {
  // Navigate to base URL to set localStorage on the right domain
  await page.goto("/login", { waitUntil: "domcontentloaded" });

  await page.evaluate(
    ({ token, user }) => {
      localStorage.setItem("koprogo_token", token);
      const roleObj = {
        id: "injected-role-1",
        role: user.role,
        organization_id: null,
        is_primary: true,
      };
      localStorage.setItem(
        "koprogo_user",
        JSON.stringify({
          id: "injected-user",
          email: user.email,
          first_name: user.first_name,
          last_name: user.last_name,
          role: user.role,
          roles: [roleObj],
          active_role: roleObj,
        }),
      );
      localStorage.setItem("koprogo_refresh_token", token);
    },
    { token, user },
  );

  // Navigate to syndic dashboard to complete "login"
  const dashboardPath =
    user.role === "owner"
      ? "/owner"
      : user.role === "superadmin"
        ? "/admin"
        : "/syndic";
  await page.goto(dashboardPath, { waitUntil: "domcontentloaded" });
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
 * Login as syndic + create a building.
 */
export async function loginAsSyndicWithBuilding(
  page: Page,
  prefix: string = "test",
): Promise<SyndicContext> {
  const auth = await loginAsSyndic(page, prefix);
  const timestamp = Date.now();

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `${prefix} Building ${timestamp}`,
      address: `${timestamp} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 12,
      construction_year: 2010,
      organization_id: auth.orgId,
    },
    headers: { Authorization: `Bearer ${auth.adminToken}` },
  });
  const building = await buildingResp.json();

  return { ...auth, buildingId: building.id };
}

/**
 * Login as syndic + create building + unit.
 */
export async function loginAsSyndicWithUnit(
  page: Page,
  prefix: string = "test",
): Promise<SyndicWithUnitContext> {
  const ctx = await loginAsSyndicWithBuilding(page, prefix);

  const unitResp = await page.request.post(`${API_BASE}/units`, {
    data: {
      organization_id: ctx.orgId,
      building_id: ctx.buildingId,
      unit_number: "1A",
      floor: 1,
      surface_area: 85.0,
      unit_type: "Apartment",
      quota: 100.0,
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
