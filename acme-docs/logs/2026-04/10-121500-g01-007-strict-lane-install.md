# 2026-04-10 12:15:00 - g01.007 Strict Lane Install

## Summary

Installed a strict execution wrapper around `g01.007` for the Acme reference
implementation.

## Completed work

- added product guardrails for retained Underlay surface formalization
- added `policy/001-working-rules.md` as the compact strict execution contract
- added `specs/` and `specs/batch-cards/` surfaces
- opened `001-retained-underlay-surface-strict-lane.md`
- opened ready card `001-audit-retained-acme-admin-underlay-surface.md`
- refreshed front-door/currentness surfaces to point at the strict lane

## Validation

- `git -C /Users/betterthanclay/Dev/projects/underlay-reference diff --check`
- `effigy acme-docs/qa:docs --repo /Users/betterthanclay/Dev/projects/underlay-reference`
- `effigy acme-docs/qa:northstar --repo /Users/betterthanclay/Dev/projects/underlay-reference`

## Next Task

Have the active `underlay-reference` thread execute
`001-audit-retained-acme-admin-underlay-surface.md` from the new strict lane
instead of from roadmap summary alone.
