import type {
  PasskeyStartResponse,
  PasskeyRegisterFinishRequest,
  PasskeyLoginStartRequest,
  PasskeyLoginFinishRequest,
  PasskeyCredential,
  PasskeyRenameRequest,
  PasskeyVerifyFinishRequest,
  EmailTotpPurpose,
  VerificationSessionResponse,
  LoginResponse,
  SingleResponse,
  ListResponse,
} from "../../types/common-types.js";
import { getHttpClient } from "../../utils/client-factory.js";

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
// Passkeys - Verification (for 2FA gates)
// ============================================================================

export async function passkeyVerifyStart(
  purpose: EmailTotpPurpose,
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
  payload: PasskeyVerifyFinishRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<VerificationSessionResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<VerificationSessionResponse>>(
    "/v1/auth/passkeys/verify/finish",
    payload
  );
  return response.data;
}
