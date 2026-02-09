# Security Audit Remediation Roadmap

Roadmap for addressing security findings from the comprehensive security audit of the Underlay Reference Implementation (Acme).

**Overall Security Grade: B+**

---

## Executive Summary

The reference implementation demonstrates strong security practices with modern authentication standards. Most critical areas are well-implemented, but several improvements are needed before production use. This roadmap prioritizes fixes from critical to low priority.

---

## Phase 1: Critical Security Fixes

**Priority: IMMEDIATE - Fix before production deployment**

### 1.1 Distributed Rate Limiting
Replace in-memory rate limiter with Redis-backed implementation to prevent bypass in multi-instance deployments.

- [ ] Add Redis rate limiting backend
  - [ ] Add `redis` or `fred` crate dependency to auth crate
  - [ ] Create `RedisRateLimitBackend` implementing rate limiter trait
  - [ ] Update `AcmeLocalAuthProvider::new()` to use Redis backend when `REDIS_URL` is set
  - [ ] Add fallback to in-memory for single-instance deployments
- [ ] Update configuration
  - [ ] Add `RATE_LIMIT_BACKEND` env var (redis|memory)
  - [ ] Add `REDIS_URL` configuration
- [ ] Testing
  - [ ] Test rate limit sharing across multiple API instances
  - [ ] Verify rate limits reset correctly

**Files:**
- `acme-api/crates/auth/src/local/rate_limit.rs` (create Redis backend)
- `acme-api/crates/auth/src/local/mod.rs` (update initialization)
- `acme-api/.env.example` (add new env vars)

---

### 1.2 Rate Limiting on Token Refresh
Add rate limiting to `/auth/refresh` endpoint to prevent refresh token enumeration attacks.

- [ ] Add refresh token rate limit
  - [ ] Add `refresh` rate limit bucket (60 requests per hour per fingerprint)
  - [ ] Apply rate limit middleware to refresh endpoint
  - [ ] Return 429 status with Retry-After header when exceeded
- [ ] Update API client to handle 429 responses
  - [ ] Add exponential backoff for retry
  - [ ] Log rate limit events for monitoring

**Files:**
- `acme-api/crates/api/src/routes/shared/auth/basic.rs` (lines 345-401)
- `acme-client/src/commands/auth/core-commands.ts` (add retry logic)

---

### 1.3 CSRF Protection for Cookie Authentication
Implement protection against cross-site request forgery for cookie-based authentication.

- [ ] Backend changes
  - [ ] Set `SameSite=Strict` on refresh token cookies by default
  - [ ] Add `X-CSRF-Token` header validation for state-changing requests
  - [ ] Add CSRF token generation endpoint `GET /auth/csrf-token`
  - [ ] Store CSRF token in session (not in cookie)
- [ ] Frontend changes
  - [ ] Fetch CSRF token on app load
  - [ ] Include CSRF token in all POST/PUT/DELETE request headers
  - [ ] Handle CSRF token expiration (401 with specific error code)
- [ ] Configuration
  - [ ] Add `CSRF_PROTECTION` env var (true|false, default true)
  - [ ] Add `SAME_SITE_COOKIE` env var (strict|lax|none)

**Files:**
- `acme-api/crates/api/src/routes/shared/auth/mod.rs` (add CSRF endpoint)
- `acme-api/crates/api/src/routes/mod.rs` (add CSRF middleware)
- `acme-client/src/utils/http-client.ts` (add CSRF header)
- `acme-client/src/utils/auth-manager.ts` (fetch/manage CSRF token)
- `acme-admin/src/lib/utils/auth-tokens.ts` (update cookie settings)

---

### 1.4 Admin Privilege Escalation Prevention
Add role hierarchy to prevent admins from promoting users to superadmin or modifying other admins.

- [ ] Backend changes
  - [ ] Add `Role::can_manage_role(&self, target: Role) -> bool` method
  - [ ] Only superadmins can promote to superadmin
  - [ ] Prevent admins from suspending other admins
  - [ ] Prevent self-suspension
  - [ ] Add audit log entries for role changes
- [ ] Frontend changes
  - [ ] Hide role options user doesn't have permission to assign
  - [ ] Disable suspend button for protected users
  - [ ] Show permission denied message appropriately
- [ ] Testing
  - [ ] Test role hierarchy enforcement
  - [ ] Test audit logging

**Files:**
- `acme-api/crates/api/src/routes/admin/users.rs` (lines 473-535)
- `acme-api/crates/domain/src/auth/role.rs` (add hierarchy)
- `acme-admin/src/routes/(app)/users/[userId]/+page.svelte` (UI restrictions)

---

## Phase 2: High Priority Security Improvements

**Priority: Fix before public beta**

### 2.1 Encrypt TOTP Secrets at Rest
Add encryption layer for TOTP secrets stored in database.

