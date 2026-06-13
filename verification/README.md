# Sifr Verification

`verification/` owns runner mechanics, schemas, profiles, policy, and area-owned
verification data.

Python verification tooling is managed by `uv` through this directory:

```bash
uv run --project verification python -m sifr_verify --self-test
uv run --project verification python -m sifr_verify profiles check
uv lock --project verification --check
```

Minimum supported `uv` version: `0.9.28`.

The public validation entrypoint remains:

```bash
scripts/run_all_tests.sh --profile create-pr
```

During the migration, `scripts/run_all_tests.sh` is still the authoritative
facade for local and CI validation. It fail-fasts when `uv` is missing or below
the minimum version so the runner foundation and lockfile stay reproducible
before profile execution is cut over to `sifr_verify`.

## Layout

- `runner/sifr_verify/` contains runner code and self-tests.
- `schemas/` contains the supported committed data contracts.
- `profiles/` contains profile JSON files selected by `scripts/run_all_tests.sh --profile`.
- `areas/` will contain area-owned manifests, fixtures, baselines, and adapters
  as each area migrates.
- `policy/` contains machine-facing runner policy such as guardrail mappings.

Schemas intentionally support only a small subset: object shape, required keys,
primitive scalar types, arrays of objects or strings, enums, booleans, integers,
and repo-relative path strings. Unsupported schema keywords are rejected.
