# g01 Roadmaps

`g01` is the active roadmap generation for the Acme reference implementation.

## Sequence

- `001` reference completion
- `002` security audit
- `003` comprehensive sweeps remediation
- `004` wasteful endpoint calls remediation
- `005` Acme reference Northstar doctrine alignment
- `006` Poodle Underlay coexistence proof
- `007` retained Underlay surface formalization — complete in merged PR 15
- `008` auth service adoption and hardening (from the g08 consumer audit)
- `009` media and blob production path (from the g08 consumer audit)
- `010` TypeScript type-safety hygiene (from the g08 consumer audit)
- `011` gate hardening and lint cleanup (from the g08 consumer audit)
- `012` Northstar instruction and language-quality audit
- `013` Underlay v0.9.7 owned media recovery — complete in PR 14

## Next roadmap

- Open the next real milestone as `g01.014`.

## g08 consumer-audit tranche (`g01.008`-`g01.011`)

The 2026-07-18 foundation consumer audit fixed the clear-cut security/quality
items in-place (open-redirect, stored XSS, login-timing, blob fail-closed,
`noImplicitAny`, non-compiling infra tests). `g01.008`-`g01.011` carried the
larger follow-ups it deferred and completed on 2026-07-19.

## Historical language boundary

- New roadmaps and actively maintained roadmap updates must use roadmap IDs and batch language.
- Imported `g01.001` through `g01.004` milestones remain historical implementation records unless they are reopened.
- Normalize local wording only when one of those imported roadmaps is reopened for active work or when an old label causes path/reference drift.

## Queue lifecycle adoption

- [g01.014 Effigy-hosted lifecycle hook](014-adopt-effigy-hosted-lifecycle-hook.md)
  is an operator-approved, configuration-only maintenance lane. It follows its
  declared Queue dependencies and may run without changing product priority.
  Existing next-task text continues to describe product sequencing; this entry
  authorizes no sibling product work.
- [g01.017 Prospective-merge protocol migration](017-prospective-merge-protocol-migration.md)
  is an operator-approved, configuration-only maintenance lane that moves the
  Queue control manifest to the accepted v4 prospective-merge shape. It runs
  without changing product priority and authorizes no sibling product work.

## Next Task

`g01.007`, `g01.012`, and `g01.013` are complete. Re-enter planning before
opening the next task; no implementation card is ready.
<!-- northstar:lifecycle:begin schema=northstar.lifecycle.projection.v2 digest=sha256:aecacf0f046763bfd806332681e867f9da9ef6757c2f78710ba89ad8b7f1360a -->
| Generation | Disposition | Runway state |
| --- | --- | --- |
| g01 | open | planning_required |
| Task | Status | Stage | Revision | Record digest |
| --- | --- | --- | --- | --- |
| g01.014 | complete | none | 8 | sha256:4dee75213a769e2735b3c74b73851aac34ad5ff43840b54b9fa902f332939473 |
| g01.015 | complete | none | 8 | sha256:943f2d28a6f065768a9174720e1b0111ff4a7a6f09b0a9742b02eddf7f58a34d |
| g01.016 | complete | none | 8 | sha256:22f379a3a2999a2ff95e5b10c116dfed358a527763875c71753eab276509a2ed |
| g01.017 | complete | none | 8 | sha256:0bd2a4aa1002467212df01e00925a1a116917b9bf7461acc5afd35c92ca98817 |
<!-- northstar:lifecycle:end -->
