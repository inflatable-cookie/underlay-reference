/**
 * Auth commands - standalone authentication functions
 */
import type {
  ChangePasswordRequest,
  GoogleOAuthCallbackRequest,
  GoogleOAuthStartResponse,
  GoogleOAuthStatusResponse,
  GoogleOAuthTokenResponse,
  LoginEmailFallbackRequest,
  LoginEmailFallbackResponse,
  LoginEmailResendRequest,
  LoginFinishRequest,
  LoginRequest,
  LoginResponse,
  LoginStartRequest,
  LoginStartResponse,
  LoginUser,
  LogoutRequest,
  PasskeyCredential,
  PasskeyLoginFinishRequest,
  PasskeyLoginStartRequest,
  PasskeyRegisterFinishRequest,
  PasskeyRenameRequest,
  PasskeyStartResponse,
  RefreshRequest,
  RegisterRequest,
  SingleResponse,
  ListResponse
} from "../types/common-types.js";
import { getHttpClient } from "../utils/client-factory.js";

// ============================================================================
// Core Auth
// ============================================================================

export async function register(
  payload: RegisterRequest,
  fetchFn: typeof fetch,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>("/v1/auth/register", payload);
  return response.data;
}

export async function login(
  payload: LoginRequest,
  fetchFn: typeof fetch,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>("/v1/auth/login", payload);
  return response.data;
}

export async function loginStart(
  payload: LoginStartRequest,
  fetchFn: typeof fetch,
): Promise<LoginStartResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginStartResponse>>("/v1/auth/login/start", payload);
  return response.data;
}

export async function loginFinish(
  payload: LoginFinishRequest,
  fetchFn: typeof fetch,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>("/v1/auth/login/finish", payload);
  return response.data;
}

/**
 * Request email fallback for a TOTP login state.
 *
 * Converts a pending TOTP login to email verification. Used when the user
 * has TOTP configured but wants to verify via email instead.
 */
export async function loginEmailFallback(
  payload: LoginEmailFallbackRequest,
  fetchFn: typeof fetch,
): Promise<LoginEmailFallbackResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginEmailFallbackResponse>>("/v1/auth/login/email-fallback", payload);
  return response.data;
}

/**
 * Resend the login email verification code.
 *
 * Used when the user is already in the email verification step and wants
 * to receive a new code.
 */
export async function loginEmailResend(
  payload: LoginEmailResendRequest,
  fetchFn: typeof fetch,
): Promise<void> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  await http.post<void>("/v1/auth/login/email-resend", payload);
}

export async function refresh(
  payload: RefreshRequest,
  fetchFn: typeof fetch,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>("/v1/auth/refresh", payload);
  return response.data;
}

export async function logout(
  payload: LogoutRequest,
  fetchFn: typeof fetch,
  accessToken?: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken, credentials: 'include' });
  await http.post<void>("/v1/auth/logout", payload);
}

export async function me(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<LoginUser> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<LoginUser>>("/v1/auth/me");
  return response.data;
}

export async function changePassword(
  payload: ChangePasswordRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.post<void>("/v1/auth/password/change", payload);
}

// ============================================================================
// TOTP (Two-Factor Authentication)
// ============================================================================

export async function totpStatus(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").TotpStatusResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<import("../types/common-types.js").TotpStatusResponse>>("/v1/auth/totp/status");
  return response.data;
}

export async function twoFactorStatus(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").TwoFactorStatusResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<import("../types/common-types.js").TwoFactorStatusResponse>>("/v1/auth/2fa-status");
  return response.data;
}

export async function totpSetup(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").TotpSetupResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<import("../types/common-types.js").TotpSetupResponse>>("/v1/auth/totp/setup", {});
  return response.data;
}

export async function totpEnable(
  payload: import("../types/common-types.js").TotpEnableRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.post<void>("/v1/auth/totp/enable", payload);
}

export async function totpDisable(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.post<void>("/v1/auth/totp/disable", {});
}

// ============================================================================
// Passkeys - Registration
// ============================================================================

export async function passkeyRegisterStart(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<PasskeyStartResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<PasskeyStartResponse>>("/v1/auth/passkeys/register/start", {});
  return response.data;
}

export async function passkeyRegisterFinish(
  payload: PasskeyRegisterFinishRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.post<void>("/v1/auth/passkeys/register/finish", payload);
}

/** @deprecated Use passkeyRegisterStart instead */
export async function passkeyConnectStart(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<PasskeyStartResponse> {
  return passkeyRegisterStart(fetchFn, accessToken);
}

/** @deprecated Use passkeyRegisterFinish instead */
export async function passkeyConnectFinish(
  payload: PasskeyRegisterFinishRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  return passkeyRegisterFinish(payload, fetchFn, accessToken);
}

// ============================================================================
// Passkeys - Management
// ============================================================================

export async function listPasskeys(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<PasskeyCredential[]> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<PasskeyCredential>>("/v1/auth/passkeys");
  return response.data;
}

export async function deletePasskey(
  credentialId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.delete<void>(`/v1/auth/passkeys/${credentialId}`);
}

export async function renamePasskey(
  credentialId: string,
  payload: PasskeyRenameRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.patch<void>(`/v1/auth/passkeys/${credentialId}`, payload);
}

// ============================================================================
// Passkeys - Login
// ============================================================================

export async function passkeyLoginStart(
  payload: PasskeyLoginStartRequest,
  fetchFn: typeof fetch,
): Promise<PasskeyStartResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<PasskeyStartResponse>>("/v1/auth/passkeys/login/start", payload);
  return response.data;
}

