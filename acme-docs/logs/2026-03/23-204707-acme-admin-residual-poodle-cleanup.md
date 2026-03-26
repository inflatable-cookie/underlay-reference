---
title: Acme admin residual Poodle cleanup
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, cleanup]
---

## Summary

The final obvious Underlay primitive action-button residue in `acme-admin` has been removed, closing the main coexistence proof execution for `g01.006`.

## What changed

- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/+page.svelte) now uses a Poodle button for the add-user action.
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/trash/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/trash/+page.svelte) now uses a Poodle button for the restore action.
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/[userId]/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/[userId]/+page.svelte) had its leftover Underlay `Button` import removed because the route was already functionally on Poodle for the remaining primitive action surface.

## Completion note

- A final scan of `acme-admin` route-level foundational controls shows no remaining meaningful cluster of Underlay primitive/button/input/select residue to chase. The remaining Underlay usage in the reference admin is structural, data-heavy, or intentionally retained domain/UI surface.
- This means the coexistence proof is no longer an active cleanup stream; it has reached its useful completion point.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`

## Next Task

`g01.006` is complete. Open `g01.007` as the next roadmap and move into the next milestone, such as formalizing the retained Underlay surface after migration, packaging the migration guidance for downstream apps, or selecting the first non-reference app rollout.
