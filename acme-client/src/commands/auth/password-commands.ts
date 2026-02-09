import type {
  ChangePasswordWithVerificationRequest,
  PasswordRequirements,
  PasswordResetRequestRequest,
  PasswordResetVerifyRequest,
  PasswordResetVerifyResponse,
  PasswordResetCompleteRequest,
  SingleResponse,
} from "../../types/common-types.js";
import { getHttpClient } from "../../utils/client-factory.js";

export async function changePasswordWithVerification(
  payload: ChangePasswordWithVerificationRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken, credentials: "include" });
  // CSRF token is automatically injected by HttpClient for mutating requests
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
): Promise<PasswordRequirements> {
  const http = getHttpClient({ fetchFn });
  const response = await http.get<SingleResponse<PasswordRequirements>>("/v1/auth/password/requirements");
  return response.data;
}

/**
 * Request a password reset code to be sent via email.
 *
 * This endpoint always returns success to prevent email enumeration.
 * If the email exists, a code is sent; if not, nothing happens.
 */
export async function requestPasswordReset(
  payload: PasswordResetRequestRequest,
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
  payload: PasswordResetVerifyRequest,
  fetchFn: typeof fetch,
): Promise<PasswordResetVerifyResponse> {
  const http = getHttpClient({ fetchFn });
  const response = await http.post<SingleResponse<PasswordResetVerifyResponse>>(
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
  payload: PasswordResetCompleteRequest,
  fetchFn: typeof fetch,
): Promise<void> {
  const http = getHttpClient({ fetchFn });
  await http.post<void>("/v1/auth/password/reset/complete", payload);
}
