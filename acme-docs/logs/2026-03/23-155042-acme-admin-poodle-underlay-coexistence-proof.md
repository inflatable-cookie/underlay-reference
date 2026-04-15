---
title: Acme admin Poodle Underlay coexistence proof
status: completed
owner: Platform
updated: 2026-03-23
tags: [execution, migration, poodle, underlay]
---

## Summary

`acme-admin` now runs a real mixed-surface proof: Poodle owns theme bootstrap at the app root, the dashboard uses direct Poodle metric primitives and composites, and retained Underlay structure still renders on the same route tree.

## What changed

- [~/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/+layout.svelte) now imports the Poodle token stylesheet, binds a root theme shell, and applies Poodle theme attributes with `theme="dark"`, `density="compact"`, and `controlSize="md"`.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+page.svelte) replaces Underlay `StatGrid` and `StatCard` usage with direct Poodle `MetricTile` and `Pill` composition for the dashboard metrics.
- [~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+layout.svelte](~/Dev/projects/underlay-reference/acme-admin/src/routes/(app)/+layout.svelte#L137) continues to host retained Underlay `ErrorBoundary`, proving that a structural Underlay surface can coexist under Poodle-owned theming.

## Coexistence notes

- The original proof used a temporary canonical-package alias bridge while Poodle packaging was still incomplete.
- `acme-admin` now installs [`@poodle/svelte-tokens`](~\/Dev\/projects\/underlay-reference\/acme-admin\/package.json#L19), [`@poodle/svelte-primitives`](~\/Dev\/projects\/underlay-reference\/acme-admin\/package.json#L20), and [`@poodle/svelte-composites`](~\/Dev\/projects\/underlay-reference\/acme-admin\/package.json#L21) directly via local `file:` dependencies.
- The temporary alias bridge has been removed from [~/Dev/projects/underlay-reference/acme-admin/svelte.config.js](~/Dev/projects/underlay-reference/acme-admin/svelte.config.js), and the root layout now imports [`@poodle/svelte-tokens/styles.css`](~\/Dev\/projects\/underlay-reference\/acme-admin\/src\/routes\/+layout.svelte#L7) instead of a raw filesystem token path.

## Validation

- `effigy check`
- `effigy validate`
- `effigy docs check-links README.md vision/README.md roadmaps/README.md logs/README.md`

## Next Task

Open the next `acme-admin` migration batch around one route family with obvious foundational Underlay usage, using the now-direct Poodle package path as the default integration model instead of any app-local bridge.
