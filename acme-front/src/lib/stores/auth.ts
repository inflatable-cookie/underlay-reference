/**
 * Client-side auth store for Acme Front SPA.
 *
 * This store wraps the api-client AuthManager and provides reactive auth state
 * for Svelte components.
 */

import { writable, derived, get } from 'svelte/store';
import { goto } from '$app/navigation';
import { browser } from '$app/environment';
import {
  createAuthManager,
} from '@api-client/utils/auth-manager.js';
import type {
  LoginUser,
  LoginResponse
} from '@api-client/types/common-types.js';

export interface AuthState {
  initialized: boolean;
  loading: boolean;
  user: LoginUser | null;
  error: string | null;
}

export interface AuthStore {
  subscribe: (run: (state: AuthState) => void) => () => void;
  initialize: () => Promise<void>;
  login: (email: string, password: string) => Promise<LoginUser>;
  register: (email: string, password: string, displayName: string) => Promise<LoginUser>;
  logout: () => Promise<void>;
  getToken: () => string | null;
  setSession: (response: LoginResponse) => LoginUser;
  getRefreshHandler: () => (fetchFn: typeof fetch) => Promise<string | null>;
}

function createAuthStore(): AuthStore {
  const manager = createAuthManager({
    onLogout: () => {
      if (browser) {
        goto('/login');
      }
    }
  });

  const state = writable<AuthState>({
    initialized: false,
    loading: false,
    user: null,
    error: null
  });

  return {
    subscribe: state.subscribe,

    async initialize() {
      const current = get(state);
      if (current.initialized) return;

      state.update(s => ({ ...s, loading: true }));

      try {
        const user = await manager.initialize(fetch);
        state.set({
          initialized: true,
          loading: false,
          user,
          error: null
        });
      } catch {
        state.set({
          initialized: true,
          loading: false,
          user: null,
          error: null
        });
      }
    },

    async login(email: string, password: string): Promise<LoginUser> {
      state.update(s => ({ ...s, loading: true, error: null }));

      try {
        const user = await manager.login({ email, password }, fetch);
        state.update(s => ({
          ...s,
          initialized: true,
          loading: false,
          user,
          error: null
        }));
        return user;
      } catch (e) {
        const message = e instanceof Error ? e.message : 'Login failed';
        state.update(s => ({ ...s, loading: false, error: message }));
        throw e;
      }
    },

    async register(email: string, password: string, displayName: string): Promise<LoginUser> {
      state.update(s => ({ ...s, loading: true, error: null }));

      try {
        const user = await manager.register({ email, password, displayName }, fetch);
        state.update(s => ({
          ...s,
          initialized: true,
          loading: false,
          user,
          error: null
        }));
        return user;
      } catch (e) {
        const message = e instanceof Error ? e.message : 'Registration failed';
        state.update(s => ({ ...s, loading: false, error: message }));
        throw e;
      }
    },

    async logout(): Promise<void> {
      state.update(s => ({ ...s, loading: true }));

      try {
        await manager.logout(fetch);
      } catch {
        // Ignore errors - logout locally anyway
      }

      state.update(s => ({
        ...s,
        initialized: true,
        loading: false,
        user: null,
        error: null
      }));

      await goto('/login');
    },

    getToken() {
      return manager.getToken();
    },

    setSession(response: LoginResponse): LoginUser {
      const user = manager.setSession(response);
      state.update(s => ({
        ...s,
        initialized: true,
        loading: false,
        user,
        error: null
      }));
      return user;
    },

    getRefreshHandler() {
      return async (fetchFn: typeof fetch): Promise<string | null> => {
        const user = await manager.refresh(fetchFn);
        if (user) {
          state.update(s => ({ ...s, user }));
          return manager.getToken();
        }
        return null;
      };
    }
  };
}

// Singleton auth store
function getOrCreateAuthStore(): AuthStore {
  if (import.meta.hot) {
    if (!import.meta.hot.data.authStore) {
      import.meta.hot.data.authStore = createAuthStore();
    }
    return import.meta.hot.data.authStore;
  }
  return createAuthStore();
}

export const auth = getOrCreateAuthStore();

// Derived stores
export const isAuthenticated = derived(auth, $auth => $auth.user !== null);
export const currentUser = derived(auth, $auth => $auth.user);
export const authLoading = derived(auth, $auth => $auth.loading || !$auth.initialized);
