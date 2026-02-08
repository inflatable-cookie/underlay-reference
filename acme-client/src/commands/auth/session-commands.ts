import type {
  Session,
  SingleResponse,
  ListResponse,
} from "../../types/common-types.js";
import { getHttpClient } from "../../utils/client-factory.js";

export async function listSessions(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<Session[]> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<Session>>("/v1/auth/sessions");
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
