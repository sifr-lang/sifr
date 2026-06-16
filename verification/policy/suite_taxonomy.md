# Verification Suite Taxonomy

This document defines the canonical suite taxonomy for compiler verification hardening.

## Canonical Suite Kinds

| suite | purpose | blocking | owner | required artifacts |
| --- | --- | --- | --- | --- |
| `coverage_matrix` | Record shipped guarantees, compiler surfaces, owners, profile evidence, and closeout enforcement. | yes | compiler-verification | guarantee registry, surface matrix, profile-assignment matrix, owner registry, profile plan evidence, negative self-tests |
| `diagnostics` | Lock compiler diagnostics contract (code/message/severity/url/spans/renderer views/exit code). | yes | compiler/diagnostics | fixtures, baselines, machine-readable run summary |
| `project_workspace` | Lock multi-file/module project behavior and deterministic project-mode outcomes. | yes | compiler/frontend | fixtures, baselines, machine-readable run summary |
| `fixedbugs` | Permanent issue-linked regressions for resolved compiler bugs. | yes | compiler/hardening | regression index, issue/root-cause metadata, run summary |
| `crashes` | Visible sentinel corpus for unresolved crashes/invariant failures. | yes | compiler/hardening | sentinel index, issue linkage, promotion log |
| `property` | Invariant tests for deterministic/high-value compiler behaviors. | yes | compiler/hardening | property manifest, deterministic run summary |
| `fuzz-smoke` | Deterministic local fuzz smoke over curated corpora. | yes | compiler/hardening | seed corpus manifest, deterministic run summary |
| `oss-curated` | Small pinned curated real-world gate that blocks merges. | yes | compiler/verification | corpus manifest, per-project outcomes, run summary |
| `ecosystem-broader` | Larger non-blocking compatibility suite for signal and backlog generation. | no | compiler/verification | suite manifest, signal report |
| `algorithmic_compatibility` | LeetCode/algorithm corpus compatibility with representative merge subset and full taxonomy deltas. | yes for representative/profile checks | algorithmic/compatibility | profile manifest, taxonomy/result baselines, per-fixture result artifacts |
| `runtime_platform` | Host/platform executable evidence, support matrix, and sanitizer smoke/full lanes. | yes for host-supported merge suites | runtime/platform | support matrix, platform evidence manifest, structured skips, sanitizer result summary |
| `package_management` | Offline registry, lockfile determinism, and package graph behavior. | yes for offline merge smoke | compiler/package-management | offline registry fixture, lockfile digest manifest, result summary |
| `stdlib_parity` | Supported stdlib namespace/module parity and example inventory checks. | yes for module merge checks | stdlib/parity | module inventory, namespace demo/LeetCode reports, result summary |

## Fixture and Baseline Conventions

### Diagnostics
- Fixture root: `verification/areas/diagnostics/fixtures/diagnostics/<case_id>/main.sifr`.
- Required baseline files:
  - `baselines/check-human.stdout.txt`
  - `baselines/check-human.stderr.txt`
  - `baselines/check-human.exit-code.txt`
  - `baselines/check-json.stdout.txt`
  - `baselines/check-json.stderr.txt`
  - `baselines/check-json.exit-code.txt`
  - `baselines/check-compact.stdout.txt`
  - `baselines/check-compact.stderr.txt`
  - `baselines/check-compact.exit-code.txt`
- Canonical expected representation:
  - Human and compact renderer output baselines are text files.
  - JSON diagnostics are canonicalized and pretty-printed before baseline compare.
  - Exit code is baseline-checked separately.
- Diagnostic code catalog: `verification/areas/diagnostics/data/code_catalog.json`.
- Diagnostic baseline coverage: `verification/areas/diagnostics/data/code_baseline_coverage.json`.
- Baseline metadata: `verification/areas/diagnostics/data/baseline_metadata.json`.
- Recovery surface coverage: `verification/areas/diagnostics/data/recovery_surface_coverage.json`.
- Synthetic baselines are allowed only when `baseline_metadata.json` marks them
  with `"synthetic": true`; executable diagnostic baselines must be owned by the
  area manifest.

