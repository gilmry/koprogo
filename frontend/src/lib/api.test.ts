/**
 * Tests d'intégration apiFetch ↔ authStore pour #550 strate 2.
 *
 * Bug observé en live console : composants (NotificationBell, listes)
 * mountent et appellent `api.get()` AVANT que `authStore.init()` n'ait
 * fini son silent-refresh → pas de token en RAM → "Missing authorization
 * header" 401 cascade.
 *
 * Fix : si l'utilisateur a un cache d'affichage (`koprogo_user`) mais pas
 * de token en RAM, apiFetch attend le refresh in-flight (mémoïsé via
 * authStore.refreshAccessToken's dedup — un seul POST /auth/refresh
 * partagé entre tous les callers concurrents).
 */
import { describe, it, expect, beforeEach, vi } from "vitest";

vi.mock("../stores/toast", () => ({
  toast: {
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
    success: vi.fn(),
  },
}));

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

vi.mock("svelte-i18n", () => ({
  locale: {
    subscribe: (fn: any) => {
      fn("fr");
      return () => {};
    },
  },
}));

const okRefreshResponse = () => ({
  ok: true,
  status: 200,
  json: async () => ({
    token: "fresh-token-from-refresh",
    user: {
      id: "u-1",
      email: "test@example.com",
      first_name: "Test",
      last_name: "User",
      role: "syndic",
      roles: [],
    },
  }),
});

const okDataResponse = () => ({
  ok: true,
  status: 200,
  json: async () => ({ items: [] }),
});

describe("apiFetch awaits refresh when cached user exists but no token (#550 strate 2)", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    if (typeof window !== "undefined") {
      localStorage.clear();
    }
  });

  it("@happy — cached user présent + token absent → déclenche refresh + fetch avec Authorization", async () => {
    localStorage.setItem(
      "koprogo_user",
      JSON.stringify({ id: "u-1", email: "test@example.com", role: "syndic" }),
    );

    const mockFetch = vi.fn().mockImplementation(async (url: string) => {
      if (url.includes("/auth/refresh")) return okRefreshResponse();
      return okDataResponse();
    });
    global.fetch = mockFetch as unknown as typeof fetch;

    const { api } = await import("./api");
    await api.get("/notifications/unread");

    // 2 fetchs : 1 pour refresh, 1 pour la cible
    expect(mockFetch).toHaveBeenCalledTimes(2);
    expect(mockFetch.mock.calls[0][0] as string).toContain("/auth/refresh");
    const dataCallHeaders = (mockFetch.mock.calls[1][1] as RequestInit)
      .headers as Headers;
    expect(dataCallHeaders.get("Authorization")).toBe(
      "Bearer fresh-token-from-refresh",
    );
  });

  it("@happy — pas de cached user → pas de refresh (login/register flow)", async () => {
    // localStorage vide → utilisateur jamais logué

    const mockFetch = vi.fn().mockImplementation(async () => okDataResponse());
    global.fetch = mockFetch as unknown as typeof fetch;

    const { api } = await import("./api");
    await api.post("/auth/login", { email: "x@x", password: "p" });

    // 1 seul fetch : pas de refresh déclenché (pas de cache user)
    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(mockFetch.mock.calls[0][0] as string).toContain("/auth/login");
  });

  it("@security — endpoint /auth/* → jamais de refresh préalable (évite récursion)", async () => {
    // Simule un cached user en mémoire (déjà logué) MAIS endpoint auth
    localStorage.setItem(
      "koprogo_user",
      JSON.stringify({ id: "u-1", email: "test@example.com", role: "syndic" }),
    );

    const mockFetch = vi.fn().mockImplementation(async () => okDataResponse());
    global.fetch = mockFetch as unknown as typeof fetch;

    const { api } = await import("./api");
    await api.post("/auth/logout", {});

    // 1 seul fetch : skipAuthRefresh sur /auth/* même avec cached user
    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(mockFetch.mock.calls[0][0] as string).toContain("/auth/logout");
  });
});
