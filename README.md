# Underlay Reference Implementation

A complete, working reference implementation for bootstrapping new Underlay-based projects. Copy these files and rename `acme` to your project name.

## Structure

```
underlay-reference/
├── apps/
│   ├── acme-api/      # Rust backend (API server + background jobs)
│   ├── acme-admin/    # SvelteKit admin frontend
│   └── acme-front/    # SvelteKit public frontend
├── packages/
│   ├── acme-client/   # TypeScript API client library
│   └── acme-ui/       # Shared UI components
├── docs/              # Documentation authority
├── config/            # Workspace-root config stack
├── package.json       # Root Bun workspace manifest
├── bun.lock           # One root lockfile
└── effigy.toml        # Root Effigy catalog
```

One Git repository owns the whole workspace. Runtime applications live under
`apps/*`, reusable internal libraries under `packages/*`, and docs authority is
root `docs/`. Names stay product-specific; the role map above is the contract.

The JavaScript workspace is declared once in the root `package.json`
(`apps/acme-admin`, `apps/acme-front`, `packages/acme-client`,
`packages/acme-ui`) with one root `bun.lock` and no child lockfiles. Internal
package edges use `workspace:*`. `apps/acme-api` is Rust-only and keeps its own
app-local Cargo workspace, so it is not a JavaScript workspace member.

Committed application dependencies resolve Underlay from the released Git tag
(`v0.9.4` at time of writing), not from sibling source paths.

## Documentation Authority

Reference-app planning and architecture live in `docs/`.

- Start with `docs/README.md`
- Use `docs/vision/001-acme-reference-implementation-vision.md` for the long-term role of the repo
- Use `docs/architecture/000-overview.md` for the package map and system layout
- Use `docs/architecture/product-guardrails.md` for the active retained-surface guardrails
- Use `docs/policy/001-working-rules.md` for the active strict execution rules
- Use `docs/processes/210-reference-implementation-notes.md` for implementation notes and validation commands

`AGENTS.md` files in this repository are intentionally kept lean and point back to that docs authority.

## Effigy-First Workspace Loop

Use Effigy as the default command surface from the workspace root:

```bash
effigy tasks
effigy workspace:js:prepare
effigy health
effigy validate
```

`workspace:js:prepare` is the one frozen root install
(`bun install --frozen-lockfile`) for the whole JavaScript workspace. Do not run
per-package installs.

First-time bring-up from another directory:

```bash
effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git
effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git --start
```

Common workspace commands:

```bash
effigy dev
effigy qa
effigy qa:docs
effigy qa:northstar
effigy db:reset
effigy db:migrate
```

`db:*` stays owned by `apps/acme-api/` and resolves through child-catalog routing from the workspace root. Root tasks should own cross-repo orchestration rather than duplicating uniquely owned child tasks.

## Config And Secrets Policy

- shared non-secret behavior belongs in the workspace-root config stack:
  `config/default.toml` plus `config/effigy.toml` (shared dev-stack overlay,
  loaded when `ENVIRONMENT=effigy`) plus optional `config/local.toml`
- **`config/local.toml` is for personal, machine-local overrides only** — it
  layers last, so anything duplicated from `effigy.toml` silently wins on your
  machine. After pulling the config convergence (2026-08), strip existing
  `local.toml` files back to personal tweaks; the shared dev-stack config now
  lives in the committed `config/effigy.toml`
- `apps/acme-admin/` and `apps/acme-front/` generate public runtime config from the root
  stack rather than reading `.env` files
- true secrets should move through Effigy-managed runtime injection or the local
  secrets vault, not committed or ad hoc `.env` files

Bootstrap notes:
- `effigy bootstrap ...` clones the reference workspace and runs `bootstrap:deps`
- setup starts the workspace container and installs dependencies inside it
- the Effigy bundle may mount sibling `../underlay` and `../poodle` for local
  framework development, cross-repo QA scripts, and docs — those mounts are not
  the committed application dependency source
- add `--start` when you want it to launch the root `dev` stack after setup

## Development Setup

The reference template consumes Underlay from the released Git repository:

```json
"@inflatable-cookie/underlay": "git+ssh://git@github.com/inflatable-cookie/underlay.git#v0.9.4"
```

```toml
underlay-core = { git = "ssh://git@github.com/inflatable-cookie/underlay.git", tag = "v0.9.4" }
```

Poodle core/Svelte packages resolve from the public npm registry at `0.2.2`.

