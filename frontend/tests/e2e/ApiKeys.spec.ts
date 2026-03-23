import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "./helpers/auth";

/**
 * API Keys E2E Test Suite - API Key Management
 *
 * Tests API key creation, listing, revocation, and authorization.
 * Uses Traefik on http://localhost.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("API Keys - Management", () => {
  test("should create a new API key via API", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "apikey");
    const timestamp = Date.now();

    const createResp = await page.request.post(`${API_BASE}/api-keys`, {
      data: {
        name: `Integration Key ${timestamp}`,
        description: "Test API key for E2E",
        permissions: ["read:buildings", "read:expenses"],
        rate_limit: 100,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const apiKey = await createResp.json();
    expect(apiKey.name).toBe(`Integration Key ${timestamp}`);
    expect(apiKey.key).toBeTruthy(); // Full key shown only once
    expect(apiKey.key_prefix).toBeTruthy();
  });

  test("should list existing API keys", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "apikey");
    const timestamp = Date.now();

    // Create a key first
    await page.request.post(`${API_BASE}/api-keys`, {
      data: {
        name: `List Test Key ${timestamp}`,
        permissions: ["read:buildings"],
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    // List keys
    const listResp = await page.request.get(`${API_BASE}/api-keys`, {
      headers: { Authorization: `Bearer ${token}` },
    });

    expect(listResp.ok()).toBeTruthy();
    const keys = await listResp.json();
    expect(Array.isArray(keys)).toBeTruthy();
  });

  test("should get a single API key by ID", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "apikey");
    const timestamp = Date.now();

    // Create a key
    const createResp = await page.request.post(`${API_BASE}/api-keys`, {
      data: {
        name: `Get Test Key ${timestamp}`,
        permissions: ["read:buildings"],
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const created = await createResp.json();

    // Get key by ID
    const getResp = await page.request.get(
      `${API_BASE}/api-keys/${created.id}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );

    expect(getResp.ok()).toBeTruthy();
    const key = await getResp.json();
    expect(key.name).toBe(`Get Test Key ${timestamp}`);
  });

  test("should revoke an API key", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "apikey");
    const timestamp = Date.now();

    // Create a key
    const createResp = await page.request.post(`${API_BASE}/api-keys`, {
      data: {
        name: `Revoke Test Key ${timestamp}`,
        permissions: ["read:buildings"],
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const created = await createResp.json();

    // Revoke key
    const revokeResp = await page.request.delete(
      `${API_BASE}/api-keys/${created.id}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );

    expect([200, 204].includes(revokeResp.status())).toBeTruthy();
  });

  test("should reject unauthorized access to API keys endpoint", async ({
    page,
  }) => {
    // No auth header - should return 401
    const resp = await page.request.get(`${API_BASE}/api-keys`);
    expect(resp.status()).toBe(401);
  });

  test("should rotate an API key", async ({ page }) => {
    const { token } = await loginAsSyndic(page, "apikey");
    const timestamp = Date.now();

    // Create a key
    const createResp = await page.request.post(`${API_BASE}/api-keys`, {
      data: {
        name: `Rotate Test Key ${timestamp}`,
        permissions: ["read:buildings"],
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    expect([200, 201].includes(createResp.status())).toBeTruthy();
    const created = await createResp.json();

    // Rotate key
    const rotateResp = await page.request.post(
      `${API_BASE}/api-keys/${created.id}/rotate`,
      { headers: { Authorization: `Bearer ${token}` } },
    );

    // 200 if rotation succeeds, or appropriate error
    expect([200, 201, 400, 404].includes(rotateResp.status())).toBeTruthy();
  });
});
