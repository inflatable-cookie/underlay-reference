# acme-client

Typed TypeScript API client for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `packages/acme-client/`:

```bash
effigy tasks
effigy health
effigy validate
```

Common repo commands:

```bash
effigy check
effigy build
effigy qa
```

`health` runs the typecheck baseline and `validate` adds the package build so the client stays transport-focused and publishable.
