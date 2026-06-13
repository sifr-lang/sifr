# Ad Hoc Phase Execution Checklist (Test Strategy and Validation Lane Redesign)

Status: complete (all implementation milestones and external review passes completed 2026-03-16)
Owner: ad_hoc_test_strategy_validation_lane_redesign execution loop
Reference planning doc:
- `issues/ad-hoc-test-strategy-and-validation-lane-redesign.md`

Loop per part: Plan -> Implement -> Validate -> Demo -> PR -> Review -> Merge -> Update docs -> Next part

## Global Gates
- [x] Scope remains constrained to the active milestone part
- [x] Root cause addressed without fallback or compatibility shims
- [x] Coverage is preserved or strengthened when expensive checks move downward
- [x] Positive-path and negative-path validation recorded for the active milestone
- [x] Milestone demo runs successfully before the PR is opened
- [x] Local validation is run before the PR is opened
- [x] PR is opened, reviewed, and merged before the next milestone starts
- [x] Docs/checklists/PR links are updated before moving on

## Full Phase To-Do Plan
1. [x] `milestone_test_1`: redesign lane taxonomy and policy so `quick`, `pr`, `nightly`, and `release` are explicit and `quick` stops running broad hardening / nested determinism work
2. [x] `milestone_test_2`: replace shell-matrix repetition with one declarative validation harness and thin wrappers
3. [x] `milestone_test_3`: downshift eligible invariants from expensive CLI/e2e paths into cheaper integration or unit coverage
4. [x] `milestone_test_4`: redesign generated-program artifact reuse and cache boundaries for repeated `run` / `test` validation
5. [x] `milestone_test_5`: refactor hardening and determinism into non-default lanes while preserving breadth
6. [x] `milestone_test_6`: add throughput/resource reporting, worker guidance, and regression visibility
7. [x] external review pass 1 completed and acted on
8. [x] production-grade review pass completed and acted on
9. [x] extra review pass 3 completed and recorded

## Entry Baseline Evidence (2026-03-16)

Baseline command:
- `/usr/bin/time -l $(pwd)/scripts/run_all_tests.sh --profile quick`

Observed progress before phase edits:
- HIR maintainability guardrails: pass
- `sifr_driver` maintainability guardrails: pass
- `cargo test -p sifr -- --skip test_e2e_pass`: pass (`35` CLI/unit tests and `23` non-pass e2e/support tests)
- `scripts/run_frontend_mode_parity_matrix.sh`: pass
- `scripts/run_phase23_graph_isolation_matrix.sh`: pass
- `scripts/run_phase24_hir_analysis_consolidation_matrix.sh`: pass
- `scripts/run_phase25_cfg_flow_activation_matrix.sh`: pass
- current `quick` then enters the broad `test_e2e_pass` run followed by phase-29 hardening, which is the default-lane overreach this phase is intended to remove
- after milestone 1, warm `quick` was remeasured at `36.86s` wall time with `0` swaps and a representative `24`-fixture pass corpus instead of the full pass suite plus hardening

Known architectural entry facts from the planning doc:
- current `quick` nests e2e and verification-hardening work behind one default command
- current shell matrix layer repeats multiple `cargo run -q -p sifr -- ...` invocations for contract-level invariants
- current generated-program caching does not let repeated `run` / `test` validation reuse stable workspaces
- current quick cache and group sizing can drive pathological memory and swap behavior

## Milestone Progress

### milestone_test_1: Lane Taxonomy and Policy Redesign
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1170
- Implementation target:
  - define checked-in lane metadata for `quick`, `pr`, `nightly`, and `release`
  - make `quick` a true developer lane with representative e2e instead of broad hardening
  - assign current validation steps to explicit lane ownership and worker policy
- Demo target:
  - `python3 scripts/validation_lane.py summary --profile quick`
- Validation target:
  - `scripts/run_all_tests.sh --profile quick`
  - negative-path profile validation command
