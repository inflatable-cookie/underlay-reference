---
title: g16.109 Underlay Reference Poodle v0.3.0 adoption worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
status: ready-for-independent-review
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
- implementation head before this handoff refresh:
  `f0dd253f` (`deps: use released Underlay v0.9.8 foundation`);
- Underlay foundation: released annotated tag `v0.9.8`, verified on origin;
  tag object `09186eecda9a48a05239cebdc4e1c26ac666a934` peels to release commit
  `97a26d9fa0a58daf198926ddcd259193daa9d5c3`;
- Poodle release authority: `v0.3.0`, published 2026-09-05;
- sibling checkouts are read-only inputs; no coordinator checkout or sibling
  repository was edited.

## Changes

- Pinned exact `@inflatable-cookie/poodle-core` and
  `@inflatable-cookie/poodle-svelte` `0.3.0` in all consumer declarations.
- Moved all consumer Underlay git declarations to released tag `v0.9.8` so its
  package metadata resolves Poodle `0.3.0`.
- Changed the sole root markdown-family import,
  `MarkdownEditor` in `packages/acme-ui/src/nightfire/notes/TaskNotesEditor.svelte`,
  to `@inflatable-cookie/poodle-svelte/markdown`.
- No shim, alias, compatibility re-export, or unrelated source change.
- No PAPERCUT entry was closed: the open entries do not meet the card rule of
  being demonstrably fixed by the `0.3.0` release note.

## Dependency and lock evidence

- Bun: `1.3.14` (repository `packageManager` version).
- Regeneration: `bun install` completed and saved `bun.lock`.
- Underlay release proof: `git ls-remote` returned annotated tag
  `v0.9.8` peeling to `97a26d9fa0a58daf198926ddcd259193daa9d5c3`.
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
- Two parallel check invocations initially hit Effigy task locks held by the
  concurrent validators; serial reruns passed. This is recorded as a tooling
  concurrency result, not a code failure.

## Review boundary

PR #16 is open against `underlay-reference` `main`. The operator resumed this
lane after Underlay `v0.9.8` was released and verified. Push the current branch
and stop for independent exact-head review; do not merge, start another lane,
edit Poodle or Underlay, or run native/desktop proofs.
