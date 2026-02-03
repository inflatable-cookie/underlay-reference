// Types
export type { ApiError } from "./types/common-types.js";
export type {
  // Auth types
  RegisterRequest,
  LoginRequest,
  LoginStartRequest,
  LoginStartResponse,
  LoginFinishRequest,
  LoginEmailFallbackRequest,
  LoginEmailFallbackResponse,
  LoginEmailResendRequest,
  RefreshRequest,
  LogoutRequest,
  ChangePasswordRequest,
  TotpSetupResponse,
  TotpEnableRequest,
  TotpStatusResponse,
  TwoFactorStatusResponse,
  PasskeyStartResponse,
  PasskeyRegisterFinishRequest,
  PasskeyLoginStartRequest,
  PasskeyLoginFinishRequest,
  PasskeyCredential,
  PasskeyRenameRequest,
  GoogleOAuthStartResponse,
  GoogleOAuthCallbackRequest,
  GoogleOAuthStatusResponse,
  GoogleOAuthTokenResponse,
  EmailTotpPurpose,
  EmailTotpRequestRequest,
  EmailTotpRequestResponse,
  EmailTotpVerifyRequest,
  EmailTotpVerifyResponse,
  VerificationSessionResponse,
  TotpVerifyRequest,
  PasskeyVerifyStartRequest,
  PasskeyVerifyFinishRequest,
  ChangePasswordWithVerificationRequest,
  PasswordRequirements,
  LoginUser,
  LoginResponse,
  Session,
  PasswordResetRequestRequest,
  PasswordResetVerifyRequest,
  PasswordResetVerifyResponse,
  PasswordResetCompleteRequest,
  ListResponse,
  SingleResponse,
} from "./types/common-types.js";

// Admin types
export type {
  Category,
  CategoryWithCounts,
  CreateCategoryPayload,
  UpdateCategoryPayload,
  Project,
  ProjectWithCounts,
  CreateProjectPayload,
  UpdateProjectPayload,
  Task,
  TaskWithLabels,
  CreateTaskPayload,
  UpdateTaskPayload,
  Label,
  CreateLabelPayload,
  SetLabelsPayload,
  ReorderPayload,
  ReorderResult,
  ValidateFieldPayload,
  ValidationResult,
  // User management types
  User,
  UserDetail,
  ListUsersQuery,
  UpdateUserRolePayload,
  UserListResponse,
  // Dashboard types
  UserCounts,
  DashboardStats,
  // Activity/Audit types
  ActivityActor,
  ActivityEntry,
  ActivityListResponse,
  ListActivityQuery,
  // Batch operation types
  BatchDeletePayload,
  BatchDeleteResult,
  BatchUpdateTaskStatusPayload,
  BatchUpdateResult,
  // Job types
  JobProgress,
  JobSummary,
  JobDetail,
  JobStats,
  ListJobsQuery,
} from "./types/admin-types.js";

export type { JobStatus } from "./types/admin-types.js";

export {
  ProjectStatus,
  TaskStatus,
  TaskPriority,
  UserRole,
  UserStatus,
} from "./types/admin-types.js";

// Utilities - re-export from Underlay
export { toUserMessage } from "@decodelabs/underlay/client";

// Query utilities - re-export from Underlay
export {
  // Types
  type SortDirection,
  type SortField,
  type FilterOperator,
  type FilterField,
  type QueryParams,
  // Functions
  orderByToSortFields,
  serializeSort,
  parseSort,
  buildQueryString,
  appendQueryParams,
  createFilterBuilder,
  parseQueryParams,
} from "@decodelabs/underlay/client";

// Selection history and suggestion utilities - re-export from Underlay
export {
  createSelectionHistory,
  formatHintsParam,
  parseHintsParam,
  buildSuggestionParams,
  appendSuggestionParams,
  type SelectionHistoryOptions,
  type SelectionHistory,
  type SuggestionRequestOptions,
} from "@decodelabs/underlay/patterns";

// Client factory (for apps to configure)
export {
  configureAcmeClient,
  getHttpClient,
  getAdminHttpClient,
  getFrontHttpClient,
  getSharedHttpClient,
} from "./utils/client-factory.js";
export type { AcmeClientConfig, HttpClientOptions, Audience } from "./utils/client-factory.js";

// Token store
export { createTokenStore, tokenStore } from "./utils/token-store.js";
export type { TokenStore, TokenState } from "./utils/token-store.js";

// Auth manager
export {
  createAuthManager,
  configureAuthManager,
  getAuthManager,
} from "./utils/auth-manager.js";
export type { AuthManager, AuthManagerConfig, AuthState } from "./utils/auth-manager.js";

// Account types
export type { UserProfile, UserProfileUpdate } from "./types/account-types.js";

// Media types (re-exported from Underlay patterns)
export {
  // Enums
  MediaKind,
  MediaVisibility,
  MediaVersionState,
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
} from "./types/media-types.js";
export type {
  MediaSummary,
  MediaDetail,
  MediaVersion,
  MediaRendition,
  MediaUsage,
  CreateMediaRequest,
  UpdateMediaRequest,
  CheckDuplicateRequest,
  CheckDuplicateResponse,
  InitiateUploadRequest,
  InitiateUploadResponse,
  MediaUploadPlan,
  FinaliseUploadRequest,
  FinaliseUploadResponse,
  MediaListQuery,
} from "./types/media-types.js";

// Commands
export * as healthCommands from "./commands/health-commands.js";
export * as authCommands from "./commands/auth-commands.js";
export * as accountCommands from "./commands/account-commands.js";
export * as adminCommands from "./commands/admin-commands.js";
export * as userCommands from "./commands/user-commands.js";
export * as mediaCommands from "./commands/media-commands.js";
export type {
  BatchDeleteMediaRequest,
  BatchDeleteMediaResult,
} from "./commands/media-commands.js";

// User types
export type {
  UserProject,
  UserTask,
  CreateUserProjectPayload,
  UpdateUserProjectPayload,
  CreateUserTaskPayload,
  UpdateUserTaskPayload,
} from "./commands/user-commands.js";
