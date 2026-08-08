# Sifr Verification

`verification/` owns runner mechanics, schemas, profiles, policy, and area-owned
verification data.

Python verification tooling is managed by `uv` through this directory:

```bash
uv run --project verification python -m sifr_verify --self-test
uv run --project verification python -m sifr_verify profiles check
uv run --project verification python -m sifr_verify profiles plan --profile merge
uv run --project verification python -m sifr_verify areas check
uv run --project verification python -m sifr_verify doctor
uv lock --project verification --check
```

Minimum supported `uv` version: `0.9.28`.

The public validation entrypoint remains:

```bash
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh --profile merge --emit-plan
```

`scripts/run_all_tests.sh` is a thin public facade over
`uv run --project verification --locked python -m sifr_verify profiles run`.
It fail-fasts when `uv` is missing or below the minimum version so profile
execution stays reproducible for local and CI validation.

`--emit-plan` prints the selected profile's machine-readable execution plan
without running suites. CI may add broader profiles, but it must not omit suites
from the local merge plan except through declared host skips.
Compare local and CI plans with:

```bash
uv run --project verification --locked python -m sifr_verify profiles compare-plans --local <local-plan.json> --ci <ci-plan.json>
```

`sifr_verify doctor` checks required local prerequisites: Python version, Rust
and Cargo availability, `uv` lock status, Cargo offline metadata resolution, and
host metadata. Optional sanitizer tools are reported as pass or skip for broader
lanes.

## Layout

- `runner/sifr_verify/` contains runner code and self-tests.
- `schemas/` contains the supported committed data schemas.
- `profiles/` contains profile JSON files selected by `scripts/run_all_tests.sh --profile`
  and executed by `sifr_verify profiles run`.
  Profile v2 data owns `crate_test_membership`, the executable list of cargo
  crate suites per profile mode. The runner rejects unknown workspace packages,
  mismatched `cargo test -p` package names, duplicate suite ids, and red blockers
  without execution deadlines.
- `areas/` contains area-owned manifests, fixtures, baselines, and adapters.
  `coverage_matrix` owns the shipped guarantee registry and compiler surface
  matrix. It also owns `data/cargo_metadata_classification.json`, which maps
  every Cargo workspace package, target, and feature to its verification
  assignment. The `coverage_matrix:readiness` suite is selected by all four
  profiles and runs strict readiness mode plus profile-assignment checks. It
  rejects temporary rows (`expected-missing`, `tests:none`, `red-blocker`),
  unknown or unassigned owners, missing profile membership, non-offline
  create-pr/merge policy, v1 stable-surface manifests, and unpinned required
  corpora.
  `diagnostics` is migrated and can be run with
  `uv run --project verification python -m sifr_verify areas run --area diagnostics`.
- `policy/` contains machine-facing runner policy such as guardrail mappings.

## Profile Ownership

- `create-pr` is a fast representative profile. It selects readiness coverage,
  diagnostics rules, runtime/platform support evidence, algorithmic manifest
  checks, static/LSP smoke tooling, generated-code smoke, performance smoke, and
  stdlib module merge checks.
- `merge` is the authoritative local gate. It selects the readiness coverage
  suite, full first-party compiler crate membership, full semantic e2e pass
  corpus, diagnostics baselines, representative generated-code/performance
  suites, CPython hand-seeded differential checks, package offline smoke,
  stdlib module merge checks, runtime/platform evidence, regression, fuzz smoke,
  and curated ecosystem checks.
- `nightly` and `release` run the same readiness coverage suite plus broader
  generated-code quality, performance, distribution, CPython differential,
  sanitizer-full, ecosystem-broader, and module-full stdlib parity suites.
  Both profiles run the complete pinned algorithm corpus and taxonomy self-test.
  Release and nightly both retain unmodified full generated-code Clippy
  coverage.

Crate test membership is data-owned by `crate_test_membership.suites` in each
profile. The coverage matrix cross-checks that first-party compiler crates with
tests are in merge membership and executed; temporary red blockers are illegal
at readiness.

## Baselines And Blessing

Verify diagnostics baselines with:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines
```

Bless only intentional baseline changes with:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless
```

Baseline metadata, source hashes, stale/unused baseline detection, and recovery
surface coverage are enforced by `diagnostics:rules`.

## Fuzz, Sanitizer, And Release Evidence

Deterministic local fuzz/property evidence:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite fuzz-smoke
```

Runtime/platform sanitizer evidence:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite sanitizer-smoke
uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite sanitizer-full
```

Release evidence is emitted under `target/validation_lane_reports/` and the
area-specific `target/verification/areas/**` result files. A readiness archive
must record commit SHA, OS/toolchain, emitted profile plans, suite counts,
report signatures, and hashes of the validation report JSON files.

Schemas intentionally support only a small subset: object shape, required keys,
primitive scalar types, arrays of objects or strings, enums, booleans, integers,
and repo-relative path strings. Unsupported schema keywords are rejected.
