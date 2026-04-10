# 001 - Retained Underlay Surface Strict Lane

Status: active
Owner: repo maintainers
Updated: 2026-04-10
Roadmap refs: g01.007
Governing refs: acme-docs/architecture/product-guardrails.md, acme-docs/policy/001-working-rules.md

## Problem

`g01.007` is the honest live owner for the reference app, but execution has
been running from roadmap prose alone. The retained-Underlay audit needs a
strict wrapper so downstream app rollout work does not outrun the approved
boundary definition.

## Goal

Freeze the retained Underlay surface in `acme-admin` as a durable reusable
boundary contract for downstream app migrations.

## Ready chain

- `001-audit-retained-acme-admin-underlay-surface.md` — ready

## Stop Conditions

- the audit turns into a broad route-conversion sweep
- retained surfaces cannot be classified without a wider planning decision
- the live code surface no longer matches the retained-boundary owner

## Next Task

Execute `001-audit-retained-acme-admin-underlay-surface.md` as the active ready
card for `g01.007`.
