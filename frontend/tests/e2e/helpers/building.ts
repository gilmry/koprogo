/**
 * Building / ACP scope helpers for Playwright E2E tests (Story Tx.2).
 *
 * Companion to `auth.ts`: lets a test pin a syndic session to a *specific*
 * building or ACP (instead of letting the helper create a fresh one), and
 * provides `seedBuildingWithUnits()` for compliance-friendly fixtures
 * (count(units) == total_units, SUM(quota) == 1000) — see memory
 * `admin-publishes-conform-buildings`.
 */
import type { Page } from "@playwright/test";
import { loginAsSyndic, ensureAcp } from "./auth";
import { createApiClient } from "./api-client";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

interface SyndicScopedContext {
  token: string;
  adminToken: string;
  orgId: string;
  email: string;
  userId: string;
  buildingId: string;
}

interface SyndicAcpContext extends SyndicScopedContext {
  acpId: string;
}

/**
 * Login as syndic and bind the session to an existing building.
 *
 * Use this when a test needs to operate on a *specific* building that was
 * seeded earlier (e.g. via `seedBuildingWithUnits` or world-builder fixtures
 * from story Tx.1 caractérisation). For ad-hoc buildings, prefer
 * `loginAsSyndicWithBuilding` from `auth.ts`.
 *
 * @param page       Playwright Page
 * @param buildingId UUID of the target building
 */
export async function loginAsSyndicWithSpecificBuilding(
  page: Page,
  buildingId: string,
): Promise<SyndicScopedContext> {
  if (!buildingId) {
    throw new Error("loginAsSyndicWithSpecificBuilding: buildingId is empty");
  }
  const auth = await loginAsSyndic(page, "syndic-scoped");

  // Sanity check: the building must exist (fail fast — no silent timeout).
  const check = await page.request.get(`${API_BASE}/buildings/${buildingId}`, {
    headers: { Authorization: `Bearer ${auth.token}` },
  });
  if (check.status() === 404) {
    throw new Error(
      `loginAsSyndicWithSpecificBuilding: building ${buildingId} not found (404)`,
    );
  }
  if (!check.ok()) {
    throw new Error(
      `loginAsSyndicWithSpecificBuilding: GET /buildings/${buildingId} -> ${check.status()}`,
    );
  }

  return { ...auth, buildingId };
}

/**
 * Login as syndic and bind the session to a specific ACP (Association des
 * copropriétaires) — the legal entity that owns one or more buildings.
 *
 * NB: in the current data model the ACP is the `organization` + a flag on
 * buildings; story 2.5 (#553) will promote ACP to a first-class entity with
 * its own UUID. Until then, `acpId` is treated as `organizationId`.
 *
 * @param page  Playwright Page
 * @param acpId UUID of the target ACP (today: organization_id)
 */
export async function loginAsSyndicWithSpecificAcp(
  page: Page,
  acpId: string,
): Promise<SyndicAcpContext> {
  if (!acpId) {
    throw new Error("loginAsSyndicWithSpecificAcp: acpId is empty");
  }
  const auth = await loginAsSyndic(page, "syndic-acp");

  // Find at least one building belonging to this ACP to anchor the session.
  const buildingsResp = await page.request.get(
    `${API_BASE}/buildings?organization_id=${encodeURIComponent(acpId)}`,
    { headers: { Authorization: `Bearer ${auth.token}` } },
  );
  if (!buildingsResp.ok()) {
    throw new Error(
      `loginAsSyndicWithSpecificAcp: list buildings -> ${buildingsResp.status()}`,
    );
  }
  const buildings = await buildingsResp.json();
  if (!Array.isArray(buildings) || buildings.length === 0) {
    throw new Error(
      `loginAsSyndicWithSpecificAcp: ACP ${acpId} has no buildings`,
    );
  }

  return { ...auth, acpId, buildingId: buildings[0].id };
}

/**
 * Seed a building with `unitsCount` units whose `quota` values sum exactly
 * to `quotaSum` (default 1000 — the Belgian PCMN canonical millième total).
 *
 * Enforces the conformity invariants required by `admin-publishes-conform-
 * buildings`:
 *   - count(units) == building.total_units
 *   - SUM(units.quota) == quotaSum
 *
 * Throws if quotas cannot be distributed evenly (caller must pick a
 * `quotaSum` divisible by `unitsCount`, or accept ±0.01 rounding on the
 * last unit which we add as a balancing adjustment).
 *
 * @param adminToken     Admin JWT (from `loginAsAdmin().adminToken`)
 * @param organizationId Target organization (ACP) UUID
 * @param unitsCount     Number of units to create (must be >= 1)
 * @param quotaSum       Total quota across all units (default 1000)
 */
