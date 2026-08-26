/**
 * Account commands - user profile operations
 */
import type { SingleResponse } from "../types/common-types.js";
import type { UserProfile, UserProfileUpdate } from "../types/account-types.js";
import { getSharedHttpClient } from "../utils/client-factory.js";

/**
 * Get the current user's profile.
 *
 * Creates a profile with defaults if one doesn't exist.
 */
export async function getProfile(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserProfile> {
  const http = getSharedHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<UserProfile>>("/v1/account/profile");
  return response.data;
}

/**
 * Update the current user's profile.
 *
 * Only provided fields are updated.
 */
export async function updateProfile(
  updates: UserProfileUpdate,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserProfile> {
  const http = getSharedHttpClient({ fetchFn, accessToken });
  const response = await http.patch<SingleResponse<UserProfile>>("/v1/account/profile", updates);
  return response.data;
}
