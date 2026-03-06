# Agents Guide: Acme Docs

## Scope

`acme-docs` is the documentation authority for the Underlay reference implementation. Keep planning, architecture, and execution history here rather than in package-local docs.

## Hard Rules

- Put reference-app roadmap work in `roadmaps/g*/` with three-digit IDs.
- Put execution history and sweep closeouts in `logs/YYYY-MM/` using `DD-HHMMSS-slug.md` filenames.
- Do not leave compatibility shim docs behind after moves; update links in place.
- Keep `vision/` high-level and stable, `architecture/` concrete, and `processes/` operational.
- Prefer Underlay source docs for shared framework doctrine and `acme-docs` for reference-app-specific application.

## Validation

```bash
rg -n "roadmap/|reports/|decisions/" acme-docs README.md AGENTS.md acme-api/AGENTS.md acme-client/AGENTS.md acme-admin/AGENTS.md acme-front/AGENTS.md
python3 - <<'PY'
from pathlib import Path
root = Path('.')
for path in [root/'README.md', root/'AGENTS.md', *Path('acme-docs').rglob('*.md')]:
    text = path.read_text()
    if 'acme-docs/roadmap/' in text or 'acme-docs/reports/' in text:
        raise SystemExit(f'stale path in {path}')
print('ok')
PY
```

## Reference Docs

- `vision/001-acme-reference-implementation-vision.md`
- `architecture/000-overview.md`
- `processes/210-reference-implementation-notes.md`
- `roadmaps/README.md`
- `logs/README.md`
