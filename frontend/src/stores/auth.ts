import { writable, get } from "svelte/store";
import type { User, UserRoleSummary } from "../lib/types";
import { UserRole } from "../lib/types";
import { syncService } from "../lib/sync";
import { localDB } from "../lib/db";
import { apiEndpoint } from "../lib/config";
import {
  getAccessToken,
  setAccessToken,
  clearAccessToken,
} from "../lib/accessToken";

// Auth store
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  // Access token reflété ici pour la réactivité UI. Source de vérité =
  // mémoire (`lib/accessToken`). JAMAIS persisté (WP-FE1). Le refresh
  // token n'est plus côté JS : cookie HttpOnly géré par le backend.
  token: string | null;
}

// Refresh token 5 minutes before expiry (access token expires in 15 minutes)
const TOKEN_REFRESH_INTERVAL = 10 * 60 * 1000; // 10 minutes
let refreshTimer: number | null = null;

// In-flight dedup pour `refreshAccessToken()`. Plusieurs composants
// `client:load` (RouteGuard + Navigation + page) mountent en parallèle et
// appellent chacun `authStore.init()` → sans dedup, N POST /auth/refresh
// concurrents réutilisent le même cookie single-use → le backend rote sur
// le 1er et rejette les suivants → clearSession() → access token vidé →
// les api.get() suivants tombent en 401 (h1 caché, listes vides, etc.).
// Cf. issue #550 (≥ 12 Playwright fails sur ce pattern API-create→UI-list).
let inflightRefresh: Promise<boolean> | null = null;

const normalizeRole = (role: string | undefined | null): UserRole => {
  switch (role) {
    case UserRole.SUPERADMIN:
    case "superadmin":
      return UserRole.SUPERADMIN;
    case UserRole.SYNDIC:
    case "syndic":
      return UserRole.SYNDIC;
    case UserRole.ACCOUNTANT:
    case "accountant":
      return UserRole.ACCOUNTANT;
    case UserRole.OWNER:
    case "owner":
      return UserRole.OWNER;
    default:
      return UserRole.OWNER;
  }
};

const mapRoleSummaryFromAny = (role: any): UserRoleSummary => {
  const rawOrg =
    role?.organizationId ?? role?.organization_id ?? role?.organization ?? null;
  const organizationId =
    rawOrg === null || rawOrg === undefined || rawOrg === ""
      ? undefined
      : String(rawOrg);

  return {
    id: String(role?.id ?? role?.role_id ?? ""),
    role: normalizeRole(role?.role ?? role?.name),
    organizationId,
    isPrimary: Boolean(role?.isPrimary ?? role?.is_primary),
  };
};

const mapBackendUser = (user: any): User => {
  let roles = (user.roles ?? []).map(mapRoleSummaryFromAny);
  let activeRole = user.active_role
    ? mapRoleSummaryFromAny(user.active_role)
    : undefined;

  if (roles.length === 0) {
    const fallbackRole = normalizeRole(user.role ?? user.active_role?.role);
    roles = [
      {
        id: String(user.active_role?.id ?? ""),
        role: fallbackRole,
        organizationId:
          user.organization_id ??
          user.organizationId ??
          user.active_role?.organization_id ??
          user.active_role?.organizationId ??
          undefined,
        isPrimary: true,
      },
    ];
  }

  if (!activeRole) {
    activeRole =
      roles.find((role: UserRoleSummary) => role.isPrimary) ?? roles[0];
  }

  roles.sort(
    (a: UserRoleSummary, b: UserRoleSummary) =>
      Number(b.isPrimary) - Number(a.isPrimary),
  );

  return {
    id: user.id,
    email: user.email,
    first_name: user.first_name ?? user.first_name ?? "",
    last_name: user.last_name ?? user.last_name ?? "",
    role: activeRole?.role ?? normalizeRole(user.role),
    organizationId:
      activeRole?.organizationId ??
      user.organization_id ??
      user.organizationId ??
      undefined,
    buildingIds: user.buildingIds ?? [],
    is_active: user.is_active ?? true,
    created_at: user.created_at,
    roles,
    activeRole,
  };
};

const ensureUserShape = (user: any): User => {
  try {
    return mapBackendUser(user);
  } catch (error) {
    console.error("Failed to normalize stored user", error);
    return {
      id: user.id ?? "",
      email: user.email ?? "",
      first_name: user.first_name ?? "",
      last_name: user.last_name ?? "",
      role: normalizeRole(user.role),
      organizationId: user.organizationId,
      buildingIds: user.buildingIds ?? [],
      roles: user.roles ?? [],
      activeRole: user.activeRole,
    } as User;
  }
};

