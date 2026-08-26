---
title: Acme admin users route Poodle form migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, users]
---

## Summary

The `users` route family now has a real direct-Poodle form cluster. The shared `UserForm` and the role-change dialog both migrated off Underlay field primitives while the surrounding Underlay route shells remained in place.

## What changed

- [~/Dev/projects/underlay-reference/acme-admin/src/lib/forms/UserForm.svelte](~/Dev/projects/underlay-reference/acme-admin/src/lib/forms/UserForm.svelte) now imports Poodle primitives directly instead of Underlay field/form components.
- The shared form no longer uses Underlay `FormValidationProvider`, `FieldSetGrid`, `SaveSplitButton`, `TextButton`, `Select`, `Switch`, `TextInput`, or `Field`.
- Validation and field messaging now flow through Poodle `Field` contracts, with direct Poodle control events (`valueChange`, `checkedChange`) and a hidden `intent` input used to preserve the existing save/save-close submission semantics.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/[userId]/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/users/[userId]/+page.svelte) now uses Poodle `Field`, `Select`, `Button`, and `FormActions` inside the retained Underlay `FormDialog`.

## Coexistence notes

- This batch intentionally did not replace `SpaFormShell`, `FormDialog`, `PageLoading`, `FormError`, or the larger page/detail shells.
- The migration line stayed at the foundational UI layer: Poodle now owns the form primitives, while Underlay still owns the route-level structural patterns around them.
- The save-intent behavior previously hidden behind Underlay `SaveSplitButton` now lives directly in the form as explicit app logic, which is the right direction for the coexistence period.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`
- `effigy check`
- `effigy validate`

## Next Task

Take the next account-oriented form batch in `acme-admin`: replace the remaining Underlay form primitives in [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/+page.svelte) and [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/password/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/password/+page.svelte), keeping the route shells stable while shrinking the remaining foundational Underlay surface.
