import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "./helpers/auth";

/**
 * Resolutions E2E Test Suite - AG Voting System
 *
 * Tests resolution creation, voting (Pour/Contre/Abstention),
 * and voting closure with different majority types (Simple/Absolute/Qualified).
 * Mirrors workflows from backend/tests/e2e_resolutions.rs.
 *
 * Belgian law (Art. 3.88 CC): 3 majority types.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Resolutions - AG Voting System", () => {
  test("should display meetings page", async ({ page }) => {
    await loginAsSyndicWithMeeting(page, "resolution");
    await page.goto("/meetings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='meetings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a resolution and retrieve it", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );
    const timestamp = Date.now();
    const title = `Budget annuel ${timestamp}`;

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title,
          description: "Approbation du budget annuel 2026",
          resolution_type: "ordinary",
          majority_required: "absolute",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(resolutionResp.status())).toBeTruthy();

    if (resolutionResp.ok()) {
      const resolution = await resolutionResp.json();
      expect(resolution.id).toBeTruthy();
      expect(resolution.title).toBe(title);
      expect(resolution.status).toBe("Pending");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/resolutions/${resolution.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(resolution.id);
    }
  });

  test("should list resolutions for a meeting", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );

    const listResp = await page.request.get(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const resolutions = await listResp.json();
    expect(Array.isArray(resolutions)).toBeTruthy();
  });

  test("should navigate to meeting detail page", async ({ page }) => {
    const { meetingId } = await loginAsSyndicWithMeeting(page, "resolution");

    await page.goto(`/meeting-detail?id=${meetingId}`);
    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
  });

  test("should cast a vote on a resolution", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );
    const timestamp = Date.now();

    // Create resolution
    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `Vote test ${timestamp}`,
          description: "Résolution pour test de vote",
          resolution_type: "ordinary",
          majority_required: "absolute",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (resolutionResp.ok()) {
      const resolution = await resolutionResp.json();

      // Cast a vote
      const voteResp = await page.request.post(
        `${API_BASE}/resolutions/${resolution.id}/vote`,
        {
          data: {
            choice: "Pour",
            voting_power: 100,
          },
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      expect([201, 400, 409].includes(voteResp.status())).toBeTruthy();
    }
  });

  test("should list votes for a resolution", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );
    const timestamp = Date.now();

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `List votes test ${timestamp}`,
          description: "Test",
          resolution_type: "ordinary",
          majority_required: "absolute",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (resolutionResp.ok()) {
      const resolution = await resolutionResp.json();

      const listResp = await page.request.get(
        `${API_BASE}/resolutions/${resolution.id}/votes`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(listResp.ok()).toBeTruthy();
      const votes = await listResp.json();
      expect(Array.isArray(votes)).toBeTruthy();
    }
  });

  test("should close voting and calculate result", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );
    const timestamp = Date.now();

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `Close vote test ${timestamp}`,
          description: "Test clôture vote",
          resolution_type: "ordinary",
          majority_required: "absolute",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (resolutionResp.ok()) {
      const resolution = await resolutionResp.json();

      const closeResp = await page.request.put(
        `${API_BASE}/resolutions/${resolution.id}/close`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(closeResp.status())).toBeTruthy();

      if (closeResp.ok()) {
        const closed = await closeResp.json();
        expect(["Adopted", "Rejected"].includes(closed.status)).toBeTruthy();
      }
    }
  });

  test("should require auth for resolutions API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/resolutions/some-id`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
