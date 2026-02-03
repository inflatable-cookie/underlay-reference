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
├── acme-api/              # Rust backend
│   ├── crates/
│   │   ├── api/           # HTTP handlers, routes, server
│   │   ├── auth/          # Authentication service
│   │   ├── core/          # Domain primitives, errors
│   │   ├── db/            # Database queries, migrations
│   │   ├── domain/        # Business logic entities
│   │   ├── infra/         # Configuration, logging
│   │   ├── jobs/          # Background job handlers
│   │   └── test-utils/    # Test fixtures, helpers
│   └── migrations/        # SQL migrations
│
├── acme-client/           # TypeScript API client
│   ├── src/
│   │   ├── commands/      # API command functions
│   │   ├── types/         # TypeScript interfaces
│   │   └── utils/         # Client utilities
│   └── package.json
│
├── acme-admin/            # Admin SvelteKit app
│   ├── src/
│   │   ├── lib/           # Components, stores, utils
│   │   └── routes/        # SvelteKit routes
│   └── tests/             # Vitest tests
│
├── acme-front/            # Public SvelteKit app
│   ├── src/
│   │   ├── lib/           # Components, stores, utils
│   │   └── routes/        # SvelteKit routes
│   └── tests/             # Vitest tests
│
├── acme-docs/             # Documentation
│   ├── architecture/      # Architecture docs
│   └── roadmap/           # Project roadmap
│
├── docker-compose.yml     # Development services
├── scripts/               # Setup scripts
└── underlay -> ...        # Symlink to Underlay library
```

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
# Start services
docker compose up -d

# Run API
cd acme-api && cargo run

# Run admin (separate terminal)
cd acme-admin && bun dev

# Run front (separate terminal)
cd acme-front && bun dev
```

### Running Tests

```bash
# Backend tests
cd acme-api && cargo test

# Frontend tests
cd acme-admin && bun test
cd acme-front && bun test
```

### Database Migrations

```bash
cd acme-api
cargo run -p acme-db --bin migrate_dev_db
```

## Configuration

### Environment Variables

The API reads configuration from environment variables. See `acme-api/.env.example` for all options.

Required:
- `DATABASE_URL` - PostgreSQL connection string
- `AUTH_JWT_PRIVATE_KEY` - JWT signing key
- `AUTH_JWT_PUBLIC_KEY` - JWT verification key

### Feature Flags

Features can be enabled/disabled via environment:

| Variable | Description |
|----------|-------------|
| `BLOB_ADAPTER` | `local` or `s3` for blob storage |
| `EMAIL_ADAPTER` | `noop`, `dev_capture`, or `smtp` |
| `ENVIRONMENT` | `local`, `dev`, `staging`, `prod` |

## Deployment

For production deployment:

1. Build the API: `cargo build --release`
2. Build frontends: `bun run build` in each frontend
3. Configure environment variables for production
4. Set up PostgreSQL and blob storage
5. Run migrations before starting the API
6. Deploy behind a reverse proxy (nginx, Caddy)

See each project's README for specific deployment instructions.
