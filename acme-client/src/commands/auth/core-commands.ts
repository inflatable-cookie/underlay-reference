import type {
  ChangePasswordRequest,
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
  RefreshRequest,
  RegisterRequest,
  SingleResponse,
} from "../../types/common-types.js";
import { getHttpClient } from "../../utils/client-factory.js";

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
  // CSRF token is automatically injected by HttpClient for mutating requests
  const response = await http.post<SingleResponse<LoginResponse>>("/v1/auth/refresh", payload);
  return response.data;
}

export async function logout(
  payload: LogoutRequest,
  fetchFn: typeof fetch,
  accessToken?: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken, credentials: 'include' });
  // CSRF token is automatically injected by HttpClient for mutating requests
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
  const http = getHttpClient({ fetchFn, accessToken, credentials: "include" });
  // CSRF token is automatically injected by HttpClient for mutating requests
  await http.post<void>("/v1/auth/password/change", payload);
}
