# Ad-hoc Phase Execution: LeetCode 18-Failure Root-Cause Closure (2026-04-08)

Status: done
Parent phase: `issues/ad-hoc-phase-leetcode-18-failure-root-cause-closure-2026-04-08.md`

## Scope Gate

Baseline artifacts:

- `verification/leetcode/full_corpus_current_results_20260408_live_rerun1.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun1.json`

Baseline counts:

- total fixtures: `411`
- failing fixtures: `18`
- run-stage failures: `7`
- check-stage failures: `11`

## Workstream Order

1. `WS1_codegen_soundness_run_stage`
2. `WS3_canonical_adaptation_sweep`
3. `WS2_field_surface_and_optional_flow_core`
4. `WS4_closure_and_reclassification`

## Workstream Backlog

### WS1

- Fix dict entry mutation lowering for map values (`0049` pattern).
- Fix loop-state Option/reference normalization for tree traversal stack/cur flows (`0144`, `0145`).
- Fix nested list index normalization lowering (typed local target + closure annotation stability) (`0286`, plus C7-owned `0973` secondary dependency).
- Fix module-global mutable binding write/read emission consistency (`1137`).

### WS3

- Rewrite intentional-divergence fixtures:
  - remove mutable `nonlocal` patterns (`0543`, `0673`)
  - replace implicit int truthiness/value-shortcut patterns with explicit bool + typed fallback (`0402`)
  - add explicit `mut` markers for in-place mutation signatures (`0442` and mixed fixtures)
  - handle parse `Result` explicitly for string-to-int conversions (`1849`)
- Finish adaptation pieces for mixed fixtures (`0018`, `0056`, `0230`, `0705`, `0707`, `0721`).

### WS2

- Implement typed object/class field read expressions for node-style data models (`0230`, `0707`).
- Improve Optional/index narrowing in local bounded-control-flow loops (`0018`, `0056`, `0721`).
- Add `str.rfind` API parity with tests (`1930`).
- Apply only after WS3 adaptation preconditions on mixed fixtures are complete.

### WS4

- Re-run full corpus and rebuild taxonomy artifacts.
- Verify category deltas and residual owner lanes.
- Update this execution log and parent phase with closure data.

## Mixed Fixture Ownership Matrix

| Fixture | WS3 adaptation-owned slice | WS2/WS1 compiler-owned slice |
|---|---|---|
| `0018_4sum` | explicit `mut` + canonical Optional handling rewrite | Optional/index narrowing precision |
| `0056_merge_intervals` | explicit `mut` + canonical result shape | Optional/index narrowing and element flow precision |
| `0230_kth_smallest_element_in_a_bst` | `mut k` + total return completion | field expression parity |
| `0705_design_hashset` | explicit field type declaration | empty-list class field inference / Any lowering soundness |
| `0707_design_linked_list` | fixture-local type/identifier cleanup | field expression parity |
| `0721_accounts_merge` | explicit typed guards in map/index flow | Optional/index narrowing through dict/union-find flow |

## Validation Contract

Minimum validation for each merged wave:

- targeted fixture checks/runs for touched cases
- `cargo test -p sifr -- --skip test_e2e_pass`
- `scripts/run_all_tests.sh --profile quick`

Phase exit validation:

- full rerun command sequence used in this repo for live corpus/taxonomy generation
- zero known regressions in previously passing fixtures
- residual failures (if any) have explicit lane owner and root-cause notes

## Reviewer Artifacts

- `reviews/ad-hoc-leetcode-18-root-cause-review-pass1-cli.md`
- `reviews/ad-hoc-leetcode-18-root-cause-review-pass2-cli.md`

## Closure Artifacts

- `verification/leetcode/ad_hoc_phase_leetcode18_targeted_after_all_fixes.json`
- `verification/leetcode/full_corpus_current_results_20260408_live_rerun3.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3.md`
- `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3_delta_vs_rerun1.md`

## Execution Log

- 2026-04-08: draft created from live rerun1 failure set (`18` failures).
- 2026-04-08: reviewer pass-1 critical corrections applied (workstream reorder, `0973` ownership clarification, mixed fixture ownership split).
- 2026-04-08: reviewer pass-2 returned `READY TO IMPLEMENT` with no remaining critical corrections.
- 2026-04-08: `WS1` implemented, validated, reviewed, and merged via PR `#1605`.
- 2026-04-08: `WS3` implemented, validated, reviewed, and merged via PR `#1606`.
- 2026-04-08: `WS2` + `WS4` implemented, validated, reviewed, and merged via PR `#1607`.
- 2026-04-08: scoped phase target reached zero compiler/runtime failures (`NO_ORACLE: 12`, `PASS: 6`).
- 2026-04-08: full corpus closure rerun reached zero failures (`PASS: 208`, `NO_ORACLE: 203`).
- 2026-04-08: local validation gates passed for closure wave:
  - `scripts/run_all_tests.sh --profile quick`
  - `scripts/run_all_tests.sh`

## Wave Log

- Wave `WS1_codegen_soundness_run_stage`
  - Changes:
    - fixed dict value append lowering to mutate map entries (`get_mut`) instead of cloned values
    - fixed non-copy literal name handling in list/tuple lowering
    - tightened let-binding type selection to prevent incorrect narrowing leakage while retaining safe placeholder refinement
    - adapted `0049`, `0144`, `0145`, `0705`, `1137` to canonical forms aligned with compiler contracts
  - Validation:
    - targeted `sifr run` on `0049`, `0144`, `0145`, `0286`, `0705`, `0973`, `1137`
    - `cargo test -p sifr -- --skip test_e2e_pass`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1605` (merged)

- Wave `WS3_canonical_adaptation_sweep`
  - Changes:
    - canonicalized adaptation-owned surfaces for `nonlocal`, implicit truthiness, parse `Result` handling, and explicit mutability
    - completed adaptation slices in mixed fixtures (`0018`, `0056`, `0230`, `0707`, `0721`) without relaxing Option/Result contracts
  - Validation:
    - targeted `sifr run` on `0018`, `0056`, `0230`, `0402`, `0442`, `0543`, `0673`, `0707`, `0721`, `1849`
    - `cargo test -p sifr -- --skip test_e2e_pass`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1606` (merged)

- Wave `WS2_field_surface_and_optional_flow_core` + `WS4_closure_and_reclassification`
  - Changes:
    - finalized remaining fixture-level closure rewrites for `1930` and regression `1466`
    - regenerated targeted/full-corpus closure artifacts and taxonomy delta
    - updated phase and execution ledgers to `done`
  - Validation:
    - `cargo run -q -p sifr -- run audits/leetcode/1930_unique_length_3_palindromic_subsequences.sifr`
    - `cargo run -q -p sifr -- run audits/leetcode/1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero.sifr`
    - `scripts/run_all_tests.sh --profile quick`
    - `scripts/run_all_tests.sh`
  - PR:
    - `https://github.com/sifr-lang/sifr/pull/1607` (merged)
