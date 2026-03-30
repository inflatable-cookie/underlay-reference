import {
  computeFileHash,
  uploadToBlob,
  type UploadProgress,
} from "@decodelabs/underlay/runtime";
import { mediaCommands, detectMediaKindFromMimeType } from "@api-client";
/**
 * Shared upload pipeline for media files.
 *
 * Encapsulates the hash → initiate → upload → finalize sequence
 * used by both the bulk upload queue and single-file replace mode.
 */

// File size limit (50MB)
export const MAX_FILE_SIZE = 50 * 1024 * 1024;

export interface UploadPipelineOptions {
  file: File;
  fetchFn: typeof fetch;
  accessToken: string;
  onProgress?: (progress: UploadProgress) => void;
}

export interface CreateAndUploadOptions extends UploadPipelineOptions {
  title?: string | null;
  visibility?: string;
}

export interface ReplaceUploadOptions extends UploadPipelineOptions {
  mediaId: string;
}

export interface UploadPipelineResult {
  mediaId: string;
  hash: string;
}

/**
 * Build the upload plan object from an initiate-upload response.
 */
function buildPlan(uploadInfo: {
  uploadPlan: {
    uploadUrl: string;
    method: string;
    headers: Record<string, string>;
    maxBytes?: number | null;
    allowedContentTypes?: string[] | null;
    expiresAt: string;
  };
}) {
  return {
    uploadUrl: uploadInfo.uploadPlan.uploadUrl,
    method: uploadInfo.uploadPlan.method,
    requiredHeaders: uploadInfo.uploadPlan.headers,
    maxBytes: uploadInfo.uploadPlan.maxBytes ?? MAX_FILE_SIZE,
    allowedContentTypes: uploadInfo.uploadPlan.allowedContentTypes ?? [],
    expiresAt: uploadInfo.uploadPlan.expiresAt,
    objectKey: "",
  };
}

/**
 * Upload a file to blob storage and finalize the version.
 *
 * Shared step used by both createAndUpload and replaceUpload.
 */
async function uploadAndFinalize(
  mediaId: string,
  file: File,
  hash: string,
  fetchFn: typeof fetch,
  accessToken: string,
  onProgress?: (progress: UploadProgress) => void,
): Promise<void> {
  const uploadInfo = await mediaCommands.initiateUpload(
    mediaId,
    { contentType: file.type, contentLength: file.size },
    fetchFn,
    accessToken,
  );

  const plan = buildPlan(uploadInfo);

  await uploadToBlob(plan, file, {
    onProgress: onProgress
      ? (progress: UploadProgress) => onProgress(progress)
      : undefined,
  });

  await mediaCommands.finaliseUpload(
    mediaId,
    uploadInfo.versionId,
    { sha256: hash, contentType: file.type },
    fetchFn,
    accessToken,
  );
}

/**
 * Create a new media record and upload the file.
 *
 * Steps: hash → create media → initiate upload → upload blob → finalize.
 */
export async function createAndUpload(
  options: CreateAndUploadOptions,
): Promise<UploadPipelineResult> {
  const { file, fetchFn, accessToken, onProgress, title, visibility } = options;

  const hash = await computeFileHash(file);

  const kind = detectMediaKindFromMimeType(file.type);
  const mediaRecord = await mediaCommands.createMedia(
    {
      kind,
      visibility: (visibility as "public" | "restricted") ?? "public",
      originalFilename: file.name,
      title: title ?? null,
    },
    fetchFn,
    accessToken,
  );

  await uploadAndFinalize(
    mediaRecord.id,
    file,
    hash,
    fetchFn,
    accessToken,
    onProgress,
  );

  return { mediaId: mediaRecord.id, hash };
}

/**
 * Upload a new version to an existing media record (replace).
 *
 * Steps: hash → initiate upload → upload blob → finalize.
 */
export async function replaceUpload(
  options: ReplaceUploadOptions,
): Promise<UploadPipelineResult> {
  const { file, mediaId, fetchFn, accessToken, onProgress } = options;

  const hash = await computeFileHash(file);

  await uploadAndFinalize(
    mediaId,
    file,
    hash,
    fetchFn,
    accessToken,
    onProgress,
  );

  return { mediaId, hash };
}

/**
 * Check if a file is a duplicate by its hash.
 */
export async function checkDuplicate(
  file: File,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<{
  hash: string;
  exists: boolean;
  media?: {
    id: string;
    title?: string | null;
    originalFilename?: string | null;
  } | null;
}> {
  const hash = await computeFileHash(file);
  const result = await mediaCommands.checkDuplicate(
    { sha256: hash },
    fetchFn,
    accessToken,
  );
  return { hash, exists: result.exists, media: result.media };
}
