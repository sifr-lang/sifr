# Ad Hoc PR Gate Speed and Validation Lane Rebalancing

status: implemented

Implementation note: lane policy is now documented in
`internal_docs/validation_lane_policy.md`. Canonical lanes are `create-pr`,
`merge`, `nightly`, and `release`.

## Objective

Make the local gates used for PR creation fast, deterministic, and compiler-relevant without weakening Sifr's correctness guarantees.

This phase is complete only when `scripts/run_all_tests.sh --profile create-pr` is a reliable fast create-PR gate, the authoritative merge gate has a measured and justified composition, every long-running bucket emits per-case timing, and non-compiler release/editor/distribution checks are moved to change-aware, merge, nightly, or release lanes with explicit ownership.

## Initial Benchmark Evidence

Benchmarks were run locally on 2026-06-04 from `/Users/yaseralnajjar/work/sifr/codebase`.

| Lane or bucket | Result | Wall time | Notes |
|---|---:|---:|---|
| `scripts/run_all_tests.sh --profile create-pr` | passed | 367.18s reported, 375.36s wrapper | Cold e2e cache; exceeds warm target but within cold target. |
| `scripts/run_all_tests.sh --profile create-pr` normal rerun | passed | 309.43s reported | Warm e2e cache; still exceeds 5-minute warm target. |
| create-pr e2e, cold | passed | 57.34s in create-pr lane, 49.00s test body | `cache_hits=0/12`, largest group 19 fixtures, median 2. |
| create-pr e2e, warm direct | passed | 2.87s wall, 2.05s test body | `cache_hits=12/12`; e2e is not the warm-cache root cause. |
| `scripts/run_all_tests.sh --profile merge` timestamped run | failed before completion | 225.36s to failure | Failed in `LSP protocol stress: FAIL: LSP exited 1:` before merge-only later buckets. |
| PR e2e, cold direct | passed | 41.29s wall, 40.42s test body | `cache_hits=0/19`, largest group 43 fixtures, median 1. |
| PR e2e, warm direct | passed | 2.66s wall, 1.70s test body | `cache_hits=19/19`. |
| PR validation contract matrix direct | passed | 89.33s wall, 87.21s test body | 15 rows; Phase 23 graph/isolation was 33.25s. |
| PR verification hardening direct | passed | 53.82s wall | 31 variants across diagnostics, project, fixedbugs, crashes, OSS-curated. |
| PR performance budget subset direct | passed | 147.53s wall | Multi-minute silent bucket. |
| Generated-code quality grouped PR bucket | interrupted after >11m | incomplete | Corpus and panic-scan passed; run was still in remaining generated-code quality scripts. |

Top cold create-pr lane sections from timestamped log:

| Section | Duration |
|---|---:|
| Developer Tooling Checks | 88.87s |
| Diagnostic source canonicalization contract | 80.48s |
| E2E pass suite | 57.34s |
| Validation contract matrix | 52.74s |
| `cargo test -p sifr -- --skip test_e2e_pass` | 30.39s |
| Distribution validation | 12.57s |
| Performance Budget Checks | 8.34s |

## Root Causes

1. The primary authoritative merge-gate blocker is generated-code quality. The grouped merge-only bucket was interrupted after more than 11 minutes: `corpus` and `panic-scan` had passed, but remaining `rustfmt`, `clippy`, `determinism`, and `demos` scripts were still not complete. The scripts repeat the full manifest across separate modes and create transient generated Rust projects. Observed process state showed release-mode dependency compilation, including crates such as `regex-automata`, inside isolated generated workspaces. The bucket also lacks per-fixture timing, so a slow fixture is hard to identify.
2. `check_diagnostic_source_canonicalization_contract.py` is a focused diagnostic contract implemented as about 42 separate `cargo run -q -p sifr -- ...` CLI invocations. It should be an in-process Rust or Python harness over one built compiler binary or frontend API, not repeated Cargo-launched checks.
3. Developer tooling checks are broad and serial. They include formatter contract, VS Code packaging, editor asset checks, analysis snapshots, LSP protocol checks, LSP large-session smoke, and many self-tests. Several are editor/tooling release contracts rather than core compiler PR-creation gates.
4. `lsp_protocol_stress.py` is sensitive to instrumentation or broad-lane process conditions. It passed alone, in a smoke-to-stress sequence, and in the normal create-pr lane, but failed twice inside timestamped broad-lane runs with `LSP exited 1` and empty stderr after a long quiet interval. Blocking checks need deterministic, actionable failure evidence.
5. The validation contract matrix repeatedly exercises project-mode/full-mode integration rows. These rows are compiler-relevant, but the current create-pr and merge gates include more expensive cross-mode project-build coverage than a fast create-PR gate should carry.
6. PR performance budgets are meaningful but silent and multi-minute. They need per-case progress and a smaller smoke subset for PR creation, with the broader subset reserved for merge or performance-sensitive changes.
7. E2E pass is not the main warm-cache problem, but cold runs have high group skew. Large groups of 19 and 43 fixtures versus medians of 2 and 1 make cold rebuilds uneven.

