---
title: Acme admin account route Poodle form migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, account]
---

## Summary

The `account` route family now uses direct Poodle form primitives for its editable profile and password surfaces while keeping the broader Underlay route, dialog, card, and workflow shells intact.

## What changed

- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/+page.svelte) now uses Poodle `Field`, `FieldSet`, `TextInput`, `Select`, `Switch`, `Button`, and `FormActions` inside the profile-edit dialog instead of the equivalent Underlay form primitives.
- The profile dialog now owns its grid layout and switch-label composition locally rather than depending on Underlay `FieldSetGrid` and Underlay switch label affordances.
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/password/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/password/+page.svelte) now uses Poodle `Field`, `TextInput`, `Button`, and `FormActions` for the password reset and verification actions.
- Underlay still owns the surrounding route-shell responsibilities for this family, including `Card`, `FormDialog`, `PageLoading`, `FormError`, `PasswordRequirements`, and `TotpInput`.

## Coexistence notes

- This batch stayed at the foundational UI boundary. It did not widen into auth-flow rewrites, account shell redesign, or replacement of Underlay route patterns.
- The account settings dialog now follows the same migration line as the earlier users batch: direct Poodle field/control contracts inside retained Underlay shells.
- The remaining obvious foundational Underlay surface in `acme-admin` is concentrated in shared reusable forms rather than in these account routes.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`
- `effigy check`
- `effigy validate`

## Next Task

Take the next shared-form migration batch in `acme-admin`: replace the remaining Underlay foundational form primitives in [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/CategoryForm.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/CategoryForm.svelte) and [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/ProjectForm.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/ProjectForm.svelte), while leaving relation-heavy selectors and route-level orchestration in Underlay until the right Poodle boundary is clearer.
