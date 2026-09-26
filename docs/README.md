# Underlay Reference — current state

Acme is the complete Underlay-based reference app: a Rust API and jobs
runtime, a typed TypeScript client, an admin frontend, and a public
frontend. Copy the repo and rename `acme`. It is a bootstrap target, not a
product strategy surface.

The workspace pins Underlay `v0.9.10` and public Poodle `0.4.2`. The retained
Underlay surface of `acme-admin` is frozen. Downstream apps use Poodle for
primitives and simple composites, and keep Underlay only where the retained
contract names it.

## By topic

- Vision: [knowledge/vision.md](knowledge/vision.md)
- Architecture: [knowledge/architecture/000-overview.md](knowledge/architecture/000-overview.md)
- Retained Underlay surface: [knowledge/architecture/004-retained-underlay-surface-contract.md](knowledge/architecture/004-retained-underlay-surface-contract.md)
- Working rules: [knowledge/contracts/working-rules.md](knowledge/contracts/working-rules.md)
- Implementation notes: [knowledge/operations/reference-implementation-notes.md](knowledge/operations/reference-implementation-notes.md)
- Everything else: [knowledge index](knowledge/README.md)

## What's next

See [plan.md](plan.md). Unresolved leads are in [triage/](triage/).

## Docs catalog

Root `docs/` is addressed through the `acme-docs` Effigy catalog alias.

```bash
effigy acme-docs/health
effigy acme-docs/qa:docs
effigy acme-docs/qa:northstar
effigy acme-docs/check:rollout admin-freshness
effigy acme-docs/check:rollout auth-security-alerting
effigy acme-docs/check:rollout reorder-conflict
```
