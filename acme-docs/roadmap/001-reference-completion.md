# Reference Implementation Completion

Roadmap for completing the Underlay Reference Implementation (Acme) to serve as a comprehensive example for consuming applications.

## Current Status

**All core phases complete.** The reference implementation now demonstrates:
- Full authentication system (passwords, 2FA, passkeys, sessions)
- Media library with versioned uploads, deduplication, and thumbnails
- Task/Project domain with categories, labels, batch operations
- Admin and public frontends with CRUD, search, and filtering
- Activity/audit logging with admin visibility
- Background job system with scheduled tasks
- Comprehensive testing patterns
- Docker-based developer setup
- Standardized configuration across all Underlay crates

## Goals

Make this a **complete reference** that demonstrates:
- All common Underlay patterns
- Real-world admin features (user management, activity logs)
- Search and filtering patterns
- Background job patterns
- Testing patterns

---

## Phase 1: Admin Dashboard & User Management ✓

Flesh out the admin experience with essential features.

### Admin Dashboard
- [x] Dashboard overview page
  - [x] User count statistic
  - [x] Media count/storage used
  - [ ] Recent activity feed (last 10 changes) — deferred to Phase 3
  - [x] System health indicators

### User Management (Admin)
- [x] Backend: User listing endpoint
  - [x] `GET /v1/admin/users` with pagination
  - [x] Filters: role, status, search by email
- [x] Backend: User detail endpoint
  - [x] `GET /v1/admin/users/{userId}`
  - [x] Include sessions, 2FA status, created date
- [x] Backend: User role management
  - [x] `PUT /v1/admin/users/{userId}/role`
  - [x] Update user's role (user → admin, etc.)
- [x] Backend: User suspension
  - [x] `POST /v1/admin/users/{userId}/suspend`
  - [x] `POST /v1/admin/users/{userId}/unsuspend`
- [x] Client: Add user admin commands
  - [x] `listUsersAdmin()`, `getUser()`, `updateUserRole()`, `suspendUser()`
- [x] Admin UI: Users list page
  - [x] Table with email, role, status, created date
  - [x] Pagination
  - [x] Role/status filters
- [x] Admin UI: User detail page
  - [x] Profile info, role badge
  - [ ] Session list with revoke action — deferred to Phase 3
  - [x] Suspend/unsuspend action

**Files:**
- `acme-api/crates/api/src/routes/admin/users.rs` (new)
- `acme-api/crates/db/src/users.rs` (new or extend auth)
- `acme-client/src/commands/user-commands.ts` (new)
- `acme-admin/src/routes/(app)/users/+page.svelte` (new)
- `acme-admin/src/routes/(app)/users/[userId]/+page.svelte` (new)

---

## Phase 2: Search & Filtering ✓

Demonstrate search patterns across the application.

### API Search Patterns
- [x] Projects search
  - [x] `GET /v1/admin/projects?q=search&status=active&sortBy=title`
  - [ ] Full-text search on title/description — deferred
- [x] Tasks search
  - [x] `GET /v1/admin/projects/{projectId}/tasks?q=search&status=pending&priority=high`
  - [x] Filter by status, priority, due date range
- [x] Media search
  - [x] `GET /v1/admin/media?q=search&kind=image&unusedOnly=true`
  - [x] Wired to UI with FilterBar

### Admin UI Search
- [x] Projects list: Search input and filters (already implemented)
- [x] Tasks list: Status/priority filters and sorting
- [x] Media list: Search input and kind/visibility filter
- [x] Demonstrate debounced search pattern (TextInput with debounce prop)
- [x] Demonstrate URL-based filter state (shareable filter URLs)

**Files:**
- `acme-api/crates/api/src/routes/admin/projects.rs` (update)
- `acme-api/crates/api/src/routes/admin/tasks.rs` (update)
- `acme-admin/src/routes/(app)/projects/+page.svelte` (update)
- `acme-admin/src/routes/(app)/media/+page.svelte` (update)

---

## Phase 3: Activity & Audit Logging ✓

