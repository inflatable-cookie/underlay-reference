# 001 Acme Reference Implementation Vision

## Purpose

Acme exists to demonstrate how to assemble a complete Underlay-based application without forcing downstream teams to invent project structure, API boundaries, or frontend patterns from scratch.

## Long-term outcome

The reference implementation should remain a credible bootstrap target for new products that want:
- a Rust API and jobs runtime aligned with Underlay contracts
- a typed TypeScript client with stable transport conventions
- an admin frontend that demonstrates shared Underlay patterns clearly
- a lightweight public frontend with the same auth and client foundations

## Scope boundaries

Acme is a reference app, not a product strategy surface.
It should prioritize canonical implementation patterns, migration examples, and repeatable architecture over bespoke feature expansion.

## What good looks like

- New teams can copy the repo and rename `acme` with minimal structural cleanup.
- Shared Underlay capabilities are used directly where they fit instead of being reimplemented locally.
- Roadmap work stays focused on reference quality, completeness, and pattern clarity.
- Execution logs explain meaningful change batches without turning the repo into a task-by-task diary.

## Next Task

Use this vision as the baseline for the next reference-implementation roadmap
batch and reopen it only when the repo's bootstrap role or boundary changes.
