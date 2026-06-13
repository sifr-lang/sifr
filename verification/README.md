# Sifr Verification

`verification/` owns runner mechanics, schemas, profiles, policy, and area-owned
verification data.

Python verification tooling is managed by `uv` through this directory:

```bash
uv run --project verification python -m sifr_verify --self-test
uv run --project verification python -m sifr_verify profiles check
uv run --project verification python -m sifr_verify areas check
uv lock --project verification --check
```

Minimum supported `uv` version: `0.9.28`.

The public validation entrypoint remains:

```bash
scripts/run_all_tests.sh --profile create-pr
```

`scripts/run_all_tests.sh` is a thin public facade over
`uv run --project verification --locked python -m sifr_verify profiles run`.
It fail-fasts when `uv` is missing or below the minimum version so profile
execution stays reproducible for local and CI validation.

## Layout

- `runner/sifr_verify/` contains runner code and self-tests.
- `schemas/` contains the supported committed data contracts.
- `profiles/` contains profile JSON files selected by `scripts/run_all_tests.sh --profile`
  and executed by `sifr_verify profiles run`.
- `areas/` contains area-owned manifests, fixtures, baselines, and adapters.
  `diagnostics` is migrated and can be run with
  `uv run --project verification python -m sifr_verify areas run --area diagnostics`.
- `policy/` contains machine-facing runner policy such as guardrail mappings.

Schemas intentionally support only a small subset: object shape, required keys,
primitive scalar types, arrays of objects or strings, enums, booleans, integers,
and repo-relative path strings. Unsupported schema keywords are rejected.
