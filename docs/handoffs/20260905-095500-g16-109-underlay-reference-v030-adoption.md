---
title: g16.109 Underlay Reference Poodle v0.3.0 adoption worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
status: paused-operator-request
owner: Tom / Poodle coordinator
created: 2026-09-05
updated: 2026-09-05
handoff_path: /Users/tom/.paseo/worktrees/119ajruu/g16-109-underlay-reference-v030-adoption/docs/handoffs/20260905-095500-g16-109-underlay-reference-v030-adoption.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, g16, poodle, underlay-reference]
---

## Assignment

Execute the Underlay Reference lane from Poodle card `g16.109`,
`/Users/tom/Dev/projects/poodle/docs/roadmaps/g16/109-v030-consumer-adoption-wave.md`.
Adopt the published public-registry Poodle `0.3.0` packages in the reference
consumer, apply the one required markdown import migration, validate with the
consumer's own headless selectors, and stop for independent exact-head review.

## Exact workspace and heads

- repository: `git@github.com:inflatable-cookie/underlay-reference.git`;
- workspace: `/Users/tom/.paseo/worktrees/119ajruu/g16-109-underlay-reference-v030-adoption`;
- worker branch: `worker/g16-109-underlay-reference-v030-adoption`;
- base: `origin/main` at `171bf940fcac9030d4b195122dc13cac21b55b0c`;
- implementation head before this handoff commit:
  `8b042f53` (`deps: adopt Poodle 0.3.0 consumer release`);
- Underlay foundation: merged PR #26 at
  `5a6709be190255190ba35bee346ea1e517897902`;
- Poodle release authority: `v0.3.0`, published 2026-09-05;
- sibling checkouts are read-only inputs; no coordinator checkout or sibling
  repository was edited.

## Changes

- Pinned exact `@inflatable-cookie/poodle-core` and
  `@inflatable-cookie/poodle-svelte` `0.3.0` in all consumer declarations.
- Moved all consumer Underlay git declarations to the merged foundation commit
  so its package metadata also resolves Poodle `0.3.0`.
- Changed the sole root markdown-family import,
  `MarkdownEditor` in `packages/acme-ui/src/nightfire/notes/TaskNotesEditor.svelte`,
  to `@inflatable-cookie/poodle-svelte/markdown`.
- No shim, alias, compatibility re-export, or unrelated source change.
- No PAPERCUT entry was closed: the open entries do not meet the card rule of
  being demonstrably fixed by the `0.3.0` release note.

## Dependency and lock evidence

- Bun: `1.3.14` (repository `packageManager` version).
- Regeneration: `bun install` completed and saved `bun.lock`.
- `effigy --json deps status bun`: clean; zero links, drift, conflicts,
  errors, warnings, and missing dependencies.
- `bun.lock` contains exactly one public Poodle core identity and one public
  Poodle Svelte identity, both `0.3.0`, with registry integrity hashes.
- Final lock grep found no `file:`, `link:`, sibling `poodle`/`underlay` path,
  or Poodle `0.2.x` marker.
- Lock diff is limited to the Poodle pins, the related merged Underlay
  foundation reference, and Bun's resulting metadata convergence.

## Headless validation

All commands ran from the workspace above through Effigy; no desktop app or
native proof was launched.

- `effigy acme-admin/validate`: pass; Svelte check clean and production build
  completed.
- `effigy acme-front/validate`: pass; 0 Svelte errors and 0 warnings.
- `effigy acme-ui/validate`: pass; 0 Svelte errors and 0 warnings.
- `effigy acme-client/validate`: pass; TypeScript check completed.
- `effigy acme-admin/test`: pass; 2 files, 32 tests.
- `effigy acme-client/test`: pass; 2 files, 3 tests.
- `effigy acme-front/check`: pass; 0 Svelte errors and 0 warnings.
- `effigy acme-ui/check`: pass; 0 Svelte errors and 0 warnings.
- `git diff --check`: pass.

## Review boundary

PR #16 is open against `underlay-reference` `main` at the current branch head,
but the operator paused this lane on 2026-09-05. The Underlay PR #26 merge is
not a released/tagged foundation version, so this lane is not complete and is
not eligible for Tier 2 gating. Resume only after a released/tagged Underlay
foundation carrying Poodle `0.3.0` exists and the consumer can pin that
released/tagged version.

Do not merge, start another lane, edit Poodle or Underlay, or run additional
validation/native/desktop proofs while paused.