For lockstep Underlay framework development inside this workspace, Effigy may
still mount a sibling `../underlay` checkout. Use `effigy deps link` when you
need to temporarily point Cargo or Bun back at that checkout; restore the tagged
dependencies before opening a consumer adoption PR.

## What's Included

### acme-api (Rust Backend)

Full-featured API server with:
- **Authentication**: JWT tokens, password auth, TOTP 2FA, passkeys, email verification
- **Session Management**: Token refresh, fingerprint validation, session listing/revocation
- **Database**: SQLx with migrations, connection pooling
- **Background Jobs**: Underlay jobs system integration
- **Email**: Template-based emails routed through SMTP and Mailpit in local dev
- **Media Library**: File uploads with versioning, deduplication, and blob storage
- **API Structure**: Health checks, auth routes, account management

Crate organization:
- `core` - Domain primitives, error types, UUID helpers
- `infra` - Configuration, logging, email setup
- `db` - Database pool, migrations, query functions
- `auth` - Authentication service, JWT handling, 2FA
- `domain` - Business logic (minimal example)
- `jobs` - Background job handlers
- `api` - HTTP handlers, routes, server setup

### acme-client (TypeScript)

API client library for frontend apps:
- Typed API commands (auth, account, health, media)
- Automatic token refresh with AuthManager
- HTTP client with request/response interceptors
- Cookie-based or in-memory token storage

### acme-admin (SvelteKit)

Admin dashboard frontend:
- Auth pages (login, forgot password)
- Account management (profile, password change, 2FA, passkeys)
- Protected route layout with auth guards
- Underlay UI Kit integration

### Admin Freshness + Conflict Contract

- Admin detail endpoints use `ETag` with `Cache-Control: private, no-cache, must-revalidate`.
- Admin detail GET requests support `If-None-Match` and can return `304`.
- Admin edit/update endpoints support `If-Match` optimistic concurrency and return `412` (`resource.precondition_failed`) on stale updates.
- Admin edit UIs reload latest server state on `412` and ask users to reapply edits.

### acme-front (SvelteKit)

Public-facing frontend:
- Landing page
- Auth integration ready
- SSR-compatible setup

## Quick Start (for Development)

Use the Effigy-owned path. The older raw Docker and localhost setup flow is not the supported bootstrap model for this workspace anymore.

### First clone from outside the repo

```bash
effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git
```

Add `--start` if you want the dev stack to launch after setup.

### If the repo is already cloned

```bash
effigy bootstrap:deps
effigy health
effigy validate
effigy dev
```

### Running the Application

```bash
# Full workspace: shell + admin + front + api + jobs + managed containers
effigy dev
```

### Development URLs

| Service | URL |
|---------|-----|
| Front | https://acme.test |
| Admin | https://admin.acme.test |
| API | https://api.acme.test |
| pgweb | https://pgweb.acme.test |
| Mailpit | https://mailpit.acme.test |
| MinIO Console | https://minio.acme.test |

Notes:
- `effigy dev` is the only supported workspace dev runner.
- `effigy dev` starts one canonical `workspace` container and runs shell, API, jobs, front, and admin inside it.
- The managed shell tab opens at the workspace root inside that running `workspace` container.
- The local shape is domain-first through the Effigy gateway: HTTPS for front/admin/API/pgweb/Mailpit/MinIO Console, plus `https://s3.acme.test` for browser-facing S3 uploads.
- Use the same `.test` aliases inside the workspace container too: `db.acme.test`, `smtp.acme.test`, `s3.acme.test`.
- Postgres and MinIO persist repo-local state under `.effigy/runtime/data/postgres` and `.effigy/runtime/data/minio`.
- Existing data in older Docker named volumes is not migrated automatically into those `.effigy/runtime/data/...` paths.

### Error Logging Smoke Test

These tasks are provided by the Effigy `underlay` bundle, so this repo
does not carry its own error-reporting script.

After `acme-api` is running, run:

```bash
effigy smoke:error-logging
```

This triggers a forced `ApiError` at `POST /v1/dev/error-smoke` against `https://api.acme.test` (debug builds only), then verifies the latest `platform.error_log` row includes:
- `error_code`
- `message`
- `context.handler_context`

To measure current `handler_context` null-rate (default last 24h):

```bash
effigy metrics:error-log
```

To run the full validation sequence (route checks + smoke + null-rate metrics):

```bash
effigy validate:error-reporting
```

## Bootstrapping a New Project

To create a new project from this reference:

### 1. Copy to Your Project