## Compiler-Gate Policy

The create-PR gate should primarily prove compiler correctness for changed code:

- parse/syntax contracts
- frontend project loading and module graph invariants
- HIR lowering, type checking, ownership, CFG/flow, and diagnostics
- codegen smoke for representative emitted Rust
- representative e2e pass/fail behavior
- deterministic cache/key behavior for local edit loops
- fast structural guardrails such as file size, dependency direction, diagnostic schema/docs/code coverage, and split-brain checks

The create-PR gate should not always include:

- VS Code packaging
- multi-editor asset release checks
- full distribution/self-update validation
- full generated-code quality corpus, generated clippy, and all demos
- broad hardening suites such as OSS-curated, crashes, and full fixedbugs
- full performance budget subset
- full LSP stress

Those checks still matter. They belong in change-aware gates, the authoritative merge gate, nightly/release lanes, or explicit pre-merge commands for PRs touching their owned surface. For LSP specifically, the create-PR lane should keep a small protocol smoke test. The full stress test should run when `crates/sifr_lsp`, `crates/sifr_analysis`, frontend query scheduling/cancellation, LSP protocol scripts, or editor integration behavior changes, and in broader merge/nightly validation.

## Gate Placement Decisions

| Check family | Create-PR gate | Merge gate | Nightly/release | Change-aware trigger |
|---|---|---|---|---|
| Generated-code quality | Smoke subset only | Broader representative subset | Full corpus: corpus, panic-scan, rustfmt, clippy, determinism, demos | Any codegen, runtime dependency selection, generated project, or emitted Rust quality change |
| LSP protocol | Smoke only | Optional representative stress | Full stress | `crates/sifr_lsp`, `crates/sifr_analysis`, frontend query scheduling/cancellation, LSP protocol scripts, editor integration behavior |
| Developer tooling and editor assets | Static/smoke guardrails only | Representative tooling contracts | Full packaging/assets matrix | Formatter, linter, editor assets, VS Code packaging, analysis/editor-query behavior |
| Distribution/self-update | No | Representative release automation checks only if touched | Full distribution validation | Installer, self-update, release metadata, stable/preview channel scripts |
| Performance budgets | Minimal smoke budgets | Representative budget subset | Full budget corpus | Performance-sensitive compiler, frontend cache, formatter, LSP latency, build/run/check changes |
| Verification hardening | Small regression smoke | Selected diagnostics/project/fixedbugs | Full hardening breadth | Diagnostics, project graph, regression corpus, crashes, OSS-curated validation |

The first implementation priority is generated-code quality because it is the only measured bucket that exceeded 11 minutes and still did not complete as a grouped PR-only check. LSP stress is explicitly not an every-PR-create check; it is a smoke-every-time, stress-when-relevant family.

## Inspiration From Other Compilers

Rust's bootstrap model is path/profile oriented: contributors run targeted `x.py` commands, compiler/library/tooling profiles differ, tidy/tool checks are explicit, and compiletest suites are separated by mode. Rust also uses cached staged artifacts and build profiles instead of treating every local check as a full release qualification pass.

TypeScript's repo separates build, lint, baseline tests, watch-mode iteration, and parallel test execution through `hereby`. It supports targeted test regexes and baseline acceptance workflows rather than making every local iteration run every broad suite.

TypeScript-Go keeps regular `go test ./...` usable, relies on Go test caching, and layers `hereby` tasks for convenience, lint, generation, baseline tracking, and release-like checks. The useful lesson is that the native language test runner remains the fast default, while broad conformance and generated artifacts are layered.

## Milestones

### milestone_gate_speed_1: Timing And Lane Taxonomy

- [x] Add first-class timing output to `scripts/run_all_tests.sh` for every top-level bucket.
- [x] Add per-case timing to generated-code quality, performance budgets, contract matrix rows, hardening variants, and distribution scripts.
- [x] Define lane taxonomy: `create-pr`, `merge`, `nightly`, and `release`.
- [x] Acceptance: one run produces a machine-readable breakdown that identifies the slowest case in every bucket.

### milestone_gate_speed_2: Diagnostic Contract Harness

- [x] Replace repeated Cargo-launched diagnostic source canonicalization checks with an in-process harness or one built-binary invocation model.
- [x] Keep JSON/human/compact renderer coverage, source-span assertions, and package/project cases.
- [x] Acceptance: the same semantic assertions run in under 10s warm on the local benchmark machine.

### milestone_gate_speed_3: Tooling And LSP Gate Isolation

- [x] Split editor packaging/assets, formatter full contract, LSP protocol stress, and LSP large-session checks into change-aware tooling lanes.
- [x] Make `lsp_protocol_stress.py` emit stderr, process lifecycle, and last protocol event evidence on failure.
- [x] Acceptance: create-PR lane keeps compiler/tooling smoke only; full tooling lane remains available and deterministic.

