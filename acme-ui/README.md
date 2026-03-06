# acme-ui

Shared UI package for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `acme-ui/`:

```bash
effigy tasks --repo .
effigy health --repo .
effigy validate --repo .
```

Common repo commands:

```bash
effigy check --repo .
effigy qa --repo .
effigy refresh:deps --repo .
```

`health` and `validate` both use the package typecheck baseline here because there is no separate production build task yet.
