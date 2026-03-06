# acme-api

Rust backend reference implementation for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `acme-api/`:

```bash
effigy tasks --repo .
effigy health --repo .
effigy validate --repo .
```

Common repo commands:

```bash
effigy dev --repo .
effigy jobs --repo .
effigy db:reset --repo .
effigy db:migrate --repo .
```

`health` and `validate` currently use the Rust build as the stable backend baseline. Workspace-level `db:*` commands resolve here through child-catalog routing rather than duplicated root wrappers.
