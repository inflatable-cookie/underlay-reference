# g01.007 Retained Underlay Surface Contract

Date: 2026-09-03
Status: dispatched — PR pending orchestrator review

Card 001 executed as one docs-only worker against live `acme-admin` source
(Underlay v0.9.7, Poodle 0.2.2). No production code changed.

## What changed

- Frozen the retained-surface contract at
  [docs/architecture/004-retained-underlay-surface-contract.md](../../architecture/004-retained-underlay-surface-contract.md).
  It classifies every meaningful surviving Underlay surface group in
  `acme-admin`:
  - Group 1 structural shell: server CSP/error mapping in `hooks.server.ts`,
    auth runtime (`configureAuth`, `useAuthenticatedData`,
    `createAuthCookieHelpers`, webauthn utils), the shared navigation
    contract (`gotoWithContext`, `NavigationContext`), the feedback runtime
    seam (Underlay toast store rendered by Poodle `ToastHost`), route
    protection, and the Underlay stylesheets under the Poodle theme shell.
  - Group 2 workflow-heavy and data-heavy stacks: entity
    list/detail/form templates with `patterns` and `client/query`, the
    versioned media workflow stack with `runtime/media/*`, the system
    operations stack, auth flow patterns (`LoginPage`, `ForgotPasswordFlow`),
    relations/selection runtime, and the Nightfire rich-text stack including
    the `acme-ui` note registrations consumed via `@acme/ui/*`.
  - Group 3 reference-app/domain-specific surface: the Poodle-first
    `system/poodle-gap-review` page and the acme composition layer under
    `lib/cards|lists|forms|menus|ui|components|stores|utils`.
  - Group 4 future Poodle review notes — explicitly not a second
    classification — inside already-classified surfaces: the group 1
    feedback-runtime toast store, slug/html/browser utilities,
    `createClientPagination`, `PasswordRequirements`.
- Recorded the downstream rule: Poodle directly for primitives and simple
  composites; Underlay only where groups 1-2 name structural, workflow-heavy,
  or data-heavy ownership; group 3 is app-local; group 4 notes wait for a
  roadmap decision.
- Closed out roadmap `g01.007`, both roadmap indexes, spec 001, Card 001,
  product guardrails, and the docs/specs/logs front doors.

## Evidence notes

- The audit found no residual Underlay primitive or simple-composite usage:
  the coexistence-proof example of a retained Underlay `ErrorBoundary` in
  `(app)/+layout.svelte` has since moved to Poodle `ErrorBoundary`.
- The classification is a contract freeze, not a migration wave; no
  migrated primitive was reopened and no route was converted.
- Review-note repair (post first PR review): the roadmap validation
  criterion "absolute path references" was rewritten to "repo-root path
  references" in the first batch; the contract uses repo-root-relative
  paths, matching the durable process-notes convention.
- Review-note repair: Card 001 `Owner` moved from "repo maintainers" to the
  executing worker, matching the archived batch-card convention
  (`specs/archive/batch-cards/003` uses `Owner: media worker`).

## Validation

- `effigy acme-docs/qa:docs`
- `effigy acme-docs/qa:northstar`
- `effigy docs check links README.md vision/README.md roadmaps/README.md logs/README.md` (from `docs/`)
- `git diff --check` and a docs-only range diff against the pushed-main base

## Next Task

Review the Card 001 PR at its exact head; downstream rollout planning
consumes the contract only after that review.
