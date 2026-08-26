/**
 * Auth token cookie helpers for Acme Admin.
 *
 * Uses Underlay's createAuthCookieHelpers factory with Acme-specific config.
 *
 * The token store in hooks.server.ts handles proactive token refresh.
 * These utilities are used by:
 * - Login routes (writeAuthTokens after successful auth)
 * - Logout routes (clearAuthTokens, readRefreshToken)
 */

import { createAuthCookieHelpers } from "@inflatable-cookie/underlay/client/sveltekit";

export const ACCESS_TOKEN_COOKIE = "acme_access_token";
export const REFRESH_TOKEN_COOKIE = "acme_refresh_token";

const authCookies = createAuthCookieHelpers({
  accessTokenName: ACCESS_TOKEN_COOKIE,
  refreshTokenName: REFRESH_TOKEN_COOKIE,
  options: {
    maxAge: 60 * 60 * 24 * 7, // 7 days
  },
});

// Re-export the helper functions for direct use
export const readAccessToken = authCookies.readAccessToken;
export const readRefreshToken = authCookies.readRefreshToken;
export const writeAuthTokens = authCookies.writeAuthTokens;
export const clearAuthTokens = authCookies.clearAuthTokens;
