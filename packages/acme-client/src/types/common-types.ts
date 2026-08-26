export interface ApiError extends Error {
  status: number;
  code: string;
  details?: Record<string, unknown> | unknown;
  requestId?: string;
  raw?: unknown;
}

export interface ListResponse<T> {
  data: T[];
}

export interface PagedListResponse<T> {
  data: T[];
  total: number;
  hasMore?: boolean;
}

export interface SingleResponse<T> {
  data: T;
}

// ============================================================================
// Auth Types
// ============================================================================

export interface RegisterRequest {
  email: string;
  password: string;
  displayName: string;
}

export interface LoginRequest {
  email: string;
  password: string;
  code?: string;
}

export interface LoginStartRequest {
  email: string;
  password: string;
  /** When true, if user has no 2FA configured, email verification will be required. */
  enforceEmailFallback?: boolean;
}

export interface LoginStartResponse {
  requiresTwoFactor: boolean;
  /** When true, the 2FA is via email code (not TOTP/passkey). */
  isEmailVerification?: boolean | null;
  loginStateId?: string | null;
  session?: LoginResponse | null;
}

export interface LoginFinishRequest {
  loginStateId: string;
  code: string;
}

export interface RefreshRequest {
  refreshToken: string;
}

export interface LogoutRequest {
  refreshToken: string;
}

export interface ChangePasswordRequest {
  currentPassword: string;
  newPassword: string;
}

export interface TotpSetupResponse {
  setupId: string;
  otpauthUri: string;
  qrSvg: string;
  backupCodes: string[];
}

export interface TotpEnableRequest {
  setupId: string;
  code: string;
}

export interface TotpStatusResponse {
  enabled: boolean;
}

export interface TwoFactorStatusResponse {
  hasTotpConfigured: boolean;
  hasPasskeyConfigured: boolean;
  totpEnabledAt?: string | null;
  passkeyCount: number;
}

export interface PasskeyStartResponse {
  options: unknown;
  stateId: string;
}

export interface PasskeyRegisterFinishRequest {
  stateId: string;
  credential: unknown;
  displayName?: string;
}

export interface PasskeyLoginStartRequest {
  email?: string;
}

export interface PasskeyLoginFinishRequest {
  stateId: string;
  credential: unknown;
}

export interface PasskeyCredential {
  id: string;
  displayName?: string | null;
  metadata: unknown;
  createdAt: string;
  lastUsedAt?: string | null;
}

export interface PasskeyRenameRequest {
  displayName: string;
}

// ============================================================================
// Email TOTP
// ============================================================================

/** Purpose for email TOTP verification */
export type EmailTotpPurpose = 'login' | 'password_change' | 'sensitive_action' | 'password_reset';

export interface EmailTotpRequestRequest {
  purpose: EmailTotpPurpose;
}

export interface EmailTotpRequestResponse {
  expiresAt: string;
}

export interface EmailTotpVerifyRequest {
  code: string;
  purpose: EmailTotpPurpose;
}

export interface EmailTotpVerifyResponse {
  verificationSessionId: string;
  expiresAt: string;
}

// Common verification response type (same structure for all 2FA methods)
export type VerificationSessionResponse = EmailTotpVerifyResponse;

// TOTP Verification (authenticator app)
export interface TotpVerifyRequest {
  code: string;
  purpose: EmailTotpPurpose;
}

// Passkey Verification
export interface PasskeyVerifyStartRequest {
  purpose: EmailTotpPurpose;
}

export interface PasskeyVerifyFinishRequest {
  stateId: string;
  credential: unknown;
}

// Password change with verification session
export interface ChangePasswordWithVerificationRequest {
  verificationSessionId: string;
  newPassword: string;
}

// Password requirements (fetched from server)
export interface PasswordRequirements {
  minLength: number;
  requireMixedCase: boolean;
  requireDigit: boolean;
  requireSpecial: boolean;
  minStrengthScore: number;
  description: string;
}

export interface LoginUser {
  userId: string;
  email: string;
  displayName: string;
  roles: string[];
}

export interface LoginResponse {
  sessionId: string;
  accessToken: string;
  refreshToken?: string;
  user: LoginUser;
}

export interface Session {
  id: string;
  userId: string;
  accessTokenExpiresAt: string;
  refreshTokenExpiresAt: string;
  createdAt: string;
  lastUsedAt: string;
  ipAddress?: string | null;
  userAgent?: string | null;
  status: "active" | "expired" | "revoked";
  revocationReason?: string | null;
  revokedAt?: string | null;
}

// ============================================================================
// Password Reset
// ============================================================================

export interface PasswordResetRequestRequest {
  email: string;
}

export interface PasswordResetVerifyRequest {
  email: string;
  code: string;
}

export interface PasswordResetVerifyResponse {
  resetToken: string;
}

export interface PasswordResetCompleteRequest {
  resetToken: string;
  newPassword: string;
}
