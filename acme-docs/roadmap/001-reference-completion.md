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

## Phase 1: Admin Dashboard & User Management

Flesh out the admin experience with essential features.

### Admin Dashboard
- [ ] Dashboard overview page
  - [ ] User count statistic
  - [ ] Media count/storage used
  - [ ] Recent activity feed (last 10 changes)
  - [ ] System health indicators

### User Management (Admin)
- [ ] Backend: User listing endpoint
  - [ ] `GET /v1/admin/users` with pagination
  - [ ] Filters: role, status, search by email
- [ ] Backend: User detail endpoint
  - [ ] `GET /v1/admin/users/{userId}`
  - [ ] Include sessions, 2FA status, created date
- [ ] Backend: User role management
  - [ ] `PUT /v1/admin/users/{userId}/role`
  - [ ] Update user's role (user → admin, etc.)
- [ ] Backend: User suspension
  - [ ] `POST /v1/admin/users/{userId}/suspend`
  - [ ] `POST /v1/admin/users/{userId}/unsuspend`
- [ ] Client: Add user admin commands
  - [ ] `listUsersAdmin()`, `getUser()`, `updateUserRole()`, `suspendUser()`
- [ ] Admin UI: Users list page
  - [ ] Table with email, role, status, created date
  - [ ] Pagination
  - [ ] Role/status filters
- [ ] Admin UI: User detail page
  - [ ] Profile info, role badge
  - [ ] Session list with revoke action
  - [ ] Suspend/unsuspend action

**Files:**
- `acme-api/crates/api/src/routes/admin/users.rs` (new)
- `acme-api/crates/db/src/users.rs` (new or extend auth)
- `acme-client/src/commands/user-commands.ts` (new)
- `acme-admin/src/routes/(app)/users/+page.svelte` (new)
- `acme-admin/src/routes/(app)/users/[userId]/+page.svelte` (new)

---

## Phase 2: Search & Filtering

Demonstrate search patterns across the application.

### API Search Patterns
- [ ] Projects search
  - [ ] `GET /v1/admin/projects?q=search&status=active&sortBy=title`
  - [ ] Full-text search on title/description
- [ ] Tasks search
  - [ ] `GET /v1/admin/projects/{projectId}/tasks?q=search&status=pending&priority=high`
  - [ ] Filter by status, priority, due date range
- [ ] Media search
  - [ ] `GET /v1/admin/media?q=search&kind=image&unusedOnly=true`
  - [ ] Already partially implemented, wire to UI

### Admin UI Search
- [ ] Projects list: Add search input and filters
- [ ] Tasks list: Add search input and status/priority filters
- [ ] Media list: Add search input and kind filter
- [ ] Demonstrate debounced search pattern
- [ ] Demonstrate URL-based filter state (shareable filter URLs)

**Files:**
- `acme-api/crates/api/src/routes/admin/projects.rs` (update)
- `acme-api/crates/api/src/routes/admin/tasks.rs` (update)
- `acme-admin/src/routes/(app)/projects/+page.svelte` (update)
- `acme-admin/src/routes/(app)/media/+page.svelte` (update)

---

## Phase 3: Activity & Audit Logging

Track and display changes for admin visibility.

### Activity Log Infrastructure
- [ ] Database: Create `platform.activity_log` table
  - [ ] Fields: id, actor_id, action, entity_type, entity_id, metadata, created_at
  - [ ] Index on entity, actor, created_at
- [ ] Backend: Activity logging helper
  - [ ] `log_activity(actor, action, entity_type, entity_id, metadata)`
  - [ ] Call from relevant endpoints (create, update, delete)
- [ ] Backend: Activity endpoints
  - [ ] `GET /v1/admin/activity` - global activity feed
  - [ ] `GET /v1/admin/activity?entityType=project&entityId=xxx` - entity-specific
  - [ ] `GET /v1/admin/users/{userId}/activity` - user's actions

### Admin UI
- [ ] Activity feed component (reusable)
  - [ ] Action icon, actor, description, timestamp
  - [ ] Relative time display
- [ ] Dashboard: Recent activity section
- [ ] Entity detail pages: Activity tab/section
- [ ] User detail: User's activity history

**Files:**
- `acme-api/migrations/YYYYMMDD__activity_log.sql` (new)
- `acme-api/crates/db/src/activity.rs` (new)
- `acme-api/crates/api/src/routes/admin/activity.rs` (new)
- `acme-client/src/commands/activity-commands.ts` (new)
- `acme-admin/src/lib/components/ActivityFeed.svelte` (new)

---

## Phase 4: Batch Operations

Demonstrate bulk action patterns.

### Backend
- [ ] Batch delete projects
  - [ ] `POST /v1/admin/projects:batch-delete`
  - [ ] Request: `{ ids: string[] }`
- [ ] Batch update task status
  - [ ] `POST /v1/admin/tasks:batch-update`
  - [ ] Request: `{ ids: string[], status: string }`
- [ ] Batch delete media
  - [ ] `POST /v1/admin/media:batch-delete`
  - [ ] Request: `{ ids: string[] }`

### Admin UI
- [ ] Selection mode for list views
  - [ ] Checkbox column
  - [ ] Select all / deselect all
  - [ ] Selection count indicator
- [ ] Batch action toolbar
  - [ ] Appears when items selected
  - [ ] Delete, change status, etc.
- [ ] Confirmation dialog for batch operations
  - [ ] Show count of affected items
  - [ ] Require explicit confirmation

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

## Phase 6: Testing Patterns

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

## Phase 7: Documentation & Developer Experience

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

## Phase 8: Advanced Features (Future)

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

### Advanced Media
- [ ] Bulk upload UI
- [ ] Image cropping/editing
- [ ] Video upload support
- [ ] Rendition preview in UI

---

## Success Criteria

The reference implementation is complete when:

- [ ] New developers can understand Underlay patterns by reading the code
- [ ] All common admin features are demonstrated (users, activity, batch ops)
- [ ] Search and filtering patterns are clear
- [ ] Testing patterns are provided
- [ ] Setup is straightforward (single command)
- [ ] Architecture is documented

## Priority Order

1. **Phase 1** - Admin dashboard & user management (most requested feature)
2. **Phase 2** - Search & filtering (common pattern)
3. **Phase 3** - Activity logging (admin essential)
4. **Phase 7** - Documentation (helps others learn)
5. **Phase 4** - Batch operations (nice to have)
6. **Phase 5** - Background jobs (already partially done)
7. **Phase 6** - Testing (important but can be added incrementally)
8. **Phase 8** - Advanced features (future work)
