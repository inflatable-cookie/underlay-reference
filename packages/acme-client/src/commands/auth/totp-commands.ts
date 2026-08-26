import type {
  TotpStatusResponse,
  TwoFactorStatusResponse,
  TotpSetupResponse,
  TotpEnableRequest,
  EmailTotpPurpose,
  VerificationSessionResponse,
  SingleResponse,
} from "../../types/common-types.js";
import { getHttpClient } from "../../utils/client-factory.js";

export async function totpStatus(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<TotpStatusResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<TotpStatusResponse>>("/v1/auth/totp/status");
  return response.data;
}

export async function twoFactorStatus(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<TwoFactorStatusResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<TwoFactorStatusResponse>>("/v1/auth/2fa-status");
  return response.data;
}

export async function totpSetup(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<TotpSetupResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<TotpSetupResponse>>("/v1/auth/totp/setup", {});
  return response.data;
}

export async function totpEnable(
  payload: TotpEnableRequest,
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

/**
 * Verify a TOTP code for 2FA gates (authenticator app).
 */
export async function verifyTotp(
  code: string,
  purpose: EmailTotpPurpose,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<VerificationSessionResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<VerificationSessionResponse>>(
    "/v1/auth/totp/verify",
    { code, purpose }
  );
  return response.data;
}
