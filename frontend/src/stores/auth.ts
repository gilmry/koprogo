import { writable } from "svelte/store";
import type { User } from "../lib/types";
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

const createAuthStore = () => {
  const { subscribe, set, update } = writable<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
    token: null,
    refreshToken: null,
  });

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

    // Initialize from localStorage and IndexedDB
    init: async () => {
      if (typeof window !== "undefined") {
        const storedUser = localStorage.getItem("koprogo_user");
        const storedToken = localStorage.getItem("koprogo_token");
        const storedRefreshToken = localStorage.getItem(
          "koprogo_refresh_token",
        );

        if (storedUser && storedToken && storedRefreshToken) {
          try {
            const user = JSON.parse(storedUser);

            // Initialize local database
            await localDB.init();

            // Initialize sync service with token
            await syncService.initialize(storedToken);

            set({
              user,
              isAuthenticated: true,
              isLoading: false,
              token: storedToken,
              refreshToken: storedRefreshToken,
            });

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
            user,
          } = data;

          // Update stored tokens
          if (typeof window !== "undefined") {
            localStorage.setItem("koprogo_token", newToken);
            localStorage.setItem("koprogo_refresh_token", newRefreshToken);

            // Update sync service token
            await syncService.setToken(newToken);
          }

          // Map backend user format to frontend format
          const mappedUser: User = {
            id: user.id,
            email: user.email,
            firstName: user.first_name,
            lastName: user.last_name,
            role: user.role,
            organizationId: user.organization_id,
            buildingIds: [],
          };

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
          const user = await response.json();

          // Map backend user format to frontend format
          const mappedUser: User = {
            id: user.id,
            email: user.email,
            firstName: user.first_name,
            lastName: user.last_name,
            role: user.role,
            organizationId: user.organization_id,
            buildingIds: [],
          };

          update((state) => ({
            ...state,
            user: mappedUser,
            isAuthenticated: true,
          }));

          return true;
        } else {
          // Try to refresh token
          const refreshToken = localStorage.getItem("koprogo_refresh_token");
          if (refreshToken) {
            return await authStore.refreshAccessToken(refreshToken);
          }

          // No refresh token, logout
          await authStore.logout();
          return false;
        }
      } catch (error) {
        console.error("Session validation error:", error);
        return false;
      }
    },
  };

  return authStore;
};

export const authStore = createAuthStore();