- [ ] Create encryption service
  - [ ] Add AES-256-GCM encryption for TOTP secrets
  - [ ] Store encryption key in environment (separate from database)
  - [ ] Create `TotpEncryptionService` with encrypt/decrypt methods
  - [ ] Handle key rotation (decrypt with old key, encrypt with new)
- [ ] Update TOTP storage
  - [ ] Encrypt secrets before storing in `auth.totp_secrets` table
  - [ ] Decrypt when needed for verification
  - [ ] Update database schema to mark column as encrypted
- [ ] Migration
  - [ ] Create migration to encrypt existing secrets
  - [ ] Allow plaintext reading for backwards compatibility during transition

**Files:**
- `acme-api/crates/auth/src/local/totp.rs` (lines 66-176)
- `acme-api/crates/infra/src/encryption.rs` (new)
- `acme-api/migrations/` (new migration)

---

### 2.2 Implement Absolute Session Timeout
Force re-authentication after maximum session lifetime regardless of activity.

- [ ] Backend changes
  - [ ] Add `max_absolute_session_duration_days` config (default 30)
  - [ ] Store `session_created_at` timestamp on session
  - [ ] Check absolute timeout on token refresh
  - [ ] Return specific error when absolute timeout exceeded
  - [ ] Revoke all user sessions when absolute timeout reached
- [ ] Frontend changes
  - [ ] Handle `session_expired_absolute` error
  - [ ] Redirect to login with appropriate message
  - [ ] Show session expiration warning (e.g., 24 hours before)
- [ ] Configuration
  - [ ] Add `SESSION_MAX_ABSOLUTE_DAYS` env var
  - [ ] Add `SESSION_EXPIRY_WARNING_HOURS` env var

**Files:**
- `acme-api/crates/auth/src/local/session.rs` (add absolute timeout check)
- `acme-api/crates/db/src/auth/session.rs` (add session_created_at field)
- `acme-client/src/utils/auth-manager.ts` (handle new error)

---

### 2.3 Proxy Trust Configuration for IP Headers
Prevent IP spoofing by validating X-Forwarded-For and X-Real-IP headers.

- [ ] Create IP extraction service
  - [ ] Add `TRUSTED_PROXIES` configuration (IP ranges or "none")
 - [ ] Parse `X-Forwarded-For` chain correctly
  - [ ] Use rightmost trusted IP in chain
  - [ ] Validate that proxy IPs are in trusted list
  - [ ] Fallback to direct connection IP if headers untrusted
- [ ] Update session fingerprinting
  - [ ] Use trusted IP extraction in `extract_client_ip`
  - [ ] Log warning when proxy headers used without trust config
- [ ] Configuration
  - [ ] Add `TRUSTED_PROXIES` env var (comma-separated IPs/CIDR)
  - [ ] Add `USE_PROXY_HEADERS` env var (true|false, default false)

**Files:**
- `acme-api/crates/api/src/routes/shared/auth/mod.rs` (lines 78-99)
- `acme-api/crates/infra/src/network.rs` (new - IP extraction)
- `acme-api/crates/api/src/main.rs` (add config)

---

### 2.4 Add Input Validation to Task/Project DTOs
Prevent DoS via unvalidated input lengths.

- [ ] Add validation to CreateProjectRequest
  - [ ] `name`: min 1, max 255 chars
  - [ ] `description`: max 2000 chars
- [ ] Add validation to UpdateProjectRequest
  - [ ] Same limits as create
- [ ] Add validation to CreateTaskRequest
  - [ ] `title`: min 1, max 255 chars
  - [ ] `description`: max 5000 chars
  - [ ] `priority`: must be valid enum value
  - [ ] `status`: must be valid enum value
- [ ] Add validation to UpdateTaskRequest
  - [ ] Same limits as create
- [ ] Update API client to enforce same limits
  - [ ] Add client-side validation before API calls
  - [ ] Show validation errors in forms

**Files:**
- `acme-api/crates/api/src/routes/tasks.rs` (lines 48-114)
- `acme-client/src/commands/task-commands.ts` (add validation)
- `acme-admin/src/lib/forms/ProjectForm.svelte` (add client validation)
- `acme-admin/src/lib/forms/TaskForm.svelte` (add client validation)

---

## Phase 3: Medium Priority Security Hardening

**Priority: Fix before production release**

### 3.1 Increase Minimum Password Length
Reject weak passwords at API validation layer.

- [ ] Update DTO validation
  - [ ] Change min password length from 1 to 12 characters
  - [ ] Add pattern validation for complexity (optional)
  - [ ] Update error message
- [ ] Update password requirements endpoint
  - [ ] Return min length of 12 in `/auth/password-requirements`
  - [ ] Ensure consistency between frontend and backend
