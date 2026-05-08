import type { PagedListResponse, SingleResponse, Session } from "../../types/common-types.js";
import type {
  User,
  UserDetail,
  CreateUserPayload,
  ListUsersQuery,
  UpdateUserPayload,
  UpdateUserRolePayload,
  UserListResponse,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";
import { getHeaderValueCaseInsensitive, type WithEtag } from "./utils.js";
import { appendQueryParams } from "@decodelabs/underlay/client/query";
import { toSnakeQueryParams } from "./utils.js";

/**
 * Create a user (admin).
 *
 * Creates a user record without logging the admin in as that user.
 * Optionally sends a password reset email so the user can set an initial password.
 */
export async function createUser(
  payload: CreateUserPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<User>>("/v1/admin/users", {
    email: payload.email,
    role: payload.role,
    status: payload.status,
    displayName: payload.displayName ?? null,
    sendPasswordReset: payload.sendPasswordReset ?? true,
  });
  return response.data;
}

/**
 * List users (admin).
 *
 * Supports filtering by role, status, and text search.
 */
export async function listUsers(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListUsersQuery
): Promise<UserListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendQueryParams("/v1/admin/users", toSnakeQueryParams(query));
  return await http.get<UserListResponse>(path);
}

/**
 * Get a single user by ID.
 */
export async function getUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<UserDetail> {
  const result = await getUserWithEtag(userId, fetchFn, accessToken);
  return result.data;
}

export async function getUserWithEtag(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<WithEtag<UserDetail>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.getWithMeta<SingleResponse<UserDetail>>(
    `/v1/admin/users/${encodeURIComponent(userId)}`
  );
  return {
    data: response.body!.data,
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
}

/**
 * Update a user (admin).
 */
export async function updateUser(
  userId: string,
  payload: UpdateUserPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const result = await updateUserWithEtag(userId, payload, fetchFn, accessToken);
  return result.data;
}

export async function updateUserWithEtag(
  userId: string,
  payload: UpdateUserPayload,
  fetchFn: typeof fetch,
  accessToken: string,
  options?: { ifMatch?: string }
): Promise<WithEtag<User>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const headers = options?.ifMatch ? { "If-Match": options.ifMatch } : undefined;
  const response = await http.putWithMeta<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}`,
    {
      email: payload.email,
      role: payload.role,
      status: payload.status,
      displayName: payload.displayName ?? null,
    },
    headers
  );
  return {
    data: response.body!.data,
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
}

/**
 * Update a user's role.
 */
export async function updateUserRole(
  userId: string,
  payload: UpdateUserRolePayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.put<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/role`,
    payload
  );
  return response.data;
}

/**
 * Suspend a user.
 */
export async function suspendUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/suspend`,
    {}
  );
  return response.data;
}

/**
 * Unsuspend (reactivate) a user.
 */
export async function unsuspendUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/unsuspend`,
    {}
  );
  return response.data;
}

/**
 * List all sessions for a user (admin).
 *
 * Returns all sessions (active, expired, revoked) for administrative purposes.
 */
export async function listUserSessions(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<PagedListResponse<Session>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.get<PagedListResponse<Session>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/sessions`
  );
}

/**
 * Revoke a specific session for a user (admin).
 *
 * Terminates the session and logs out the user from that device.
 */
export async function revokeUserSession(
  userId: string,
  sessionId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(
    `/v1/admin/users/${encodeURIComponent(userId)}/sessions/${encodeURIComponent(sessionId)}/revoke`,
    {}
  );
}