Track and display changes for admin visibility.

### Activity Log Infrastructure
- [x] Database: `platform.audit_log` table (already exists in baseline)
  - [x] Fields: id, user_id, action, resource_type, resource_id, details, occurred_at
  - [x] Index on entity, actor, created_at
- [x] Backend: Activity logging helper
  - [x] `log_activity(actor, action, entity_type, entity_id, metadata)`
  - [x] Call from relevant endpoints (create, update, delete)
- [x] Backend: Activity endpoints
  - [x] `GET /v1/admin/activity` - global activity feed
  - [x] `GET /v1/admin/activity/entity/:type/:id` - entity-specific
  - [x] `GET /v1/admin/users/{userId}/activity` - user's actions

### Admin UI
- [x] Activity feed component (reusable)
  - [x] Action icon, actor, description, timestamp
  - [x] Relative time display
- [x] Dashboard: Recent activity section
- [ ] Entity detail pages: Activity tab/section — deferred
- [x] User detail: User's activity history

**Files:**
- `acme-api/crates/db/src/activity.rs` (new)
- `acme-api/crates/api/src/routes/admin/activity.rs` (new)
- `acme-client/src/commands/admin-commands.ts` (updated with activity commands)
- `acme-admin/src/lib/components/LogList.svelte` (new)

---

## Phase 4: Batch Operations ✓

Demonstrate bulk action patterns.

### Backend
- [x] Batch delete projects
  - [x] `POST /v1/admin/projects:batch-delete`
  - [x] Request: `{ ids: string[] }`
- [x] Batch update task status
  - [x] `POST /v1/admin/tasks:batch-update`
  - [x] Request: `{ ids: string[], status: string }`
- [x] Batch delete media
  - [x] `POST /v1/admin/media:batch-delete`
  - [x] Request: `{ ids: string[] }`

### Admin UI
- [x] Selection mode for list views
  - [x] Checkbox column
  - [x] Select all / deselect all
  - [x] Selection count indicator
- [x] Batch action toolbar
  - [x] Appears when items selected
  - [x] Delete, change status, etc.
- [x] Confirmation dialog for batch operations
  - [x] Show count of affected items
  - [x] Require explicit confirmation

**Files:**
- `acme-api/crates/api/src/routes/admin/projects.rs` (update)
- `acme-api/crates/api/src/routes/admin/tasks.rs` (update)
- `acme-api/crates/api/src/routes/admin/media.rs` (update)
- `acme-admin/src/lib/components/BatchActionBar.svelte` (new)
- `acme-admin/src/lib/components/SelectableList.svelte` (new)

---

## Phase 5: Background Jobs Example ✓

Demonstrate background job patterns beyond media.

### Email Notification Jobs
- [x] Welcome email on registration
  - [x] Job type: `email.welcome`
  - [x] Enqueue from registration endpoint
- [x] Task due date reminder
  - [x] Job type: `tasks.check_due_reminders` (scheduled job)
  - [x] Job type: `tasks.send_reminder` (individual reminder)
  - [x] Scheduled job to check upcoming due dates

### Cleanup Jobs
- [x] Expired session cleanup
  - [x] Job type: `auth.cleanup_sessions`
  - [x] Remove sessions older than X days
- [x] Orphan media cleanup
  - [x] Job type: `media.cleanup_orphans`
  - [x] Soft-delete media with zero usage after X days

### Admin Visibility
- [x] Jobs API endpoints
  - [x] `GET /v1/admin/jobs` - list jobs with filters
  - [x] `GET /v1/admin/jobs/stats` - job queue statistics
  - [x] `GET /v1/admin/jobs/:jobId` - job details
  - [x] `POST /v1/admin/jobs/:jobId/cancel` - cancel job
  - [x] `POST /v1/admin/jobs/:jobId/retry` - retry failed job
- [x] TypeScript client commands
- [x] Jobs dashboard page (frontend)
  - [x] Job queue statistics (pending, running, failed, succeeded)
  - [x] Recent job runs with status filtering
  - [x] Failed jobs with error details
  - [x] Manual retry and cancel buttons

