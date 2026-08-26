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

`health` uses `fmt` plus a cheap `cargo check` baseline; `validate` still runs the Rust build. Workspace-level `migration:*` commands resolve here through child-catalog routing rather than duplicated root wrappers. From the workspace root, use `effigy state plan` and `effigy state apply local --yes` for the local stack.
