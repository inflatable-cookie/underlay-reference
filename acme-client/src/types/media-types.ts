/**
 * Media Library types for acme-client.
 *
 * Re-exports types from Underlay client helpers for use in Acme applications.
 */

// Re-export all media types from Underlay client helpers
export {
  // Enums
  MediaKind,
  MediaVisibility,
  MediaVersionState,
  // DTOs
  type MediaSummary,
  type MediaDetail,
  type MediaVersion,
  type MediaRendition,
  type MediaUsage,
  // Request/Response types
  type CreateMediaRequest,
  type UpdateMediaRequest,
  type CheckDuplicateRequest,
  type CheckDuplicateResponse,
  type InitiateUploadRequest,
  type InitiateUploadResponse,
  type MediaUploadPlan,
  type FinaliseUploadRequest,
  type FinaliseUploadResponse,
  // Query parameters
  type MediaListQuery,
  // Utility functions
  getMediaKindLabel,
  getMediaKindAccent,
  getMediaVisibilityLabel,
  getMediaVisibilityAccent,
  getMediaVersionStateLabel,
  getMediaVersionStateAccent,
  detectMediaKindFromMimeType,
  isMediaDeleted,
  getMediaDisplayName,
} from "@decodelabs/underlay/client/media";

// Re-export pagination types that may be used with media
export type {
  PaginatedResponse,
  PaginationParams,
} from "@decodelabs/underlay/client/pagination";
