import type {
  EmailTotpPurpose,
  EmailTotpRequestResponse,
  EmailTotpVerifyResponse,
  SingleResponse,
} from "../../types/common-types.js";
import { getHttpClient } from "../../utils/client-factory.js";

export async function requestEmailTotp(
  purpose: EmailTotpPurpose,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<EmailTotpRequestResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<EmailTotpRequestResponse>>(
    "/v1/auth/email-totp/request",
    { purpose }
  );
  return response.data;
}

export async function verifyEmailTotp(
  code: string,
  purpose: EmailTotpPurpose,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<EmailTotpVerifyResponse> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<EmailTotpVerifyResponse>>(
    "/v1/auth/email-totp/verify",
    { code, purpose }
  );
  return response.data;
}
