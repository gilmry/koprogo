import { writable } from 'svelte/store';
import type { User } from '../lib/types';
import { syncService } from '../lib/sync';
import { localDB } from '../lib/db';

// Auth store
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
}

const createAuthStore = () => {
  const { subscribe, set, update } = writable<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
    token: null,
  });

  return {
    subscribe,

    // Initialize from localStorage and IndexedDB
    init: async () => {
      if (typeof window !== 'undefined') {
        const storedUser = localStorage.getItem('koprogo_user');
        const storedToken = localStorage.getItem('koprogo_token');

        if (storedUser && storedToken) {
          try {
            const user = JSON.parse(storedUser);

            // Initialize local database
            await localDB.init();

            // Initialize sync service with token
            await syncService.initialize(storedToken);

            set({ user, isAuthenticated: true, isLoading: false, token: storedToken });
          } catch (error) {
            console.error('Failed to initialize auth:', error);
            set({ user: null, isAuthenticated: false, isLoading: false, token: null });
          }
        } else {
          set({ user: null, isAuthenticated: false, isLoading: false, token: null });
        }
      }
    },

    // Login
    login: async (user: User, token: string) => {
      if (typeof window !== 'undefined') {
        localStorage.setItem('koprogo_user', JSON.stringify(user));
        localStorage.setItem('koprogo_token', token);

        // Initialize local database
        await localDB.init();

        // Save user to local DB
        await localDB.saveUser(user);

        // Initialize sync service
        await syncService.initialize(token);
      }

      set({ user, isAuthenticated: true, isLoading: false, token });
    },

    // Logout
    logout: async () => {
      if (typeof window !== 'undefined') {
        localStorage.removeItem('koprogo_user');
        localStorage.removeItem('koprogo_token');

        // Clear local data
        await syncService.clearLocalData();
      }

      set({ user: null, isAuthenticated: false, isLoading: false, token: null });
    },

    // Update user
    updateUser: async (user: User) => {
      if (typeof window !== 'undefined') {
        localStorage.setItem('koprogo_user', JSON.stringify(user));
        await localDB.saveUser(user);
      }
      update(state => ({ ...state, user }));
    },

    // Get token
    getToken: () => {
      if (typeof window !== 'undefined') {
        return localStorage.getItem('koprogo_token');
      }
      return null;
    },
  };
};

export const authStore = createAuthStore();
