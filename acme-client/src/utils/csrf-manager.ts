/**
 * CSRF token manager for automatic token handling.
 *
 * This module provides:
 * - Automatic CSRF token fetching and caching
 * - Token injection for mutating requests (POST, PUT, PATCH, DELETE)
 * - Token refresh on expiration
 *
 * ## Usage
 *
 * ```typescript
 * import { csrfManager } from '@acme/client';
 *
 * // CSRF tokens are automatically injected by the HttpClient
 * // when using cookie-based authentication.
 * ```
 */

import { getHttpClient } from "./client-factory.js";

interface CsrfTokenState {
  token: string | null;
  fetchedAt: number | null;
  ttlMs: number;
}

const CSRF_TTL_MS = 55 * 60 * 1000; // 55 minutes (tokens last 60 minutes)

let state: CsrfTokenState = {
  token: null,
  fetchedAt: null,
  ttlMs: CSRF_TTL_MS,
};

/**
 * Check if we have a valid cached CSRF token.
 */
function hasValidToken(): boolean {
  if (!state.token || !state.fetchedAt) {
    return false;
  }
  const age = Date.now() - state.fetchedAt;
  return age < state.ttlMs;
}

/**
 * Fetch a new CSRF token from the server.
 */
async function fetchCsrfToken(fetchFn: typeof fetch): Promise<string> {
  const http = getHttpClient({ fetchFn, credentials: "include" });
  const response = await http.get<{ data: { csrfToken: string } }>(
    "/v1/auth/csrf-token"
  );
  return response.data.csrfToken;
}

/**
 * Get a valid CSRF token, fetching a new one if needed.
 */
export async function getCsrfToken(fetchFn: typeof fetch): Promise<string> {
  if (hasValidToken() && state.token) {
    return state.token;
  }

  const token = await fetchCsrfToken(fetchFn);
  state = {
    token,
    fetchedAt: Date.now(),
    ttlMs: CSRF_TTL_MS,
  };
  return token;
}

/**
 * Clear the cached CSRF token (call on logout).
 */
export function clearCsrfToken(): void {
  state = {
    token: null,
    fetchedAt: null,
    ttlMs: CSRF_TTL_MS,
  };
}

/**
 * Get CSRF headers for a mutating request.
 * Returns empty object if no token is available.
 */
export async function getCsrfHeaders(
  fetchFn: typeof fetch
): Promise<Record<string, string>> {
  try {
    const token = await getCsrfToken(fetchFn);
    return { "X-CSRF-Token": token };
  } catch {
    // If we can't get a CSRF token (e.g., not authenticated),
    // return empty headers and let the request proceed.
    // The server will reject it if CSRF is required.
    return {};
  }
}
