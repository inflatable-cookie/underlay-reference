# Retired Roadmap Backlog: Advanced Features (Deferred Candidates)

Status: triage (non-authoritative; not scheduled, approved, or runnable)
Captured: 2026-09-09
Source: the retired backlog advanced-features note (Git history); originally
Phase 9 of the reference-completion milestone, moved aside so core patterns
and infrastructure stayed in focus

This is the single triage record for every deferred item the retired backlog
held. None of this work was approved, scheduled, or started; promotion of any
candidate happens through planning, never by executing this note.

## Candidates

### 1. OAuth/SSO

Social login and single-sign-on integration: Google OAuth provider
(credentials, authorization flow, token exchange), GitHub OAuth provider
(same pattern, different scopes), OAuth callback handling (state validation,
error handling, redirect flows), and account linking (link/unlink, email
conflicts).

- Constraints: useful but not essential for demonstrating Underlay patterns;
  the auth crate already carries the primitives (`underlay-auth-oauth`;
  Acme placeholder code in `acme-auth/src/local.rs`).
- Open questions: which provider goes first; how account-linking conflicts
  resolve against existing users.
- Promotion condition: planning scopes a reference-app auth milestone;
  suggested start is Google, then GitHub.

### 2. Real-time Features

WebSocket-based live updates: connection management, token authentication,
heartbeat/ping-pong, broadcast task changes to connected clients, optimistic
UI updates, presence indicators.

- Constraints: needs additional infrastructure (WebSocket server, possibly
  Redis pub/sub at scale); orthogonal to the core CRUD patterns the
  reference app demonstrates.
- Open questions: transport choice (`axum` WebSocket support vs
  `tokio-tungstenite`); broadcast topology at scale.
- Promotion condition: planning accepts the infrastructure cost and scopes
  a real-time milestone.

### 3. Multi-tenancy Example

Organization/workspace isolation patterns: organization model with member
roles, tenant isolation (row-level security vs schema-per-tenant vs shared
schema), permission inheritance with resource-specific overrides.

- Constraints: significant schema changes required; likely complex enough
  to warrant its own reference implementation rather than being bolted
  onto Acme.
- Open questions: whether this belongs in this repo at all; research
  PostgreSQL row-level security first.
- Promotion condition: planning rules it in scope for this repo (or charters
  the separate implementation).

### 4. Advanced Media

Extended media library: client-side cropping before upload plus
server-side processing, video upload support, drag-and-drop media
reordering with manual gallery ordering.

- Constraints: current media library is already comprehensive; video upload
  is explicitly blocked (needs a transcoding pipeline; storage cost
  considerations).
- Open questions: server-side library choice (`image-rs` was suggested);
  gallery ordering semantics.
- Promotion condition: planning scopes a media milestone. Video upload
  additionally needs the transcode-pipeline and storage-cost decisions
  reversed first.

## Owner

No owner assigned. These candidates stay deferred until planning promotes
one into `docs/plan.md`.

## Disposition

All four candidates above stay deferred in triage. None were approved or
started. No durable rules or accepted design were found (the source's
implementation notes are candidate guidance, preserved above as
constraints, not promoted to any contract).
