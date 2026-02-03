# Agents Guide: Acme Reference Implementation

This directory contains a complete reference implementation for bootstrapping new Underlay-based projects. When working with this code, follow these guidelines.

## Purpose

The `acme-*` projects are **templates for copying**, not running applications. They demonstrate the canonical structure and patterns for Underlay projects.

## When Helping Users Bootstrap a New Project

1. **Copy, don't modify** - Copy files from `reference/` to the user's project, then rename
2. **Rename systematically** - Use the substitution table in README.md
3. **Verify each step** - Run `cargo build` and `bun check` after copying

## Project Structure

```
reference/
├── acme-api/          # Rust backend
│   ├── crates/        # Workspace crates
│   │   ├── core/      # Primitives (re-exports underlay-core)
│   │   ├── infra/     # Config, email, logging
│   │   ├── db/        # Database layer
│   │   ├── auth/      # Authentication service
│   │   ├── domain/    # Business logic (add your entities here)
│   │   ├── jobs/      # Background jobs
│   │   └── api/       # HTTP handlers
│   └── migrations/    # SQL migrations
├── acme-client/       # TypeScript API client
├── acme-admin/        # SvelteKit admin
└── acme-front/        # SvelteKit public site
```

## Key Patterns

### Authentication Flow
- Password + optional 2FA (TOTP or email code)
- JWT access/refresh tokens
- Session fingerprinting for security
- Passkey support

### API Structure
- Routes organized by domain in `crates/api/src/routes/`
- Shared routes (health, auth, account) in `routes/shared/`
- DTOs in `crates/api/src/dto/`
- State in `crates/api/src/state.rs`

### Database Access
- SQLx with compile-time query checking
- Migrations in `migrations/` directory
- Query functions in `crates/db/src/`

### Frontend Auth
- Token refresh handled by `AuthManager`
- Cookies set by backend, read by `auth-tokens.ts`
- Auth stores in `lib/stores/auth.ts`

## Common Tasks

### Adding a New API Endpoint

1. Add types to `crates/api/src/dto/your_domain.rs`
2. Add handler to `crates/api/src/routes/your_domain.rs`
3. Register route in `crates/api/src/routes/mod.rs`
4. Add client command in `api-client/src/commands/`
5. Export from `api-client/src/index.ts`

### Adding a Database Table

1. Create migration in `migrations/YYYYMMDDHHMI__description.sql`
2. Add query functions in `crates/db/src/your_domain.rs`
3. Export from `crates/db/src/lib.rs`

### Adding an Admin Page

1. Create route in `admin/src/routes/(app)/your-page/+page.svelte`
2. Add navigation in `admin/src/lib/ui/AdminNavList.svelte`

## Package Manager

**Always use `bun`** for all TypeScript/JavaScript operations (not npm or pnpm).

```bash
bun install    # Install dependencies
bun run build  # Build
bun check      # Type check
bun dev        # Dev server
```

## Dependencies

These projects depend on:
- Underlay Rust crates (linked via workspace)
- Underlay TypeScript packages (via npm/bun)
- Underlay UI Kit for Svelte components

## Testing the Reference

```bash
# Verify Rust builds
cd acme-api && cargo build

# Verify TypeScript builds
cd acme-client && bun install && bun run build
cd acme-admin && bun install && bun check
cd acme-front && bun install && bun check
```
