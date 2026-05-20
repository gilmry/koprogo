/**
 * Test de dedup in-flight de `authStore.refreshAccessToken()` (#550).
 *
 * Bug : N composants `client:load` (RouteGuard + Navigation + page) mountent
 * en parallèle et appellent chacun `authStore.init()` → N POST /auth/refresh
 * concurrents avec le même cookie refresh single-use → backend rote sur le 1er
 * et rejette les suivants → `clearSession()` → tous les `api.get()` suivants
 * tombent en 401 (Playwright : ~12 tests rouges sur ce pattern).
 *
 * Fix : un seul POST partagé entre callers concurrents (Promise mémoisée).
 */
import { describe, it, expect, beforeEach, vi } from "vitest";

vi.mock("../lib/sync", () => ({
  syncService: {
    setToken: vi.fn(),
    initialize: vi.fn(),
    clearLocalData: vi.fn(),
  },
}));

vi.mock("../lib/db", () => ({
  localDB: {
    init: vi.fn(),
    saveUser: vi.fn(),
  },
}));

vi.mock("../lib/accessToken", () => ({
  getAccessToken: vi.fn(() => null),
  setAccessToken: vi.fn(),
  clearAccessToken: vi.fn(),
}));

vi.mock("../lib/config", () => ({
  apiEndpoint: (path: string) => `http://test.local${path}`,
}));

const okRefreshResponse = () => ({
  ok: true,
  status: 200,
  json: async () => ({
    token: "new-access-token",
    user: {
      id: "u-1",
      email: "syndic@example.com",
      first_name: "Test",
      last_name: "Syndic",
      role: "syndic",
      roles: [],
    },
  }),
});

const failRefreshResponse = () => ({
  ok: false,
  status: 401,
  json: async () => ({ error: "Token expired" }),
});

describe("authStore.refreshAccessToken — in-flight dedup (#550)", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    if (typeof window !== "undefined") {
      localStorage.clear();
    }
  });

  it("@happy — 3 callers concurrents → 1 seul POST /auth/refresh", async () => {
    const mockFetch = vi
      .fn()
      .mockImplementation(async () => okRefreshResponse());
    global.fetch = mockFetch as unknown as typeof fetch;

    const { authStore } = await import("./auth");

    const [r1, r2, r3] = await Promise.all([
      authStore.refreshAccessToken(),
      authStore.refreshAccessToken(),
      authStore.refreshAccessToken(),
    ]);

    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(r1).toBe(true);
    expect(r2).toBe(true);
    expect(r3).toBe(true);
  });

  it("@happy — appels séquentiels après résolution → 1 POST par appel", async () => {
    const mockFetch = vi
      .fn()
      .mockImplementation(async () => okRefreshResponse());
    global.fetch = mockFetch as unknown as typeof fetch;

    const { authStore } = await import("./auth");

    const r1 = await authStore.refreshAccessToken();
    const r2 = await authStore.refreshAccessToken();

    expect(mockFetch).toHaveBeenCalledTimes(2);
    expect(r1).toBe(true);
    expect(r2).toBe(true);
  });

  it("@edge — 10 callers concurrents → toujours 1 POST", async () => {
    const mockFetch = vi
      .fn()
      .mockImplementation(async () => okRefreshResponse());
    global.fetch = mockFetch as unknown as typeof fetch;

    const { authStore } = await import("./auth");

    const results = await Promise.all(
      Array.from({ length: 10 }, () => authStore.refreshAccessToken()),
    );

    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(results.every((r) => r === true)).toBe(true);
  });

  it("@negative — refresh échoue : tous les callers concurrents reçoivent false avec 1 seul POST", async () => {
    const mockFetch = vi
      .fn()
      .mockImplementation(async () => failRefreshResponse());
    global.fetch = mockFetch as unknown as typeof fetch;

    const { authStore } = await import("./auth");

    const [r1, r2, r3] = await Promise.all([
      authStore.refreshAccessToken(),
      authStore.refreshAccessToken(),
      authStore.refreshAccessToken(),
    ]);

    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(r1).toBe(false);
    expect(r2).toBe(false);
    expect(r3).toBe(false);
  });

  it("@security — nouveau cluster d'appels APRÈS échec → nouveau POST (pas de stuck-state)", async () => {
    let firstCallDone = false;
    const mockFetch = vi.fn().mockImplementation(async () => {
      if (!firstCallDone) {
        firstCallDone = true;
        return failRefreshResponse();
      }
      return okRefreshResponse();
    });
    global.fetch = mockFetch as unknown as typeof fetch;

    const { authStore } = await import("./auth");

    const first = await authStore.refreshAccessToken();
    expect(first).toBe(false);

    const second = await authStore.refreshAccessToken();
    expect(second).toBe(true);

    expect(mockFetch).toHaveBeenCalledTimes(2);
  });
});
