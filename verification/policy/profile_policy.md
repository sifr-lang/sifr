# Validation Profile Policy

Status: active

Sifr local validation uses four profiles:

- `create-pr`: fast local create-PR signal, target <=5m warm and <=15m cold,
  with blocking per-step budgets for regression control.
- `merge`: authoritative merge gate for compiler correctness.
- `nightly`: broad hardening, full generated-code quality, and full e2e pass corpus.
- `release`: highest-confidence release qualification profile.

Profile policy lives in `verification/profiles/{create-pr,merge,nightly,release}.json`.
`uv run --project verification --locked python -m sifr_verify profiles shell --profile <profile>`
is the shell-facing resolver for profile metadata while the bash facade remains
the public validation entrypoint.

Profiles at schema version 2 carry local-first execution policy:

- `network_policy.mode=offline` for create-pr and merge governs the validation
  workload after the cache-setup prelude. The profile runner executes the
  canonical `cargo fetch --locked` prelude before forcing offline Cargo
  execution. That prelude is the only registry-network opportunity; a failed
  fetch aborts the profile instead of allowing later steps to self-heal
  online. It is reported as the `cargo_cache_setup` lane step, while every
  subsequent step runs locked and offline. The execution sandbox's external
  network rule applies to validation commands, generated binaries, and fixture
  subprocesses after this prelude.
- `crate_test_membership.suites` is the source of truth for profile-owned crate
  tests. Each suite has a stable id, workspace package, exact `cargo test`
  command, profile modes, status, and merge-execution marker. The runner rejects
  duplicate suite ids, unknown packages, and commands whose `-p/--package`
  argument does not match the suite package.
- generated binaries and external programs run under the execution sandbox
  rules: tempdir-only writes, no external network, declared loopback-only
  networking, subprocess cleanup, and bounded captured output.
- `profile_plan.emit_command` is the local source of truth for CI parity checks.
- `uv run --project verification --locked python -m sifr_verify doctor`
  diagnoses local prerequisites. Cargo cache population is owned by the
  profile runner's reported setup prelude.

The `coverage_matrix` area is selected by create-pr, merge, nightly, and release
through the blocking `readiness` suite. The suite runs strict mode with
`SIFR_COVERAGE_MATRIX_STRICT=1`, the profile-assignment matrix check, and
negative self-tests. It rejects `expected-missing`, `tests:none`, `red-blocker`,
ownerless rows, unknown owners, expired quarantine, v1 stable-surface manifests,
unpinned required corpora, live-network create-pr/merge policy, missing locked
or offline Cargo policy, and first-party compiler crates without executed merge
membership.

Cargo workspace packages, targets, and features are inventoried from
`cargo metadata --locked --no-deps --format-version 1` and classified in
`verification/areas/coverage_matrix/data/cargo_metadata_classification.json`.
Every first-party compiler crate must have full-mode merge membership. Temporary
`red-blocker` membership is no longer an allowed readiness state. Targets and
features use `merge-red-blocker` only for historical classification data that is
not part of the current stable readiness surface.

## Create-PR Profile

`scripts/run_all_tests.sh --profile create-pr` proves fast compiler-relevant behavior:

- static guardrails and diagnostic registry/docs ruless
- strict readiness coverage-matrix consistency for shipped guarantees and surfaces
- parser/frontend cache and split-brain guardrails
- static tooling ruless and LSP protocol smoke
- smoke performance budgets
- generated-code quality smoke over a bounded fixture subset
- library crate unit tests, CLI unit tests, and representative e2e pass fixtures

It intentionally excludes editor packaging, editor asset release checks, distribution/self-update checks, LSP stress, LSP large-session smoke, broad verification hardening, broad project-mode matrices, full generated clippy/corpus, full performance budgets, and the slower generated-build crate integration tests.

Generated-build crate tests use Rust `#[ignore]` only to keep the default smoke
crate commands fast. They are not disabled tests: full profiles run the
`sifr_cli_generated_builds` and `sifr_driver_generated_builds` suites with
`--ignored --test-threads=1`. Do not use `#[ignore]` in the `sifr` binary or
`sifr_driver` lib suites for broken/quarantined tests; add a separate explicit
profile suite instead.

Create-pr also carries blocking per-step budgets in `step_budgets`. These
budgets are intentionally broader than the long-term wall-time target while the
fast lane is being restored, but they make each step's cost a contract instead
of a decorative observation. Local investigation may temporarily set
`SIFR_VERIFY_DISABLE_STEP_BUDGETS=1`; PR validation must not rely on that escape
hatch.

## Merge Profile

`scripts/run_all_tests.sh --profile merge` is the authoritative merge gate. It preserves broader compiler coverage through:

- full core rules matrices listed in the manifest
- strict readiness coverage-matrix consistency for shipped guarantees and surfaces
- representative hardening suites
- representative performance budget subset
- representative generated-code quality checks with shared generated artifacts and Cargo target reuse
- distribution representative checks
- formatter, analysis, static tooling, and LSP smoke checks
- full crate tests, including the slower generated-build crate integration suites

## Change-Aware Ownership

Use the broader profile or targeted family command when touching these surfaces:

- Generated-code quality: codegen, runtime dependency selection, generated project layout, or emitted Rust quality.
- LSP stress and large-session checks: `crates/sifr_lsp`, `crates/sifr_analysis`, frontend query scheduling/cancellation, LSP protocol scripts, or editor integration behavior.
- Developer tooling/editor release: formatter, linter, editor assets, VS Code packaging, analysis snapshots, or editor query behavior.
- Distribution/self-update: installer scripts, self-update metadata, release channel scripts, or stable/preview channel behavior.
- Performance budgets: compiler performance-sensitive paths, frontend cache, formatter, LSP latency, or build/run/check workflows.
- Verification hardening: diagnostics, project workspace graph, regression corpus, crash fixtures, and OSS-curated validation.

## Timing Evidence

`scripts/run_all_tests.sh` emits `[sifr-lane-step]` records for the Cargo cache
setup prelude and every top-level validation bucket. Slow sub-tools emit
`[sifr-case-timing]` records. `uv run --project verification --locked python -m
sifr_verify reports summarize` writes
`target/validation_lane_reports/<profile>.latest.json` with wall time, step
timings, slowest cases, e2e cache/group stats, generated artifact cache hits,
and advisories.
