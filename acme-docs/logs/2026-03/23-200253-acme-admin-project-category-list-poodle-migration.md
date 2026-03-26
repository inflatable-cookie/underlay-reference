---
title: Acme admin project and category list Poodle migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, projects, categories]
---

## Summary

The project and category listing routes now use direct Poodle primitives for their remaining foundational toolbar and filter controls while leaving the higher-order Underlay list, selection, and reorder patterns intact.

## What changed

- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/+page.svelte) now uses Poodle `Button`, `Field`, `SearchField`, and `Select` for the top-level toolbar and filter row instead of the old Underlay button/input/select surfaces.
- The projects list route now owns two pieces of behavior explicitly in route logic: the debounced name filter and the category filter option loading for `listCategoriesForSuggestions`, which are now visible route concerns rather than hidden inside an Underlay async select.
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/categories/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/categories/+page.svelte) now uses the same Poodle toolbar and filter primitives for reorder/add actions plus name/status filtering.

## Coexistence notes

- This batch stays on the same architectural line as the previous media and form migrations: Poodle owns primitive actions and simple filters, while Underlay still owns `FilterBar`, `PageHeader`, `ReorderableList`, `OrderBy`, `ListGrid`, `ListCard`, batch selection, and reorder conflict handling.
- No list orchestration moved layers. Project selection mode, batch delete, category reorder, and project reorder flows all remain on the existing Underlay route patterns.
- The one async filter concern in this batch, project category options, was moved into explicit route-local state rather than preserved through another Underlay compatibility dependency.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`

## Next Task

Take the next project-area batch in `acme-admin`: replace the remaining Underlay `Button`, `Field`, and `Select` surfaces around task filtering and simple actions in [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/+page.svelte) and [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/+page.svelte), while keeping detail shells, confirm actions, progress, and route-level orchestration stable.