export async function seedBuildingWithUnits(
  adminToken: string,
  organizationId: string,
  unitsCount: number,
  quotaSum: number = 1000,
): Promise<{ buildingId: string; unitIds: string[] }> {
  if (unitsCount < 1) {
    throw new Error("seedBuildingWithUnits: unitsCount must be >= 1");
  }
  if (quotaSum <= 0) {
    throw new Error("seedBuildingWithUnits: quotaSum must be > 0");
  }

  // page.request is a Page-bound API — when called outside a Page context,
  // fall back to a manual fetch with the admin bearer. We expose a thin
  // wrapper so callers can pass `apiClient` results if they prefer.
  const headers = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${adminToken}`,
  };

  const timestamp = Date.now();

  // Hotfix #602 — buildings.acp_id (FK acps.id) replaced organization_id.
  // Inline ACP creation : caller is outside a Page context so we can't reuse
  // ensureAcp(page, ...) — we keep the same shape (lookup then create).
  const listAcpResp = await fetch(`${API_BASE}/acps`, {
    method: "GET",
    headers,
  });
  let acpId: string | undefined;
  if (listAcpResp.ok) {
    const acps = (await listAcpResp.json()) as Array<{
      id: string;
      organization_id?: string | null;
    }>;
    acpId = acps.find((a) => a.organization_id === organizationId)?.id;
  }
  if (!acpId) {
    const createAcpResp = await fetch(`${API_BASE}/acps`, {
      method: "POST",
      headers,
      body: JSON.stringify({
        organization_id: organizationId,
        name: `Seed ACP ${timestamp}`,
        address_street: `${timestamp} Rue Seed`,
        address_postal_code: "1000",
        address_city: "Brussels",
      }),
    });
    if (!createAcpResp.ok) {
      throw new Error(
        `seedBuildingWithUnits: POST /acps -> ${createAcpResp.status}`,
      );
    }
    acpId = ((await createAcpResp.json()) as { id: string }).id;
  }

  const buildingResp = await fetch(`${API_BASE}/buildings`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      acp_id: acpId,
      name: `Seeded Building ${timestamp}`,
      address: `${timestamp} Rue Seed`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: unitsCount,
      construction_year: 2020,
    }),
  });
  if (!buildingResp.ok) {
    throw new Error(
      `seedBuildingWithUnits: POST /buildings -> ${buildingResp.status}`,
    );
  }
  const building = (await buildingResp.json()) as { id: string };

  // Distribute quotaSum evenly across units. Rounding error (if any) goes
  // on the last unit so SUM(quota) == quotaSum exactly.
  const baseQuota = Math.floor((quotaSum / unitsCount) * 100) / 100;
  const lastQuota =
    Math.round((quotaSum - baseQuota * (unitsCount - 1)) * 100) / 100;

  const unitIds: string[] = [];
  for (let i = 0; i < unitsCount; i++) {
    const quota = i === unitsCount - 1 ? lastQuota : baseQuota;
    const unitResp = await fetch(`${API_BASE}/units`, {
      method: "POST",
      headers,
      body: JSON.stringify({
        building_id: building.id,
        unit_number: `${i + 1}A`,
        floor: Math.floor(i / 2),
        surface_area: 60 + i * 5,
        unit_type: "Apartment",
        quota,
      }),
    });
    if (!unitResp.ok) {
      throw new Error(
        `seedBuildingWithUnits: POST /units #${i + 1} -> ${unitResp.status}`,
      );
    }
    const unit = (await unitResp.json()) as { id: string };
    unitIds.push(unit.id);
  }

  return { buildingId: building.id, unitIds };
}

/**
 * Page-bound variant of `seedBuildingWithUnits` — uses the typed API client
 * so the call shares cookies / proxy with the browser context. Prefer this
 * when you already have a `Page` (e.g. inside a Playwright test fixture).
 */
export async function seedBuildingWithUnitsViaPage(
  page: Page,
  adminToken: string,
  organizationId: string,
  unitsCount: number,
  quotaSum: number = 1000,
): Promise<{ buildingId: string; unitIds: string[] }> {
  if (unitsCount < 1) {
    throw new Error("seedBuildingWithUnitsViaPage: unitsCount must be >= 1");
  }
  const api = createApiClient(page, adminToken);

  const timestamp = Date.now();

  // Hotfix #602 — buildings.acp_id (FK acps.id) replaced organization_id.
  const acpId = await ensureAcp(page, organizationId, adminToken, "seed-page");

  const buildingRes = await api.post(
    "/buildings" as never,
    {
      acp_id: acpId,
      name: `Seeded Building ${timestamp}`,
      address: `${timestamp} Rue Seed`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: unitsCount,
      construction_year: 2020,
    } as never,
  );
  if (!buildingRes.ok) {
    throw new Error(
      `seedBuildingWithUnitsViaPage: POST /buildings -> ${buildingRes.status}`,
    );
  }
  const buildingId = (buildingRes.data as { id: string }).id;

  const baseQuota = Math.floor((quotaSum / unitsCount) * 100) / 100;
  const lastQuota =
    Math.round((quotaSum - baseQuota * (unitsCount - 1)) * 100) / 100;

  const unitIds: string[] = [];
  for (let i = 0; i < unitsCount; i++) {
    const quota = i === unitsCount - 1 ? lastQuota : baseQuota;
    const unitRes = await api.post(
      "/units" as never,
      {
        organization_id: organizationId,
        building_id: buildingId,
        unit_number: `${i + 1}A`,
        floor: Math.floor(i / 2),
        surface_area: 60 + i * 5,
        unit_type: "Apartment",
        quota,
      } as never,
    );
    if (!unitRes.ok) {
      throw new Error(
        `seedBuildingWithUnitsViaPage: POST /units #${i + 1} -> ${unitRes.status}`,
      );
    }
    unitIds.push((unitRes.data as { id: string }).id);
  }

  return { buildingId, unitIds };
}