- [ ] Testing
  - [ ] Test password validation with various lengths
  - [ ] Verify error messages are helpful

**Files:**
- `acme-api/crates/api/src/dto/auth.rs` (lines 18-23)
- `acme-api/crates/auth/src/local/mod.rs` (update password analyzer to match)

---

### 3.2 Default Cookie Secure Flag to True
Ensure cookies are only sent over HTTPS by default in production.

- [ ] Update configuration
  - [ ] Change default `COOKIE_SECURE` from `false` to `true`
  - [ ] Add explicit warning when set to `false` in production
  - [ ] Add `ENVIRONMENT` check - allow `false` only in local/dev
- [ ] Documentation
  - [ ] Document requirement for HTTPS in production
  - [ ] Add troubleshooting guide for local development

**Files:**
- `acme-api/crates/api/src/main.rs` (lines 52-55)
- `acme-api/.env.example` (update documentation)

---

### 3.3 Enforce CSP in Production
Change CSP from report-only to enforcement mode.

- [ ] Admin frontend
  - [ ] Set `reportOnly: false` in production builds
  - [ ] Review CSP violations in logs before enabling
  - [ ] Add `unsafe-inline` exceptions if needed for Svelte
- [ ] Public frontend
  - [ ] Same changes as admin
- [ ] Add security headers
  - [ ] `X-Content-Type-Options: nosniff`
  - [ ] `X-Frame-Options: DENY` (or `SAMEORIGIN`)
  - [ ] `Referrer-Policy: strict-origin-when-cross-origin`
- [ ] Configuration
  - [ ] Add `CSP_REPORT_ONLY` env var (default false in prod)

**Files:**
- `acme-admin/src/hooks.server.ts` (lines 21)
- `acme-front/src/hooks.server.ts` (lines 21)

---

### 3.4 Strengthen Argon2 Parameters
Increase password hashing work factor for new deployments.

- [ ] Update default parameters
  - [ ] Memory: 64MB → 128MB (131,072 KB)
  - [ ] Iterations: 3 → 4
  - [ ] Keep parallelism at 4
- [ ] Migration strategy
  - [ ] Store Argon2 parameters with each password hash
  - [ ] Verify using stored parameters
  - [ ] Re-hash with new parameters on successful login (gradual migration)
- [ ] Configuration
  - [ ] Document resource requirements (128MB per hash operation)
  - [ ] Add warnings about memory usage in multi-tenant environments

**Files:**
- `acme-api/crates/auth/src/local/mod.rs` (lines 247-262)
- `acme-api/crates/auth/src/local/password.rs` (lines 77-118)

---

## Phase 4: Defense in Depth Improvements

**Priority: Nice to have, implement as time allows**

### 4.1 Configurable Session Fingerprint Strictness
Allow applications to choose between lenient and strict fingerprint validation.

- [ ] Add configuration
  - [ ] Add `SESSION_FINGERPRINT_MODE` env var (lenient|strict)
  - [ ] Lenient: Log mismatch warnings only (current behavior)
  - [ ] Strict: Require additional verification for significant changes
- [ ] Strict mode implementation
  - [ ] When fingerprint changes, require email verification
  - [ ] Send security alert email on suspicious changes
  - [ ] Allow user to approve new device/location
- [ ] Testing
  - [ ] Test both modes
  - [ ] Verify security alerts are sent

**Files:**
- `acme-api/crates/auth/src/local/session.rs` (lines 44-58)
- `acme-api/crates/auth/src/local/mod.rs` (add config)

---

### 4.2 JWT Key Rotation Mechanism
Support periodic rotation of JWT signing keys.

- [ ] Create key rotation system
  - [ ] Store multiple JWT keys (active + previous)
  - [ ] Sign new tokens with active key
  - [ ] Verify tokens with any non-expired key
  - [ ] Add `key_id` to JWT header
- [ ] Key generation
  - [ ] Add `generate-jwt-env.rs` support for rotation
  - [ ] Document rotation schedule (e.g., every 90 days)
  - [ ] Add key expiration (e.g., 180 days after rotation)
- [ ] Monitoring
  - [ ] Log when keys are near expiration
  - [ ] Alert on key validation failures

**Files:**
- `acme-api/crates/auth/src/bin/generate-jwt-env.rs`
- `acme-api/crates/auth/src/local/mod.rs` (lines 210-213)

---

### 4.3 Use citext for Email Storage
Improve performance and correctness of email lookups.

- [ ] Database changes
  - [ ] Enable `citext` PostgreSQL extension
  - [ ] Change `auth.users.email` from `TEXT` to `CITEXT`
  - [ ] Update indexes for case-insensitive lookups
- [ ] Code updates
  - [ ] Remove `LOWER(email)` from queries
  - [ ] Update email normalization in registration
