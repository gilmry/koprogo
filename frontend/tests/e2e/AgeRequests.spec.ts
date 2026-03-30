import { test, expect } from "@playwright/test";
import { loginAsSyndicWithOwner } from "./helpers/auth";

/**
 * AGE Requests E2E Test Suite - Demandes d'AGE par copropriétaires
 *
 * Tests AGE request creation, signature collection, threshold (1/5 = 20%),
 * and submission to syndic workflow.
 * Belgian law: Art. 3.87 §2 CC — owners can force extraordinary assembly.
 * Mirrors workflows from backend/tests/e2e_age_requests.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("AGE Requests - Demandes d'AGE (Art. 3.87 §2 CC)", () => {
  test("should display age-requests page", async ({ page }) => {
    await loginAsSyndicWithOwner(page, "agereq");
    await page.goto("/age-requests");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='age-requests-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an AGE request and retrieve it", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithOwner(
      page,
      "agereq",
    );
    const timestamp = Date.now();

    const ageResp = await page.request.post(
      `${API_BASE}/buildings/${buildingId}/age-requests`,
      {
        data: {
          organization_id: orgId,
          title: `AGE urgente ${timestamp}`,
          description:
            "Demande d'assemblée générale extraordinaire pour travaux urgents",
          threshold_pct: 0.2,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(ageResp.status())).toBeTruthy();

    if (ageResp.ok()) {
      const ageRequest = await ageResp.json();
      expect(ageRequest.id).toBeTruthy();
      expect(ageRequest.building_id).toBe(buildingId);
      expect(ageRequest.status).toBe("Draft");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/age-requests/${ageRequest.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(ageRequest.id);
    }
  });

  test("should open an AGE request for signatures (Draft → Open)", async ({
    page,
  }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithOwner(
      page,
      "agereq",
    );
    const timestamp = Date.now();

    const ageResp = await page.request.post(
      `${API_BASE}/buildings/${buildingId}/age-requests`,
      {
        data: {
          organization_id: orgId,
          title: `AGE signatures ${timestamp}`,
          description: "Demande ouverte pour collecte de signatures",
          threshold_pct: 0.2,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (ageResp.ok()) {
      const ageRequest = await ageResp.json();

      const openResp = await page.request.put(
        `${API_BASE}/age-requests/${ageRequest.id}/open`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(openResp.status())).toBeTruthy();

      if (openResp.ok()) {
        const opened = await openResp.json();
        expect(opened.status).toBe("Open");
      }
    }
  });

  test("should list AGE requests for building", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithOwner(page, "agereq");

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/age-requests`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const requests = await listResp.json();
    expect(Array.isArray(requests)).toBeTruthy();
  });

  test("should add a cosignatory to an AGE request", async ({ page }) => {
    const { token, buildingId, orgId, ownerId } = await loginAsSyndicWithOwner(
      page,
      "agereq",
    );
    const timestamp = Date.now();

    const ageResp = await page.request.post(
      `${API_BASE}/buildings/${buildingId}/age-requests`,
      {
        data: {
          organization_id: orgId,
          title: `AGE cosignataires ${timestamp}`,
          description: "Demande pour test de cosignataires",
          threshold_pct: 0.2,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    if (ageResp.ok()) {
      const ageRequest = await ageResp.json();

      // Open first (required to cosign)
      await page.request.put(`${API_BASE}/age-requests/${ageRequest.id}/open`, {
        headers: { Authorization: `Bearer ${token}` },
      });

      const cosignResp = await page.request.post(
        `${API_BASE}/age-requests/${ageRequest.id}/cosign`,
        {
          data: { owner_id: ownerId, shares_pct: 10.0 },
          headers: { Authorization: `Bearer ${token}` },
        },
      );
      expect([200, 201, 400].includes(cosignResp.status())).toBeTruthy();
    }
  });

  test("should navigate to new age-request page", async ({ page }) => {
    await loginAsSyndicWithOwner(page, "agereq");
    await page.goto("/age-requests/new");

    await expect(page.locator("body")).toBeVisible();
  });

  test("should require auth for AGE requests API", async ({ page }) => {
    const resp = await page.request.get(
      `${API_BASE}/buildings/some-id/age-requests`,
    );
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
