import type { ListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  CapturedEmailSummary,
  CapturedEmailDetail,
  ListCapturedEmailsQuery,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

export async function listCapturedEmails(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListCapturedEmailsQuery
): Promise<CapturedEmailSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.to_address) params.set("to_address", query.to_address);
  if (query?.from_address) params.set("from_address", query.from_address);
  if (query?.since) params.set("since", query.since);
  if (query?.until) params.set("until", query.until);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/captured-emails?${queryString}` : "/v1/admin/captured-emails";

  const response = await http.get<ListResponse<CapturedEmailSummary>>(path);
  return response.data;
}

export async function getCapturedEmail(
  emailId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<CapturedEmailDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<CapturedEmailDetail>>(
    `/v1/admin/captured-emails/${encodeURIComponent(emailId)}`
  );
  return response.data;
}

export async function deleteCapturedEmail(
  emailId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(`/v1/admin/captured-emails/${encodeURIComponent(emailId)}`);
}