- Validation evidence:
  - positive path: `python3 scripts/validation_lane.py summary --profile quick` -> lane summary reports `quick` as a bounded developer lane with `frontend_mode_parity`, `phase23_graph_isolation`, and a `24`-fixture representative e2e manifest
  - positive path: `python3 scripts/validation_lane.py summary --profile full` -> legacy `full` canonicalizes to `pr` and reports the merge-gate lane with full matrix coverage, a `64`-fixture representative e2e manifest, and selected hardening suites
  - positive path: `cargo test -p sifr --test e2e fixture_selection -- --nocapture` -> passed (`1 passed, 0 failed`), proving the new manifest parser rejects empty fixture selections while preserving deterministic set semantics
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> passed with no hardening suites invoked and `24` pass fixtures completed successfully
  - positive path: `/usr/bin/time -l $(pwd)/scripts/run_all_tests.sh --profile quick` -> passed in `36.86s` warm wall time with `0` swaps, `6` e2e cache hits, and `24` representative pass fixtures
  - negative path: `bash scripts/run_all_tests.sh --profile invalid` -> exits `2` with the unsupported-profile contract preserved
  - merge evidence: PR `#1170` merged into `main` as `d604b6ba24966d4c83a37b50e01e131c30c3b743` on `2026-03-16`
  - closure basis: lane semantics are now checked in under `verification/validation_lanes/manifest.json`, `quick` no longer executes broad hardening by default, and representative e2e selection is enforced by the Rust harness through fixture manifests rather than shell-only convention

### milestone_test_2: Declarative Validation Harness
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1172
- Implementation target:
  - replace shell-matrix orchestration with one manifest-driven harness
  - keep top-level scripts as thin wrappers only
- Demo target:
  - `bash scripts/run_validation_contract_matrix.sh --suite frontend_mode_parity --suite phase23_graph_isolation`
- Validation target:
  - `bash scripts/run_validation_contract_matrix.sh --suite frontend_mode_parity --suite phase23_graph_isolation --suite phase24_hir_analysis --suite phase25_cfg_flow`
  - `bash scripts/run_frontend_mode_parity_matrix.sh`
  - `$(pwd)/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `bash scripts/run_validation_contract_matrix.sh --suite frontend_mode_parity --suite phase23_graph_isolation` -> passed, with one shared timing report across `7` contract rows
  - positive path: `bash scripts/run_validation_contract_matrix.sh --suite frontend_mode_parity --suite phase23_graph_isolation --suite phase24_hir_analysis --suite phase25_cfg_flow` -> passed, with one shared timing report across all `19` contract rows
  - positive path: `bash scripts/run_frontend_mode_parity_matrix.sh` -> passed through the compatibility wrapper, proving the legacy command name now delegates to the declarative harness
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> passed with the quick lane invoking `scripts/run_validation_contract_matrix.sh` instead of the legacy shell loops
  - negative path: the initial harness revision surfaced a real workspace-boundary regression when `<TMP>` paths lived under the repo `target/` tree; the harness was corrected to allocate temp roots under the system temp directory so generated `build` outputs stay outside the workspace, matching the old shell behavior
  - merge evidence: PR `#1172` merged into `main` as `fee3e67110adf9668a261dc931ccab097c26dbc6` on `2026-03-16`
  - closure basis: the contract matrix now lives in `verification/validation_contracts/manifest.json`, `tests/validation_contracts.rs` is the single Rust-native execution harness, `scripts/run_validation_contract_matrix.sh` is the one harness entrypoint, and the old matrix scripts have been reduced to thin suite-selecting wrappers

### milestone_test_3: Invariant Downshifting
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1175
- Implementation target:
  - move diagnostic/lowering/codegen invariants into cheaper integration or unit coverage where possible
  - keep traceability from removed expensive checks to the new cheaper proof
- Demo target:
  - `cargo test -p sifr emit_entrypoint_downshifts_phase -- --nocapture`
