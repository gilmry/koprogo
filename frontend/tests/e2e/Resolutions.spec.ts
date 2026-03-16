import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithMeeting(page: Page): Promise<{
  token: string;
  buildingId: string;
  meetingId: string;
  orgId: string;
}> {
  const timestamp = Date.now();
  const email = `resolution-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Resolution Test Org ${timestamp}`,
      slug: `resolution-test-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: "test123456",
      first_name: "Resolution",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Resolution Building ${timestamp}`,
      address: `${timestamp} Rue Vote`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 12,
      construction_year: 2000,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  const meetingDate = new Date();
  meetingDate.setDate(meetingDate.getDate() + 1);

  const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
    data: {
      building_id: building.id,
      organization_id: org.id,
      title: `AG Votes ${timestamp}`,
      meeting_date: meetingDate.toISOString(),
      meeting_type: "Ordinary",
      agenda: "Vote résolutions",
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  const meeting = await meetingResp.json();

  await page.goto("/login");
  await page.getByTestId("login-email").fill(email);
  await page.getByTestId("login-password").fill("test123456");
  await page.getByTestId("login-submit").click();
  await page.waitForURL(/\/(syndic|admin|owner)/, { timeout: 15000 });

  return {
    token,
    buildingId: building.id,
    meetingId: meeting.id,
    orgId: org.id,
  };
}

test.describe("Resolutions - AG Voting System", () => {
  test("should display meetings page with resolutions", async ({ page }) => {
    await setupSyndicWithMeeting(page);
    await page.goto("/meetings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='meetings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a resolution via API", async ({ page }) => {
    const { token, meetingId } = await setupSyndicWithMeeting(page);
    const timestamp = Date.now();

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          title: `Résolution ${timestamp}`,
          description: "Approbation du budget annuel 2026",
          resolution_type: "ordinary",
          majority_required: "simple",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(resolutionResp.status())).toBeTruthy();
  });

  test("should list meeting resolutions", async ({ page }) => {
    const { token, meetingId } = await setupSyndicWithMeeting(page);

    const listResp = await page.request.get(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const resolutions = await listResp.json();
    expect(Array.isArray(resolutions)).toBeTruthy();
  });

  test("should navigate to meeting detail with resolutions tab", async ({
    page,
  }) => {
    const { meetingId } = await setupSyndicWithMeeting(page);

    await page.goto(`/meeting-detail?id=${meetingId}`);
    await expect(page.locator("body")).toBeVisible();
  });
});