export async function passkeyLoginFinish(
  payload: PasskeyLoginFinishRequest,
  fetchFn: typeof fetch,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>("/v1/auth/passkeys/login/finish", payload);
  return response.data;
}

// ============================================================================
// Google OAuth
// ============================================================================

export async function googleOAuthStart(
  fetchFn: typeof fetch,
  accessToken?: string,
): Promise<GoogleOAuthStartResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<GoogleOAuthStartResponse>>("/v1/auth/oauth/google/start", {});
  return response.data;
}

export async function googleOAuthCallback(
  payload: GoogleOAuthCallbackRequest,
  fetchFn: typeof fetch,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>("/v1/auth/oauth/google/callback", payload);
  return response.data;
}

export async function googleOAuthStatus(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<GoogleOAuthStatusResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<GoogleOAuthStatusResponse>>("/v1/auth/oauth/google/status");
  return response.data;
}

export async function googleOAuthRefresh(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<GoogleOAuthTokenResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<GoogleOAuthTokenResponse>>("/v1/auth/oauth/google/refresh", {});
  return response.data;
}

export async function googleOAuthDisconnect(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.post<void>("/v1/auth/oauth/google/disconnect", {});
}

// ============================================================================
// Session Management
// ============================================================================

export async function listSessions(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").Session[]> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<import("../types/common-types.js").Session>>("/v1/auth/sessions");
  return response.data;
}

export async function revokeSession(
  sessionId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.post<void>(`/v1/auth/sessions/${sessionId}/revoke`, {});
}

// ============================================================================
// Email TOTP (Email-based One-Time Password)
// ============================================================================

export async function requestEmailTotp(
  purpose: import("../types/common-types.js").EmailTotpPurpose,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").EmailTotpRequestResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<import("../types/common-types.js").EmailTotpRequestResponse>>(
    "/v1/auth/email-totp/request",
    { purpose }
  );
  return response.data;
}

export async function verifyEmailTotp(
  code: string,
  purpose: import("../types/common-types.js").EmailTotpPurpose,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").EmailTotpVerifyResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<import("../types/common-types.js").EmailTotpVerifyResponse>>(
    "/v1/auth/email-totp/verify",
    { code, purpose }
  );
  return response.data;
}

// ============================================================================
// TOTP Verification (for 2FA gates - authenticator app)
// ============================================================================

export async function verifyTotp(
  code: string,
  purpose: import("../types/common-types.js").EmailTotpPurpose,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").VerificationSessionResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<import("../types/common-types.js").VerificationSessionResponse>>(
    "/v1/auth/totp/verify",
    { code, purpose }
  );
  return response.data;
}

// ============================================================================
// Passkey Verification (for 2FA gates)
// ============================================================================

export async function passkeyVerifyStart(
  purpose: import("../types/common-types.js").EmailTotpPurpose,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<PasskeyStartResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<PasskeyStartResponse>>(
    "/v1/auth/passkeys/verify/start",
    { purpose }
  );
  return response.data;
}

export async function passkeyVerifyFinish(
  payload: import("../types/common-types.js").PasskeyVerifyFinishRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<import("../types/common-types.js").VerificationSessionResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<import("../types/common-types.js").VerificationSessionResponse>>(
    "/v1/auth/passkeys/verify/finish",
    payload
  );
  return response.data;
}

// ============================================================================
// Password Change with Verification Session
// ============================================================================

export async function changePasswordWithVerification(
  payload: import("../types/common-types.js").ChangePasswordWithVerificationRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.post<void>("/v1/auth/password/change-2fa", payload);
}

/**
 * Get password requirements from the server.
 *
 * Returns the password requirements configuration so UIs can display
 * accurate feedback without hardcoding values.
 */
export async function passwordRequirements(
  fetchFn: typeof fetch,
): Promise<import("../types/common-types.js").PasswordRequirements> {
  const http = getHttpClient({ fetchFn });
  const response = await http.get<SingleResponse<import("../types/common-types.js").PasswordRequirements>>("/v1/auth/password/requirements");
  return response.data;
}

// ============================================================================
// Password Reset (Forgot Password)
// ============================================================================

/**
 * Request a password reset code to be sent via email.
 *
 * This endpoint always returns success to prevent email enumeration.
 * If the email exists, a code is sent; if not, nothing happens.
 */
export async function requestPasswordReset(
  payload: import("../types/common-types.js").PasswordResetRequestRequest,
  fetchFn: typeof fetch,
): Promise<void> {
  const http = getHttpClient({ fetchFn });
  await http.post<void>("/v1/auth/password/reset/request", payload);
}

/**
 * Verify a password reset code.
 *
 * Returns a reset token that can be used to set a new password.
 */
export async function verifyPasswordReset(
  payload: import("../types/common-types.js").PasswordResetVerifyRequest,
  fetchFn: typeof fetch,
): Promise<import("../types/common-types.js").PasswordResetVerifyResponse> {
  const http = getHttpClient({ fetchFn });
  const response = await http.post<SingleResponse<import("../types/common-types.js").PasswordResetVerifyResponse>>(
    "/v1/auth/password/reset/verify",
    payload
  );
  return response.data;
}

/**
 * Complete a password reset by setting a new password.
 *
 * Requires a valid reset token from verifyPasswordReset.
 */
export async function completePasswordReset(
  payload: import("../types/common-types.js").PasswordResetCompleteRequest,
  fetchFn: typeof fetch,
): Promise<void> {
  const http = getHttpClient({ fetchFn });
  await http.post<void>("/v1/auth/password/reset/complete", payload);
}