```bash
mkdir my-project && cd my-project

mkdir -p apps packages

cp -r /path/to/underlay-reference/apps/acme-api ./apps/api
cp -r /path/to/underlay-reference/apps/acme-admin ./apps/admin
cp -r /path/to/underlay-reference/apps/acme-front ./apps/front
cp -r /path/to/underlay-reference/packages/acme-client ./packages/api-client
cp -r /path/to/underlay-reference/packages/acme-ui ./packages/ui
cp /path/to/underlay-reference/package.json ./package.json
```

Keep the `apps/*` and `packages/*` split. Do not flatten packages back to the
repository root.

### 2. Rename Everything

Replace `acme` with your project name throughout:

| Pattern | Replace With |
|---------|--------------|
| `acme-api`, `acme-client`, etc. | `myapp-api`, `myapp-client`, etc. |
| `acme_*` (package/crate names) | `myapp_*` |
| `acme_access_token` | `myapp_access_token` |
| `acme_refresh_token` | `myapp_refresh_token` |
| `AcmeLocalAuthService` | `MyAppLocalAuthService` |
| `configureAcmeClient` | `configureMyAppClient` |
| `ACME_*` env vars | `MYAPP_*` |

### 3. Update Workspace Wiring

Keep the workspace-level bootstrap assumptions aligned when you rename the reference:

- root `package.json` `workspaces` paths and `packageManager` pin
- root `effigy.toml` `catalog.alias`
- root `effigy.toml` `[bundle.dirs]` physical package paths
- root `effigy.toml` `[containers.stack]` `profile`, `project_name`, and `dns.domain`
- root `effigy.toml` `ready_message`
- child `effigy.toml` aliases where package names change
- regenerate the single root `bun.lock`; do not add child lockfiles

When you need a different Underlay release, update the tag in every web manifest
and `apps/acme-api/Cargo.toml` workspace dependency, then regenerate the Bun and
Cargo locks narrowly.

## Environment Variables

`acme-api` uses layered config precedence (repo-root `config/` stack):

1. `config/default.toml` (committed, all environments)
2. `config/<environment>.toml` (env-named overlay — `config/effigy.toml` for
   the shared dev stack)
3. `config/local.toml` (personal overrides, gitignored)
4. allowlisted environment variables (secrets and runtime wiring)

Use TOML for app behavior defaults, and env vars for secrets/runtime wiring
and per-environment overrides. `.env` files are not part of the runtime
contract.

For the current `underlay-reference` local shape:

- keep app-owned runtime wiring in `apps/acme-api/.env`
- keep typed app behavior overrides in `apps/acme-api/config/local.toml`
- keep `apps/acme-api/effigy.toml` as plain task orchestration (`cargo run ...`), not an env dump

### Required for API

```bash
DATABASE_URL=postgres://postgres:postgres@db.acme.test:5432/acme
AUTH_JWT_PRIVATE_KEY=...  # Generated by generate-jwt-env
AUTH_JWT_PUBLIC_KEY=...   # Generated by generate-jwt-env
```

### Optional

```bash
ENVIRONMENT=local|dev|staging|prod
HOST=0.0.0.0
PORT=41001
PUBLIC_HOST=api.acme.test
RUST_LOG=debug
CORS_ORIGINS=https://acme.test,https://admin.acme.test
COOKIE_DOMAIN=.acme.test
COOKIE_SECURE=true
EMAIL_ADAPTER=noop|smtp|ses
SMTP_HOST=smtp.acme.test
SMTP_PORT=1025
SMTP_TLS=none
BLOB_ADAPTER=s3
BLOB_S3_BUCKET=acme-media
BLOB_S3_ENDPOINT_URL=http://s3.acme.test:9000
BLOB_S3_PUBLIC_URL_BASE=https://s3.acme.test/acme-media
BLOB_S3_PRESIGN_URL_BASE=https://s3.acme.test
```

## Adding Your Domain

1. **Define entities** in `apps/api/crates/domain/src/`
2. **Add database tables** in `apps/api/migrations/`
3. **Create query functions** in `apps/api/crates/db/src/`
4. **Add API routes** in `apps/api/crates/api/src/routes/`
5. **Add client commands** in `packages/api-client/src/commands/`
6. **Build UI** in `apps/admin/` and `apps/front/`

## Documentation

See the Underlay docs for detailed patterns:
- LLM Bootstrap Guide - step-by-step bootstrap rules
- Rust Backend Guide - API patterns
- TypeScript Client Guide - client patterns
- Admin Guide - admin frontend patterns
- Frontend Guide - public frontend patterns
