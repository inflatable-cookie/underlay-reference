# 017: Migrate local state stack to repository task (Effigy v0.15)

- Replaces the retired `[state]` / `[state.local]` layered state stack with an
  equivalent `[tasks."state:local"]` sequence running
  `acme-api/migration:reset:schema`, `acme-api/migration:apply`, and
  `acme-api/migration:apply:overlay` in order.
- Raises root `minimum_effigy_version` to `0.15.0`, matching the Effigy bundle.
- Updates `README.md`, `apps/acme-api/README.md`, and
  `docs/knowledge/architecture/000-overview.md` to invoke `effigy state:local`.
- Held for Effigy v0.15 upgrade day; do not merge before then.
