# Validation Profile Policy

Status: active

Sifr local validation uses four profiles:

- `create-pr`: fast local create-PR signal, target <=120s warm and <=300s cold.
- `merge`: authoritative merge gate for compiler correctness.
- `nightly`: broad hardening, full generated-code quality, and full e2e pass corpus.
- `release`: highest-confidence release qualification profile.

Profile policy lives in `verification/profiles/{create-pr,merge,nightly,release}.json`.
`uv run --project verification --locked python -m sifr_verify profiles shell --profile <profile>`
is the shell-facing resolver for profile metadata while the bash facade remains
the public validation entrypoint.

Profiles at schema version 2 carry local-first execution policy:

- `network_policy.mode=offline` for create-pr and merge.
- Cargo profile execution is locked and offline; `cargo fetch --locked` is setup,
  not part of profile execution.
- generated binaries and external programs run under the execution sandbox
  contract: tempdir-only writes, no external network, declared loopback-only
  networking, subprocess cleanup, and bounded captured output.
- `profile_plan.emit_command` is the local source of truth for CI parity checks.
- `uv run --project verification --locked python -m sifr_verify doctor` is the
  setup boundary for local prerequisites before profile execution.

The `coverage_matrix` area is selected by create-pr and merge in advisory mode
during the gate-closure phase. It fails schema, owner, status, profile-policy,
and expiry errors immediately while permitting the closed Wave 0 list of
temporary `expected-missing` and `red-blocker` rows. Closeout promotes the same
check to strict mode with `SIFR_COVERAGE_MATRIX_STRICT=1`.

## Create-PR Profile

`scripts/run_all_tests.sh --profile create-pr` proves fast compiler-relevant behavior:

- static guardrails and diagnostic registry/docs contracts
- advisory coverage-matrix consistency for shipped guarantees and surfaces
- parser/frontend cache and split-brain guardrails
- static tooling contracts and LSP protocol smoke
- smoke performance budgets
- generated-code quality smoke over a bounded fixture subset
- library crate unit tests, CLI unit tests, and representative e2e pass fixtures

It intentionally excludes editor packaging, editor asset release checks, distribution/self-update checks, LSP stress, LSP large-session smoke, broad verification hardening, broad project-mode matrices, full generated clippy/corpus, full performance budgets, and the slower `sifr` integration/e2e-support crate tests.

## Merge Profile

`scripts/run_all_tests.sh --profile merge` is the authoritative merge gate. It preserves broader compiler coverage through:

- full core contract matrices listed in the manifest
- advisory coverage-matrix consistency for shipped guarantees and surfaces
- representative hardening suites
- representative performance budget subset
- representative generated-code quality checks with shared generated artifacts and Cargo target reuse
- distribution representative checks
- formatter, analysis, static tooling, and LSP smoke checks
- full crate tests, including the slower `sifr` integration/e2e-support tests

## Change-Aware Ownership

Use the broader profile or targeted family command when touching these surfaces:

- Generated-code quality: codegen, runtime dependency selection, generated project layout, or emitted Rust quality.
- LSP stress and large-session checks: `crates/sifr_lsp`, `crates/sifr_analysis`, frontend query scheduling/cancellation, LSP protocol scripts, or editor integration behavior.
- Developer tooling/editor release: formatter, linter, editor assets, VS Code packaging, analysis snapshots, or editor query behavior.
- Distribution/self-update: installer scripts, self-update metadata, release channel scripts, or stable/preview channel behavior.
- Performance budgets: compiler performance-sensitive paths, frontend cache, formatter, LSP latency, or build/run/check workflows.
- Verification hardening: diagnostics, project workspace graph, regression corpus, crash fixtures, and OSS-curated validation.

## Timing Evidence

`scripts/run_all_tests.sh` emits `[sifr-lane-step]` records for every top-level bucket. Slow sub-tools emit `[sifr-case-timing]` records. `uv run --project verification --locked python -m sifr_verify reports summarize` writes `target/validation_lane_reports/<profile>.latest.json` with wall time, step timings, slowest cases, e2e cache/group stats, generated artifact cache hits, and advisories.
