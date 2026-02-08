import type { SingleResponse } from "../../types/common-types.js";
import type {
  ErrorLogDetail,
  ErrorLogStats,
  ListErrorLogsQuery,
  ErrorLogsListResponse,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

/**
 * List error logs with optional filters.
 *
 * Supports filtering by status code, error code, endpoint, and time range.
 */
export async function listErrorLogs(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListErrorLogsQuery
): Promise<ErrorLogsListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.status_code !== undefined) params.set("status_code", String(query.status_code));
  if (query?.error_code) params.set("error_code", query.error_code);
  if (query?.endpoint) params.set("endpoint", query.endpoint);
  if (query?.since) params.set("since", query.since);
  if (query?.until) params.set("until", query.until);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/error-logs?${queryString}` : "/v1/admin/error-logs";

  return await http.get<ErrorLogsListResponse>(path);
}

/**
 * Get details of a specific error log entry.
 */
export async function getErrorLog(
  errorLogId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ErrorLogDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<ErrorLogDetail>>(
    `/v1/admin/error-logs/${encodeURIComponent(errorLogId)}`
  );
  return response.data;
}

/**
 * Get error log statistics.
 */
export async function getErrorLogStats(
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ErrorLogStats> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<ErrorLogStats>>(
    "/v1/admin/error-logs/stats"
  );
  return response.data;
}
