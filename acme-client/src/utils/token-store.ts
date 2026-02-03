/**
 * Token store for managing authentication tokens.
 *
 * This provides a Svelte-compatible store for access tokens that:
 * - Stores tokens in memory (NOT localStorage for security)
 * - Provides reactive access via Svelte's store contract
 * - Tracks token expiry for proactive refresh
 *
 * ## Usage
 *
 * ```typescript
 * import { tokenStore } from '@acme/client';
 *
 * // Set token after login
 * tokenStore.setToken(response.accessToken, 900); // 900 seconds = 15 min
 *
 * // Get current token
 * const token = tokenStore.getToken();
 *
 * // Subscribe to changes (Svelte store contract)
 * const unsubscribe = tokenStore.subscribe((state) => {
 *   console.log('Auth state:', state.accessToken ? 'logged in' : 'logged out');
 * });
 *
 * // Clear on logout
 * tokenStore.clearToken();
 * ```
 */

/** Token state held by the store. */
export interface TokenState {
	/** Current access token, or null if not authenticated. */
	accessToken: string | null;
	/** Unix timestamp (ms) when the access token expires, or null. */
	expiresAt: number | null;
	/** Current refresh token, or null if not authenticated. */
	refreshToken: string | null;
}

/** Subscriber callback for Svelte store contract. */
type Subscriber = (state: TokenState) => void;

/** Unsubscribe function returned by subscribe(). */
type Unsubscriber = () => void;

/** Svelte-compatible store interface. */
export interface TokenStore {
	/** Subscribe to state changes (Svelte store contract). */
	subscribe(subscriber: Subscriber): Unsubscriber;
	/** Set the access token with expiry, and optionally the refresh token. */
	setToken(accessToken: string, expiresInSeconds: number, refreshToken?: string): void;
	/** Clear the token (logout). */
	clearToken(): void;
	/** Get the current access token synchronously. */
	getToken(): string | null;
	/** Get the current refresh token synchronously. */
	getRefreshToken(): string | null;
	/** Check if the token is expired or will expire soon. */
	isExpired(bufferSeconds?: number): boolean;
	/** Get the current state synchronously. */
	getState(): TokenState;
}

/**
 * Create a token store instance.
 *
 * This is a factory function so each app instance gets its own store.
 * For SSR, each request should have its own store to avoid cross-request leakage.
 */
export function createTokenStore(): TokenStore {
	let state: TokenState = {
		accessToken: null,
		expiresAt: null,
		refreshToken: null
	};

	const subscribers = new Set<Subscriber>();

	function notify(): void {
		for (const subscriber of subscribers) {
			subscriber(state);
		}
	}

	return {
		subscribe(subscriber: Subscriber): Unsubscriber {
			subscribers.add(subscriber);
			// Immediately call with current state (Svelte store contract)
			subscriber(state);
			return () => {
				subscribers.delete(subscriber);
			};
		},

		setToken(accessToken: string, expiresInSeconds: number, refreshToken?: string): void {
			state = {
				accessToken,
				expiresAt: Date.now() + expiresInSeconds * 1000,
				// Keep existing refresh token if not provided (e.g., during token refresh)
				refreshToken: refreshToken ?? state.refreshToken
			};
			notify();
		},

		clearToken(): void {
			state = {
				accessToken: null,
				expiresAt: null,
				refreshToken: null
			};
			notify();
		},

		getToken(): string | null {
			return state.accessToken;
		},

		getRefreshToken(): string | null {
			return state.refreshToken;
		},

		isExpired(bufferSeconds = 30): boolean {
			if (!state.accessToken || !state.expiresAt) {
				return true;
			}
			// Consider expired if within buffer of expiry
			return Date.now() >= state.expiresAt - bufferSeconds * 1000;
		},

		getState(): TokenState {
			return { ...state };
		}
	};
}

/**
 * Default global token store instance.
 *
 * For browser SPAs, this singleton is typically sufficient.
 * For SSR, use `createTokenStore()` per-request instead.
 */
export const tokenStore = createTokenStore();
