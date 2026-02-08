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
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/activity?${queryString}` : "/v1/admin/activity";

  return await http.get<ActivityListResponse>(path);
}

/**
 * List activity for a specific entity.
 */
export async function listActivityForEntity(
  entityType: string,
  entityId: string,
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListActivityQuery
): Promise<ActivityListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const basePath = `/v1/admin/activity/entity/${encodeURIComponent(entityType)}/${encodeURIComponent(entityId)}`;
  const path = queryString ? `${basePath}?${queryString}` : basePath;

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
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const basePath = `/v1/admin/users/${encodeURIComponent(userId)}/activity`;
  const path = queryString ? `${basePath}?${queryString}` : basePath;

  return await http.get<ActivityListResponse>(path);
}
