# Validation Lane Policy

Status: active

Sifr local validation uses four lanes:

- `create-pr`: fast local create-PR signal, target <=120s warm and <=300s cold.
- `merge`: authoritative merge gate for compiler correctness.
- `nightly`: broad hardening, full generated-code quality, and full e2e pass corpus.
- `release`: highest-confidence release qualification lane.

The lane manifest is `verification/validation_lanes/manifest.json`. `scripts/validation_lane.py` is the shell-facing resolver for lane metadata.

## Create-PR Lane

`scripts/run_all_tests.sh --profile create-pr` proves fast compiler-relevant behavior:

- static guardrails and diagnostic registry/docs contracts
- parser/frontend cache and split-brain guardrails
- static tooling contracts and LSP protocol smoke
- smoke performance budgets
- generated-code quality smoke over a bounded fixture subset
- library crate unit tests, CLI unit tests, and representative e2e pass fixtures

It intentionally excludes editor packaging, editor asset release checks, distribution/self-update checks, LSP stress, LSP large-session smoke, broad verification hardening, broad project-mode matrices, full generated clippy/corpus, full performance budgets, and the slower `sifr` integration/e2e-support crate tests.

## Merge Lane

`scripts/run_all_tests.sh --profile merge` is the authoritative merge gate. It preserves broader compiler coverage through:

- full core contract matrices listed in the manifest
- representative hardening suites
- representative performance budget subset
- representative generated-code quality checks with shared generated artifacts and Cargo target reuse
- distribution representative checks
- formatter, analysis, static tooling, and LSP smoke checks
- full crate tests, including the slower `sifr` integration/e2e-support tests

## Change-Aware Ownership

Use the broader lane or targeted family command when touching these surfaces:

- Generated-code quality: codegen, runtime dependency selection, generated project layout, or emitted Rust quality.
- LSP stress and large-session checks: `crates/sifr_lsp`, `crates/sifr_analysis`, frontend query scheduling/cancellation, LSP protocol scripts, or editor integration behavior.
- Developer tooling/editor release: formatter, linter, editor assets, VS Code packaging, analysis snapshots, or editor query behavior.
- Distribution/self-update: installer scripts, self-update metadata, release channel scripts, or stable/preview channel behavior.
- Performance budgets: compiler performance-sensitive paths, frontend cache, formatter, LSP latency, or build/run/check workflows.
- Verification hardening: diagnostics, project graph, regression corpus, crash fixtures, and OSS-curated validation.

## Timing Evidence

`scripts/run_all_tests.sh` emits `[sifr-lane-step]` records for every top-level bucket. Slow sub-tools emit `[sifr-case-timing]` records. `scripts/validation_lane_report.py` writes `target/validation_lane_reports/<profile>.latest.json` with wall time, step timings, slowest cases, e2e cache/group stats, generated artifact cache hits, and advisories.
