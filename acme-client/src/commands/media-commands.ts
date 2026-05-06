import { getAcmeClientConfig, getAdminHttpClient } from "../utils/client-factory.js";
import {
  appendQueryParams,
  type QueryParams,
} from "@decodelabs/underlay/client/query";
/**
 * Media Library commands - media operations for admin UI
 */
import type { ListResponse, PagedListResponse, SingleResponse } from "../types/common-types.js";
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
import {
  getHeaderValueCaseInsensitive,
  toSnakeQueryParams,
  type WithEtag,
} from "./admin/utils.js";

export type MediaListProfile = "list" | "filter";

export interface ListMediaOptions {
  profile?: MediaListProfile;
  query?: QueryParams;
}

function normalizeLocalUploadPlanUrl(url: string): string {
  try {
    const parsed = new URL(url);
    if (
      !parsed.pathname.startsWith("/v1/dev-uploads/")
      && !parsed.pathname.startsWith("/v1/dev-blobs/")
    ) {
      return url;
    }

    const apiBase = new URL(getAcmeClientConfig().baseUrl);
    parsed.protocol = apiBase.protocol;
    parsed.hostname = apiBase.hostname;
    parsed.port = apiBase.port;
    return parsed.toString();
  } catch {
    return url;
  }
}

function normalizeMediaSummary(media: MediaSummary): MediaSummary {
  return {
    ...media,
    thumbnailUrl: media.thumbnailUrl
      ? normalizeLocalUploadPlanUrl(media.thumbnailUrl)
      : media.thumbnailUrl,
  };
}

function normalizeMediaVersion(version: MediaVersion): MediaVersion {
  return {
    ...version,
    url: version.url ? normalizeLocalUploadPlanUrl(version.url) : version.url,
    renditions: (version.renditions ?? []).map((rendition) => ({
      ...rendition,
      url: rendition.url ? normalizeLocalUploadPlanUrl(rendition.url) : rendition.url,
    })),
  };
}

function normalizeMediaDetail(media: MediaDetail): MediaDetail {
  return {
    ...media,
    currentVersion: media.currentVersion
      ? normalizeMediaVersion(media.currentVersion)
      : media.currentVersion,
  };
}

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
  accessToken: string,
): Promise<CheckDuplicateResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<CheckDuplicateResponse>(
    "/v1/admin/media/check-duplicate",
    request,
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
export async function listMedia(
  fetchFn: typeof fetch,
  accessToken: string,
  options?: ListMediaOptions,
): Promise<PagedListResponse<MediaSummary>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  let path = "/v1/admin/media";
  if (options?.query) {
    path = appendQueryParams(path, toSnakeQueryParams(options.query));
  }
  if (options?.profile) {
    path += `${path.includes("?") ? "&" : "?"}profile=${encodeURIComponent(options.profile)}`;
  }
  const response = await http.get<PagedListResponse<MediaSummary>>(path);
  return {
    ...response,
    data: response.data.map(normalizeMediaSummary),
  };
}

export async function listMediaAdmin(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: QueryParams,
): Promise<MediaSummary[]> {
  const response = await listMedia(fetchFn, accessToken, {
    profile: "list",
    query,
  });
  return response.data;
}

/**
 * List soft-deleted media items (trash).
 */
export async function listMediaTrash(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<MediaSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<MediaSummary>>(
    "/v1/admin/media/trash",
  );
  return response.data.map(normalizeMediaSummary);
}

/**
 * Create a new media item (metadata only, no bytes yet).
 */
export async function createMedia(
  request: CreateMediaRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<MediaDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<MediaDetail>>(
    "/v1/admin/media",
    request,
  );
  return normalizeMediaDetail(response.data);
}

/**
 * Get a single media item by ID.
 */
export async function getMedia(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<MediaDetail> {
  const result = await getMediaWithEtag(mediaId, fetchFn, accessToken);
  return result.data;
}

export async function getMediaWithEtag(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<WithEtag<MediaDetail>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.getWithMeta<SingleResponse<MediaDetail>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}`,
  );
  return {
    data: normalizeMediaDetail(response.body!.data),
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
}

/**
 * Update a media item's metadata.
 */
export async function updateMedia(
  mediaId: string,
  request: UpdateMediaRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<MediaDetail> {
  const result = await updateMediaWithEtag(
    mediaId,
    request,
    fetchFn,
    accessToken,
  );
  return result.data;
}

export async function updateMediaWithEtag(
  mediaId: string,
  request: UpdateMediaRequest,
  fetchFn: typeof fetch,
  accessToken: string,
  options?: { ifMatch?: string },
): Promise<WithEtag<MediaDetail>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const headers = options?.ifMatch
    ? { "If-Match": options.ifMatch }
    : undefined;
  const response = await http.putWithMeta<SingleResponse<MediaDetail>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}`,
    request,
    headers,
  );
  return {
    data: normalizeMediaDetail(response.body!.data),
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
}

/**
 * Soft-delete a media item.
 */
export async function softDeleteMedia(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/soft-delete`,
    {},
  );
}

/**
 * Restore a soft-deleted media item.
 */
export async function restoreMedia(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string,
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
  accessToken: string,
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
  accessToken: string,
): Promise<InitiateUploadResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<InitiateUploadResponse>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/initiate-upload`,
    request,
  );
  return {
    ...response,
    uploadPlan: {
      ...response.uploadPlan,
      uploadUrl: normalizeLocalUploadPlanUrl(response.uploadPlan.uploadUrl),
    },
  };
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
  accessToken: string,
): Promise<FinaliseUploadResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<FinaliseUploadResponse>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/${encodeURIComponent(versionId)}/finalise-upload`,
    request,
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
  accessToken: string,
): Promise<MediaVersion[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<MediaVersion>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions`,
  );
  return response.data.map(normalizeMediaVersion);
}

/**
 * Activate (set as current) a specific version.
 */
export async function activateVersion(
  mediaId: string,
  versionId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/${encodeURIComponent(versionId)}/activate`,
    {},
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
  accessToken: string,
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/versions/${encodeURIComponent(versionId)}`,
  );
}

/**
 * List all usages of a media item.
 */
export async function listUsages(
  mediaId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<MediaUsage[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<MediaUsage>>(
    `/v1/admin/media/${encodeURIComponent(mediaId)}/usage`,
  );
  return response.data;
}

// ============================================================================
// Batch Operations
// ============================================================================

/** Batch delete request payload. */
export interface BatchDeleteMediaRequest {
  ids: string[];
}

/** Batch delete result. */
export interface BatchDeleteMediaResult {
  ok: boolean;
  deleted: number;
}

/**
 * Batch soft delete media items.
 */
export async function batchDeleteMedia(
  request: BatchDeleteMediaRequest,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<BatchDeleteMediaResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.post<BatchDeleteMediaResult>(
    "/v1/admin/media:batch-delete",
    request,
  );
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
export function getMediaDownloadUrl(
  mediaId: string,
  restricted = false,
): string {
  if (restricted) {
    return `/v1/media/${encodeURIComponent(mediaId)}/download`;
  }
  return `/v1/media/${encodeURIComponent(mediaId)}`;
}
