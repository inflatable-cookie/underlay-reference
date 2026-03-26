---
title: Acme admin Poodle Underlay coexistence proof handoff
status: completed
owner: Platform
updated: 2026-03-23
tags: [coordination, handoff]
---

## Objective

Implement the first mixed-surface proof in `acme-admin` so Poodle owns theme bootstrap while at least one retained Underlay surface and one direct Poodle replacement coexist on the same app.

## Scope

- Add the required Poodle dependencies or an equivalent canonical-package local alias bridge and root theme bootstrap to `acme-admin`.
- Use `/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte` as the theme ownership entrypoint.
- Use `/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte` as the first page-level replacement target.
- Keep at least one retained Underlay surface live under the new Poodle-owned theme selection.
- Do not widen this batch into a full admin migration, auth-flow rewrite, or broad component inventory work.

## Inputs

- [/Users/betterthanclay/Dev/projects/underlay/contracts/ui/poodle-underlay-coexistence-contract.json](/Users/betterthanclay/Dev/projects/underlay/contracts/ui/poodle-underlay-coexistence-contract.json)
- [/Users/betterthanclay/Dev/projects/underlay/contracts/ui/poodle-adoption-underlay-surface-groups.json](/Users/betterthanclay/Dev/projects/underlay/contracts/ui/poodle-adoption-underlay-surface-groups.json)
- [/Users/betterthanclay/Dev/projects/underlay/docs/roadmaps/g01/042-poodle-adoption-and-underlay-ui-contraction.md](/Users/betterthanclay/Dev/projects/underlay/docs/roadmaps/g01/042-poodle-adoption-and-underlay-ui-contraction.md)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/roadmaps/g01/006-poodle-underlay-coexistence-proof.md](/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/roadmaps/g01/006-poodle-underlay-coexistence-proof.md)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/package.json](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/package.json)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+layout.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+layout.svelte)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte)
- [/Users/betterthanclay/Dev/projects/poodle/docs/guides/svelte-developer-guide.md](/Users/betterthanclay/Dev/projects/poodle/docs/guides/svelte-developer-guide.md)

## Constraints

- Follow the repo instructions in `/Users/betterthanclay/Dev/projects/underlay-reference/AGENTS.md`.
- Keep edits aligned with the active Northstar vision, roadmap, and log flow.
- Do not widen scope beyond the listed tasks.
- Poodle owns canonical theme selection for this proof; do not introduce a second app-facing theme API.
- Prefer a page that already has simple Underlay usage and clear replacements over a richer but noisier admin surface.
- Treat retained Underlay surfaces as temporary migration companions, not as proof that the old primitive surface should remain canonical.

## Deliverables

- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/svelte.config.js](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/svelte.config.js)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/poodle/composites.ts](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/poodle/composites.ts)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/poodle/primitives.ts](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/poodle/primitives.ts)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/poodle/runtime.ts](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/poodle/runtime.ts)
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/logs/2026-03/23-155042-acme-admin-poodle-underlay-coexistence-proof.md](/Users/betterthanclay/Dev/projects/underlay-reference/acme-docs/logs/2026-03/23-155042-acme-admin-poodle-underlay-coexistence-proof.md)

## Acceptance Criteria

- `acme-admin` imports the Poodle token stylesheet and applies Poodle theme attributes from the app root.
- The proof keeps at least one retained Underlay surface active under the new Poodle-owned theme selection.
- `/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte` replaces at least one existing Underlay primitive or generic composite with direct Poodle usage.
- The proof log states exactly which retained Underlay surface remained and which direct Poodle surface replaced prior Underlay usage.
- Validation passes with `effigy validate` in `acme-admin`, plus the direct `acme-docs` link checks already established for this roadmap surface.

## Notes

- Current context: this batch advances Underlay roadmap `g01.042` and Acme reference roadmap `g01.006`.
- Decisions: the Underlay field review is complete, and the next real risk is mixed-surface execution rather than more component taxonomy analysis.
- Suggested proof target: keep `ErrorBoundary` in `/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+layout.svelte` as the retained Underlay surface, and replace the dashboard’s `StatCard` / `StatGrid` usage in `/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte` with direct Poodle `MetricTile` plus Poodle token ownership.
- Watch-outs: `/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte` still maps a large set of `--underlay-*` variables manually, and direct package consumption is currently blocked because the local Poodle package manifests still use `workspace:*` internals. The current `acme-docs` Effigy QA tasks also carry a pre-existing docs-policy/path configuration issue, so use the direct docs checks listed above for this batch instead of waiting on task repair.
- Outcome: the proof landed with a Poodle-owned theme shell at the root, retained `ErrorBoundary` under that shell, and direct Poodle `MetricTile`/`Pill` usage on the dashboard. App-local canonical-package aliases in `svelte.config.js` and `src/lib/poodle/*` were used instead of package installation to avoid baking a fake compatibility layer into Underlay.

## Completion Protocol

1. Update the proof log and tie it back to roadmap `g01.006`.
2. Record any temporary token aliasing or coexistence compromises explicitly.
3. If the proof changes the Underlay migration posture, update `/Users/betterthanclay/Dev/projects/underlay/contracts/ui/poodle-underlay-coexistence-contract.json`.
4. Leave one clear next task for the following thread.