- Validation target:
  - `cargo test -p sifr emit_entrypoint_downshifts_phase -- --nocapture`
  - `bash scripts/run_validation_contract_matrix.sh --suite frontend_mode_parity --suite phase23_graph_isolation --suite phase24_hir_analysis --suite phase25_cfg_flow`
  - `$(pwd)/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -p sifr emit_entrypoint_downshifts_phase -- --nocapture` -> passed (`2 passed, 0 failed`), proving the phase 24/25 positive analysis demos through `emit_entrypoint`-level Rust shape checks instead of CLI `run` execution
  - positive path: `bash scripts/run_validation_contract_matrix.sh --suite frontend_mode_parity --suite phase23_graph_isolation --suite phase24_hir_analysis --suite phase25_cfg_flow` -> passed after removing the downshifted positive analysis rows, reducing the full contract harness from `19` rows / `66302ms` to `14` rows / `42173ms`
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> passed with the new lower-layer phase 24/25 checks folded into `cargo test -p sifr -- --skip test_e2e_pass`
  - negative path: the removed contract rows were not deleted blindly; each moved invariant is now pinned in `crates/sifr/src/main.rs` against emitted Rust shape, and the remaining phase 24/25 negative contract rows continue to enforce diagnostic parity in the declarative harness
  - merge evidence: PR `#1175` merged into `main` as `208192d3497e84d689dfb3c7f3548469ea192caf` on `2026-03-16`
  - closure basis: phase 24/25 positive analysis invariants no longer depend on expensive CLI `run` execution, while the contract harness retains only the rows that still need true CLI-mode parity coverage

### milestone_test_4: Artifact Reuse and Cache Boundary Redesign
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1178
- Implementation target:
  - introduce reusable generated-program workspaces and repeat-run cache visibility
  - materially improve unchanged reruns for `run` and `test`
- Demo target:
  - `cargo run -q -p sifr -- run demos/m22_4_parity_regression_matrix_demo/main.sifr`
  - `cargo run -q -p sifr -- test demos/m22_4_parity_regression_matrix_demo`
- Validation target:
  - `cargo test -p sifr_driver test_cached_project_binary -- --nocapture`
  - `cargo test -p sifr_driver test_run_tests_reuses_cached_workspace_for_unchanged_project -- --nocapture`
  - `cargo test -p sifr_driver test_run_tests_invalidates_cached_workspace_when_sources_change -- --nocapture`
  - `cargo run -q -p sifr -- run demos/m22_4_parity_regression_matrix_demo/main.sifr` twice
  - `cargo run -q -p sifr -- test demos/m22_4_parity_regression_matrix_demo` twice
  - `$(pwd)/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `cargo test -p sifr_driver test_cached_project_binary -- --nocapture` -> passed (`2 passed, 0 failed`), proving repeated cached `run` builds reuse the same binary path for unchanged project inputs and emit a new key/path when the source graph changes
  - positive path: `cargo test -p sifr_driver test_run_tests_reuses_cached_workspace_for_unchanged_project -- --nocapture` -> passed, proving repeated `sifr test` invocations on unchanged inputs reuse the same generated Cargo workspace
  - positive path: `cargo test -p sifr_driver test_run_tests_invalidates_cached_workspace_when_sources_change -- --nocapture` -> passed, proving `sifr test` invalidates to a new cache key/workspace once reachable sources change
  - positive path: `cargo run -q -p sifr -- run demos/m22_4_parity_regression_matrix_demo/main.sifr` -> first invocation emitted `[sifr-artifact-cache] ... cache_hit=false ... miss_reason=not_found`, second invocation emitted the same cache key with `cache_hit=true`
  - positive path: `cargo run -q -p sifr -- test demos/m22_4_parity_regression_matrix_demo` -> first invocation emitted `[sifr-artifact-cache] ... cache_hit=false ... miss_reason=not_found`, second invocation emitted the same cache key with `cache_hit=true`
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> passed, with the authoritative quick lane preserving correctness while the `run`/`test` paths now surface explicit cache-hit accounting
  - negative path: cache invalidation is now explicit instead of implicit temp-dir churn; generated artifacts rebuild only when the rooted scope, generated Rust/Cargo inputs, or toolchain/env signature changes, and the tests above pin the changed-source invalidation contract for both `run` and `test`
  - merge evidence: PR `#1178` merged into `main` as `b2c5b48fc53b593c1668e87e3c73b0fa835b3e0d` on `2026-03-16`
  - closure basis: `sifr run` and `sifr test` now materialize through content-addressed caches under the system temp root, promote cache misses atomically from staging directories, and emit explicit cache hit/miss status lines so repeated local validation no longer always pays the generated-program rebuild cost

