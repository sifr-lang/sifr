# Ad Hoc Phase Execution Checklist (Test Strategy and Validation Lane Redesign)

Status: in progress (started 2026-03-16)
Owner: ad_hoc_test_strategy_validation_lane_redesign execution loop
Reference planning doc:
- `issues/ad-hoc-test-strategy-and-validation-lane-redesign.md`

Loop per part: Plan -> Implement -> Validate -> Demo -> PR -> Review -> Merge -> Update docs -> Next part

## Global Gates
- [ ] Scope remains constrained to the active milestone part
- [ ] Root cause addressed without fallback or compatibility shims
- [ ] Coverage is preserved or strengthened when expensive checks move downward
- [ ] Positive-path and negative-path validation recorded for the active milestone
- [ ] Milestone demo runs successfully before the PR is opened
- [ ] Local validation is run before the PR is opened
- [ ] PR is opened, reviewed, and merged before the next milestone starts
- [ ] Docs/checklists/PR links are updated before moving on

## Full Phase To-Do Plan
1. [x] `milestone_test_1`: redesign lane taxonomy and policy so `quick`, `pr`, `nightly`, and `release` are explicit and `quick` stops running broad hardening / nested determinism work
2. [x] `milestone_test_2`: replace shell-matrix repetition with one declarative validation harness and thin wrappers
3. [ ] `milestone_test_3`: downshift eligible invariants from expensive CLI/e2e paths into cheaper integration or unit coverage
4. [ ] `milestone_test_4`: redesign generated-program artifact reuse and cache boundaries for repeated `run` / `test` validation
5. [ ] `milestone_test_5`: refactor hardening and determinism into non-default lanes while preserving breadth
6. [ ] `milestone_test_6`: add throughput/resource reporting, worker guidance, and regression visibility
7. [ ] external review pass 1 completed and acted on
8. [ ] production-grade review pass completed and acted on

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
- Implementation PR: https://github.com/yaseralnajjar/sifr/pull/1170
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
- Implementation PR: https://github.com/yaseralnajjar/sifr/pull/1172
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
- Status: pending
- Implementation target:
  - move diagnostic/lowering/codegen invariants into cheaper integration or unit coverage where possible
  - keep traceability from removed expensive checks to the new cheaper proof
- Demo target:
  - pending
- Validation target:
  - pending
- Validation evidence:
  - pending

### milestone_test_4: Artifact Reuse and Cache Boundary Redesign
- Status: pending
- Implementation target:
  - introduce reusable generated-program workspaces and repeat-run cache visibility
  - materially improve unchanged reruns for `run` and `test`
- Demo target:
  - pending
- Validation target:
  - pending
- Validation evidence:
  - pending

### milestone_test_5: Hardening Lane Refactor
- Status: pending
- Implementation target:
  - preserve broad hardening and determinism coverage while removing it from the default local loop
  - make lane placement explicit rather than script-local
- Demo target:
  - pending
- Validation target:
  - pending
- Validation evidence:
  - pending

### milestone_test_6: Throughput and Resource Governance
- Status: pending
- Implementation target:
  - report lane timing, cache behavior, fixture/group skew, and memory-related guidance
  - make performance and resource regressions visible from local runs
- Demo target:
  - pending
- Validation target:
  - pending
- Validation evidence:
  - pending

## External Review Passes

### review_pass_1
- Reviewer artifact: pending
- Status: pending
- Review summary:
  - pending
- Validation evidence:
  - pending
- Follow-up PR:
  - pending

### review_pass_2
- Reviewer artifact: pending
- Status: pending
- Review summary:
  - pending
- Validation evidence:
  - pending
- Follow-up PR:
  - pending
