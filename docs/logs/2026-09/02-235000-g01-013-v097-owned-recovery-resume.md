# g01.013 v0.9.7 Owned-Recovery Resume

Date: 2026-09-02
Status: dispatched

## Decision

Underlay v0.9.7 supplies the positive ownership primitive that blocked PR 14.
Resume the same lane rather than replace it. The operator-approved design is a
fresh private token plus immutable destination authority persisted before
exclusive create; recovery accepts only matching token-bound object metadata.

The narrow private migration is now explicit authority. `media_version.object_key`
already carries staging identity, while `storage_provider` and `bucket` alone are
only intent. Card 003 may therefore add a private token column and distinct owned
destination-key column. Public DTOs remain unchanged.

## Required Closeout

- exact Underlay `v0.9.7` / `8a7ce84b0501f6902da3ec1daf03f67ef0f42d4f`;
- owned promotion and staging-independent recovery;
- cleanup failure retains database identity;
- full adversarial handler/Postgres/blob oracle;
- new exact PR 14 head for review.

## Next

Worker resumes from
`docs/handoffs/20260902-235000-underlay-v0-9-7-owned-recovery-resume.md`.
