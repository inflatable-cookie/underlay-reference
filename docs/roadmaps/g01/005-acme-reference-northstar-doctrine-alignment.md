# g01.005 Acme Reference Northstar Doctrine Alignment

## Goal

Cut the Acme reference docs over to the Northstar contract so the reference app has one clear documentation authority, segmented roadmaps, and month-sharded logs.

## Why this matters now

The repo had a mixed docs shape with flat `roadmap/`, flat `reports/`, and a separate root `docs/` note file. That structure is easy to drift and harder for fresh agents to follow consistently.

## Scope

- [x] add `vision/` and document the reference-app purpose clearly
- [x] migrate flat roadmap files into `roadmaps/g01/`
- [x] migrate flat reports into `logs/YYYY-MM/`
- [x] move root implementation notes into `acme-docs/processes/`
- [x] update root and package `AGENTS.md` / `README.md` references to the new authority
- [x] remove retired local `roadmap/` and `reports/` paths with no shim files

## Validation

- [x] run a local markdown/path contract sweep for `acme-docs`, root docs, and package guides
- [x] commit and push the migration batch

## Next

After the migration lands, future reference-app execution should open `g01.006` for the next real milestone and record its batch evidence in `logs/YYYY-MM/`.