**Files:**
- `acme-api/crates/jobs/src/lib.rs` (updated with new handlers)
- `acme-api/crates/api/src/routes/admin/jobs.rs` (new)
- `acme-client/src/commands/admin-commands.ts` (updated)
- `acme-admin/src/routes/(app)/system/jobs/+page.svelte` (pending)

---

## Phase 6: Media Library Enhancements ✓

Complete the media library implementation with missing features.

### Upload Flow Improvements ✓
- [x] Deduplication check before upload
  - [x] Compute sha256 hash client-side using `computeFileHash()` from Underlay
  - [x] Call `checkDuplicate()` before creating media record
  - [x] Show prompt if duplicate found: "Use existing" or "Upload as new"
  - [x] Skip upload if user chooses existing
- [x] Upload progress indicator
  - [x] Use XHR or fetch with progress events (via `uploadToBlob`)
  - [x] Show progress bar during upload
  - [x] Show finalisation state
- [x] Proper hash verification
  - [x] Pass computed sha256 to `finaliseUpload()` instead of empty string
  - [ ] Backend verifies hash matches uploaded file — deferred
- [x] Client-side file validation
  - [x] Validate file type against allowed list (images, PDFs only)
  - [x] Validate file size against limit (50MB)
  - [x] Show clear error messages
  - [x] Explicitly reject video uploads

### Server-side Validation
- [x] Magic byte detection
  - [x] Verify file content matches declared MIME type
  - [x] Reject files with mismatched extension/content
- [x] Video format rejection (done client-side)
  - [x] Explicitly reject video uploads with clear error
- [x] Size limit enforcement (server-side)
  - [x] Max file size 50MB (constant, can be made configurable)
  - [x] Early rejection in initiate-upload if declared size exceeds limit
  - [x] Post-upload verification in finalise-upload with cleanup on rejection
  - [x] Return clear error with file size details

### Rendition Generation
- [x] Thumbnail generation for images
  - [x] Generate on upload finalisation (via job queue)
  - [x] Store in media_rendition table
  - [x] Display in media list/detail views
- [ ] PDF first-page thumbnail (future)
  - [ ] Extract first page as image
  - [ ] Requires PDF rendering library

### UI Enhancements
- [x] Display thumbnails in media list grid
- [x] Show renditions in media detail view
- [x] Bulk upload support
  - [x] Multiple file selection
  - [x] Queue and progress for each file

**Files:**
- `acme-admin/src/routes/(app)/media/upload/+page.svelte` (update)
- `acme-admin/src/routes/(app)/media/+page.svelte` (update for thumbnails)
- `acme-api/crates/api/src/routes/admin/media.rs` (update for validation)
- `acme-api/crates/jobs/src/tasks/media.rs` (rendition generation)

---

## Phase 7: Testing Patterns ✓

Provide example tests for reference.

### Backend Tests
- [x] Unit test examples
  - [x] Domain logic tests (acme-domain: status parsing, roundtrip, serialization)
  - [x] Validation tests (API endpoint validation patterns in integration tests)
- [x] Integration test examples
  - [x] API endpoint tests with test database (api_tests.rs)
  - [x] Request/response testing patterns
- [x] Test utilities
  - [x] Test fixtures / factories (acme-test-utils crate)
  - [x] Database cleanup helpers (cleanup module)

### Frontend Tests
- [x] Component test examples
  - [x] Event handler testing patterns
  - [x] Data display patterns
- [x] Integration test examples
  - [x] Utility function tests
  - [x] Validation patterns

**Files:**
- `acme-api/crates/api/tests/api_tests.rs` (new)
- `acme-api/crates/test-utils/` (new crate)
- `acme-api/crates/domain/src/lib.rs` (unit tests added)
- `acme-admin/tests/` (new)

---

## Phase 8: Documentation & Developer Experience ✓

Make it easy for others to use as a reference.

### API Documentation
- [ ] OpenAPI/Swagger generation — deferred
  - [ ] Add utoipa annotations to routes
  - [ ] Swagger UI endpoint
