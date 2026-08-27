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
(`v0.9.5` at time of writing), not from sibling source paths.

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

### Tests

`effigy test --plan` prints the resolved plan before anything runs — the
targets, the suite chosen per target, and the evidence Effigy used to choose
it. Read it first when test shape matters:

```bash
effigy test --plan
effigy test
effigy test acme-api
```

The workspace resolves to four targets. `acme-api` runs the configured `rust`
suite (`cargo test --workspace`); `acme-admin`, `acme-front`, and `acme-client`
run `vitest`. Sibling `underlay` and `poodle` are excluded from the root plan.
Database-backed Rust tests skip themselves unless `DATABASE_URL` or
`TEST_DATABASE_URL` is set, so a plain `effigy test` stays useful without a
running stack.

### Conformance

```bash
effigy qa:conformance
```

Runs the two released Underlay checkers — workspace shape and env/secret
authority — from the installed `@inflatable-cookie/underlay` package. They read
this repository only; no sibling Underlay checkout is required.

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
effigy state plan
effigy state apply local --yes
effigy acme-api/migration:reset
```

Root `effigy state plan` / `effigy state apply local --yes` orchestrate the local schema and dev-overlay stack. Package-owned `migration:*` tasks stay in `apps/acme-api/` and resolve through child-catalog routing from the workspace root.

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
- `effigy bootstrap ...` clones the reference workspace and applies the
  repo-owned `[bootstrap]` contract
- setup starts the workspace container and runs one frozen root workspace
  install; there is no per-package install step
- the Effigy bundle may mount sibling `../underlay` and `../poodle` for local
  framework development, cross-repo QA scripts, and docs — those mounts are not
  the committed application dependency source
- add `--start` when you want it to launch the root `dev` stack after setup

## Development Setup

The reference template consumes Underlay from the released Git repository:

```json
"@inflatable-cookie/underlay": "git+ssh://git@github.com/inflatable-cookie/underlay.git#v0.9.5"
```

```toml
underlay-core = { git = "ssh://git@github.com/inflatable-cookie/underlay.git", tag = "v0.9.5" }
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
- **API Structure**: Explicit runtime, shared, front, and admin route families

Route families and their access posture:

| Family | Source | Paths | Posture |
|--------|--------|-------|---------|
| runtime | `routes/runtime.rs` | `/v1/health`, `/favicon.ico`, `/api/openapi.json`, `/api/docs` | unauthenticated, no CSRF, never requires `X-Api-Version` |
| shared | `routes/shared/router.rs` | `/v1/auth/*`, `/v1/account/*` | mixed bootstrap and authenticated; CSRF on cookie-backed mutations |
| front | `routes/front/router.rs` | `/v1/projects/*` | authenticated product-user routes |
| admin | `routes/admin/router.rs` | `/v1/admin/*` | `AdminUser` gate |

#### OpenAPI exposure

The OpenAPI document is served at `/api/openapi.json` with Swagger UI at
`/api/docs`. Both belong to the runtime family and are **exposed only in
development environments** — `main.rs` passes
`app_config.env.is_development()` into the router builder, so `staging`,
`production`, and any unrecognised environment name serve neither. Changing
that is a deployment policy decision, not a route change.

Business endpoints are path-versioned under `/v1/*`. This app has declared the
optional `X-Api-Version` header: the TypeScript client sends it on every
request and the server validates it across all three business families. Runtime
endpoints are exempt by contract.

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
effigy workspace:js:prepare
effigy health
effigy validate
effigy dev
```

`workspace:js:prepare` is the single frozen root workspace install. There is no
per-package install step.

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
- Postgres persists in the repo-scoped named volume `underlay-reference-dev-postgres-data`; MinIO uses `underlay-reference-dev-minio-data`.
- Older host bind-mount paths under `.effigy/runtime/data/` are not migrated automatically into those named volumes.

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

Use TOML for app behavior defaults, and env vars for secrets and runtime
wiring. `.env` files are not part of the runtime contract: there is no
`.env`, `.env.local`, or `.env.example` in the target posture, and nothing in
this workspace reads one.

Two tracked files are the env authority:

- `config/env-manifest.txt` — the complete environment surface any runtime
  process may read, with each key's condition recorded inline
- `config/required-secrets.txt` — the startup-critical subset

Both are static key inventories. They never carry values; secret presence stays
an operator and runtime concern. `effigy qa:conformance` proves they exist,
parse, and agree.

For the current `underlay-reference` local shape:

- keep non-secret dev values in the committed root `config/` stack
- keep personal, machine-local overrides in `config/local.toml`
- keep local secrets in the Effigy vault, injected at task/container runtime
- keep `apps/acme-api/effigy.toml` as plain task orchestration (`cargo run ...`), not an env dump

### Environment classes

`ENVIRONMENT` selects both the behavior class and the config overlay. An unset
or unrecognised name fails closed to deployed production behavior.

| Class | Names | Posture |
|-------|-------|---------|
| Non-deployed | `local`, `effigy`, `test` | dev seeds, CORS origin mirroring, and bounded startup warnings are allowed |
| Deployed | `dev`, `staging`, `production` | fail closed: malformed config, `COOKIE_SECURE=false`, and CSRF disablement are startup errors |

### Startup-critical

```bash
DATABASE_URL=postgres://postgres:postgres@db.acme.test:5432/acme
AUTH_JWT_PRIVATE_KEY=...  # Generated by generate-jwt-env
AUTH_JWT_PUBLIC_KEY=...   # Generated by generate-jwt-env
ENCRYPTION_KEY=...        # Required in deployed environments; warns in local/effigy/test
```

### Commonly set

```bash
ENVIRONMENT=local|effigy|test|dev|staging|production
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
ACME_S3_BUCKET=acme-media
ACME_S3_ENDPOINT=http://s3.acme.test:9000
ACME_S3_PUBLIC_URL_BASE=https://s3.acme.test/acme-media
```

Everything else, including the conditional Redis, SES/AWS, OAuth, and
trusted-proxy keys, is listed with its condition in
`config/env-manifest.txt`. Do not add a runtime env read without adding it
there.

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
