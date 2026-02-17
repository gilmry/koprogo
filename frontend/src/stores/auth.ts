import { writable, get } from "svelte/store";
import type { User, UserRoleSummary } from "../lib/types";
import { UserRole } from "../lib/types";
import { syncService } from "../lib/sync";
import { localDB } from "../lib/db";
import { apiEndpoint } from "../lib/config";

// Auth store
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
  refreshToken: string | null;
}

// Refresh token 5 minutes before expiry (access token expires in 15 minutes)
const TOKEN_REFRESH_INTERVAL = 10 * 60 * 1000; // 10 minutes
let refreshTimer: number | null = null;

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
    refreshToken: null,
  };

  if (typeof window !== "undefined") {
    const storedUser = localStorage.getItem("koprogo_user");
    const storedToken = localStorage.getItem("koprogo_token");
    const storedRefreshToken = localStorage.getItem("koprogo_refresh_token");

    if (storedUser && storedToken && storedRefreshToken) {
      try {
        const user = ensureUserShape(JSON.parse(storedUser));
        initialState = {
          user,
          isAuthenticated: true,
          isLoading: true, // still loading until init() completes async ops
          token: storedToken,
          refreshToken: storedRefreshToken,
        };
      } catch {
        // Invalid stored data, keep defaults
      }
    } else {
      initialState.isLoading = false;
    }
  }

  const { subscribe, set, update } = writable<AuthState>(initialState);

  const startTokenRefresh = () => {
    if (refreshTimer) {
      clearInterval(refreshTimer);
    }

    refreshTimer = window.setInterval(async () => {
      const refreshToken = localStorage.getItem("koprogo_refresh_token");
      if (refreshToken) {
        await authStore.refreshAccessToken(refreshToken);
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

    // Initialize async operations (localDB, sync, token refresh).
    // User data is already pre-populated from localStorage at store creation.
    init: async () => {
      if (typeof window !== "undefined") {
        const storedToken = localStorage.getItem("koprogo_token");
        const storedRefreshToken = localStorage.getItem(
          "koprogo_refresh_token",
        );

        if (initialState.user && storedToken && storedRefreshToken) {
          try {
            // Initialize local database
            await localDB.init();

            // Initialize sync service with token
            await syncService.initialize(storedToken);

            // Mark loading complete
            update((state) => ({ ...state, isLoading: false }));

            // Start auto-refresh
            startTokenRefresh();
          } catch (error) {
            console.error("Failed to initialize auth:", error);
            set({
              user: null,
              isAuthenticated: false,
              isLoading: false,
              token: null,
              refreshToken: null,
            });
          }
        } else {
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            token: null,
            refreshToken: null,
          });
        }
      }
    },

    // Login
    login: async (user: User, token: string, refreshToken: string) => {
      if (typeof window !== "undefined") {
        localStorage.setItem("koprogo_user", JSON.stringify(user));
        localStorage.setItem("koprogo_token", token);
        localStorage.setItem("koprogo_refresh_token", refreshToken);

        // Initialize local database
        await localDB.init();

        // Save user to local DB
        await localDB.saveUser(user);

        // Initialize sync service
        await syncService.initialize(token);

        // Start auto-refresh
        startTokenRefresh();
      }

      set({
        user,
        isAuthenticated: true,
        isLoading: false,
        token,
        refreshToken,
      });
    },

    // Logout
    logout: async () => {
      stopTokenRefresh();

      if (typeof window !== "undefined") {
        localStorage.removeItem("koprogo_user");
        localStorage.removeItem("koprogo_token");
        localStorage.removeItem("koprogo_refresh_token");

        // Clear local data
        await syncService.clearLocalData();
      }

      set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        token: null,
        refreshToken: null,
      });
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
    getToken: () => {
      if (typeof window !== "undefined") {
        return localStorage.getItem("koprogo_token");
      }
      return null;
    },

    // Refresh access token
    refreshAccessToken: async (refreshToken: string): Promise<boolean> => {
      try {
        const response = await fetch(apiEndpoint("/auth/refresh"), {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ refresh_token: refreshToken }),
        });

        if (response.ok) {
          const data = await response.json();
          const {
            token: newToken,
            refresh_token: newRefreshToken,
            user: userPayload,
          } = data;

          // Update stored tokens
          if (typeof window !== "undefined") {
            localStorage.setItem("koprogo_token", newToken);
            localStorage.setItem("koprogo_refresh_token", newRefreshToken);

            // Update sync service token
            await syncService.setToken(newToken);
          }

          // Map backend user format to frontend format
          const mappedUser: User = mapBackendUser(userPayload);

          if (typeof window !== "undefined") {
            localStorage.setItem("koprogo_user", JSON.stringify(mappedUser));
          }
          await localDB.saveUser(mappedUser);

          update((state) => ({
            ...state,
            token: newToken,
            refreshToken: newRefreshToken,
            user: mappedUser,
          }));

          return true;
        } else {
          // Token refresh failed, logout user
          console.error("Token refresh failed");
          await authStore.logout();
          if (typeof window !== "undefined") {
            window.location.href = "/login";
          }
          return false;
        }
      } catch (error) {
        console.error("Token refresh error:", error);
        await authStore.logout();
        if (typeof window !== "undefined") {
          window.location.href = "/login";
        }
        return false;
      }
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
        const {
          token: newToken,
          refresh_token: newRefreshToken,
          user: userPayload,
        } = data;

        const mappedUser: User = mapBackendUser(userPayload);

        if (typeof window !== "undefined") {
          localStorage.setItem("koprogo_token", newToken);
          localStorage.setItem("koprogo_refresh_token", newRefreshToken);
          localStorage.setItem("koprogo_user", JSON.stringify(mappedUser));
        }

        await syncService.setToken(newToken);
        await localDB.saveUser(mappedUser);

        startTokenRefresh();

        update((state) => ({
          ...state,
          token: newToken,
          refreshToken: newRefreshToken,
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
      const token = authStore.getToken();
      if (!token) {
        return false;
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
          const refreshToken = localStorage.getItem("koprogo_refresh_token");
          if (refreshToken) {
            const refreshed = await authStore.refreshAccessToken(refreshToken);
            if (refreshed) {
              return true;
            }
          }

          await authStore.logout();
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
