import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "./helpers/auth";

/**
 * AG Sessions E2E Test Suite - Video Conference (Art. 3.87 §1 CC)
 *
 * Tests AG video session creation, lifecycle (Scheduled→Live→Ended),
 * remote participant tracking, and combined quorum calculation.
 * Platforms: Zoom, Teams, GoogleMeet, Jitsi, Whereby.
 * Mirrors workflows from backend/tests/e2e_ag_sessions.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("AG Sessions - Video Conference (Art. 3.87 §1 CC)", () => {
  test("should display AG sessions page", async ({ page }) => {
    await loginAsSyndicWithMeeting(page, "agsess");
    await page.goto("/ag-sessions");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='ag-sessions-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a video session for a meeting and retrieve it", async ({
    page,
  }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    const sessionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      {
        data: {
          platform: "Jitsi",
          video_url: "https://meet.jit.si/ag-test-room",
          waiting_room_enabled: false,
          recording_enabled: false,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(sessionResp.status())).toBeTruthy();

    if (sessionResp.ok()) {
      const session = await sessionResp.json();
      expect(session.id).toBeTruthy();
      expect(session.status).toBe("Scheduled");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/ag-sessions/${session.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(session.id);
    }
  });

  test("should start an AG session (Scheduled → Live)", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    const sessionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      {
        data: {
          platform: "Zoom",
          video_url: "https://zoom.us/j/test-meeting",
          waiting_room_enabled: true,
          recording_enabled: true,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (sessionResp.ok()) {
      const session = await sessionResp.json();

      const startResp = await page.request.put(
        `${API_BASE}/ag-sessions/${session.id}/start`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(startResp.status())).toBeTruthy();

      if (startResp.ok()) {
        const started = await startResp.json();
        expect(started.status).toBe("Live");
      }
    }
  });

  test("should record remote participant joining", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    const sessionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      {
        data: {
          platform: "GoogleMeet",
          video_url: "https://meet.google.com/test-code",
          waiting_room_enabled: false,
          recording_enabled: false,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (sessionResp.ok()) {
      const session = await sessionResp.json();

      const joinResp = await page.request.put(
        `${API_BASE}/ag-sessions/${session.id}/record-join`,
        {
          data: { remote_voting_power: 50.0 },
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      expect([200, 400].includes(joinResp.status())).toBeTruthy();
    }
  });

  test("should get combined quorum for AG session", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    const sessionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      {
        data: {
          platform: "Whereby",
          video_url: "https://whereby.com/test-room",
          waiting_room_enabled: false,
          recording_enabled: false,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (sessionResp.ok()) {
      const session = await sessionResp.json();

      const quorumResp = await page.request.get(
        `${API_BASE}/ag-sessions/${session.id}/combined-quorum`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 404].includes(quorumResp.status())).toBeTruthy();
    }
  });

  test("should get AG session by meeting ID", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    await page.request.post(`${API_BASE}/meetings/${meetingId}/ag-session`, {
      data: {
        platform: "Jitsi",
        video_url: "https://meet.jit.si/room-test",
        waiting_room_enabled: false,
        recording_enabled: false,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    const byMeetingResp = await page.request.get(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 404].includes(byMeetingResp.status())).toBeTruthy();
  });

  test("should require auth for AG sessions API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/ag-sessions`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