### milestone_gate_speed_4: Generated-Code Quality Reuse

- [x] Generate the corpus once per run and reuse artifacts across panic scan, rustfmt, clippy, determinism, and demo checks.
- [x] Share or intentionally scope Cargo target directories so dependency builds are not repeated unnecessarily.
- [x] Introduce a small generated-code smoke subset for create-PR and keep full corpus for codegen-facing merge/nightly gates.
- [x] Acceptance: generated-code smoke is under 30s warm; full corpus reports per-fixture timings and does not silently rebuild duplicate release dependencies without justification.

### milestone_gate_speed_5: Compiler Contract Rebalancing

- [x] Reduce create-PR contract matrix to unique compiler invariants not already covered by unit/e2e tests.
- [x] Move broad project-mode matrices and hardening breadth to merge or nightly lanes unless touched paths require them.
- [x] Keep representative e2e pass warm-cache behavior and add group-skew rebalancing for cold runs.
- [x] Acceptance: create-PR lane stays under 120s warm and under 300s cold, while merge lane preserves documented coverage.

### milestone_gate_speed_6: Policy And Validation

- [x] Update `scripts/run_all_tests.sh`, `verification/validation_lanes/manifest.json`, and docs to make the lane policy explicit.
- [x] Document which paths trigger change-aware tooling, distribution, generated-code, performance, and hardening lanes.
- [x] Acceptance: local validation and CI use the same commands; no CI-only gate behavior is introduced.

## Required Validation

Before closing this ad hoc phase:

- `scripts/run_all_tests.sh --profile create-pr`
- create-PR lane command after implementation
- authoritative merge lane command after implementation
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- file-size guardrail

The final report must include warm and cold wall-clock times, per-bucket timings, and a before/after comparison against the initial benchmark table above.

## Implementation Measurements

Measured locally after implementation on 2026-06-05 from `/Users/yaseralnajjar/work/sifr/codebase`.

| Lane or bucket | Before | After | Status |
|---|---:|---:|---|
| create-PR lane cold (`scripts/run_all_tests.sh --profile create-pr`) | 367.18s reported, 375.36s wrapper | 206.74s reported | Under 300s cold target. |
| create-PR lane warm (`scripts/run_all_tests.sh --profile create-pr`) | 309.43s reported | 74.82s reported | Under 120s warm target. |
| merge lane warm/cold-local (`scripts/run_all_tests.sh --profile merge`) | failed before completion at 225.36s | 595.66s reported | Under 15-minute warm target; e2e cache was cold for this run. |
| Diagnostic source canonicalization contract | 80.48s | 3.18s | Under 10s warm target. |
| Generated-code smoke | PR grouped bucket interrupted after >11m | 18.11s in create-PR lane | Under 30s warm target. |
| create-pr e2e cold | 57.34s lane bucket, `cache_hits=0/12`, largest group 19, median 2 | 55.12s test body, `cache_hits=0/18`, largest group 8, median 2 | Group skew capped. |
| create-pr e2e warm | 2.87s direct | 1.66s test body, `cache_hits=18/18` | Warm cache preserved. |

Latest create-PR per-bucket timings from `target/validation_lane_reports/create-pr.latest.json`:

| Bucket | Wall time |
|---|---:|
| core guardrails | 5.91s |
| diagnostic contracts | 7.97s |
| frontend/syntax guardrails | 1.77s |
| developer tooling smoke | 9.29s |
| performance budget smoke | 7.06s |
| verification hardening self-tests | 0.49s |
| distribution validation | 0.23s skipped by lane policy |
| generated-code quality smoke | 18.11s |
| crate tests | 17.04s |
| validation contract matrix | 0.36s no-op by lane policy |
| e2e pass suite | 2.43s |
| verification hardening suites | 0.25s no-op by lane policy |
| extra e2e checks | 0.25s no-op by lane policy |

Latest merge-lane per-bucket timings from `target/validation_lane_reports/merge.latest.json`:

| Bucket | Wall time |
|---|---:|
| core guardrails | 5.82s |
| diagnostic contracts | 15.37s |
| frontend/syntax guardrails | 1.73s |
| developer tooling representative | 35.36s |
| performance budget representative | 138.51s |
| verification hardening self-tests | 0.51s |
| distribution representative | 28.27s |
| generated-code quality representative | 127.12s |
| crate tests full | 67.01s |
| validation contract matrix | 81.44s |
| e2e pass suite | 38.22s |
| verification hardening suites | 52.44s |
| extra e2e checks | 0.35s |

Merge e2e used a cold cache for this run: `cache_hits=0/22`,
`largest_group_fixtures=12`, `median_group_fixtures=1`. The lane passed with
the non-blocking advisory `group skew is high; investigate batching balance or
fixture clustering`.
