# acme-client

Typed TypeScript API client for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `acme-client/`:

```bash
effigy tasks --repo .
effigy health --repo .
effigy validate --repo .
```

Common repo commands:

```bash
effigy check --repo .
effigy build --repo .
effigy qa --repo .
```

`health` runs the typecheck baseline and `validate` adds the package build so the client stays transport-focused and publishable.
