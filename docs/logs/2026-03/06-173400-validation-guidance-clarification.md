# Validation guidance clarification

## Summary

- Clarified that the AGENTS validation snippet intentionally checks for deprecated flat docs paths.
- Left the validation logic unchanged.

## Files changed

- `acme-docs/AGENTS.md`

## Why

The final doctrine sweep kept surfacing the validation snippet as if it were live path doctrine. Adding one explicit note keeps the guard in place while making its purpose obvious to future agents.

## Next actions

- Keep using the validation snippet to block deprecated flat docs paths from reappearing.
- Treat hits inside validation snippets as expected unless the checks themselves become stale.
