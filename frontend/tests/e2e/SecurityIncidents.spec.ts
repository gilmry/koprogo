import { test, expect } from "@playwright/test";
import { loginAsAdmin, loginAsSyndic } from "./helpers/auth";

/**
 * Security Incidents E2E Test Suite - GDPR Article 33
 *
 * Tests security incident creation, listing, detail view,
 * and severity filtering. Endpoints are admin-only
 * (POST/GET /admin/security-incidents).
 * Uses Traefik on http://localhost.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Security Incidents - GDPR Art. 33 Breach Notification", () => {
  test("should create a security incident via API", async ({ page }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    const createResp = await page.request.post(
      `${API_BASE}/admin/security-incidents`,
      {
        data: {
          severity: "high",
          incident_type: "data_breach",
          title: `Test Breach ${timestamp}`,
          description:
            "Unauthorized access to owner personal data detected in logs",
          data_categories_affected: ["personal_data", "financial_data"],
          affected_subjects_count: 15,
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      },
    );

    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const incident = await createResp.json();
    expect(incident.title).toBe(`Test Breach ${timestamp}`);
    expect(incident.severity).toBe("high");
  });

  test("should list security incidents", async ({ page }) => {
    const { adminToken } = await loginAsAdmin(page);

    const listResp = await page.request.get(
      `${API_BASE}/admin/security-incidents`,
      { headers: { Authorization: `Bearer ${adminToken}` } },
    );

    expect(listResp.ok()).toBeTruthy();
    const incidents = await listResp.json();
    expect(Array.isArray(incidents)).toBeTruthy();
  });

  test("should get a security incident by ID", async ({ page }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    // Create incident
    const createResp = await page.request.post(
      `${API_BASE}/admin/security-incidents`,
      {
        data: {
          severity: "critical",
          incident_type: "unauthorized_access",
          title: `Critical Incident ${timestamp}`,
          description: "Multiple failed authentication attempts from single IP",
          data_categories_affected: ["authentication_data"],
          affected_subjects_count: 1,
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      },
    );

    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const created = await createResp.json();

    // Get by ID
    const getResp = await page.request.get(
      `${API_BASE}/admin/security-incidents/${created.id}`,
      { headers: { Authorization: `Bearer ${adminToken}` } },
    );

    expect(getResp.ok()).toBeTruthy();
    const incident = await getResp.json();
    expect(incident.title).toBe(`Critical Incident ${timestamp}`);
    expect(incident.severity).toBe("critical");
  });

  test("should list overdue incidents requiring APD notification", async ({
    page,
  }) => {
    const { adminToken } = await loginAsAdmin(page);

    const overdueResp = await page.request.get(
      `${API_BASE}/admin/security-incidents/overdue`,
      { headers: { Authorization: `Bearer ${adminToken}` } },
    );

    expect(overdueResp.ok()).toBeTruthy();
    const overdue = await overdueResp.json();
    expect(Array.isArray(overdue)).toBeTruthy();
  });

  test("should reject non-admin access to security incidents", async ({
    page,
  }) => {
    const { token } = await loginAsSyndic(page, "secincident");

    // Syndic should not have access to admin endpoints
    const resp = await page.request.get(
      `${API_BASE}/admin/security-incidents`,
      { headers: { Authorization: `Bearer ${token}` } },
    );

    // Should return 403 Forbidden or 401 Unauthorized
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });

  test("should report incident to APD (Belgian Data Protection Authority)", async ({
    page,
  }) => {
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();

    // Create incident
    const createResp = await page.request.post(
      `${API_BASE}/admin/security-incidents`,
      {
        data: {
          severity: "high",
          incident_type: "data_breach",
          title: `APD Report Test ${timestamp}`,
          description: "Personal data exposed to unauthorized third party",
          data_categories_affected: ["personal_data"],
          affected_subjects_count: 50,
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      },
    );

    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const created = await createResp.json();

    // Report to APD
    const reportResp = await page.request.put(
      `${API_BASE}/admin/security-incidents/${created.id}/report-apd`,
      {
        data: {
          apd_reference_number: `APD-${timestamp}`,
          investigation_notes:
            "Breach contained within 2 hours. All affected users notified.",
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      },
    );

    expect([200, 201, 204].includes(reportResp.status())).toBeTruthy();
  });
});
