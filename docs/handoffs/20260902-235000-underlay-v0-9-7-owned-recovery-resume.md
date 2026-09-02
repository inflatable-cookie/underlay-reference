---
title: Resume Underlay Reference owned-media recovery on v0.9.7
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
status: ready-to-resume
owner: Tom / Northstar orchestrator
created: 2026-09-02
updated: 2026-09-02
base_required: pushed-main
tags: [coordination, handoff, worker, underlay, media, integrity, recovery]
---

## Assignment

Resume Card 003 in the existing worker, workspace, branch, and PR 14. Integrate
current pushed `main` without rewriting history. Move every Underlay declaration
and lock to released `v0.9.7` at
`8a7ce84b0501f6902da3ec1daf03f67ef0f42d4f`, then replace intent/byte-based
recovery with the released owned-promotion surface.

## Existing Lane Identity

- agent: `0bc493dc-7044-46f0-bd6f-a6d35ccfbe67`;
- workspace: `wks_c08140ce3908419e`;
- branch: `worker/underlay-v0-9-6-immutable-media-adoption`;
- PR: https://github.com/inflatable-cookie/underlay-reference/pull/14;
- prior head: `bfb6a41b87a3a8c39e2fd0c93a98c889ffd3facb`.

Do not create a replacement branch, workspace, agent, or PR.

## Required Repair

1. Add the spec-authorized private migration: a non-public `bytea` ownership
   token and a distinct owned destination key, with complete-or-absent
   constraints. Existing provider/bucket columns may complete the authority.
2. Generate a fresh cryptographically random token of at least 32 bytes per
   publication. Persist token plus provider/bucket/destination before create.
   Never expose the raw token through DTOs, logs, errors, URLs, or Debug output.
3. Use `promote_verified_owned` for first publication. Use
   `recover_owned_publication` only after a collision/restart and only with the
   database-owned token and exact immutable authority. Never read staging for
   recovery; never use intent, key secrecy, ETag, or byte equality as ownership.
4. Keep ready/current activation atomic. Persist only server-derived digest,
   size, MIME, provider, bucket, and published key.
5. Delete and purge staging and owned destination objects before deleting the
   row. Propagate blob cleanup failure and retain the row plus all identities so
   retry can converge.
6. Keep public DTOs and successful response shapes unchanged. Do not edit
   Underlay, Poodle, workflows, deployments, unrelated planning, or Bughunt
   review state.

## Oracle

Drive the real handler, Postgres, and a failure-capable blob adapter:

- pre-create crash plus foreign incumbent refuses;
- post-owned-create crash recovers after staging is deleted, mutated, or hostile;
- wrong token, provider, bucket, or destination refuses without mutation;
- identical foreign bytes refuse;
- activation rollback leaves owned recovery identity intact;
- delete and purge blob failure retain the row and succeed on retry;
- success yields exactly one immutable object, one ready version, and one current
  pointer;
- all Cargo/JavaScript declarations and locks resolve only `v0.9.7` at the
  released commit.

## Stop Conditions

Stop for public DTO change, schema work beyond the two authorized private
fields and their constraints, retention thresholds, unsupported production
adapter, token exposure, or an oracle that cannot execute. Do not weaken
collision refusal or ownership proof.

Push a new exact head to PR 14 and stop for orchestrator review.
