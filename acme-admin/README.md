# acme-admin

SvelteKit admin reference app for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `acme-admin/`:

```bash
effigy tasks
effigy health
effigy validate
```

Common repo commands:

```bash
effigy dev
effigy check
effigy build
effigy qa
```

`health` runs the Svelte typecheck baseline and `validate` adds the production build so admin changes stay aligned with the shared reference-app contract.
