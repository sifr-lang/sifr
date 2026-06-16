# Sifr documentation (Mintlify)

Public docs: [docs.sifr.sh](https://docs.sifr.sh)

Source in this directory is deployed by [Mintlify](https://mintlify.com) from `sifr-lang/sifr` (subdirectory `docs`).

## Deployment

Mintlify must be configured as a monorepo deployment:

- Dashboard: **Git Settings**
- Repository: `sifr-lang/sifr`
- Branch: `main`
- Documentation path: `/docs`

Do not include a trailing slash in the documentation path. Mintlify expects `docs.json` and `.mintignore` relative to this directory.

## Local preview

```bash
cd docs
npx mint@latest dev
```

## Assistant instructions

Mintlify assistant instructions live in `.mintlify/Assistant.md`. Keep the file inside `.mintlify/` so it is not publicly served.

## Internal reference (not published)

Compiler reference markdown lives alongside Mintlify pages but is excluded via `.mintignore`:

- `errors/` — generated diagnostic code reference
- `schemas/` — internal schemas
- `*.md` flat files — CLI semantics, formatter, linter, etc.

Migrate these to MDX and add them to `docs.json` when ready to publish.
