import type {
  ChangePasswordRequest,
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

export interface AuthTokenModeOptions {
  tokenMode?: "cookie" | "body";
}

function authTokenModeHeader(options?: AuthTokenModeOptions): Record<string, string> | undefined {
  if (options?.tokenMode === "body") {
    return { "X-Auth-Token-Mode": "body" };
  }
  return undefined;
}

export async function register(
  payload: RegisterRequest,
  fetchFn: typeof fetch,
  options?: AuthTokenModeOptions,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>(
    "/v1/auth/register",
    payload,
    authTokenModeHeader(options),
  );
  return response.data;
}

export async function login(
  payload: LoginRequest,
  fetchFn: typeof fetch,
  options?: AuthTokenModeOptions,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>(
    "/v1/auth/login",
    payload,
    authTokenModeHeader(options),
  );
  return response.data;
}

export async function loginStart(
  payload: LoginStartRequest,
  fetchFn: typeof fetch,
  options?: AuthTokenModeOptions,
): Promise<LoginStartResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginStartResponse>>(
    "/v1/auth/login/start",
    payload,
    authTokenModeHeader(options),
  );
  return response.data;
}

export async function loginFinish(
  payload: LoginFinishRequest,
  fetchFn: typeof fetch,
  options?: AuthTokenModeOptions,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  const response = await http.post<SingleResponse<LoginResponse>>(
    "/v1/auth/login/finish",
    payload,
    authTokenModeHeader(options),
  );
  return response.data;
}

export async function refresh(
  payload: RefreshRequest,
  fetchFn: typeof fetch,
  options?: AuthTokenModeOptions,
): Promise<LoginResponse> {
  const http = getHttpClient({ fetchFn, credentials: 'include' });
  // CSRF token is automatically injected by HttpClient for mutating requests
  const response = await http.post<SingleResponse<LoginResponse>>(
    "/v1/auth/refresh",
    payload,
    authTokenModeHeader(options),
  );
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