### milestone_test_5: Hardening Lane Refactor
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1180
- Implementation target:
  - preserve broad hardening and determinism coverage while removing it from the default local loop
  - make lane placement explicit rather than script-local
- Demo target:
  - `bash scripts/run_smoke_fuzz_property.sh`
- Validation target:
  - `bash scripts/check_e2e_report_determinism.sh --profile quick`
  - `bash scripts/check_e2e_sequential_parallel_equivalence.sh --profile quick`
  - `python3 scripts/run_verification_hardening.py --profile pr --suite diagnostics --suite project`
  - `bash scripts/run_smoke_fuzz_property.sh`
  - `$(pwd)/scripts/run_all_tests.sh --profile quick`
- Validation evidence:
  - positive path: `python3 scripts/run_verification_hardening.py --profile pr --suite diagnostics --suite project` -> passed (`verification ok: variants=7, failures=0, blocking_failures=0, non_blocking_failures=0`), proving the selected PR hardening subset still runs cleanly after the lane-boundary refactor
  - positive path: `bash scripts/run_smoke_fuzz_property.sh` -> passed, and the wrapper now drives the property/fuzz-smoke suites through `run_verification_hardening.py --profile nightly` instead of `quick`
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> passed, with no hardening or determinism checks reintroduced into the default local loop
  - negative path: `bash scripts/check_e2e_report_determinism.sh --profile quick` -> exits `2` with `determinism checks are not part of the quick lane`
  - negative path: `bash scripts/check_e2e_sequential_parallel_equivalence.sh --profile quick` -> exits `2` with `sequential-vs-parallel equivalence is not part of the quick lane`
  - hardening-compatibility fix: milestone 4 cache-status lines initially broke the hardening `project` baseline suite; `scripts/run_verification_hardening.py` now normalizes `[sifr-artifact-cache] ...` lines out of baseline-checked outputs so cache accounting remains visible in normal CLI logs without narrowing hardening coverage
  - merge evidence: PR `#1180` merged into `main` as `8d6621d7031e2858d172af512ad7b8af6a9b1ef9` on `2026-03-16`
  - closure basis: determinism and broad hardening no longer have `quick` as a hidden execution profile, determinism-scale now inherits the selected non-default lane profile instead of hardcoding the quick representative subset, and the smoke-hardening wrapper is aligned to `nightly`

### milestone_test_6: Throughput and Resource Governance
- Status: complete
- Implementation PR: https://github.com/sifr-lang/sifr/pull/1183
- Implementation target:
  - report lane timing, cache behavior, fixture/group skew, and memory-related guidance
  - make performance and resource regressions visible from local runs
- Demo target:
  - `$(pwd)/scripts/run_all_tests.sh --profile quick`
- Validation target:
  - `python3 -m py_compile scripts/validation_lane_report.py`
  - `cargo fmt --check`
  - `$(pwd)/scripts/run_all_tests.sh --profile quick` (cold run)
  - `$(pwd)/scripts/run_all_tests.sh --profile quick` (warm rerun)
