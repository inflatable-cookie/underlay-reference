# Architecture Overview

The Acme reference implementation demonstrates a complete Underlay-based application with a Rust backend, TypeScript API client, and SvelteKit frontends.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              Frontends                                   │
│  ┌─────────────────────┐              ┌─────────────────────┐           │
│  │    acme-admin       │              │    acme-front       │           │
│  │    (SvelteKit)      │              │    (SvelteKit)      │           │
│  │    Port: 40012      │              │    Port: 40013      │           │
│  └──────────┬──────────┘              └──────────┬──────────┘           │
│             │                                    │                       │
│             └────────────────┬───────────────────┘                       │
│                              │                                           │
│                    ┌─────────┴─────────┐                                │
│                    │   acme-client     │                                │
│                    │   (TypeScript)    │                                │
│                    └─────────┬─────────┘                                │
└──────────────────────────────┼──────────────────────────────────────────┘
                               │ HTTP/JSON
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           acme-api (Rust)                                │
│                            Port: 40011                                   │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        API Layer                                 │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │    │
│  │  │  Auth   │ │ Account │ │  Admin  │ │  Media  │ │  Tasks  │   │    │
│  │  │ Routes  │ │ Routes  │ │ Routes  │ │ Routes  │ │ Routes  │   │    │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘   │    │
│  │       └───────────┴───────────┴───────────┴───────────┘        │    │
│  └───────────────────────────────┬─────────────────────────────────┘    │
│                                  │                                       │
│  ┌───────────────────────────────┴─────────────────────────────────┐    │
│  │                     Service Layer                                │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                │    │
│  │  │   Auth      │ │   Media     │ │  Activity   │                │    │
│  │  │  Service    │ │  Service    │ │   Logger    │                │    │
│  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘                │    │
│  │         └───────────────┴───────────────┘                        │    │
│  └─────────────────────────┬────────────────────────────────────────┘    │
│                            │                                             │
│  ┌─────────────────────────┴────────────────────────────────────────┐    │
│  │                    Database Layer (acme-db)                       │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │    │
│  │  │  Auth   │ │  Tasks  │ │  Media  │ │Activity │ │  Stats  │    │    │
│  │  │Queries  │ │Queries  │ │Queries  │ │Queries  │ │Queries  │    │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘    │    │
│  └──────────────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────────────┘
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   PostgreSQL    │     │   Blob Storage  │     │   Job Queue     │
│   (Database)    │     │  (MinIO/S3)     │     │  (PostgreSQL)   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Project Structure

```
underlay-reference/
├── apps/
│   ├── acme-api/          # Rust backend (app-local Cargo workspace)
│   │   ├── crates/
│   │   │   ├── api/       # HTTP handlers, route families, server
│   │   │   ├── auth/      # Authentication service
│   │   │   ├── core/      # Domain primitives, errors
│   │   │   ├── db/        # Database queries, migrations
│   │   │   ├── domain/    # Business logic entities
│   │   │   ├── infra/     # Configuration, logging
│   │   │   ├── jobs/      # Background job handlers
│   │   │   └── test-utils/# Test fixtures, helpers
│   │   └── migrations/    # SQL migrations
│   │
│   ├── acme-admin/        # Admin SvelteKit app
│   │   ├── src/
│   │   │   ├── lib/       # Components, stores, utils
│   │   │   └── routes/    # SvelteKit routes
│   │   └── tests/         # Vitest tests
│   │
│   └── acme-front/        # Public SvelteKit app
│       ├── src/
│       │   ├── lib/       # Components, stores, utils
│       │   └── routes/    # SvelteKit routes
│       └── tests/         # Vitest tests
│
├── packages/
│   ├── acme-client/       # TypeScript API client
│   │   ├── src/
│   │   │   ├── commands/  # API command functions
│   │   │   ├── types/     # TypeScript interfaces
│   │   │   └── utils/     # Client utilities
│   │   └── package.json
│   │
│   └── acme-ui/           # Shared UI package
│
├── docs/                  # Documentation authority
│   ├── vision/            # Long-term reference-app role
│   ├── architecture/      # Architecture docs
│   ├── processes/         # Implementation notes and runbooks
│   ├── policy/            # Execution and authority rules
│   ├── specs/             # Active strict-lane wrappers
│   ├── roadmaps/          # Segmented roadmap generations
│   │   ├── g01/           # Active roadmap generation
│   │   └── backlog/       # Unscheduled candidate milestones
│   ├── scripts/           # Effigy Rhai rollout-check helpers
│   ├── handoffs/          # Cross-thread worker handoffs
│   └── logs/              # Month-sharded execution history
│
├── config/                # Workspace-root config stack
├── package.json           # Root Bun workspace manifest
├── bun.lock               # One root lockfile
└── effigy.toml            # Bundle-backed local dev entrypoint
```

The JavaScript workspace members are `apps/acme-admin`, `apps/acme-front`,
`packages/acme-client`, and `packages/acme-ui`. `apps/acme-api` is Rust-only and
keeps its Cargo workspace app-local. Underlay and Poodle arrive as released
dependencies; sibling checkouts are QA/tooling mounts only.

## Technology Stack

### Backend (acme-api)

| Technology | Purpose |
|------------|---------|
| Rust | Systems programming language |
| Axum | Web framework |
| SQLx | Database toolkit (compile-time checked queries) |
| PostgreSQL | Primary database |
| Underlay crates | Auth, jobs, blob storage, etc. |

### Frontend (acme-admin, acme-front)

