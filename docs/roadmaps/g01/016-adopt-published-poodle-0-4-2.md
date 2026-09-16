# g01.016 Adopt Published Poodle 0.4.2

Owner: repo maintainers
Created: 2026-09-16
Governing refs: Poodle CHANGELOG 0.3.0/0.4.0/0.4.2 at tag `v0.4.2`
Depends on: no other Queue task
UI classification: none

## Outcome

underlay-reference moves its Poodle pins onto the current published release, `0.4.2`,
and absorbs the 0.3 → 0.4 breaking changes where they apply. Behaviour is
unchanged except where the release itself changed it.

## Context

Poodle is published on npm at `0.4.2`. This repository pins
`@inflatable-cookie/poodle-core` / `-svelte` at `0.3.0` in `apps/acme-admin/package.json`, `apps/acme-front/package.json`, and `packages/acme-ui/package.json`.


Poodle 0.4 removed or re-broke: Slider/RangeSlider `appearance`→`variant` plus
eighteen recipe hooks; markdown sanitize-by-default (`renderHtml` no longer
implies trust); `poodle-specs` Slider* items; `poodle-headless` text helpers;
`poodle-node` `NodeRole` variants and `NodeA11y.initial_focus` (exhaustive
matches break).

## Work

1. Move every `@inflatable-cookie/poodle-core` and `@inflatable-cookie/poodle-svelte`
   pin from `0.3.0` to `0.4.2` in `apps/acme-admin/package.json`, `apps/acme-front/package.json`, and `packages/acme-ui/package.json`.
2. Absorb any breaking change the build or tests surface. No compatibility
   shims.
3. Run this repository's own checks (`effigy qa`).

## Acceptance and review oracle

| Invariant | Required proof |
| --- | --- |
| No stale Poodle pin survives | no `@inflatable-cookie/poodle-*` pin below `0.4.2` in this repository |
| The apps still build and render | this repository's checks pass |
| No shim was added | the diff changes pins and call sites only |

## Stop conditions

Stop and report if a change needs a material product, visual, or interaction
decision the operator has not made, or if a removed Poodle API has no direct
replacement. Do not add a compatibility shim or vendor a Poodle copy.

## Evidence

On completion, record: the changed manifests, the resolved versions, and the
`effigy qa` result.
