# acme-api

Rust backend reference implementation for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `acme-api/`:

```bash
effigy tasks
effigy health
effigy validate
```

Common repo commands:

```bash
effigy dev
effigy jobs
effigy db:reset
effigy db:migrate
```

`health` and `validate` currently use the Rust build as the stable backend baseline. Workspace-level `db:*` commands resolve here through child-catalog routing rather than duplicated root wrappers.
