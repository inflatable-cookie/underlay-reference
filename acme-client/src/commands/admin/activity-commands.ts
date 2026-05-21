import type {
  ActivityListResponse,
  ListActivityQuery,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

/**
 * List all activity (global feed).
 */
export async function listActivity(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListActivityQuery
): Promise<ActivityListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.action) params.set("action", query.action);
  if (query?.resourceType) params.set("resource_type", query.resourceType);
  if (query?.page !== undefined) params.set("page", String(query.page));
  if (query?.limit !== undefined) params.set("limit", String(query.limit));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/activity?${queryString}` : "/v1/admin/activity";

  return await http.get<ActivityListResponse>(path);
}

/**
 * List activity performed by a specific user.
 */
export async function listActivityForUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListActivityQuery
): Promise<ActivityListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.page !== undefined) params.set("page", String(query.page));
  if (query?.limit !== undefined) params.set("limit", String(query.limit));

  const queryString = params.toString();
  const basePath = `/v1/admin/users/${encodeURIComponent(userId)}/activity`;
  const path = queryString ? `${basePath}?${queryString}` : basePath;

  return await http.get<ActivityListResponse>(path);
}
