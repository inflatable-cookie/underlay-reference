# Active docs guidance normalization

## Summary

- Added an explicit historical language boundary to the active roadmap indexes.
- Tightened the docs-authority agent guidance so new roadmap work stays in roadmap-ID and batch language.
- Left imported roadmap bodies unchanged because they are historical implementation records.

## Files changed

- `acme-docs/roadmaps/README.md`
- `acme-docs/roadmaps/g01/README.md`
- `acme-docs/AGENTS.md`

## Why

The reference implementation had already moved to the Northstar folder contract, but the active roadmap guidance did not yet explain how to treat inherited phase-era roadmap language. Making that boundary explicit reduces repeated cleanup churn and keeps fresh agents from extending the older planning model.

## Next actions

- Open the next real milestone as `g01.006` when new reference-app work is ready.
- Normalize imported roadmap wording only when a historical roadmap is reopened or when an old label causes live drift.
