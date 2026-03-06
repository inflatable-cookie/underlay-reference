# Underlay Reference Effigy Adoption Closeout

Date: 2026-03-06

## Summary

- Established Effigy as the default workspace command surface for the Underlay reference root.
- Normalized the active child repos around repo-local Effigy catalogs, repo `AGENTS.md`, `README.md`, and package-script entrypoints.
- Integrated `acme-docs` into the same contract using rollout audit tasks instead of leaving the docs authority outside the normal repo loop.
- Preserved child-task ownership so `db:*` stays owned by `acme-api/` and resolves from the workspace root through child-catalog routing.

## Main outcomes

- Workspace root now owns cross-repo orchestration through `health`, `validate`, `qa`, and `dev`.
- `acme-api`, `acme-client`, `acme-admin`, `acme-front`, and `acme-ui` now all expose repo-local Effigy baselines for agents and contributors.
- `acme-docs` now participates as a first-class Effigy repo with `health`, `validate`, and `qa` backed by the rollout audit scripts in the workspace `scripts/` directory.
- Workspace and child package scripts now route back through Effigy instead of teaching raw package commands as the default loop.

## Repos normalized

- workspace root
- `acme-api`
- `acme-client`
- `acme-admin`
- `acme-front`
- `acme-ui`
- `acme-docs`

## Validation highlights

- Workspace task discovery now exposes root `health`, `validate`, and `qa` along with child catalog tasks.
- Root `db:reset` resolution selects `acme-api/db:reset` directly rather than relying on duplicated root wrappers.
- `acme-docs` now has a real local validation surface through `check:admin-freshness-rollout`, `check:auth-security-alerting-rollout`, and `check:reorder-conflict-rollout`.
- Root and child package-script shims were validated against their Effigy task surfaces.

## Remaining caveats

- This rollout establishes the day-to-day command surface; it does not attempt to clear any broader `doctor` scan backlog that may exist.
- Historical docs and archived roadmap or log bodies may still contain raw tool commands where they are serving as records or reusable reference material rather than active repo entry guidance.

## Next actions

- Use this reference workspace as the template for future Underlay-based consumer repo adoption.
- Add deeper repo-local test tasks only when the individual packages gain meaningful owned test surfaces.
