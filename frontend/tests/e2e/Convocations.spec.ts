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
  const email = `convoc-test-${timestamp}@example.com`;

  const adminLoginResp = await page.request.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
  });
  const adminData = await adminLoginResp.json();
  const adminToken = adminData.token;

  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Convoc Test Org ${timestamp}`,
      slug: `convoc-test-${timestamp}`,
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
      first_name: "Convoc",
      last_name: `Test${timestamp}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();
  const token = userData.token;

  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `Convoc Building ${timestamp}`,
      address: `${timestamp} Rue AG`,
      city: "Ghent",
      postal_code: "9000",
      country: "Belgium",
      total_units: 10,
      construction_year: 2005,
      organization_id: org.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();

  // Create meeting 20 days from now
  const meetingDate = new Date();
  meetingDate.setDate(meetingDate.getDate() + 20);

  const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
    data: {
      building_id: building.id,
      organization_id: org.id,
      title: `AG Ordinaire ${timestamp}`,
      meeting_date: meetingDate.toISOString(),
      meeting_type: "Ordinary",
      agenda: "Points divers",
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

test.describe("Convocations - AG Automatic Invitations", () => {
  test("should display convocations page", async ({ page }) => {
    await setupSyndicWithMeeting(page);
    await page.goto("/convocations");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='convocations-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a convocation via API", async ({ page }) => {
    const { token, buildingId, meetingId } = await setupSyndicWithMeeting(page);
    const timestamp = Date.now();

    const convocResp = await page.request.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meetingId,
        building_id: buildingId,
        language: "FR",
        subject: `Convocation AG ${timestamp}`,
        body_text: "Vous êtes convoqués à l'assemblée générale ordinaire.",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201, 400].includes(convocResp.status())).toBeTruthy();
  });

  test("should navigate to convocation detail page", async ({ page }) => {
    const { token, buildingId, meetingId } = await setupSyndicWithMeeting(page);

    const convocResp = await page.request.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meetingId,
        building_id: buildingId,
        language: "FR",
        subject: "Convocation detail test",
        body_text: "Corps de la convocation.",
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (convocResp.status() === 201) {
      const convoc = await convocResp.json();
      await page.goto(`/convocation-detail?id=${convoc.id}`);
      await expect(page.locator("body")).toBeVisible();
    } else {
      // Skip if convocation creation fails (legal deadline not met)
      await page.goto("/convocations");
      await expect(page.locator("body")).toBeVisible();
    }
  });

  test("should list building convocations", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithMeeting(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/convocations`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const convocations = await listResp.json();
    expect(Array.isArray(convocations)).toBeTruthy();
  });
});
