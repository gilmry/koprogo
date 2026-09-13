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

  it("@edge — isLoading=true (init en cours) sans cache user → déclenche refresh quand même", async () => {
    // Cas réel observé après "Clear site data" du navigateur :
    // localStorage vide MAIS cookie HttpOnly refresh encore présent côté
    // browser → silent-refresh peut réussir → API call doit attendre.
    // authStore.isLoading=true au moment du 1er appel (état initial).

    const mockFetch = vi.fn().mockImplementation(async (url: string) => {
      if (url.includes("/auth/refresh")) return okRefreshResponse();
      return okDataResponse();
    });
    global.fetch = mockFetch as unknown as typeof fetch;

    const { api } = await import("./api");
    // Note : pas de localStorage.setItem("koprogo_user", ...) avant l'appel.
    // authStore.isLoading=true par défaut (état initial) → my fix declenche
    // refreshAccessToken() même sans cache. Le refresh peuple le cache et
    // le token → la requête cible part avec Authorization.
    await api.get("/notification-preferences/u-1");

    // 2 fetchs : 1 pour refresh (déclenché par isLoading=true), 1 pour la cible
    expect(mockFetch).toHaveBeenCalledTimes(2);
    expect(mockFetch.mock.calls[0][0] as string).toContain("/auth/refresh");
    const dataCallHeaders = (mockFetch.mock.calls[1][1] as RequestInit)
      .headers as Headers;
    expect(dataCallHeaders.get("Authorization")).toBe(
      "Bearer fresh-token-from-refresh",
    );
  });
});

/**
 * Régression #782 — le corps de la réponse d'erreur doit survivre au `throw`.
 *
 * `apiFetch` levait `new Error(errorMessage)` : une exception NUE. Ni le code
 * HTTP, ni `details`, ni le corps ne survivaient. Trois actions d'écriture ont
 * échoué en silence pendant cinq recettes — `acp_id`, `recipient_owner_ids`,
 * `total_voting_power` — parce que le nom du champ fautif, que le serveur
 * donne pourtant, n'atteignait jamais l'écran.
 *
 * Conséquence moins visible et plus grave : `lib/utils/conformity.ts` et
 * `lib/utils/meetingCompletion.ts` interrogent `err.details` et ne pouvaient
 * JAMAIS correspondre. Deux fonctionnalités mortes sous des tests verts, parce
 * que ces tests fabriquaient l'objet d'erreur à la main sans jamais exercer
 * `apiFetch`. C'est précisément ce trou que ces cas-ci comblent.
 */
describe("apiFetch — le corps d'une réponse d'erreur survit au throw (#782)", () => {
  /// Ce que le serveur répond réellement, relevé en recette le 2026-09-06.
  const reponse400 = {
    error: "Invalid request body",
    details:
      "Json deserialize error: missing field `acp_id` at line 1 column 192",
  };

  const repondre = (statut: number, corps: unknown) =>
    ({
      ok: false,
      status: statut,
      text: async () => JSON.stringify(corps),
    }) as any;

  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    if (typeof window !== "undefined") localStorage.clear();
  });

  it("lève une ApiError qui porte le statut, le détail et le corps", async () => {
    global.fetch = vi.fn().mockImplementation(async (url: string) => {
      if (url.includes("/auth/refresh")) return okRefreshResponse();
      return repondre(400, reponse400);
    }) as unknown as typeof fetch;

    const { api, ApiError } = await import("./api");
    const erreur = await api.post("/buildings", {}).catch((e: any) => e);

    expect(erreur).toBeInstanceOf(ApiError);
    expect(erreur.status).toBe(400);
    expect(erreur.message).toBe("Invalid request body");
    // La propriété que les deux extracteurs existants interrogent.
    expect(erreur.details).toContain("acp_id");
  });

  it("affiche le nom du champ fautif sous le titre du toast", async () => {
    global.fetch = vi.fn().mockImplementation(async (url: string) => {
      if (url.includes("/auth/refresh")) return okRefreshResponse();
      return repondre(400, reponse400);
    }) as unknown as typeof fetch;

    const { api } = await import("./api");
    const { toast } = await import("../stores/toast");
    await api.post("/buildings", {}).catch(() => undefined);

    expect(toast.error).toHaveBeenCalledWith(
      "Invalid request body",
      expect.any(Number),
      expect.stringContaining("acp_id"),
    );
  });

  it("n'invente pas de détail quand le serveur n'en donne pas", async () => {
    global.fetch = vi.fn().mockImplementation(async (url: string) => {
      if (url.includes("/auth/refresh")) return okRefreshResponse();
      return repondre(400, { error: "Trop court" });
    }) as unknown as typeof fetch;

    const { api } = await import("./api");
    const { toast } = await import("../stores/toast");
    await api.post("/x", {}).catch(() => undefined);

    expect(toast.error).toHaveBeenCalledWith(
      "Trop court",
      expect.any(Number),
      undefined,
    );
  });

  /// Les erreurs métier typées gardent leur forme d'objet : elles ont leurs
  /// propres gestionnaires (`conformity.ts`, `meetingCompletion.ts`) et ne
  /// doivent pas être aplaties en chaîne.
  it("conserve un détail structuré tel quel", async () => {
    global.fetch = vi.fn().mockImplementation(async (url: string) => {
      if (url.includes("/auth/refresh")) return okRefreshResponse();
      return repondre(422, {
        error: "Immeuble non conforme",
        kind: "building_not_conformant",
        details: { code: "BUILDING_NOT_CONFORMANT", units_delta: 2 },
      });
    }) as unknown as typeof fetch;

    const { api } = await import("./api");
    const erreur = await api.post("/expenses", {}).catch((e: any) => e);

    expect(erreur.details).toEqual({
      code: "BUILDING_NOT_CONFORMANT",
      units_delta: 2,
    });
  });
});
