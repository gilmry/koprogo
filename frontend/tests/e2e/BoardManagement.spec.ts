import { test, expect } from "@playwright/test";
import { loginAsSyndicWithOwner } from "./helpers/auth";

/**
 * Board Management E2E Test Suite - Conseil de Copropriété
 *
 * Tests board member election, board decisions, and stats.
 * Belgian law: Conseil de copropriété — elected board to oversee syndic (Art. 3.84 CC).
 * Mirrors workflows from backend/tests/e2e_board.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Board Management - Conseil de Copropriété", () => {
  test("should display board dashboard page", async ({ page }) => {
    await loginAsSyndicWithOwner(page, "boardmgmt");
    await page.goto("/board-dashboard");

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("main").first()).toBeVisible({ timeout: 10000 });
  });

  test("should elect a board member and retrieve it", async ({ page }) => {
    const { token, buildingId, orgId, ownerId, adminToken } =
      await loginAsSyndicWithOwner(page, "boardmgmt");
    const mandateStart = new Date();
    const mandateEnd = new Date();
    mandateEnd.setDate(mandateEnd.getDate() + 365);

    // Create a building with >20 units (required by Belgian law for conseil)
    const lgBuildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Board Building ${Date.now()}`,
        address: "1 Rue du Conseil",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        total_units: 25,
        construction_year: 2010,
        organization_id: orgId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const lgBuilding = await lgBuildingResp.json();

    // Create a meeting for this building (needed for elected_by_meeting_id)
    const meetingDate = new Date();
    meetingDate.setDate(meetingDate.getDate() + 30);
    const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
      data: {
        building_id: lgBuilding.id,
        organization_id: orgId,
        title: `AG Board Election ${Date.now()}`,
        scheduled_date: meetingDate.toISOString(),
        meeting_type: "Ordinary",
        location: "Salle communale",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const meeting = await meetingResp.json();

    const electResp = await page.request.post(`${API_BASE}/board-members`, {
      data: {
        building_id: lgBuilding.id,
        owner_id: ownerId,
        position: "President",
        mandate_start: mandateStart.toISOString(),
        mandate_end: mandateEnd.toISOString(),
        elected_by_meeting_id: meeting.id,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(electResp.status())).toBeTruthy();

    if (electResp.ok()) {
      const member = await electResp.json();
      expect(member.id).toBeTruthy();
      expect(member.position).toBe("president");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/board-members/${member.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(member.id);
    }
  });

  test("should list active board members for building", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithOwner(
      page,
      "boardmgmt",
    );

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/board-members/active`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const members = await listResp.json();
    expect(Array.isArray(members)).toBeTruthy();
  });

  test("should create a board decision and retrieve it", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithOwner(
      page,
      "boardmgmt",
    );
    const timestamp = Date.now();
    const dueDate = new Date();
    dueDate.setDate(dueDate.getDate() + 30);

    // Create a meeting first (needed for meeting_id field)
    const meetingDate = new Date();
    meetingDate.setDate(meetingDate.getDate() + 30);
    const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        title: `AG Decision ${timestamp}`,
        scheduled_date: meetingDate.toISOString(),
        meeting_type: "Ordinary",
        location: "Salle communale",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const meeting = await meetingResp.json();

    const decisionResp = await page.request.post(
      `${API_BASE}/board-decisions`,
      {
        data: {
          building_id: buildingId,
          meeting_id: meeting.id,
          subject: `Remplacement chaudière ${timestamp}`,
          decision_text:
            "Suite à l'AG du 15 mars, remplacement de la chaudière",
          deadline: dueDate.toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(decisionResp.status())).toBeTruthy();

    if (decisionResp.ok()) {
      const decision = await decisionResp.json();
      expect(decision.id).toBeTruthy();
      expect(decision.building_id).toBe(buildingId);
      expect(decision.status).toBe("pending");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/board-decisions/${decision.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
    }
  });

  test("should list board decisions for building", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithOwner(
      page,
      "boardmgmt",
    );

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/board-decisions`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const decisions = await listResp.json();
    expect(Array.isArray(decisions)).toBeTruthy();
  });

  test("should get board member statistics", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithOwner(
      page,
      "boardmgmt",
    );

    const statsResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/board-members/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 404].includes(statsResp.status())).toBeTruthy();
  });

  test("should require auth for board management API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/board-members/some-id`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