- [x] Endpoint documentation comments (in architecture docs)

### Developer Setup
- [x] Docker Compose for local development
  - [x] PostgreSQL
  - [x] MinIO (S3-compatible) for blob storage
  - [x] MailHog for email testing
- [x] Setup script
  - [x] Database migration
  - [x] Seed data instructions
- [x] Environment templates
  - [x] `.env.example` for each project (api, admin, front)

### Reference Guides
- [x] `acme-docs/architecture/000-overview.md` - system overview
- [x] `acme-docs/architecture/001-authentication.md` - auth patterns
- [x] `acme-docs/architecture/002-media-library.md` - media patterns
- [x] `acme-docs/architecture/003-domain-patterns.md` - CRUD, soft-delete, ordering

**Files:**
- `docker-compose.yml` (new)
- `scripts/setup.sh` (new)
- `acme-admin/.env.example` (new)
- `acme-front/.env.example` (new)
- `acme-docs/architecture/*.md` (new)

---

## Phase 9: Advanced Features → Backlog

*Moved to backlog: See [backlog/advanced-features.md](./backlog/advanced-features.md)*

Lower priority items (OAuth/SSO, Real-time, Multi-tenancy, Advanced Media) have been deferred to allow focus on core patterns and infrastructure.

---

## Phase 10: Underlay Configuration Audit ✓

Review all Underlay Rust crates and expose configurable parameters through standardized config structs (like `MediaConfig` in `underlay-blob`).

### Goals
- Identify all hardcoded values that should be configurable
- Create config structs with sensible defaults
- Use builder pattern for easy customization
- Allow consuming apps to override only what they need

### Crates to Audit

#### underlay-blob ✓
- [x] `MediaConfig` - file size limits, thumbnail dimensions
- [ ] S3 config - timeouts, retry settings
- [ ] Local adapter - temp file handling

#### underlay-auth
- [ ] Session config
  - [ ] Session duration / TTL
  - [ ] Max concurrent sessions per user
  - [ ] Session token length
- [ ] JWT config
  - [ ] Access token expiry
  - [ ] Refresh token expiry
  - [ ] Token issuer/audience
- [ ] Rate limiting
  - [ ] Login attempt limits
  - [ ] Lockout duration
  - [ ] Failed attempt threshold

#### underlay-auth-email-totp ✓
- [x] TOTP config (already has `EmailTotpConfig` with builder pattern)
  - [x] Code validity period (`code_expiry_minutes`)
  - [x] Code length (`code_length`)
  - [x] Max verification attempts (`max_attempts`)
  - [x] Cooldown between sends (`max_codes_per_hour`)

#### underlay-auth-totp ✓
- [x] Authenticator config (now has builder pattern)
  - [x] TOTP period (`with_period_seconds`)
  - [x] TOTP digits (`with_digits`)
  - [x] Algorithm selection (`with_algorithm`)
  - [x] Issuer name for QR codes (`with_issuer`)
  - [x] Skew tolerance (`with_skew_steps`)

#### underlay-auth-webauthn ✓
- [x] WebAuthn config (already fully configurable via `WebAuthnConfig`)
  - [x] Relying party ID/name (`rp_id`, `rp_name`)
  - [x] Origin (`rp_origin`)

#### underlay-http ✓
- [x] Cookie config (now has full builder pattern)
  - [x] Domain, secure flag, max age
  - [x] SameSite policy (`with_same_site`)
  - [x] Cookie name prefixes (`with_cookie_prefix`)
  - [x] Refresh token path (`with_refresh_token_path`)
- [x] CORS config (now has full builder pattern)
  - [x] Allowed origins (`with_origins`)
  - [x] Allowed headers (`with_header`, `with_headers`)
  - [x] Max age (`with_max_age`)
  - [x] Credentials (`with_credentials`)
  - [x] Origin mirroring (`with_mirror_origin`)
- [ ] Request limits — deferred (handled at reverse proxy level)
  - [ ] Body size limits
  - [ ] Request timeout

