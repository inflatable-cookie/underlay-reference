---
title: Acme admin shared form Poodle migration
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay, acme-admin, forms]
---

## Summary

The shared `CategoryForm` and `ProjectForm` surfaces now use direct Poodle primitives for foundational form behavior, which moves the reusable create/edit layer off Underlay primitives without disturbing the higher-order Underlay patterns wrapped around them.

## What changed

- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/CategoryForm.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/CategoryForm.svelte) now uses Poodle `Field`, `FieldSet`, `TextInput`, `TextArea`, `ColorPicker`, `Switch`, `Button`, `FormActions`, and `SplitButton` directly.
- The category form no longer depends on Underlay `FieldSetGrid`, `FormValidationProvider`, `SaveSplitButton`, or `TextButton`; validity and save-intent handling now live explicitly in local form logic.
- [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/ProjectForm.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/lib/forms/ProjectForm.svelte) now uses Poodle `Field`, `FieldSet`, `TextInput`, `TextArea`, `Select`, `Button`, `FormActions`, and `SplitButton` directly.
- The inline category-create affordance inside `ProjectForm` also now uses Poodle buttons, so the shared form surface is consistent even when the retained Underlay `RelationSelector` opens its embedded create flow.

## Coexistence notes

- This batch kept the architectural line intact: Underlay `SlugField` and `RelationSelector` remain in place because they are still higher-order patterns, not foundational primitives.
- Hidden inputs now carry values for fields like `intent`, `color`, `isActive`, and `categoryId`, so the server-side payload shape remains stable while the primitive layer changes underneath.
- The remaining obvious Underlay foundational UI in `acme-admin` is now concentrated in route-local controls rather than the reusable form library.

## Validation

- `bun x svelte-check --tsconfig ./tsconfig.json`
- `effigy check`
- `effigy validate`

## Next Task

Take the next account-security migration batch in `acme-admin`: replace the remaining Underlay `Field`, `TextInput`, `FormActions`, and adjacent simple action controls in [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/2fa/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/2fa/+page.svelte) and [/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/passkeys/+page.svelte](/Users/betterthanclay/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/account/passkeys/+page.svelte), while keeping the surrounding workflows and route shells stable.
