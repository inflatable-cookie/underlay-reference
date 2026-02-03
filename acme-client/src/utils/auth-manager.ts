/**
 * Authentication manager for coordinating token lifecycle.
 *
 * This module provides:
 * - Coordinated login/logout flows
 * - Automatic token refresh on 401
 * - Session initialization from cookies
 * - Logout callbacks for apps to respond to session end
 *
 * ## Usage
 *
 * ```typescript
 * import { createAuthManager } from '@acme/client';
 *
 * // Create manager (once per app instance)
 * const auth = createAuthManager({
 *   onLogout: () => {
 *     // Redirect to login page
 *     window.location.href = '/login';
 *   }
 * });
 *
 * // Login
 * const user = await auth.login({ email, password }, fetch);
 *
 * // Initialize from cookies (on page load)
 * const user = await auth.initialize(fetch);
 *
 * // Make authenticated requests
 * const token = auth.getToken();
 * const data = await someCommand(payload, fetch, token);
 *
 * // Or use the fetch wrapper with auto-refresh
 * const response = await auth.authenticatedFetch('/v1/some/endpoint', { method: 'GET' });
 *
 * // Logout
 * await auth.logout(fetch);
 * ```
 */

import type { TokenStore } from './token-store.js';
import { createTokenStore } from './token-store.js';
import type { LoginRequest, LoginResponse, LoginUser, LoginStartRequest, LoginStartResponse } from '../types/common-types.js';
import * as authCommands from '../commands/auth-commands.js';

/** Configuration for the auth manager. */
export interface AuthManagerConfig {
	/** Token store instance. If not provided, creates a new one. */
	tokenStore?: TokenStore;
	/** Callback when user is logged out (session expired, explicit logout, etc). */
	onLogout?: () => void;
	/** Buffer time (seconds) before token expiry to trigger refresh. Default: 60. */
	refreshBufferSeconds?: number;
}

/** Auth manager state. */
export interface AuthState {
	/** Whether auth has been initialized. */
	initialized: boolean;
	/** Whether we're currently initializing/refreshing. */
	loading: boolean;
	/** Current user, or null if not authenticated. */
	user: LoginUser | null;
	/** Error message if last auth operation failed. */
	error: string | null;
}

/** Auth manager interface. */
export interface AuthManager {
	/** Get the token store for subscribing to changes. */
	readonly store: TokenStore;

	/** Get current auth state. */
	getState(): AuthState;

	/** Get the current access token (or null). */
	getToken(): string | null;

	/** Check if user is authenticated (has valid token). */
	isAuthenticated(): boolean;

	/**
	 * Initialize auth state from cookies.
	 * Call this on app startup to restore session.
	 * Returns the user if session was restored, null otherwise.
	 */
	initialize(fetchFn: typeof fetch): Promise<LoginUser | null>;

	/**
	 * Login with email and password.
	 * Returns the user on success.
	 * @throws {Error} On login failure
	 */
	login(payload: LoginRequest, fetchFn: typeof fetch): Promise<LoginUser>;

	/**
	 * Start two-factor login flow.
	 * Returns whether 2FA is required and the login state ID if so.
	 * Set enforceEmailFallback: true for admin apps that require 2FA even without TOTP configured.
	 */
	loginStart(
		payload: LoginStartRequest,
		fetchFn: typeof fetch
	): Promise<LoginStartResponse>;

	/**
	 * Complete two-factor login with TOTP code.
	 * Returns the user on success.
	 */
	loginFinish(
		payload: { loginStateId: string; code: string },
		fetchFn: typeof fetch
	): Promise<LoginUser>;

	/**
	 * Logout the current user.
	 * Clears tokens and calls the onLogout callback.
	 */
	logout(fetchFn: typeof fetch): Promise<void>;

	/**
	 * Manually refresh the access token.
	 * Usually not needed - refresh happens automatically on 401.
	 * Returns the user on success, null if refresh failed.
	 */
	refresh(fetchFn: typeof fetch): Promise<LoginUser | null>;

