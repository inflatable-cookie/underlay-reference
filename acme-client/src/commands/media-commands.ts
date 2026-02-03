/**
 * Media Library commands - media operations for admin UI
 */
import type { ListResponse, SingleResponse } from "../types/common-types.js";
import type {
  MediaSummary,
  MediaDetail,
  MediaVersion,
  MediaUsage,
  CreateMediaRequest,
  UpdateMediaRequest,
  CheckDuplicateRequest,
  CheckDuplicateResponse,
  InitiateUploadResponse,
  FinaliseUploadRequest,
  FinaliseUploadResponse,
} from "../types/media-types.js";
import { getAdminHttpClient } from "../utils/client-factory.js";
import {
  appendQueryParams,
  type QueryParams,
} from "@decodelabs/underlay/client";
import {
  appendPaginationParams,
  type PaginatedResponse,
  type PaginationParams,
} from "@decodelabs/underlay/patterns";

// ============================================================================
// Deduplication
// ============================================================================

/**
 * Check if a file with the given hash already exists.
 *
 * Call this before uploading to detect duplicates and offer reuse.
 */
export async function checkDuplicate(
  request: CheckDuplicateRequest,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<CheckDuplicateResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<CheckDuplicateResponse>(
    "/v1/admin/media/check-duplicate",
    request
  );
  return response;
}

// ============================================================================
// CRUD Operations
// ============================================================================

/**
 * List all media items (admin view).
 *
 * Supports filtering and sorting via QueryParams:
 * - `filter[kind]` - Filter by kind (image, pdf, etc.)
 * - `filter[visibility]` - Filter by visibility (public, restricted)
 * - `filter[title][like]` - Search by title (use %value% for contains)
 * - `sort` - Sort order (e.g., "title:asc,updatedAt:desc")
 */
export async function listMediaAdmin(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: QueryParams
): Promise<MediaSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendQueryParams("/v1/admin/media", query ?? {});
  const response = await http.get<ListResponse<MediaSummary>>(path);
  return response.data;
}

/**
 * List media items with pagination (admin view).
 *
 * Supports the same filtering and sorting as listMediaAdmin, plus pagination.
 */
export async function listMediaPaginatedAdmin(
  fetchFn: typeof fetch,
  accessToken: string,
  pagination?: PaginationParams,
  query?: QueryParams
): Promise<PaginatedResponse<MediaSummary>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  let path = "/v1/admin/media/paginated";
  if (query) {
    path = appendQueryParams(path, query);
  }
  path = appendPaginationParams(path, pagination ?? {});
  return await http.get<PaginatedResponse<MediaSummary>>(path);
}

/**
 * List soft-deleted media items (trash).
 */
export async function listMediaTrash(
  fetchFn: typeof fetch,
  accessToken: string
): Promise<MediaSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<MediaSummary>>(
    "/v1/admin/media/trash"
  );
  return response.data;
}

/**
 * Create a new media item (metadata only, no bytes yet).
 */
export async function createMedia(
  request: CreateMediaRequest,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<MediaDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<MediaDetail>>(
    "/v1/admin/media",
    request
  );
  return response.data;
}

/**
 * Get a single media item by ID.
 */
export async function getMedia(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<MediaDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<MediaDetail>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}`
  );
  return response.data;
}

/**
 * Update a media item's metadata.
 */
export async function updateMedia(
  mediaId: string,
  request: UpdateMediaRequest,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<MediaDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.put<SingleResponse<MediaDetail>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}`,
    request
  );
  return response.data;
}

/**
 * Soft-delete a media item.
 */
export async function softDeleteMedia(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(`/v1/admin/media/${encodeURIComponent(mediaId)}/soft-delete`, {});
}

/**
 * Restore a soft-deleted media item.
 */
export async function restoreMedia(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(`/v1/admin/media/${encodeURIComponent(mediaId)}/restore`, {});
}

/**
 * Permanently delete a media item and all its versions.
 */
export async function purgeMedia(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(`/v1/admin/media/${encodeURIComponent(mediaId)}`);
}

// ============================================================================
// Upload Flow
// ============================================================================

/**
 * Request to initiate an upload
 */
export interface InitiateUploadRequest {
  contentType: string;
  contentLength: number;
}

/**
 * Initiate an upload for a media item.
 *
 * Returns a pre-signed URL for direct-to-blob upload.
 */
export async function initiateUpload(
  mediaId: string,
  request: InitiateUploadRequest,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<InitiateUploadResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<InitiateUploadResponse>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/initiate-upload`,
    request
  );
  return response;
}

/**
 * Finalise an upload after the file has been uploaded to the blob store.
 *
 * This validates the upload and transitions the version to 'ready' state.
 */
export async function finaliseUpload(
  mediaId: string,
  versionId: string,
  request: FinaliseUploadRequest,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<FinaliseUploadResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<FinaliseUploadResponse>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/${encodeURIComponent(versionId)}/finalise-upload`,
    request
  );
  return response;
}

// ============================================================================
// Versions and Usage
// ============================================================================

/**
 * List all versions of a media item.
 */
export async function listVersions(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<MediaVersion[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<MediaVersion>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions`
  );
  return response.data;
}

/**
 * Activate (set as current) a specific version.
 */
export async function activateVersion(
  mediaId: string,
  versionId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/${encodeURIComponent(versionId)}/activate`,
    {}
  );
}

/**
 * Delete a version.
 *
 * Note: Cannot delete the currently active version.
 */
export async function deleteVersion(
  mediaId: string,
  versionId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/${encodeURIComponent(versionId)}`
  );
}

/**
 * List all usages of a media item.
 */
export async function listUsages(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<MediaUsage[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<MediaUsage>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/usage`
  );
  return response.data;
}

// ============================================================================
// Download
// ============================================================================

/**
 * Get the download URL for a media item.
 *
 * For public media, this returns a direct URL.
 * For restricted media, this requires authentication and returns a signed URL.
 *
 * Note: The actual download redirects to the blob URL, so this function
 * just returns the API endpoint URL that will redirect.
 */
export function getMediaDownloadUrl(mediaId: string, restricted = false): string {
  if (restricted) {
    return `/v1/media/${encodeURIComponent(mediaId)}/download`;
  }
  return `/v1/media/${encodeURIComponent(mediaId)}`;
}
