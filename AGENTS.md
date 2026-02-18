# Agents Guide: Underlay Reference Implementation

## Purpose

This repository is a **reference template** for bootstrapping Underlay-based apps. Prefer canonical, reusable patterns over one-off customization.

## Keep AGENTS Lean

`AGENTS.md` files should contain only:

1. Scope and intent
2. Hard operational rules
3. Minimal validation commands
4. Links to detailed docs

Detailed implementation notes are documented in:
- `/Users/betterthanclay/Dev/apps/underlay-reference/docs/reference-implementation-notes.md`
- `/Users/betterthanclay/Dev/apps/underlay-reference/README.md`

## Hard Rules

- For bootstrap work, copy and rename from the reference packages; avoid inventing alternate structure without a clear reason.
- Use `bun` for TypeScript/Svelte tasks.
- Keep wire JSON naming and API conventions aligned with Underlay guides.
- Keep changes scoped; avoid unrelated refactors.

## Validation

Run only what matches touched areas:

```bash
cd acme-api && cargo build
cd acme-client && bun check
cd acme-admin && bun check
cd acme-front && bun check
```

## Source of Truth

For architecture and conventions, prefer Underlay docs in `/Users/betterthanclay/Dev/libraries/underlay/docs/guides/`.
