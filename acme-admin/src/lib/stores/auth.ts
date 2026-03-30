/**
 * Client-side auth store for Acme Admin SPA.
 *
 * This store wraps the api-client AuthManager and provides reactive auth state
 * for Svelte components. It handles:
 * - Login/logout flows
 * - Session initialization from cookies
 * - Automatic token refresh
 * - Redirect on auth failure
 *
 * Usage:
 *   import { auth } from '$lib/stores/auth';
 *
 *   // In components
 *   {#if $auth.loading}
 *     <LoadingSpinner />
 *   {:else if $auth.user}
 *     <Dashboard user={$auth.user} />
 *   {:else}
 *     <LoginPage />
 *   {/if}
 *
 *   // Login
 *   await auth.login(email, password);
 *
 *   // Logout
 *   await auth.logout();
 */

import { writable, derived, get } from 'svelte/store';
import { goto } from '$app/navigation';
import { browser } from '$app/environment';
import {
  createAuthManager,
  type LoginUser,
  type LoginResponse
} from '@api-client';

export interface AuthState {
  /** Whether auth has been initialized (checked cookies). */
  initialized: boolean;
  /** Whether we're currently loading (initializing, logging in, etc). */
  loading: boolean;
  /** The authenticated user, or null if not logged in. */
  user: LoginUser | null;
  /** Error message from last auth operation, or null. */
  error: string | null;
}

export interface AuthStore {
  subscribe: (run: (state: AuthState) => void) => () => void;

  /** Initialize auth state by checking for existing session. */
  initialize: () => Promise<void>;

  /** Log in with email and password. Returns user on success, throws on failure. */
  login: (email: string, password: string) => Promise<LoginUser>;

  /**
   * Start two-factor login. Returns whether 2FA is required.
   * If 2FA is required, call loginFinish with the code.
   * If isEmailVerification is true, the code was sent via email (not TOTP/passkey).
   */
  loginStart: (email: string, password: string) => Promise<{
    requiresTwoFactor: boolean;
    isEmailVerification?: boolean;
    loginStateId?: string;
    user?: LoginUser;
  }>;

  /** Complete two-factor login with TOTP code. */
  loginFinish: (loginStateId: string, code: string) => Promise<LoginUser>;

  /** Log out the current user. */
  logout: () => Promise<void>;

  /** Get a token provider for making authenticated API calls. */
  getTokenProvider: () => () => Promise<string | null>;

  /** Get the current access token (synchronous). */
  getToken: () => string | null;

  /** Set session from an external login response (e.g., passkey login). */
  setSession: (response: LoginResponse) => LoginUser;

  /**
   * Get a refresh handler for use with HTTP clients.
   * This allows automatic token refresh on 401 errors.
   */
  getRefreshHandler: () => (fetchFn: typeof fetch) => Promise<string | null>;
}

function createAuthStore(): AuthStore {
  // Create auth manager with logout callback
  const manager = createAuthManager({
    onLogout: () => {
      if (browser) {
        goto('/login');
      }
    }
  });

  // Internal state store
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

    async loginStart(email: string, password: string) {
      state.update(s => ({ ...s, loading: true, error: null }));

      try {
        // Admin login enforces email fallback for users without 2FA
        const result = await manager.loginStart({ email, password, enforceEmailFallback: true }, fetch);

        if (result.session) {
          // Login complete (no 2FA)
          const user = result.session.user;
          state.update(s => ({
            ...s,
            initialized: true,
            loading: false,
            user,
            error: null
          }));
          return { requiresTwoFactor: false, user };
        }

        // 2FA required
        state.update(s => ({ ...s, loading: false }));
        return {
          requiresTwoFactor: true,
          isEmailVerification: result.isEmailVerification ?? false,
          loginStateId: result.loginStateId ?? undefined
        };
      } catch (e) {
        const message = e instanceof Error ? e.message : 'Login failed';
        state.update(s => ({ ...s, loading: false, error: message }));
        throw e;
      }
    },

    async loginFinish(loginStateId: string, code: string): Promise<LoginUser> {
      state.update(s => ({ ...s, loading: true, error: null }));

      try {
        const user = await manager.loginFinish({ loginStateId, code }, fetch);
        state.update(s => ({
          ...s,
          initialized: true,
          loading: false,
          user,
          error: null
        }));
        return user;
      } catch (e) {
        const message = e instanceof Error ? e.message : 'Verification failed';
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

    getTokenProvider() {
      return manager.getTokenProvider(fetch);
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

// Singleton auth store - preserved across HMR in development
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

// Derived stores for convenience
export const isAuthenticated = derived(auth, $auth => $auth.user !== null);
export const currentUser = derived(auth, $auth => $auth.user);
export const authLoading = derived(auth, $auth => $auth.loading || !$auth.initialized);
