# Ad-hoc Phase: Codegen Runtime Build Gap Closure (2026-04-05)

status: completed

## Goal
Close the `codegen_runtime_build_gap` category from `58` to `0` for the current full LeetCode corpus baseline, while preserving Sifr core guarantees and avoiding unsupported semantic broadening.

## Baseline (frozen)
- Source run: `verification/leetcode/full_corpus_current_results_20260405_live_rerun1.json`
- Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260405_live_rerun1.json`
- Scoped category size: `58`
- Per-case breakdown: `verification/leetcode/codegen_runtime_build_gap_breakdown_20260405_v3.csv`
- Root-cause report: `issues/codegen-runtime-build-gap-root-cause-breakdown-2026-04-05-v3.md`

## Review Sign-off
- `reviews/codegen-runtime-build-gap-root-cause-review-pass3.md` -> `READY`
- `reviews/codegen-runtime-build-gap-root-cause-review-pass4.md` -> `READY`

## Architecture Decisions (locked)
- `nonlocal` mutable capture remains unsupported by design.
- No runtime panic-based behavior in user paths.
- No implicit unsafe unwrapping of `Option`/`Result`.
- Prefer compiler correctness fixes over fixture rewrites unless a case is explicitly adaptation-owned.

## Ownership Split (verified)
- `compiler_fix`: `35`
- `both` (compiler + Sifr-surface adaptation): `21`
- `sifr_adaptation`: `2`

Sentinel clarification:
- `NO_RUST_CODE` is context-dependent in this phase:
  - compiler-path failure/panic (`0394`, `0513`, `0838`, `1609`, `0662`) is compiler-owned,
  - runtime-oracle cases (`1968`, `2215`) compiled and ran; sentinel only means no Rust error code was present.

Root-cause families in scope:
- `recursive_field_surface_leaks_to_codegen_without_gate`: `21` (`both`)
- `type_contract_emission_gap`: `20` (`compiler_fix`)
- `ownership_and_borrow_emission_gap`: `6` (`compiler_fix`)
- `binding_scope_and_capture_emission_gap`: `3` (`compiler_fix`)
- `other_codegen_build_gap`: `4` (`compiler_fix`)
- `codegen_production_panic_missing_structured_emission`: `1` (`compiler_fix`)
- `truthiness_bool_lowering_gap`: `1` (`compiler_fix`)
- `runtime_oracle_canonicalization_needed`: `2` (`sifr_adaptation`)

## Workstreams

### workstream_crbg_1_type_contract_emission_closure
Owner: compiler
Priority: P0
Scope size: `20`
Lane: `compiler_fix`

Scope:
- Remove invalid generated Rust type surfaces (mismatched Option wrapping, invalid trait obligations, invalid comparisons, arithmetic on wrapped scalars).
- Ensure emitted contracts match HIR typing invariants.

Anchor cases:
- `0211` (`String == None` invalid compare path)
- `0729` (spurious generated `Display` contract on node internals)
- `0783` (compiler-introduced scalar `Option<i64>` arithmetic)

Definition of done:
- All 20 fixtures in this family compile and run or move to a different explicitly justified family with evidence.
- No new `E0308/E0277/E0369` regressions in existing e2e pass fixtures.
- Add regression tests for `0211`, `0729`, `0783` patterns.

### workstream_crbg_2_recursive_field_surface_contract
Owner: compiler + fixture adaptation
Priority: P0
Scope size: `21`
Lane: `both`

Scope:
- Compiler: close invalid field-projection/lowering surfaces on optional recursive nodes.
- Sifr adaptation: canonicalize fixtures that rely on non-canonical recursive access forms after compiler closure.

Anchor pattern:
- Repeated `E0609` optional-recursive field projection failures.

Definition of done:
- Remaining failures in these 21 fixtures are either eliminated or explicitly documented as canonical adaptation cases with concrete patch diffs.
- No broadening that violates core principles (no implicit unsafe unwrap).
- Target: this family reaches `0` in final rerun.

### workstream_crbg_3_ownership_binding_capture_emission
Owner: compiler
Priority: P1
Scope size: `9` (`6 + 3`)
Lane: `compiler_fix`

Scope:
- Close borrow/move emission defects (`E0382/E0502`) and scope/capture emission defects (`E0425/E0424/E0434`).
- For unsupported capture forms, emit deterministic compile-time diagnostics instead of invalid Rust.

Architecture guardrail:
- Keep `nonlocal mutable capture unsupported` decision intact.

Definition of done:
- All 9 fixtures in these families either pass or fail with intentional deterministic diagnostics that are not codegen/runtime-build-gap failures.
- Any intentional unsupported-capture residuals are reclassified under `intentional_unsupported_capture` (not this phase bucket).
- Regressions added for move/borrow and unsupported-capture diagnostics.

### workstream_crbg_4_codegen_resilience_and_bool_lowering
Owner: compiler
Priority: P1
Scope size: `6` (`other_codegen_build_gap=4`, `panic=1`, `truthiness=1`)
Lane: `compiler_fix`

Scope:
- Remove compiler panic path (`0662`) and replace with robust structured handling.
- Close `NO_RUST_CODE` residual build-gap lane (`0394`, `0513`, `0838`, `1609`) with explicit deterministic compiler outputs.
- Fix bool-unary/truthiness lowering gap (`0020`, `E0600`).

Definition of done:
- `codegen_production_panic_missing_structured_emission` = `0`
- `truthiness_bool_lowering_gap` = `0`
- `other_codegen_build_gap` = `0`

### workstream_crbg_5_runtime_oracle_canonicalization
Owner: fixture adaptation
Priority: P2
Scope size: `2`
Lane: `sifr_adaptation`

Fixtures:
- `1968_array_with_elements_not_equal_to_average_of_neighbors`
- `2215_find_the_difference_of_two_arrays`

Scope:
- Canonicalize output comparison semantics for valid non-deterministic order cases.

Definition of done:
- Both fixtures pass without compiler changes.
- Oracle policy is explicit and reproducible.

## Sequencing (implementation order)
1. `workstream_crbg_1_type_contract_emission_closure`
2. `workstream_crbg_2_recursive_field_surface_contract`
3. `workstream_crbg_4_codegen_resilience_and_bool_lowering`
4. `workstream_crbg_3_ownership_binding_capture_emission`
5. `workstream_crbg_5_runtime_oracle_canonicalization`
6. Full-corpus rerun + taxonomy refresh + closure decision

Sequencing rationale:
- `workstream_crbg_4` is intentionally executed before `workstream_crbg_3` because it is a small isolated closure lane (panic removal + residual build-gap hardening + one bool-lowering fix) that gives fast deterministic signal and reduces noise before ownership/capture boundary work.

## Validation Protocol
Per wave:
- Targeted fixture rerun for changed family.
- `scripts/run_all_tests.sh --profile quick`

Phase exit gate:
- Fresh full-corpus artifact produced.
- Fresh taxonomy artifact produced.
- `codegen_runtime_build_gap` count is `0`.
- Any remaining failures (if any) are recategorized outside this phase with explicit evidence, owner, and target category label (for unsupported capture residuals use `intentional_unsupported_capture`).

## Required Deliverables
- Execution log issue (wave-by-wave deltas)
- Updated full-corpus results JSON (new dated artifact)
- Updated taxonomy JSON (new dated artifact)
- Final closure report linking compiler changes and adaptation patches
- Closure report artifact: `issues/codegen-runtime-build-gap-closure-report-2026-04-06.md`
- Closure PR: https://github.com/sifr-lang/sifr/pull/1575 (merged `2026-04-06`)

## Ready-to-implement Checklist
- [x] Create execution issue for this phase and track wave status.
- [x] Implement `workstream_crbg_1` and publish targeted delta artifact.
- [x] Implement `workstream_crbg_2` and publish targeted delta artifact.
- [x] Implement `workstream_crbg_4` and publish targeted delta artifact.
- [x] Implement `workstream_crbg_3` and publish targeted delta artifact.
- [x] Implement `workstream_crbg_5` and publish targeted delta artifact.
- [x] Run full-corpus rerun and regenerate taxonomy.
- [x] Confirm `codegen_runtime_build_gap = 0` and publish closure report.
