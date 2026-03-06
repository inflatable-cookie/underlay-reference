# Acme Docs

Northstar-aligned documentation authority for the Underlay reference implementation.

Use this repo section for four things:
- `vision/` for the long-term purpose of the reference app
- `architecture/` for structural and technical decisions
- `roadmaps/` for executable work queues
- `logs/` for timestamped execution history tied to roadmap work

## Core structure

- `vision/`
- `architecture/`
- `processes/`
- `roadmaps/`
- `logs/`

## Reference app scope

The reference implementation covers:
- `acme-api` for the Rust backend and jobs runtime
- `acme-client` for the typed TypeScript API boundary
- `acme-admin` for the admin frontend
- `acme-front` for the public frontend
- `acme-ui` for app-local shared UI pieces

## How to use this docs set

- Start with [vision/001-acme-reference-implementation-vision.md](/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/vision/001-acme-reference-implementation-vision.md).
- Use [architecture/000-overview.md](/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/architecture/000-overview.md) for the package map and system layout.
- Read [processes/210-reference-implementation-notes.md](/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/processes/210-reference-implementation-notes.md) for implementation notes and validation commands.
- Track active execution in [roadmaps/README.md](/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/roadmaps/README.md).
- Record meaningful work batches in [logs/README.md](/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/logs/README.md).
