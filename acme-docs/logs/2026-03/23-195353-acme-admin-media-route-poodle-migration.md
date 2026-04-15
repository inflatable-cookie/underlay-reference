---
title: Acme admin media route Poodle migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, media]
---

## Summary

The media route family now uses direct Poodle primitives for its remaining foundational fields, selects, search, and simple action controls while keeping the media workflows and higher-order Underlay route patterns unchanged.

## What changed

- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/+page.svelte) now uses Poodle `Field`, `SearchField`, `Select`, and `Button` for list filtering and top-level toolbar actions.
- The media list route now owns its title-filter debounce explicitly in route logic rather than relying on old Underlay input behavior.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/upload/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/upload/+page.svelte) now uses Poodle buttons throughout the replace/upload queue surface, including ghost-button replacements for the old text-button actions.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/[mediaId]/+page.svelte) now uses Poodle `Button`, `Field`, `TextInput`, `Select`, and `FormActions` inside the retained Underlay media detail and edit-dialog shells.

## Coexistence notes

- This batch stayed within the same architectural line as the earlier form migrations: Poodle owns primitive controls and simple actions, while Underlay still owns `FilterBar`, `FileUpload`, `ProgressBar`, `FormDialog`, detail shells, and route-level workflow composition.
- The media upload pipeline, duplicate handling, and media detail edit concurrency path stayed unchanged.
- The next concentrated Underlay foundational surface is now the project/category list and filter layer rather than the media routes.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`
- `effigy check`
- `effigy validate`

## Next Task

Take the next project/category listing batch in `acme-admin`: replace the remaining Underlay `Field`, `TextInput`, `Select`, and adjacent simple toolbar button usage in [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/+page.svelte) and [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/categories/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/categories/+page.svelte), while keeping selection and reorder orchestration stable.
