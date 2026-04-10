# Underlay Reference Implementation

A complete, working reference implementation for bootstrapping new Underlay-based projects. Copy these files and rename `acme` to your project name.

## Structure

```
underlay-reference/
├── acme-api/          # Rust backend (API server + background jobs)
├── acme-client/       # TypeScript API client library
├── acme-admin/        # SvelteKit admin frontend
├── acme-front/        # SvelteKit public frontend
├── acme-ui/           # Shared UI components
└── underlay -> ...    # Symlink to underlay library
```

## Documentation Authority

Reference-app planning and architecture live in `acme-docs/`.

- Start with `acme-docs/README.md`
- Use `acme-docs/vision/001-acme-reference-implementation-vision.md` for the long-term role of the repo
- Use `acme-docs/architecture/000-overview.md` for the package map and system layout
- Use `acme-docs/architecture/product-guardrails.md` for the active retained-surface guardrails
- Use `acme-docs/policy/001-working-rules.md` for the active strict execution rules
- Use `acme-docs/processes/210-reference-implementation-notes.md` for implementation notes and validation commands

`AGENTS.md` files in this repository are intentionally kept lean and point back to that docs authority.

## Effigy-First Workspace Loop

Use Effigy as the default command surface from the workspace root:

```bash
effigy tasks
effigy health
effigy validate
```

First-time bring-up from another directory:

```bash
effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git
effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git --start
```

Common workspace commands:

```bash
effigy dev
effigy dev front
effigy dev admin
effigy qa
effigy qa:docs
effigy qa:northstar
effigy db:reset
effigy db:migrate
```

`db:*` stays owned by `acme-api/` and resolves through child-catalog routing from the workspace root. Root tasks should own cross-repo orchestration rather than duplicating uniquely owned child tasks.

Bootstrap notes:
- `effigy bootstrap ...` clones the reference workspace and runs `bootstrap:deps`
- setup fetches/install dependencies for `underlay`, `acme-api`, `acme-client`, `acme-ui`, `acme-front`, and `acme-admin`
- add `--start` when you want it to launch the root `dev` stack after setup

## Development Setup

This repository uses a **symlink** to reference the Underlay library for development:

```bash
# Create symlink to your underlay clone
ln -s /path/to/underlay ./underlay
```

All `package.json` files reference `@decodelabs/underlay` via `file:../underlay`.
The `Cargo.toml` uses path dependencies like `../underlay/rust/crates/...`.

## What's Included

### acme-api (Rust Backend)

Full-featured API server with:
- **Authentication**: JWT tokens, password auth, TOTP 2FA, passkeys, email verification
- **Session Management**: Token refresh, fingerprint validation, session listing/revocation
- **Database**: SQLx with migrations, connection pooling
- **Background Jobs**: Underlay jobs system integration
- **Email**: Template-based emails with dev capture mode
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

### Option A: Effigy Bootstrap (Recommended)

```bash
effigy bootstrap git@github.com:inflatable-cookie/underlay-reference.git
```

Add `--start` if you want the dev stack to launch after setup.

### Option B: Automated Setup

```bash
git clone <this-repo> underlay-reference
cd underlay-reference

# Link to your underlay library
ln -s /path/to/underlay ./underlay

# Run automated setup (requires Docker)
./scripts/setup.sh
```

The setup script will:
- Start PostgreSQL, MinIO, and MailHog via Docker
- Create environment files from templates
- Run database migrations
- Generate JWT keys
- Install frontend dependencies

### Option C: Manual Setup

#### 1. Clone and Set Up

```bash
git clone <this-repo> underlay-reference
cd underlay-reference

# Link to your underlay library
ln -s /path/to/underlay ./underlay
```

#### 2. Start Services (Docker)

```bash
# Start all development services
docker compose up -d

# Or start individually
docker compose up -d postgres    # PostgreSQL only
docker compose up -d minio       # MinIO (S3) only
docker compose up -d mailhog     # MailHog (email) only
```

#### 3. Set Up Database

