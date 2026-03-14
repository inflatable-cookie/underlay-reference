# acme-ui

Shared UI package for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `acme-ui/`:

```bash
effigy tasks
effigy health
effigy validate
```

Common repo commands:

```bash
effigy check
effigy qa
effigy refresh:deps
```

`health` and `validate` both use the package typecheck baseline here because there is no separate production build task yet.
