# acme-admin

SvelteKit admin reference app for the Underlay reference workspace.

## Effigy-First Repo Loop

Use Effigy as the default command surface inside `acme-admin/`:

```bash
effigy tasks --repo .
effigy health --repo .
effigy validate --repo .
```

Common repo commands:

```bash
effigy dev --repo .
effigy check --repo .
effigy build --repo .
effigy qa --repo .
```

`health` runs the Svelte typecheck baseline and `validate` adds the production build so admin changes stay aligned with the shared reference-app contract.