```bash
# Configure connection
cp acme-api/.env.example acme-api/.env
# Edit acme-api/.env with your DATABASE_URL

# Optional: copy local config overrides (otherwise defaults come from config/default.toml)
cp acme-api/config/local.toml.example acme-api/config/local.toml

# Run migrations
cd acme-api
cargo run -p acme-db --bin migrate_dev_db
```

#### 4. Generate Auth Keys

```bash
cd acme-api
cargo run -p acme-auth --bin generate-jwt-env >> .env
```

#### 5. Install Dependencies

```bash
# Use bun for all TypeScript projects
cd acme-client && bun install
cd acme-admin && bun install
cd acme-front && bun install
```

### Running the Application

```bash
# Full workspace
effigy dev

# Focused profiles
effigy dev admin
effigy dev front
```

### Development URLs

| Service | URL |
|---------|-----|
| API | http://localhost:40011 |
| Admin | http://localhost:40012 |
| Front | http://localhost:40013 |
| MailHog | http://localhost:8025 |
| MinIO Console | http://localhost:9001 |

### Error Logging Smoke Test

After `acme-api` is running, run:

```bash
./scripts/smoke-error-logging.sh
```

This triggers a forced `ApiError` at `POST /v1/dev/error-smoke` (debug builds only), then verifies the latest `platform.error_log` row includes:
- `error_code`
- `message`
- `context.handler_context`

To measure current `handler_context` null-rate (default last 24h):

```bash
./scripts/error-log-metrics.sh
```

To run the full validation sequence (route checks + smoke + null-rate metrics):

```bash
./scripts/validate-error-reporting.sh
```

## Bootstrapping a New Project

To create a new project from this reference:

### 1. Copy to Your Project

```bash
mkdir my-project && cd my-project

cp -r /path/to/underlay-reference/acme-api ./api
cp -r /path/to/underlay-reference/acme-client ./api-client
cp -r /path/to/underlay-reference/acme-admin ./admin
cp -r /path/to/underlay-reference/acme-front ./front

# Link underlay
ln -s /path/to/underlay ./underlay
```

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

### 3. Update Package Paths

If your underlay is at a different relative path, update `package.json` files:

```json
"@decodelabs/underlay": "file:../underlay"
```

And `Cargo.toml` workspace dependencies:

```toml
underlay-core = { path = "../underlay/rust/crates/underlay-core" }
```

## Environment Variables

`acme-api` uses layered config precedence:

1. `acme-api/config/default.toml`
2. `acme-api/config/local.toml` (optional, gitignored)
3. `.env` / environment variables (override layer)

Use TOML for app behavior defaults, and env vars for secrets/runtime wiring and per-environment overrides.

### Required for API

```bash
DATABASE_URL=postgres://user@localhost:5432/acme
AUTH_JWT_PRIVATE_KEY=...  # Generated by generate-jwt-env
AUTH_JWT_PUBLIC_KEY=...   # Generated by generate-jwt-env
```

### Optional

```bash
ENVIRONMENT=local|dev|staging|prod
HOST=127.0.0.1
PORT=3000
RUST_LOG=debug
CORS_ORIGINS=http://localhost:4173,http://localhost:4174
COOKIE_DOMAIN=.acme.com
COOKIE_SECURE=true
EMAIL_ADAPTER=noop|dev_capture|smtp|ses
BLOB_ADAPTER=local
BLOB_LOCAL_PATH=./storage
```

## Adding Your Domain

1. **Define entities** in `api/crates/domain/src/`
2. **Add database tables** in `api/migrations/`
3. **Create query functions** in `api/crates/db/src/`
4. **Add API routes** in `api/crates/api/src/routes/`
5. **Add client commands** in `api-client/src/commands/`
6. **Build UI** in `admin/` and `front/`

## Documentation

See the Underlay docs for detailed patterns:
- LLM Bootstrap Guide - Step-by-step instructions
- Rust Backend Guide - API patterns
- TypeScript Client Guide - Client patterns
- Admin Guide - Admin frontend patterns
- Frontend Guide - Public frontend patterns