- [ ] Migration
  - [ ] Create migration to convert existing emails
  - [ ] Handle duplicates that differ only in case

**Files:**
- `acme-api/migrations/` (new migration)
- `acme-api/crates/db/src/auth/users.rs` (remove LOWER calls)
- `acme-api/crates/api/src/routes/shared/auth/password_reset.rs` (lines 18-28)

---

### 4.4 HSM/KMS Integration Documentation
Guide for enterprise key management.

- [ ] Documentation
  - [ ] Document AWS KMS integration for JWT keys
  - [ ] Document HashiCorp Vault integration
  - [ ] Document Azure Key Vault integration
- [ ] Example implementation
  - [ ] Create example code for KMS-based signing
  - [ ] Show key rotation with external KMS
  - [ ] Document performance considerations

**Files:**
- `acme-docs/architecture/004-key-management.md` (new)

---

## Phase 5: Security Monitoring & Auditing

**Priority: Implement for production monitoring**

### 5.1 Security Event Logging
Add structured logging for security-relevant events.

- [ ] Define security events
  - [ ] Failed login attempts
  - [ ] Successful logins (with location/device info)
  - [ ] Password changes
  - [ ] 2FA enable/disable
  - [ ] Session revocations
  - [ ] Role changes
  - [ ] Rate limit violations
- [ ] Implementation
  - [ ] Add `SecurityEvent` enum
  - [ ] Create security logger with structured JSON output
  - [ ] Include request ID, user ID, IP, timestamp
- [ ] Integration
  - [ ] Send to SIEM if configured
  - [ ] Store in separate database table for querying

**Files:**
- `acme-api/crates/infra/src/security/` (new directory)
- `acme-api/crates/api/src/middleware/security_logging.rs` (new)

---

### 5.2 Failed Login Monitoring
Detect and alert on brute force attacks.

- [ ] Metrics collection
  - [ ] Track failed login attempts per IP
  - [ ] Track failed login attempts per account
  - [ ] Track password reset attempts
- [ ] Alerting
  - [ ] Alert when threshold exceeded (e.g., 100 failed attempts/minute from IP)
  - [ ] Alert on distributed attacks (same account, multiple IPs)
  - [ ] Integration with PagerDuty/Slack
- [ ] Dashboard
  - [ ] Add security metrics to admin dashboard
  - [ ] Show failed login attempts graph
  - [ ] List blocked IPs

**Files:**
- `acme-api/crates/infra/src/metrics.rs` (new)
- `acme-admin/src/routes/(app)/dashboard/+page.svelte` (add security section)

---

### 5.3 Audit Log for Security Events
Extend activity logging to cover all security-relevant actions.

- [ ] Add security events to audit log
  - [ ] Login/logout events
  - [ ] Permission changes
  - [ ] Configuration changes
  - [ ] API key generation/revocation
- [ ] Tamper protection
  - [ ] Consider append-only audit log table
  - [ ] Hash chain for log integrity
  - [ ] Separate write permissions for audit table
- [ ] Retention
  - [ ] Configurable retention policy
  - [ ] Archive old logs to cold storage

**Files:**
- `acme-api/crates/db/src/activity.rs`
- `acme-api/migrations/` (audit log enhancements)

---

## Success Criteria

This security roadmap is complete when:

- [ ] All Phase 1 critical issues are resolved
- [ ] All Phase 2 high priority issues are resolved
- [ ] Security tests pass (rate limiting, CSRF, etc.)
- [ ] Security documentation is complete
- [ ] Penetration testing is performed
- [ ] Security monitoring is in place

## Implementation Order

1. **Phase 1** - Critical fixes (immediate)
2. **Phase 2** - High priority (before beta)
3. **Phase 3** - Medium priority (before production)
4. **Phase 4** - Defense in depth (ongoing)
5. **Phase 5** - Monitoring (production readiness)

## Security Testing Checklist

Before marking each phase complete, verify:

- [ ] Rate limiting works across multiple instances
- [ ] CSRF tokens prevent cross-site requests
- [ ] Admin privilege escalation is blocked
- [ ] TOTP secrets are encrypted in database
- [ ] Sessions expire after absolute timeout
- [ ] IP spoofing is prevented
- [ ] Input validation rejects oversized data
- [ ] Password requirements are enforced
- [ ] Cookies are secure and SameSite
- [ ] CSP headers are enforced
- [ ] Argon2 uses secure parameters
- [ ] Fingerprint validation works in both modes
- [ ] JWT keys can be rotated
- [ ] Security events are logged
- [ ] Failed login monitoring alerts appropriately

## Notes

- Each fix should include tests
- Update security documentation as fixes are implemented
- Consider third-party security audit after Phase 3
- Keep dependencies updated (cargo audit, npm audit)
- Document any intentional security trade-offs
