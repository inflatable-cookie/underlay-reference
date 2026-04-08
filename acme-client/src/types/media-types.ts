/**
 * Media Library types for acme-client.
 *
 * Re-exports types from Underlay runtime helpers for use in Acme applications.
 */

// Re-export all media types from Underlay runtime helpers
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
} from "@decodelabs/underlay/runtime/media";

// Re-export pagination types that may be used with media
export type {
  PaginatedResponse,
  PaginationParams,
} from "@decodelabs/underlay/runtime/data";
