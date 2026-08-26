---
title: Acme admin account security Poodle migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, account, security]
---

## Summary

The remaining account security controls in `2fa` and `passkeys` now use direct Poodle primitives for foundational fields and actions, while the larger Underlay security workflows and shells remain unchanged.

## What changed

- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/2fa/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/2fa/+page.svelte) now uses Poodle `Button` and `FormActions` for enable/disable/setup actions instead of the Underlay equivalents.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/passkeys/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/passkeys/+page.svelte) now uses Poodle `Field`, `TextInput`, `Button`, and `FormActions` for rename, create-name, and add-passkey controls.
- The passkey naming dialog no longer depends on the old Underlay input ref API; it now focuses the Poodle input by DOM id after the retained Underlay `AlertDialog` opens.
- The rename and delete actions now use explicit Poodle button treatments, including smaller ghost actions for row-level controls and a danger tone for delete.

## Coexistence notes

- This batch stayed within the established coexistence boundary. Underlay still owns `AlertDialog`, `TotpInput`, `TimeAgo`, page loading/error surfaces, and the broader account-security workflow composition.
- The route behavior stayed stable: passkey naming remains optional during creation, rename still validates a non-empty name, and TOTP setup/disable flows still run through the same auth commands.
- The remaining foundational Underlay form surface is now concentrated more in project/media route-local forms than in the account area.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`
- `effigy check`
- `effigy validate`

## Next Task

Take the next task-form migration batch in `acme-admin`: replace the remaining Underlay `Field`, `TextInput`, `TextArea`, and `Select` usage in [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/new/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/new/+page.svelte) and [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/edit/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/projects/[projectId]/tasks/[taskId]/edit/+page.svelte), while keeping route-level orchestration and command wiring stable.
