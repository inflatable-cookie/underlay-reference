---
title: Acme admin project detail Poodle migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, projects, tasks]
---

## Summary

The project detail and task detail routes now use direct Poodle primitives for their remaining simple action buttons and task-list filter controls while leaving Underlay in place for the higher-order detail, confirmation, and orchestration surfaces.

## What changed

- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/+page.svelte) now uses Poodle `Button`, `Field`, and `Select` for the edit/add/select/reorder task actions plus the task status and priority filter controls.
- The project detail route still uses Underlay `ConfirmAction`, `FilterBar`, `OrderBy`, `Badge`, `ListCard`, `ProgressBar`, `DetailsCard`, and `BatchActionBar`; this batch only moved the foundational control layer.
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/+page.svelte) now uses a Poodle `Button` for the edit action while retaining Underlay `ConfirmAction`, detail sections, badges, pills, and metadata display.

## Coexistence notes

- This batch stayed disciplined about the migration boundary: Poodle owns simple actions and selects, Underlay still owns the route shells, detail composition, and operational behavior.
- No task selection, reorder, delete, or navigation orchestration changed layers. The same route-local handlers and Underlay workflow components remain in place.
- The remaining concentrated foundational Underlay surface is now in the `system` route cluster rather than the project area.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`

## Next Task

Take the next system-operations batch in `acme-admin`: replace the remaining Underlay `Button`, `Field`, `Select`, and `TextInput` surfaces in [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/+page.svelte), [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/[id]/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/[id]/+page.svelte), [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/scheduled-tasks/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/scheduled-tasks/+page.svelte), [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/errors/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/errors/+page.svelte), and [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/emails/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/emails/+page.svelte), while keeping tables, cards, logs, and operational workflow shells stable.