| Technology | Purpose |
|------------|---------|
| SvelteKit | Full-stack framework |
| Svelte 5 | UI framework with runes |
| TypeScript | Type-safe JavaScript |
| Vite | Build tool |
| Underlay components | Shared UI components |

### API Client (acme-client)

| Technology | Purpose |
|------------|---------|
| TypeScript | Type-safe API calls |
| Fetch API | HTTP requests |

## Key Patterns

### Authentication Flow

See [001-authentication.md](./001-authentication.md) for detailed auth patterns.

- JWT access tokens (short-lived, 15 minutes)
- Refresh tokens (longer-lived, 30 days)
- Session management with fingerprint validation
- Optional 2FA (TOTP, passkeys)

### Media Library

See [002-media-library.md](./002-media-library.md) for media patterns.

- Versioned uploads with blob storage
- Client-side deduplication via SHA-256
- Automatic thumbnail generation
- Soft delete with retention

### Domain Patterns

See [003-domain-patterns.md](./003-domain-patterns.md) for CRUD patterns.

- Soft delete with `deleted_at` timestamps
- Manual ordering with `weight` columns
- Batch operations for bulk actions
- Activity logging for audit trails

## Development Workflow

### Starting Development

```bash
effigy workspace:js:prepare
effigy health
effigy validate
effigy dev
```

`workspace:js:prepare` runs one frozen root workspace install
(`bun install --frozen-lockfile`). There is no per-package install step.

### Running Tests

```bash
# Inspect the resolved plan first
effigy test --plan

# Whole workspace
effigy test

# One target
effigy acme-api/test
effigy acme-admin/test
effigy acme-front/test
effigy acme-client/test
```

### Database Migrations

```bash
effigy state plan
effigy state apply local --yes
effigy acme-api/migration:apply
effigy acme-api/migration:reset
```

Root state plan/apply owns the local schema plus dev-overlay order. The API package owns the concrete `migration:*` apply and reset/replay tasks.

## Configuration

### Layered Configuration

The API reads configuration with the following precedence:

1. `config/default.toml` (workspace root, committed)
2. `config/<environment>.toml` (env-named overlay, e.g. `config/effigy.toml`)
3. `config/local.toml` (optional personal overrides, gitignored)
4. environment variables from runtime injection (override layer)

`config/env-manifest.txt` is the complete env surface, with each key's
condition recorded inline. `config/required-secrets.txt` is the
startup-critical subset. There is no `.env` or `.env.example`: `.env` files are
not part of the runtime contract, and nothing in this workspace reads one.

Startup-critical:
- `DATABASE_URL` - PostgreSQL connection string
- `AUTH_JWT_PRIVATE_KEY` - JWT signing key
- `AUTH_JWT_PUBLIC_KEY` - JWT verification key
- `ENCRYPTION_KEY` - required in deployed environments; may be absent with an
  explicit warning in `local`, `effigy`, and `test`

### Environment Classes

`ENVIRONMENT` names both the behavior class and the config overlay. `local`,
`effigy`, and `test` are the bounded non-deployed set. `dev`, `staging`, and
`production` are deployed. An unset or unrecognised name fails closed to
production behavior.

Outside the non-deployed set the API fails to start on malformed layered
config, on `COOKIE_SECURE=false`, and on an attempt to disable CSRF.

### Route Families And Access Model

`crates/api/src/routes/` exposes four explicit family builders merged by one
root builder. Family ownership is visible in the source layout; public paths
are unchanged by it.

| Family | Builder | Auth | CSRF | `X-Api-Version` |
|--------|---------|------|------|-----------------|
| runtime | `routes/runtime.rs` | none | none | exempt |
| shared | `routes/shared/router.rs` | bootstrap or `AuthenticatedUser` | on cookie mutations | required |
| front | `routes/front/router.rs` | `AuthenticatedUser` | on cookie mutations | required |
| admin | `routes/admin/router.rs` | `AdminUser` | on cookie mutations | required |

Cross-cutting policy lives in `routes/middleware.rs` and is layered above the
merged router, so it applies uniformly rather than per family.

Path versioning under `/v1/*` is the baseline. This app has additionally
declared the optional `X-Api-Version` header — the TypeScript client sends it
on every request — so the server applies it consistently across all three
business families. Runtime endpoints are exempt by contract even when they sit
under `/v1`.

### Client IP And Proxy Trust

Client IP used as policy input — auth fingerprints, lockout, rate limiting,
audit — comes from Underlay's peer-aware `RequestContext`, resolved once per
request. Handlers never parse forwarding headers.

`TRUSTED_PROXY` declares the deployment topology. Unset or unrecognised trusts
no forwarding header and uses the socket peer, so a typo cannot widen the trust
boundary. A header is honoured only because the declared topology says a
trusted proxy sets it, never because the header is present.

### Adapter Selection

| Variable | Description |
|----------|-------------|
| `ACME_S3_BUCKET` | selects the real S3 blob adapter outside local/effigy; `ACME_ALLOW_NOOP_BLOB=1` is the explicit storage-less opt-in |
| `EMAIL_ADAPTER` | `noop`, `smtp`, or `ses` |
| `RATE_LIMIT_BACKEND` | `in_memory` or `redis` |
| `ENVIRONMENT` | `local`, `effigy`, `test`, `dev`, `staging`, `production` |

## Deployment

For production deployment:

1. Build the API: `cargo build --release`
2. Build frontends: `bun run build` in each frontend
3. Configure environment variables for production
4. Set up PostgreSQL and blob storage
5. Run migrations before starting the API
6. Deploy behind a reverse proxy (nginx, Caddy)

See each project's README for specific deployment instructions.
