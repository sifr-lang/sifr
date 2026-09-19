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

All maintained first-party Python projects require the canonical GIL-enabled
CPython 3.14.7 interpreter exactly. Install it with `uv python install 3.14.7`;
`uv run` selects it from each project's exact `requires-python` pin.

Required `uv` version: `0.12.10`. Every maintained uv project carries the exact
native `required-version` pin, and CI reads the same pin from this project's
`pyproject.toml`.

The public validation entrypoint remains:

```bash
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh --profile merge --emit-plan
```

`scripts/run_all_tests.sh` is a thin public facade over
`uv run --project verification --locked python -m sifr_verify profiles run`.
It fail-fasts when `uv` is missing or differs from the exact version so profile
execution stays reproducible for local and CI validation.

`--emit-plan` prints the selected profile's machine-readable execution plan
without running suites. CI may add broader profiles, but it must not omit suites
from the local merge plan except through declared host skips.
Compare local and CI plans with:

```bash
uv run --project verification --locked python -m sifr_verify profiles compare-plans --local <local-plan.json> --ci <ci-plan.json>
```

`sifr_verify doctor` checks required local prerequisites: exact Python version, Rust
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
  `github_actions.json` owns the exact release label and commit for each
  external action in maintained workflows. Use
  `python3 scripts/check_github_action_pins.py` to validate all references.
  The validator rejects mutable refs, unknown actions, label drift, and weak
  artifact-digest behavior.

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

## Focused fixture collection

The existing area manifests remain the fixture authorities. Shared baseline
adapters accept exact suite/case selections and failure reports:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --case baselines/parser_bad_indent --no-fail-fast
uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --rerun-failures target/verification/areas/diagnostics-results.json --no-fail-fast
uv run --project verification --locked python -m sifr_verify profiles run --profile merge --no-fail-fast
uv run --project verification --locked python -m sifr_verify areas inventory --output /tmp/inputs.json
uv run --project verification --locked python -m sifr_verify areas inventory --output /tmp/current-inputs.json --compare /tmp/inputs.json
```

Inventory comparison authorizes reuse only when inputs are unchanged; it never
creates a new test execution. Added fixtures and changed validation inputs change
the inventory digest. Phase records alone do not. An inventory is an input
manifest, not a replacement for the canonical area/profile result. Ownership
roots cover compiler sources, generators, fixtures, vendored inputs, scripts,
workflows and tested diagnostic documentation without a suffix allowlist.
Initialized submodules contribute both their declared/checked-out identity and
nonignored contents, including dirty files and additions. Outside source ownership roots, inventory derives file constants from the
guardrail policy's entrypoints and the documentation inventory's active consumers.
It also reads the active taxonomy check's declared roots (including the root README)
and uses the compatibility check's own scan roots and skip predicate for its
document/config sweep, and the documentation consumer's public-page selection.
Consequently tested prose is an input; unconsumed prose and phase records remain
excluded. New declared config inputs and newly scanned public pages invalidate
evidence without editing an inventory suffix list.
Files inside an input root remain inputs even when their suffix resembles prose:
generators and tests can consume Markdown and arbitrary future asset types.

The shared adapter prepares its compiler through Cargo once per process and
checks configured compiler overrides against the prepared artifact. Setup,
signals, output truncation and deadlines cannot satisfy a language-negative
baseline. Failed and blocked cases remain failures during collection and bless.
Raw diagnostics remain in the area report and mismatch artifacts; failures stream
as each case finishes. Specialized area runners retain their suite selectors.

Generated-code smoke, representative and full modes now run explicit release
link/runtime assertions for the two safe codegen demo companions in addition to
their existing Rust-check, snapshot, formatting and quality obligations. Their
declared stdout is independent of equivalence comparisons. Full E2E/release
coverage remains selected. Cargo freshness never skips a selected runtime
assertion. Native stage details are referenced from the area result.

The fixture comparison hook requires two executable providers and an independent
expected result. DX.4 exercises source/native controls only; metadata and
persistent providers are not qualified until their provider qualification suites connect and
test them. Diagnostic example checks execute explicitly selected standalone
check-fail/check-pass pairs and their explain/help surface; contextual package and
runtime examples are not indiscriminately executed.