#### Diagnostic Recovery Surfaces

| surface id | compiler layer | required multi-error behavior |
| --- | --- | --- |
| `parser_recovery` | parser | one malformed source can emit more than one stable parser diagnostic without losing code/span identity |
| `hir_mixed_recovery` | HIR/type checking | mixed independent semantic errors are preserved in one run without collapse to a catch-all diagnostic |
| `repeated_type_recovery` | HIR/type checking | repeated similar type errors are bounded by recovery caps while preserving a summary diagnostic |

Every surface listed here must have at least one multi-error fixture in `recovery_surface_coverage.json`.

### Project Workspace
- Fixture root: `verification/areas/project_workspace/fixtures/project/<case_id>/`.
- Entrypoint: `main.sifr`.
- Required baseline files per configured command variant:
  - `<variant>.stdout.txt`
  - `<variant>.stderr.txt`
  - `<variant>.exit-code.txt`
- Project fixtures are multi-file and use import/module behavior representative of real project use.

### Fixedbugs
- Fixture root: fixed bug locks usually point at the canonical e2e/demo fixture that reproduces the resolved bug.
- Index source of truth: `verification/areas/regression/data/fixedbugs.json`.
- Required metadata:
  - issue or finding id
  - root-cause category
  - owning suite location
  - concise context note (when needed)

### Crashes
- Sentinel root: `verification/areas/regression/fixtures/crashes/<case_id>.sifr`.
- Index source of truth: `verification/areas/regression/data/crashes.json`.
- Required metadata:
  - issue id
  - crash or sentinel classification
  - current status (`unresolved` or `promoted`)
  - minimized `reproducer_fixture` path
  - promotion target once fixed

### Property and Fuzz-Smoke
- Policy and manifests live under `verification/areas/fuzz_property/`.
- Seeds and deterministic generation rules are version-controlled.
- Each run emits machine-readable case outcomes and seed provenance.
- Operational policy: `verification/policy/fuzz_property.md`.

### OSS-Curated and Ecosystem-Broader
- Manifests live under `verification/areas/ecosystem_compatibility/data/`.
- `oss-curated` entries must include pinned revision, source checksum, SPDX license, owner, rationale, commands, timeout, and expected class.
- `ecosystem-broader` entries use the same metadata contract, but remain non-blocking signal unless promoted by profile policy.
- `ecosystem-broader` is explicitly non-blocking and emits signal-only reports.
- Operational policy: `verification/policy/ecosystem_compatibility.md`.

## Manifest Source of Truth

- Shipped guarantee coverage lives in
  `verification/areas/coverage_matrix/shipped_guarantees.json`.
- Compiler-surface profile assignments live in
  `verification/areas/coverage_matrix/compiler_surface_matrix.json`.
- Decisions-table profile assignments live in
  `verification/areas/coverage_matrix/profile_assignment_matrix.json` and are
  checked against `verification/profiles/*.json` by
  `coverage_matrix:closeout`.
- Owner ids live in `verification/owners.json`; `unassigned` is invalid.
- Runner-owned hardening suite data lives under
  `verification/runner/sifr_verify/hardening/data/`.
- Stable-surface area manifests use schema version 2 and declare owner,
  `network_mode`, pinned-corpus policy, skip policy, and baseline metadata
  contract. Create-pr and merge stable-surface suites must be offline.
- Suite runners must not hardcode fixture lists outside their owning manifest.
- Manifest updates are review artifacts and follow normal PR review.
- Corpus lifecycle and promotion rules: `verification/policy/regression_corpus.md`.
- Deterministic sharding + flake policy: `verification/policy/deterministic_sharding_and_flake_policy.md`.
- Structured artifact schema + retention: `verification/policy/artifact_schema_and_retention.md`.
