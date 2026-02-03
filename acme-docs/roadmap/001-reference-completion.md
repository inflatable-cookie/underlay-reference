# Reference Implementation Completion

Roadmap for completing the Underlay Reference Implementation (Acme) to serve as a comprehensive example for consuming applications.

## Current Status

The reference implementation has solid foundations:
- Full authentication system (passwords, 2FA, passkeys, sessions)
- Media library with versioned uploads and blob storage
- Task/Project domain with categories, labels, and comments
- Admin and public frontends with core CRUD patterns
- TypeScript client with typed commands

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
- `acme-admin/src/lib/components/ActivityFeed.svelte` (new)

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

## Phase 5: Background Jobs Example

Demonstrate background job patterns beyond media.

### Email Notification Jobs
- [ ] Welcome email on registration
  - [ ] Job type: `email.welcome`
  - [ ] Enqueue from registration endpoint
- [ ] Task due date reminder
  - [ ] Job type: `task.due_reminder`
  - [ ] Scheduled job to check upcoming due dates

### Cleanup Jobs
- [ ] Expired session cleanup
  - [ ] Job type: `auth.cleanup_sessions`
  - [ ] Remove sessions older than X days
- [ ] Orphan media cleanup
  - [ ] Job type: `media.cleanup_orphans`
  - [ ] Soft-delete media with zero usage after X days

### Admin Visibility
- [ ] Jobs dashboard page
  - [ ] Recent job runs
  - [ ] Failed jobs with error details
  - [ ] Manual retry button

**Files:**
- `acme-api/crates/jobs/src/tasks/email.rs` (new)
- `acme-api/crates/jobs/src/tasks/cleanup.rs` (new)
- `acme-admin/src/routes/(app)/system/jobs/+page.svelte` (new)

---

## Phase 6: Media Library Enhancements

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
- [ ] Magic byte detection
  - [ ] Verify file content matches declared MIME type
  - [ ] Reject files with mismatched extension/content
- [x] Video format rejection (done client-side)
  - [x] Explicitly reject video uploads with clear error
- [x] Size limit enforcement (server-side)
  - [x] Max file size 50MB (constant, can be made configurable)
  - [x] Early rejection in initiate-upload if declared size exceeds limit
  - [x] Post-upload verification in finalise-upload with cleanup on rejection
  - [x] Return clear error with file size details

### Rendition Generation
- [ ] Thumbnail generation for images
  - [ ] Generate on upload finalisation
  - [ ] Store in media_rendition table
  - [ ] Display in media list/detail views
- [ ] PDF first-page thumbnail (future)
  - [ ] Extract first page as image
  - [ ] Requires PDF rendering library

### UI Enhancements
- [ ] Display thumbnails in media list grid
- [ ] Show renditions in media detail view
- [ ] Bulk upload support
  - [ ] Multiple file selection
  - [ ] Queue and progress for each file

**Files:**
- `acme-admin/src/routes/(app)/media/upload/+page.svelte` (update)
- `acme-admin/src/routes/(app)/media/+page.svelte` (update for thumbnails)
- `acme-api/crates/api/src/routes/admin/media.rs` (update for validation)
- `acme-api/crates/jobs/src/tasks/media.rs` (rendition generation)

---

## Phase 7: Testing Patterns

Provide example tests for reference.

### Backend Tests
- [ ] Unit test examples
  - [ ] Domain logic tests
  - [ ] Validation tests
- [ ] Integration test examples
  - [ ] API endpoint tests with test database
  - [ ] Authentication flow tests
- [ ] Test utilities
  - [ ] Test fixtures / factories
  - [ ] Database cleanup helpers

### Frontend Tests
- [ ] Component test examples
  - [ ] Form component tests
  - [ ] List component tests
- [ ] Integration test examples
  - [ ] Page load tests
  - [ ] User flow tests (login, create project)

**Files:**
- `acme-api/crates/api/tests/` (new)
- `acme-admin/tests/` (examples)
- `acme-front/tests/` (examples)

---

## Phase 8: Documentation & Developer Experience

Make it easy for others to use as a reference.

### API Documentation
- [ ] OpenAPI/Swagger generation
  - [ ] Add utoipa annotations to routes
  - [ ] Swagger UI endpoint
- [ ] Endpoint documentation comments

### Developer Setup
- [ ] Docker Compose for local development
  - [ ] PostgreSQL
  - [ ] MinIO (S3-compatible) for blob storage
  - [ ] Optional: MailHog for email testing
- [ ] Setup script
  - [ ] Database migration
  - [ ] Seed data (admin user, sample projects)
- [ ] Environment templates
  - [ ] `.env.example` for each project

### Reference Guides
- [ ] `acme-docs/architecture/000-overview.md` - system overview
- [ ] `acme-docs/architecture/001-authentication.md` - auth patterns
- [ ] `acme-docs/architecture/002-media-library.md` - media patterns
- [ ] `acme-docs/architecture/003-domain-patterns.md` - CRUD, soft-delete, ordering

**Files:**
- `docker-compose.yml` (new)
- `scripts/setup.sh` (new)
- `acme-docs/architecture/*.md` (new)

---

## Phase 9: Advanced Features (Future)

Lower priority items for comprehensive coverage.

### OAuth/SSO
- [ ] Google OAuth provider
- [ ] GitHub OAuth provider
- [ ] OAuth callback handling
- [ ] Account linking

### Real-time Features
- [ ] WebSocket infrastructure
- [ ] Live task updates
- [ ] Presence indicators

### Multi-tenancy Example
- [ ] Organization/workspace model
- [ ] Tenant isolation patterns
- [ ] Permission inheritance

### Advanced Media (beyond Phase 6)
- [ ] Image cropping/editing tools
- [ ] Video upload support (if decided to allow)
- [ ] Drag-and-drop media reordering

---

## Success Criteria

The reference implementation is complete when:

- [ ] New developers can understand Underlay patterns by reading the code
- [ ] All common admin features are demonstrated (users, activity, batch ops)
- [ ] Search and filtering patterns are clear
- [ ] Media library is fully featured (deduplication, validation, thumbnails)
- [ ] Testing patterns are provided
- [ ] Setup is straightforward (single command)
- [ ] Architecture is documented

## Priority Order

1. **Phase 1** - Admin dashboard & user management (most requested feature)
2. **Phase 2** - Search & filtering (common pattern)
3. **Phase 6** - Media library enhancements (complete existing feature)
4. **Phase 3** - Activity logging (admin essential)
5. **Phase 8** - Documentation (helps others learn)
6. **Phase 4** - Batch operations (nice to have)
7. **Phase 5** - Background jobs (already partially done)
8. **Phase 7** - Testing (important but can be added incrementally)
9. **Phase 9** - Advanced features (future work)
