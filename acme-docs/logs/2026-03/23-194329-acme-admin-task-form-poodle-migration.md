---
title: Acme admin task form Poodle migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, tasks]
---

## Summary

The task create/edit routes now use direct Poodle primitives for foundational form controls, keeping the task workflow and command orchestration intact while removing the remaining Underlay field/input/select layer from those routes.

## What changed

- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/new/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/new/+page.svelte) now uses Poodle `Field`, `TextInput`, `TextArea`, `Select`, `Button`, and `FormActions` for task creation.
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/edit/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/edit/+page.svelte) now uses the same Poodle surfaces for task editing.
- Title validation is now expressed directly in the route forms through Poodle field error state instead of relying on the old Underlay field layer.
- The label-chip chooser and all task command logic remain local to the routes, so this batch changes the foundational UI surface without widening into workflow or API behavior changes.

## Coexistence notes

- This batch stayed within the current boundary: Poodle owns primitive and simple composite form controls, while Underlay still owns `PageHeader`, `PageLoading`, `FormError`, and the broader route composition around the task screens.
- The edit route’s optimistic concurrency handling and ETag refresh path remain untouched.
- The next concentrated foundational Underlay surface is now in the media route family rather than in the task routes.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`
- `effigy check`
- `effigy validate`

## Next Task

Take the next media migration batch in `acme-admin`: replace the remaining Underlay `Field`, `TextInput`, `Select`, `TextButton`, and adjacent simple button usage in [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/+page.svelte), [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/upload/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/upload/+page.svelte), and [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte), while keeping media upload/detail workflow orchestration stable.
