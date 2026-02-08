import type {
  GoogleOAuthStartResponse,
  GoogleOAuthCallbackRequest,
  GoogleOAuthStatusResponse,
  GoogleOAuthTokenResponse,
  LoginResponse,
  SingleResponse,
} from "../../types/common-types.js";
import { getHttpClient } from "../../utils/client-factory.js";

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
