# Sifr documentation (Mintlify)

Public docs: [docs.sifr.sh](https://docs.sifr.sh)

Source in this directory is deployed by [Mintlify](https://mintlify.com) from `sifr-lang/sifr` (subdirectory `docs`).

## Local preview

```bash
cd docs
npx mint@latest dev
```

## Internal reference (not published)

Compiler reference markdown lives alongside Mintlify pages but is excluded via `.mintignore`:

- `errors/` — generated diagnostic code reference
- `schemas/` — internal schemas
- `*.md` flat files — CLI semantics, formatter, linter, etc.

Migrate these to MDX and add them to `docs.json` when ready to publish.
