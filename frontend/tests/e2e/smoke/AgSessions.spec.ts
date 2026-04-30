import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "../helpers/auth";

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
          meeting_id: meetingId,
          platform: "jitsi",
          video_url: "https://meet.jit.si/ag-test-room",
          scheduled_start: "2026-06-15T10:00:00Z",
          waiting_room_enabled: false,
          recording_enabled: false,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(sessionResp.status()).toBe(201);

    const session = await sessionResp.json();
    expect(session.id).toBeTruthy();
    expect(session.status).toBe("scheduled");

    // Retrieve by ID
    const getResp = await page.request.get(
      `${API_BASE}/ag-sessions/${session.id}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(getResp.status()).toBe(200);
    const retrieved = await getResp.json();
    expect(retrieved.id).toBe(session.id);
  });

  test("should start an AG session (Scheduled → Live)", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    const sessionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      {
        data: {
          meeting_id: meetingId,
          platform: "zoom",
          video_url: "https://zoom.us/j/test-meeting",
          scheduled_start: "2026-06-15T10:00:00Z",
          waiting_room_enabled: true,
          recording_enabled: true,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(sessionResp.status()).toBe(201);
    const session = await sessionResp.json();

    const startResp = await page.request.put(
      `${API_BASE}/ag-sessions/${session.id}/start`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(startResp.status()).toBe(200);

    const started = await startResp.json();
    expect(started.status).toBe("live");
  });

  test("should record remote participant joining", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    const sessionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      {
        data: {
          meeting_id: meetingId,
          platform: "google_meet",
          video_url: "https://meet.google.com/test-code",
          scheduled_start: "2026-06-15T10:00:00Z",
          waiting_room_enabled: false,
          recording_enabled: false,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(sessionResp.status()).toBe(201);
    const session = await sessionResp.json();

    // Session must be Live before recording a remote join
    const startResp = await page.request.put(
      `${API_BASE}/ag-sessions/${session.id}/start`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(startResp.status()).toBe(200);

    const joinResp = await page.request.post(
      `${API_BASE}/ag-sessions/${session.id}/join`,
      {
        data: { voting_power: 50.0, total_building_quotas: 1000.0 },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(joinResp.status()).toBe(200);
  });

  test("should get combined quorum for AG session", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    const sessionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      {
        data: {
          meeting_id: meetingId,
          platform: "whereby",
          video_url: "https://whereby.com/test-room",
          scheduled_start: "2026-06-15T10:00:00Z",
          waiting_room_enabled: false,
          recording_enabled: false,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(sessionResp.status()).toBe(201);
    const session = await sessionResp.json();

    // Start session and add a remote participant so quorum has data
    const startResp = await page.request.put(
      `${API_BASE}/ag-sessions/${session.id}/start`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(startResp.status()).toBe(200);

    await page.request.post(`${API_BASE}/ag-sessions/${session.id}/join`, {
      data: { voting_power: 150.0, total_building_quotas: 1000.0 },
      headers: { Authorization: `Bearer ${token}` },
    });

    // Endpoint is /ag-sessions/:id/quorum with query params
    const quorumResp = await page.request.get(
      `${API_BASE}/ag-sessions/${session.id}/quorum?physical_quotas=300&total_building_quotas=1000`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(quorumResp.status()).toBe(200);
  });

  test("should get AG session by meeting ID", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(page, "agsess");

    await page.request.post(`${API_BASE}/meetings/${meetingId}/ag-session`, {
      data: {
        meeting_id: meetingId,
        platform: "jitsi",
        video_url: "https://meet.jit.si/room-test",
        scheduled_start: "2026-06-15T10:00:00Z",
        waiting_room_enabled: false,
        recording_enabled: false,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    const byMeetingResp = await page.request.get(
      `${API_BASE}/meetings/${meetingId}/ag-session`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(byMeetingResp.status()).toBe(200);
  });

  test("should require auth for AG sessions API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/ag-sessions`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
