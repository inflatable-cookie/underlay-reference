# g01 Roadmaps

`g01` is the active roadmap generation for the Acme reference implementation.

## Sequence

- `001` reference completion
- `002` security audit
- `003` comprehensive sweeps remediation
- `004` wasteful endpoint calls remediation
- `005` Acme reference Northstar doctrine alignment
- `006` Poodle Underlay coexistence proof
- `007` retained Underlay surface formalization
- `008` auth service adoption and hardening (from the g08 consumer audit)
- `009` media and blob production path (from the g08 consumer audit)
- `010` TypeScript type-safety hygiene (from the g08 consumer audit)
- `011` gate hardening and lint cleanup (from the g08 consumer audit)
- `012` Northstar instruction and language-quality audit
- `013` Underlay v0.9.7 owned media recovery

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

## Next Task

Resume `g01.013` Card 003 on existing PR 14. `g01.012` is complete. `g01.007`
remains paused during the dependency and media rollout; resume it after merge.
