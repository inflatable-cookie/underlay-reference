import {
  DEFAULT_MEDIA_UPLOAD_MAX_FILE_SIZE,
  createMediaUploadPipeline,
  type UploadProgress
} from "@decodelabs/underlay/runtime/media/upload";
import { detectMediaKindFromMimeType, mediaCommands } from "@api-client";

export const MAX_FILE_SIZE = DEFAULT_MEDIA_UPLOAD_MAX_FILE_SIZE;

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

const pipeline = createMediaUploadPipeline({
  detectKind: detectMediaKindFromMimeType,
  createMedia: (request, context: { fetchFn: typeof fetch; accessToken: string }) =>
    mediaCommands.createMedia(request, context.fetchFn, context.accessToken),
  initiateUpload: (mediaId, request, context: { fetchFn: typeof fetch; accessToken: string }) =>
    mediaCommands.initiateUpload(mediaId, request, context.fetchFn, context.accessToken),
  finaliseUpload: (mediaId, versionId, request, context: { fetchFn: typeof fetch; accessToken: string }) =>
    mediaCommands.finaliseUpload(mediaId, versionId, request, context.fetchFn, context.accessToken),
  checkDuplicate: (request, context: { fetchFn: typeof fetch; accessToken: string }) =>
    mediaCommands.checkDuplicate(request, context.fetchFn, context.accessToken)
});

export async function createAndUpload(
  options: CreateAndUploadOptions
): Promise<UploadPipelineResult> {
  return pipeline.createAndUpload(options);
}

export async function replaceUpload(
  options: ReplaceUploadOptions
): Promise<UploadPipelineResult> {
  return pipeline.replaceUpload(options);
}

export async function checkDuplicate(
  file: File,
  fetchFn: typeof fetch,
  accessToken: string
) {
  return pipeline.checkDuplicate(file, { fetchFn, accessToken });
}
