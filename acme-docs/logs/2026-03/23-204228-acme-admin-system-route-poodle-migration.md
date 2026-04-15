---
title: Acme admin system route Poodle migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, system]
---

## Summary

The system operations route cluster now uses direct Poodle primitives for refresh actions and filter controls while retaining Underlay for data-heavy operational surfaces such as tables, cards, dropdowns, detail displays, and workflow composition.

## What changed

- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/+page.svelte) now uses Poodle `Button`, `Field`, and `Select` for the refresh action and status filter.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/[id]/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/jobs/[id]/+page.svelte) now uses Poodle buttons for cancel, retry, and refresh actions.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/scheduled-tasks/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/scheduled-tasks/+page.svelte) now uses Poodle `Field` and `Select` for the enabled-status filter.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/errors/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/errors/+page.svelte) now uses Poodle `Button`, `Field`, and `Select` for refresh and status-code filtering.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/emails/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/system/emails/+page.svelte) now uses Poodle `Button`, `Field`, and `TextInput` for the captured-email GET filter form.

## Coexistence notes

- This batch deliberately left Underlay in place for `DataTable`, `DropdownMenu`, `Card`, `ListCard`, `ListGrid`, log/detail layouts, and the broader operational workflow surfaces.
- The only behavior change of note is explicit control ownership: jobs/errors filters are now driven by route state through Poodle selects, and the email filter form now uses Poodle uncontrolled inputs with URL-backed defaults.
- At this point the remaining foundational Underlay usage in `acme-admin` is no longer clustered by subsystem; it is mostly residual simple action controls in a few routes.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`

## Next Task

Take the next cleanup-sized batch in `acme-admin`: replace the last obvious Underlay primitive action surfaces in [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/+page.svelte), [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/[userId]/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/[userId]/+page.svelte), and [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/trash/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/media/trash/+page.svelte), while leaving retained Underlay shells, menus, dialogs, and detail surfaces intact.