- Validation evidence:
  - positive path: `python3 -m py_compile scripts/validation_lane_report.py` -> passed, proving the new lane-report helper is syntactically valid
  - positive path: `cargo fmt --check` -> passed after the follow-up formatting-only commit, keeping the new e2e summary output aligned with workspace formatting rules
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> cold run passed in `55.32s` wall time / `48.67s` CPU time with `0` swaps, `377.0MiB` peak RSS, `cache_hits=0/6`, `rebuilt_groups=6`, and `cache_footprint=e2e=250.4MiB/1512files`, proving the new report surfaces first-run cache cost and resource footprint from the default lane
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> immediate warm rerun passed in `32.43s` wall time / `22.03s` CPU time with `0` swaps, `104.9MiB` peak RSS, `cache_hits=6/6`, `build=1ms`, and the same cache footprint, proving the representative lane now makes warm-cache wins and steady-state resource behavior visible in the default local report
  - demo evidence: the same warm quick rerun emitted `target/validation_lane_reports/quick.latest.{json,log,time}`, and the JSON report captures wall/CPU time, cache hit rate, rebuilt groups, group skew ratio, worker defaults, and advisory signals for downstream regression inspection
  - negative path: the first implementation pass exposed two root-cause reporting bugs rather than papering over them: macOS `/usr/bin/time -l` emits `real/user/sys` on one combined line, and the e2e cache dir was being reported as a relative path while the test harness resolved it from the crate working directory; milestone 6 fixes both by parsing BSD `time -l` output directly and resolving the e2e cache root to an absolute path inside `run_e2e_pass.sh`
  - merge evidence: PR `#1183` merged into `main` as `0e3ad38da2710afe7684478a1d1e3a4e7fd70fc6` on `2026-03-16`
  - closure basis: throughput/resource visibility is now part of the default lane wrapper itself, cache growth and group skew are surfaced in both terminal output and JSON artifacts, and the quick lane now exposes worker defaults plus swap/RSS signals without reintroducing heavy validation families

## External Review Passes

### review_pass_1
- Reviewer artifact: `reviews/ad-hoc-test-strategy-and-validation-lane-redesign-review-pass-1a.md`
- Status: complete
- Review summary:
  - Reviewer raised five concerns: `pr`/`nightly` lane placement for determinism-equivalence checks, quick-lane matrix breadth, contract-manifest demo-path fragility, and report-signature extraction robustness.
  - Validation result: no code changes were required. Determinism/equivalence remaining outside `quick` and below `release` is an explicit phase-design choice, quick intentionally omits phase 24/25 matrix rows because milestone 3 downshifted those invariants into cheaper `cargo test -p sifr` coverage, contract manifests already fail on missing fixture/demo paths with direct path context, and both signature-check scripts already hard-fail on missing signatures while matching hex-only report IDs.
- Validation evidence:
  - planning-doc validation: `issues/ad-hoc-test-strategy-and-validation-lane-redesign.md` assigns repeated-run determinism and sequential-vs-parallel equivalence to Layer 4 hardening/determinism and explicitly says those checks should default to `nightly`/`release` or explicit invocation, so reviewer findings 1 and 3 were rejected as design disagreements rather than regressions
  - milestone-3 validation: `issues/ad-hoc-test-strategy-and-validation-lane-redesign-execution.md` records that phase 24/25 positive invariants were intentionally downshifted into cheaper `emit_entrypoint`/`cargo test -p sifr` coverage, so reviewer finding 2 was rejected as stale against the merged milestone-3 architecture
  - script validation: `scripts/check_e2e_report_determinism.sh` and `scripts/check_e2e_sequential_parallel_equivalence.sh` already exit with an explicit error when `signature` is empty, and the extraction regex only matches `[0-9a-f]+`, so reviewer finding 5 did not expose a real bug in the merged scripts
  - reviewer-artifact preservation: the external review output was recorded verbatim under `reviews/ad-hoc-test-strategy-and-validation-lane-redesign-review-pass-1a.md` for traceability even though no code delta followed from this pass
- Follow-up PR:
  - PR `#1185` (`https://github.com/sifr-lang/sifr/pull/1185`) records the reviewer artifact and the no-code-change disposition for this pass