	/**
	 * Set session from an external login response (e.g., passkey login).
	 * Use this when you've obtained a LoginResponse from a direct API call
	 * and need to update the auth manager state.
	 */
	setSession(response: LoginResponse): LoginUser;

	/**
	 * Get a token provider function for use with HTTP client.
	 * This provider auto-refreshes if token is expired.
	 */
	getTokenProvider(fetchFn: typeof fetch): () => Promise<string | null>;
}

/**
 * Create an auth manager instance.
 */
export function createAuthManager(config: AuthManagerConfig = {}): AuthManager {
	const store = config.tokenStore ?? createTokenStore();
	const refreshBuffer = config.refreshBufferSeconds ?? 60;

	let state: AuthState = {
		initialized: false,
		loading: false,
		user: null,
		error: null
	};

	// Track if we're currently refreshing to avoid concurrent refreshes
	let refreshPromise: Promise<LoginUser | null> | null = null;

	function handleLoginResponse(response: LoginResponse): LoginUser {
		// Extract expiry from response (typically in seconds)
		// The API returns expiresIn in the session, but we'll use a default if not present
		const expiresIn = 900; // 15 minutes default, matching Acme config

		// Store both access token and refresh token (hybrid approach)
		// The refresh token is also set as an httpOnly cookie by the backend,
		// but we store it in memory to support cross-origin requests where
		// SameSite=Lax cookies aren't sent.
		store.setToken(response.accessToken, expiresIn, response.refreshToken);
		state = {
			...state,
			initialized: true,
			loading: false,
			user: response.user,
			error: null
		};

		return response.user;
	}

	function handleLogout(): void {
		store.clearToken();
		state = {
			...state,
			initialized: true,
			loading: false,
			user: null,
			error: null
		};

		config.onLogout?.();
	}

	/**
	 * Attempt to refresh the access token using the stored refresh token.
	 *
	 * Uses a hybrid approach: sends the refresh token in the request body if
	 * available (for cross-origin support), otherwise relies on the httpOnly
	 * cookie (for same-origin requests after page refresh).
	 *
	 * @param fetchFn - The fetch implementation to use
	 * @param triggerLogoutCallback - If true, calls the onLogout callback on failure.
	 *   Set to false during initialization (when we don't know if user had a session).
	 *   Set to true when refreshing a known active session.
	 */
	async function doRefresh(
		fetchFn: typeof fetch,
		triggerLogoutCallback: boolean = true
	): Promise<LoginUser | null> {
		try {
			// Use stored refresh token if available (hybrid approach for cross-origin),
			// otherwise send empty string and let the backend try the httpOnly cookie
			const storedRefreshToken = store.getRefreshToken() ?? '';
			const response = await authCommands.refresh({ refreshToken: storedRefreshToken }, fetchFn);
			return handleLoginResponse(response);
		} catch (error) {
			// Refresh failed - clear local state
			store.clearToken();
			state = {
				...state,
				initialized: true,
				loading: false,
				user: null,
				error: null
			};

			// Only trigger logout callback if this was a known active session,
			// not during initial auth check (first visit with no session)
			if (triggerLogoutCallback) {
				config.onLogout?.();
			}

			return null;
		}
	}

	return {
		get store() {
			return store;
		},

		getState(): AuthState {
			return { ...state };
		},

		getToken(): string | null {
			return store.getToken();
		},

		isAuthenticated(): boolean {
			return store.getToken() !== null && !store.isExpired(0);
		},

		async initialize(fetchFn: typeof fetch): Promise<LoginUser | null> {
			if (state.initialized) {
				return state.user;
			}

			// Always attempt refresh on initialization.
			// The refresh_token is stored in an httpOnly cookie that the browser
			// will send automatically. We can't check for it directly, so we
			// optimistically try to refresh. If there's no valid refresh token,
			// the server will return an error and we'll return null (no session).
			//
			// Note: We don't rely on the logged_in cookie because in cross-origin
			// setups (frontend on localhost:40002, API on 127.0.0.1:40001), the
			// cookie is set on the API origin and not visible to document.cookie.
			//
			// We pass false for triggerLogoutCallback because this is initial auth
			// check - we don't know if the user ever had a session, so we shouldn't
			// trigger the logout callback (which might redirect to login page).

			state = { ...state, loading: true };

			// Don't trigger onLogout during initialization - this is just checking
			// for an existing session, not losing one
			const user = await doRefresh(fetchFn, false);
			return user;
		},

		async login(payload: LoginRequest, fetchFn: typeof fetch): Promise<LoginUser> {
			state = { ...state, loading: true, error: null };

			try {
				const response = await authCommands.login(payload, fetchFn);
				return handleLoginResponse(response);
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Login failed';
				state = { ...state, loading: false, error: message };
				throw error;
			}
		},

		async loginStart(
			payload: LoginStartRequest,
			fetchFn: typeof fetch
		): Promise<LoginStartResponse> {
			state = { ...state, loading: true, error: null };

			try {
				const response = await authCommands.loginStart(payload, fetchFn);

				// If login completed (no 2FA), handle the session
				if (response.session) {
					handleLoginResponse(response.session);
				} else {
					state = { ...state, loading: false };
				}

				return response;
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Login failed';
				state = { ...state, loading: false, error: message };
				throw error;
			}
		},

		async loginFinish(
			payload: { loginStateId: string; code: string },
			fetchFn: typeof fetch
		): Promise<LoginUser> {
			state = { ...state, loading: true, error: null };

			try {
				const response = await authCommands.loginFinish(payload, fetchFn);
				return handleLoginResponse(response);
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Login failed';
				state = { ...state, loading: false, error: message };
				throw error;
			}
		},

		async logout(fetchFn: typeof fetch): Promise<void> {
			state = { ...state, loading: true };

			try {
				// Send refresh token in body if available (hybrid approach),
				// otherwise rely on cookie
				const storedRefreshToken = store.getRefreshToken() ?? '';
				await authCommands.logout({ refreshToken: storedRefreshToken }, fetchFn, store.getToken() ?? undefined);
			} catch {
				// Ignore errors - we're logging out anyway
			}

			handleLogout();
		},

		async refresh(fetchFn: typeof fetch): Promise<LoginUser | null> {
			// Avoid concurrent refresh calls
			if (refreshPromise) {
				return refreshPromise;
			}

			state = { ...state, loading: true };

			// Trigger logout callback since explicit refresh implies active session
			refreshPromise = doRefresh(fetchFn, true);

			try {
				return await refreshPromise;
			} finally {
				refreshPromise = null;
			}
		},

		getTokenProvider(fetchFn: typeof fetch): () => Promise<string | null> {
			return async (): Promise<string | null> => {
				// If token is about to expire, refresh first
				if (store.isExpired(refreshBuffer)) {
					// Avoid concurrent refresh calls
					if (refreshPromise) {
						await refreshPromise;
					} else {
						// Trigger logout callback since this is a known active session
						refreshPromise = doRefresh(fetchFn, true);
						try {
							await refreshPromise;
						} finally {
							refreshPromise = null;
						}
					}
				}

				return store.getToken();
			};
		},

		setSession(response: LoginResponse): LoginUser {
			return handleLoginResponse(response);
		}
	};
}

/**
 * Default global auth manager instance.
 *
 * For browser SPAs, this singleton is typically sufficient.
 * Configure it once at app startup with `configureAuthManager()`.
 */
let defaultAuthManager: AuthManager | null = null;

/**
 * Configure the default auth manager.
 * Call this once at app startup.
 */
export function configureAuthManager(config: AuthManagerConfig = {}): AuthManager {
	defaultAuthManager = createAuthManager(config);
	return defaultAuthManager;
}

/**
 * Get the default auth manager.
 * Throws if not configured.
 */
export function getAuthManager(): AuthManager {
	if (!defaultAuthManager) {
		throw new Error(
			'Auth manager not configured. Call configureAuthManager() at app startup.'
		);
	}
	return defaultAuthManager;
}
