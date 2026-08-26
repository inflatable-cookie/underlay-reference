# Acme Reference Northstar Doctrine Alignment

## Change summary

- migrated the Acme docs authority from flat `roadmap/` and `reports/` folders to segmented `roadmaps/g01/` and month-sharded `logs/`
- added a vision layer and core Northstar readmes so the reference app documents are self-describing
- moved root reference implementation notes into `acme-docs/processes/` and rewired package-level guidance to the new authority
- removed retired docs paths and the stray root `docs/` island with no compatibility shims

## Files touched

- `acme-docs/README.md`
- `acme-docs/AGENTS.md`
- `acme-docs/vision/`
- `acme-docs/roadmaps/`
- `acme-docs/logs/`
- `acme-docs/processes/210-reference-implementation-notes.md`
- `README.md`
- `AGENTS.md`
- `acme-api/AGENTS.md`
- `acme-client/AGENTS.md`
- `acme-admin/AGENTS.md`
- `acme-front/AGENTS.md`

## Why

The reference repo needs one obvious docs authority so fresh agents can follow the same contract as the rest of the migrated project set. Flattened legacy planning folders and orphaned notes make that harder.

## Next actions

- open `g01.006` only when there is a real new milestone to execute
- keep future execution evidence in `acme-docs/logs/YYYY-MM/`
- avoid reintroducing package-local roadmap or report docs