### review_pass_2
- Reviewer artifact: `reviews/ad-hoc-test-strategy-and-validation-lane-redesign-production-grade-review-pass-2a.md`
- Status: complete
- Review summary:
  - Reviewer raised five production-grade concerns: lane-profile defaults in determinism/equivalence scripts, missing fixture-manifest existence checks, fuzz-smoke temp-file accumulation, lane-report temp-file accumulation, and missing contract-script existence checks.
  - Validation result: findings 2, 3, and 4 were accepted and fixed; finding 1 was rejected because `--profile` parsing overrides the script default and repeated-run determinism intentionally exercises the cached representative lane, while finding 5 was rejected as a low-signal shell-launch failure mode that already produces a direct missing-script error.
- Validation evidence:
  - positive path: `python3 scripts/validation_lane.py summary --profile quick` -> passed, proving valid fixture manifests still resolve and summarize correctly after the new early validation helper
  - negative path: `python3 scripts/validation_lane.py --manifest <temp-manifest-with-missing-fixture> summary --profile quick` -> now fails immediately with `fixture manifest not found: .../verification/validation_lanes/does-not-exist.json`, replacing the old delayed cargo-side failure
  - positive path: `python3 scripts/run_verification_hardening.py --profile nightly --suite fuzz-smoke` -> passed (`verification ok: variants=33, failures=0, blocking_failures=0, non_blocking_failures=0`), proving the fuzz-smoke temp-file cleanup does not break the hardening runner
  - positive path: `find target/verification/tmp -maxdepth 1 -name 'fuzz_smoke_*.sifr' | wc -l` -> `0` after the passing fuzz-smoke run, proving successful generated fuzz snippets are now pruned instead of accumulating indefinitely
  - positive path: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> passed in `36.06s` wall time with the expected warm-cache e2e report after the temp-capture cleanup change
  - positive path: `find target/validation_lane_reports -type f -newer /tmp/validation-lane-marker.G1cX7v | sort` -> only `quick.latest.{json,log,time}`, proving `run_all_tests.sh` no longer leaves behind per-run `lane.<profile>.*` capture files once the latest artifacts are written
  - rejected finding validation: `scripts/check_e2e_report_determinism.sh` and `scripts/check_e2e_sequential_parallel_equivalence.sh` only default `PROFILE` before argument parsing; the passed `--profile` value still wins, so the reported “always release” bug was not reproducible
- Follow-up PR:
  - PR `#1186` (`https://github.com/sifr-lang/sifr/pull/1186`) carries the accepted production-grade review fixes and records the reviewer artifact/disposition

### review_pass_3
- Reviewer artifact: `reviews/ad-hoc-test-strategy-and-validation-lane-redesign-review-pass-3a.md`
- Status: complete
- Review summary:
  - Reviewer reported no critical bugs and assessed the implementation as strong overall, with only minor concerns around default-profile ergonomics for determinism scripts, future cache-pruning enhancements, cache invalidation coverage visibility, configurable RSS thresholds, and optional contract-manifest path validation.
  - Validation result: no additional code changes were required from this pass. The reviewer explicitly marked the current implementation correct in the five requested assessment areas and framed the remaining items as future enhancements rather than blocking production defects.
- Validation evidence:
  - reviewer-artifact preservation: the external review output is recorded verbatim under `reviews/ad-hoc-test-strategy-and-validation-lane-redesign-review-pass-3a.md`
  - disposition validation: the review’s only actionable-looking cache/temp-file concerns were already addressed in review pass 2 (`#1186`), and the remaining comments were enhancement suggestions rather than regressions in the merged phase work
  - closure basis: after three external passes, no unresolved critical or high-severity defects remain against the completed phase scope
- Follow-up PR:
  - PR `#1188` (`https://github.com/sifr-lang/sifr/pull/1188`) records the reviewer artifact and closes the phase status after the additional external review pass
