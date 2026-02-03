# Acme Docs

Documentation, architecture, and planning for the Underlay Reference Implementation (Acme).

This repository serves as both:
- **Reference documentation** for developers learning Underlay patterns
- **Living spec** for the reference implementation itself

## Structure

- `architecture/` – system overview and design decisions
- `roadmap/` – feature roadmaps and implementation plans
- `reports/` – session summaries and progress reports

## Sub-projects

The reference implementation consists of:

- **acme-api** – Rust backend (Axum + SQLx) demonstrating Underlay patterns
- **acme-client** – TypeScript API client library
- **acme-admin** – SvelteKit admin dashboard
- **acme-front** – SvelteKit public-facing frontend
- **acme-ui** – Shared UI components (optional, for app-specific components)

## Purpose

This reference implementation demonstrates how to build applications using Underlay infrastructure:

- Authentication (passwords, 2FA, passkeys, sessions)
- Media library (versioned uploads, blob storage)
- Domain patterns (CRUD, soft-delete, ordering)
- Frontend patterns (authenticated data fetching, forms, lists)
- TypeScript client generation patterns

See `roadmap/001-reference-completion.md` for the current implementation status and roadmap.