const createAuthStore = () => {
  // Pre-populate from localStorage synchronously so page scripts
  // can access user data immediately via get(authStore)
  let initialState: AuthState = {
    user: null,
    isAuthenticated: false,
    isLoading: true,
    token: null,
  };

  // WP-FE1 : aucun token en localStorage. `koprogo_user` est un cache
  // d'affichage NON sensible (peinture instantanée), jamais une preuve
  // d'authentification : `init()` fait un silent-refresh via le cookie
  // HttpOnly pour obtenir un access token frais et confirmer la session.
  if (typeof window !== "undefined") {
    const storedUser = localStorage.getItem("koprogo_user");
    if (storedUser) {
      try {
        initialState.user = ensureUserShape(JSON.parse(storedUser));
        // isAuthenticated reste false tant que le silent-refresh n'a pas
        // (re)produit un access token : pas de session sans token.
      } catch {
        // Invalid cached user, ignore
      }
    }
  }

  const { subscribe, set, update } = writable<AuthState>(initialState);

  const startTokenRefresh = () => {
    if (refreshTimer) {
      clearInterval(refreshTimer);
    }

    refreshTimer = window.setInterval(async () => {
      // Silent-refresh périodique via le cookie HttpOnly (aucun token JS).
      const ok = await authStore.refreshAccessToken();
      if (!ok && typeof window !== "undefined") {
        window.location.href = "/login";
      }
    }, TOKEN_REFRESH_INTERVAL);
  };

  const stopTokenRefresh = () => {
    if (refreshTimer) {
      clearInterval(refreshTimer);
      refreshTimer = null;
    }
  };

  const authStore = {
    subscribe,

    // Initialize: silent-refresh via le cookie HttpOnly pour (re)produire
    // un access token en mémoire après un reload (aucun token persisté).
    init: async () => {
      if (typeof window === "undefined") {
        return;
      }

      const refreshed = await authStore.refreshAccessToken();
      if (refreshed) {
        try {
          await localDB.init();
          const token = getAccessToken();
          if (token) {
            await syncService.initialize(token);
          }
          startTokenRefresh();
          update((state) => ({ ...state, isLoading: false }));
        } catch (error) {
          console.error("Failed to initialize auth:", error);
          await authStore.clearSession();
        }
      } else {
        // Pas de cookie valide → session absente (pas une erreur).
        await authStore.clearSession();
      }
    },

    // Login: access token en mémoire ; le refresh token est déjà posé
    // par le backend dans un cookie HttpOnly (réponse de /auth/login).
    login: async (user: User, token: string) => {
      setAccessToken(token);

      if (typeof window !== "undefined") {
        // Cache d'affichage non sensible uniquement (pas de credential).
        localStorage.setItem("koprogo_user", JSON.stringify(user));
        await localDB.init();
        await localDB.saveUser(user);
        await syncService.initialize(token);
        startTokenRefresh();
      }

      set({
        user,
        isAuthenticated: true,
        isLoading: false,
        token,
      });
    },

    // Clear session côté client (mémoire + cache user + sync + state).
    // N'appelle PAS le backend (utilisé quand le cookie est déjà invalide).
    clearSession: async () => {
      stopTokenRefresh();
      clearAccessToken();
      if (typeof window !== "undefined") {
        localStorage.removeItem("koprogo_user");
        await syncService.clearLocalData();
      }
      set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        token: null,
      });
    },

    // Logout: révocation serveur (expire le cookie + invalide les refresh)
    // puis nettoyage client. Best-effort sur le réseau.
    logout: async () => {
      const token = getAccessToken();
      if (typeof window !== "undefined" && token) {
        try {
          await fetch(apiEndpoint("/auth/logout"), {
            method: "POST",
            credentials: "include",
            headers: { Authorization: `Bearer ${token}` },
          });
        } catch {
          // best-effort : on nettoie le client quoi qu'il arrive
        }
      }
      await authStore.clearSession();
    },

    // Update user
    updateUser: async (user: User) => {
      if (typeof window !== "undefined") {
        localStorage.setItem("koprogo_user", JSON.stringify(user));
        await localDB.saveUser(user);
      }
      update((state) => ({ ...state, user }));
    },

    // Get token
    // Access token courant (mémoire seule, WP-FE1).
    getToken: () => getAccessToken(),

    // Silent-refresh : POST /auth/refresh sans corps ; le refresh token
    // est lu côté serveur dans le cookie HttpOnly (credentials:"include").
    // Aucun token n'est jamais lu/écrit en localStorage.
    refreshAccessToken: async (): Promise<boolean> => {
      // Dedup in-flight : un seul POST /auth/refresh à la fois (#550).
      if (inflightRefresh) return inflightRefresh;

      inflightRefresh = (async (): Promise<boolean> => {
        try {
          const response = await fetch(apiEndpoint("/auth/refresh"), {
            method: "POST",
            credentials: "include",
          });

          if (response.ok) {
            const data = await response.json();
            // WP-FE1 : la réponse ne contient PLUS de refresh_token (cookie
            // roté côté backend). Seul l'access token revient au JS.
            const { token: newToken, user: userPayload } = data;

            setAccessToken(newToken);

            const mappedUser: User = mapBackendUser(userPayload);

            if (typeof window !== "undefined") {
              localStorage.setItem("koprogo_user", JSON.stringify(mappedUser));
              // Cache local best-effort : NE DOIT JAMAIS faire échouer le
              // refresh. `init()` appelle refreshAccessToken() AVANT
              // `localDB.init()` ; la persistance locale est rattrapée juste
              // après (init → localDB.init → syncService.initialize). Auth ≠
              // couche de cache. (#548 / WP-D1 — ripple FE1 JWT→cookie.)
              try {
                await syncService.setToken(newToken);
                await localDB.saveUser(mappedUser);
              } catch (cacheErr) {
                console.warn(
                  "Local cache not ready during token refresh (non-fatal):",
                  cacheErr,
                );
              }
            }

            update((state) => ({
              ...state,
              token: newToken,
              user: mappedUser,
              isAuthenticated: true,
            }));

            return true;
          }

          // Cookie absent/expiré/révoqué → session client nettoyée.
          // (Pas d'appel backend logout : le cookie est déjà invalide.)
          await authStore.clearSession();
          return false;
        } catch (error) {
          console.error("Token refresh error:", error);
          await authStore.clearSession();
          return false;
        } finally {
          inflightRefresh = null;
        }
      })();

      return inflightRefresh;
    },

    switchRole: async (roleId: string): Promise<boolean> => {
      const currentState = get(authStore);
      const currentUser = currentState.user;
      if (!currentUser) {
        return false;
      }

      const token = currentState.token ?? authStore.getToken();
      if (!token) {
        return false;
      }

      try {
        const response = await fetch(apiEndpoint("/auth/switch-role"), {
          method: "POST",
          // credentials:"include" → le cookie refresh roté est stocké.
          credentials: "include",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({ role_id: roleId }),
        });

        if (!response.ok) {
          const errorData = await response.json().catch(() => ({}));
          console.error("Switch role failed", errorData);
          return false;
        }

        const data = await response.json();
        // WP-FE1 : pas de refresh_token dans le corps (cookie HttpOnly).
        const { token: newToken, user: userPayload } = data;

        setAccessToken(newToken);

        const mappedUser: User = mapBackendUser(userPayload);

        if (typeof window !== "undefined") {
          localStorage.setItem("koprogo_user", JSON.stringify(mappedUser));
        }

        await syncService.setToken(newToken);
        await localDB.saveUser(mappedUser);

        startTokenRefresh();

        update((state) => ({
          ...state,
          token: newToken,
          user: mappedUser,
        }));

        return true;
      } catch (error) {
        console.error("Switch role error", error);
        return false;
      }
    },

    // Validate current session
    validateSession: async (): Promise<boolean> => {
      let token = authStore.getToken();
      if (!token) {
        // Pas d'access token en mémoire (reload) : tenter le silent-refresh
        // via le cookie HttpOnly avant de conclure à l'absence de session.
        const refreshed = await authStore.refreshAccessToken();
        if (!refreshed) {
          return false;
        }
        token = authStore.getToken();
        if (!token) {
          return false;
        }
      }

      try {
        const response = await fetch(apiEndpoint("/auth/me"), {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });

        if (response.ok) {
          const userPayload = await response.json();
          const mappedUser: User = mapBackendUser(userPayload);

          // Only update if user data has actually changed
          const currentState = get(authStore);
          const currentUser = currentState.user;

          // Compare user IDs and roles to avoid unnecessary updates
          const hasChanged =
            !currentUser ||
            currentUser.id !== mappedUser.id ||
            currentUser.role !== mappedUser.role ||
            currentUser.email !== mappedUser.email;

          if (hasChanged) {
            if (typeof window !== "undefined") {
              localStorage.setItem("koprogo_user", JSON.stringify(mappedUser));
            }
            await localDB.saveUser(mappedUser);

            update((state) => ({
              ...state,
              user: mappedUser,
              isAuthenticated: true,
            }));
          }

          return true;
        }

        if (response.status === 401) {
          // Access token expiré : tenter un silent-refresh via le cookie.
          const refreshed = await authStore.refreshAccessToken();
          if (refreshed) {
            return true;
          }

          await authStore.clearSession();
          return false;
        }

        console.warn(
          "Session validation received non-OK response",
          response.status,
        );

        return true;
      } catch (error) {
        console.error("Session validation error:", error);
        return true;
      }
    },
  };

  return authStore;
};

export const authStore = createAuthStore();
export const mapUserFromBackend = mapBackendUser;
export const mapRoleSummary = mapRoleSummaryFromAny;
