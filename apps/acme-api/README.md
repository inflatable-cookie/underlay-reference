# acme-api

Rust backend reference implementation for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `apps/acme-api/`:

```bash
effigy tasks
effigy health
effigy validate
```

Common repo commands:

```bash
effigy dev
effigy jobs
effigy migration:apply
effigy migration:reset
```

`health` uses `fmt` plus a cheap `oxide check` baseline; `validate` still runs the Rust build. Host Rust goes through Oxide (`oxide build`, `oxide test`, `oxide run -p acme-api`); the managed Oxide block in the root `AGENTS.md` is the command rule. Workspace-level `migration:*` commands resolve here through child-catalog routing rather than duplicated root wrappers, and run their Rust on the host inside the bundle's `hybrid:exec` profile environment. From the workspace root, use `effigy state plan` and `effigy state apply local --yes` for the local stack.