#### underlay-jobs ✓
- [x] Job runner config (already configurable via `JobRunnerConfig`)
  - [x] Poll interval
  - [x] Batch size
- [x] Job config backoff (now has builder methods)
  - [x] Retry backoff settings (`with_backoff`, `with_exponential_backoff`, `with_fixed_backoff`)
  - [x] Default constants exported (`DEFAULT_BACKOFF_BASE_SECS`, `DEFAULT_BACKOFF_MAX_SECS`)
- [x] Scheduler config
  - [x] Tick interval (`SchedulerConfig` with `with_tick_interval_secs`)
  - [ ] Timezone handling — deferred (cron expressions handle this)
- [ ] History retention
  - [ ] Days to keep completed jobs
  - [ ] Days to keep failed jobs

#### underlay-email
- [ ] Email config — deferred (handled at app level via `EmailConfig`)
  - [ ] Default from address
  - [ ] Reply-to handling
  - [ ] Rate limiting per recipient
- [ ] Template config
  - [ ] Template directory
  - [ ] Default locale

#### underlay-db ✓
- [x] Pool config (now has builder methods)
  - [x] Min/max connections (`with_min_connections`, `with_max_connections`)
  - [x] Connection timeout (`with_acquire_timeout_secs`)
  - [x] Idle timeout (`with_idle_timeout_secs`)
  - [x] Default constants exported
- [x] Pagination config (already has constants)
  - [x] Default page size (`DEFAULT_PAGE_SIZE`)
  - [x] Max page size (`MAX_PAGE_SIZE`)

#### underlay-observability ✓
- [x] Tracing config (now has builder methods)
  - [x] Log level (`with_level`)
  - [x] Log format (`with_format`, `with_json`, `with_pretty`)
  - [ ] Sampling rate — deferred (requires OpenTelemetry integration)
  - [ ] Span attributes to include — deferred
- [x] Request ID config (already configurable)
  - [x] Header name (`RequestIdLayer::new(header_name)`)
  - [x] ID format (uses UUIDv7)

### Implementation Pattern

Each config struct should follow this pattern:

```rust
/// Configuration for [feature] with sensible defaults.
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    /// Description of field.
    /// Default: X
    pub field_name: Type,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            field_name: sensible_default,
        }
    }
}

impl FeatureConfig {
    pub fn new() -> Self { Self::default() }

    // Builder methods
    pub fn field_name(mut self, value: Type) -> Self {
        self.field_name = value;
        self
    }
}
```

### Deliverables
- [x] Audit each crate and document current hardcoded values
- [x] Create config structs for each crate
- [x] Update Acme to use the new configs (demonstrating usage)
- [x] Document config options in crate-level docs

**Files (Underlay):**
- `underlay-auth/src/config.rs` (new)
- `underlay-auth-*/src/config.rs` (new for each)
- `underlay-http/src/config.rs` (extend)
- `underlay-jobs/src/config.rs` (new)
- `underlay-email/src/config.rs` (new)
- `underlay-db/src/config.rs` (new)
- `underlay-observability/src/config.rs` (new)

---

## Success Criteria

The reference implementation is complete when:

- [x] New developers can understand Underlay patterns by reading the code
- [x] All common admin features are demonstrated (users, activity, batch ops)
- [x] Search and filtering patterns are clear
- [x] Media library is fully featured (deduplication, validation, thumbnails)
- [x] Testing patterns are provided
- [x] Setup is straightforward (single command)
- [x] Architecture is documented

## Priority Order

1. **Phase 1** - Admin dashboard & user management ✓
2. **Phase 2** - Search & filtering ✓
3. **Phase 3** - Activity logging ✓
4. **Phase 4** - Batch operations ✓
5. **Phase 5** - Background jobs ✓
6. **Phase 6** - Media library enhancements ✓
7. **Phase 7** - Testing patterns ✓
8. **Phase 8** - Documentation & developer experience ✓
9. **Phase 10** - Underlay configuration audit ✓
10. **Phase 9** - Advanced features → Backlog
