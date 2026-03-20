# Ad Hoc Phase Execution Checklist (Canonical Iteration Model and Lazy Parity Closure)

Status: in_progress (started 2026-03-20; `wave_psp_iter_fix_0` implementation merged; review pass-1 and pass-2 approved)
Owner: ad_hoc_canonical_iteration execution loop
Reference planning doc:
- `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 0
- [x] Scope remains constrained to active wave
- [x] Root cause is fixed without compatibility shims
- [x] Positive-path and negative-path validation recorded for each wave
- [x] Demo runs before opening each wave PR
- [x] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [x] PR opened/reviewed/merged before next wave starts
- [x] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_psp_iter_fix_0`: contract freeze and governance lock
2. [ ] `wave_psp_iter_fix_1`: type-system capability layer
3. [ ] `wave_psp_iter_fix_2`: canonical iterator HIR
4. [ ] `wave_psp_iter_fix_3`: concrete iterator codegen pipelines
5. [ ] `wave_psp_iter_fix_4`: generator backend unification
6. [ ] `wave_psp_iter_fix_5`: builtin surface cleanup
7. [ ] `wave_psp_iter_fix_6`: `sifr.itertools` and iterator-returning stdlib closure
8. [ ] `wave_psp_iter_fix_7`: user-defined iterable protocol participation
9. [ ] `wave_psp_iter_fix_8`: downstream phase alignment and final closure
10. [ ] wave-level extra completion review cycle done
11. [ ] wave-level extra production-grade review cycle done
12. [ ] milestone-level completion review cycle done
13. [ ] milestone-level production-grade review cycle done
14. [ ] phase-level completion review cycle done
15. [ ] phase-level production-grade review cycle done
16. [ ] closure telegram notification sent

## Entry Baseline Evidence (2026-03-20)

Baseline command:
- `scripts/run_all_tests.sh --profile quick`

Observed baseline result before iterator-fix waves:
- HIR maintainability guardrails: PASS
- `sifr_driver` maintainability guardrails: PASS
- `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
- e2e fail/runtime/corpus lane: PASS (`25` tests)
- validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`): PASS (`7` rows)
- e2e pass suite quick profile: PASS (`24` fixtures, report signature `e1bf653aaa770517`)
- quick lane report: PASS (wall `42.59s`, max RSS `104.6MiB`, swaps `0`)

Baseline fracture reproductions captured for wave ownership:
- `any(iter(xs))`:
  - `check` passes, `run` fails rust build with `no method named 'iter' found for struct 'Box<dyn Iterator<Item = i64>>'`
- `filter(pred, iter(xs))`:
  - `check` passes, `run` fails rust build with clone/trait-bound mismatch on `Box<dyn Iterator<Item = i64>>`
- `reversed(iter(xs))`:
  - `check` passes, `run` fails rust build with `dyn Iterator<Item = i64>: DoubleEndedIterator` bound failure
- `sorted(iter(xs))`:
  - `check` passes, `run` fails rust build with unresolved `sorted` symbol in emitted Rust
- tuple iteration mismatch:
  - homogeneous tuple `for`-iteration currently fails type-check (`for-loop iterable must have a statically-known element type, got 'tuple[int, int, int]'`)

Architecture/governance lock artifacts added in wave 0:
- `verification/stdlib/phase_psp_iter_fix_architecture_lock.md`
- `verification/stdlib/wave_psp_iter_fix_0_cpython_traceability.md`
- phase governance inventory alignment in `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`

Contract lock for wave progression:
- canonical types: `Iterable[T]`, `Iterator[T]`, `Reversible[T]`
- lazy/eager builtin boundary is fixed for all later waves
- tuple iteration rule is explicit: homogeneous planned for support, heterogeneous implicit union-yield remains unsupported
- capability-aware iterator semantics are required across typing/lowering/codegen (no fallback to erased backend assumptions)

## Wave Progress

### wave_psp_iter_fix_0: Contract Freeze and Governance Lock
- Status: completed (implementation merged; completion and production-grade review passes approved)
- Implementation PR:
  - `#1339` (merged): https://github.com/yaseralnajjar/sifr/pull/1339
- Scope:
  - freeze canonical iteration semantics and permanent divergences
  - update architecture and governance docs before implementation waves begin
- Validation target:
  - architecture + waiver artifacts updated (PASS)
  - explicit baseline repro cases recorded (PASS)
  - CPython-family mapping recorded for `test_iter`, `test_generators`, `test_itertools`, and tuple-iteration coverage (PASS)
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_0_architecture_lock.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave0_contract_lock_demo.sifr` -> PASS (`ad_hoc_iter_fix_wave0_contract_lock_demo: ok`)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_0_itertools_tee_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_0_itertools_groupby_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_0_tuple_heterogeneous_iteration_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_itertools_starmap_non_binary_callable.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-20)

### wave_psp_iter_fix_1: Type-System Capability Layer
- Status: planned
- Scope:
  - add reversible/capability-aware iteration typing
  - align tuple iterability and assignability with the frozen contract
- Validation target:
  - positive typing tests for iterable/iterator/reversible capability use
  - negative typing tests for invalid reversibility, tuple misuse, and invalid single-pass reuse

### wave_psp_iter_fix_2: Canonical Iterator HIR
- Status: planned
- Scope:
  - add dedicated iterator HIR nodes
  - lower `for`, protocol entry, iterator builtins, and generator-expression sources through canonical iterator IR
- Validation target:
  - HIR snapshot tests for canonical iterator forms
  - no remaining generic builtin-call fallback for covered iterator operations

### wave_psp_iter_fix_3: Concrete Iterator Codegen Pipelines
- Status: planned
- Scope:
  - emit concrete Rust iterator chains
  - centralize collection-to-iterator lowering
  - remove clone-based fake re-iteration of true iterators
- Validation target:
  - end-to-end closure for `any(iter(xs))`, `filter(pred, iter(xs))`, and `sorted(iter(xs))`
  - capability-aware acceptance/rejection for `reversed(iter(xs))`
  - generated Rust inspection confirms no invalid `.iter()` / `.clone()` assumptions on iterator values

### wave_psp_iter_fix_4: Generator Backend Unification
- Status: planned
- Scope:
  - align generator functions and generator expressions with the canonical iterator backend
  - remove current narrow backend-shape dependence
  - retire the current single-top-level-`while` plus single-yield-site restriction as the default supported model
- Validation target:
  - positive generator-function and filtered generator-expression coverage
  - negative unsupported-shape diagnostics remain precise and do not degenerate into backend panics

### wave_psp_iter_fix_5: Builtin Surface Cleanup
- Status: planned
- Scope:
  - make builtin lazy/eager boundaries match the final contract
  - convert `filter` to true lazy semantics
- Validation target:
  - positive `map` / `filter` / `zip` / `enumerate` / `reversed` / `sorted` coverage
  - negative explicit-materialization diagnostics

### wave_psp_iter_fix_6: `sifr.itertools` and Iterator-Returning Stdlib Closure
- Status: planned
- Scope:
  - rewrite iterable signatures and buffering semantics where required
  - align iterator-returning stdlib APIs with builtin iterator consumers
- Validation target:
  - stdlib lazy-composition demos and e2e fixtures
  - explicit documentation for buffered helpers that remain intentionally non-streaming

### wave_psp_iter_fix_7: User-Defined Iterable Protocol Participation
- Status: planned
- Scope:
  - add user-defined iterable participation
  - validate user-defined iterable protocol conformance across typing, lowering, and codegen
- Validation target:
  - user-defined iterable positive/negative fixtures
  - protocol-conformance diagnostics for invalid `__iter__`, `__next__`, and `__reversed__` implementations

### wave_psp_iter_fix_8: Downstream Phase Alignment and Final Closure
- Status: planned
- Scope:
  - audit inherited iterator-sensitive surfaces from the earlier implemented ad hoc phases
  - revalidate bytes, runtime/file, and earlier stdlib iterator-returning APIs against the canonical iteration contract
  - land closure demo, final negative-case coverage, and parity-governance alignment without rewriting earlier historical phase claims
- Validation target:
  - inherited-surface regression fixtures and demos pass under the final iterator model
  - residual differences are documented as intentional divergences
  - full phase gate via `scripts/run_all_tests.sh`

## External Review Passes

### wave_psp_iter_fix_0 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-0-review-pass-1.md`
- Status: completed (approved; no contract-lock, governance, or validation omissions found)

### wave_psp_iter_fix_0 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-0-review-pass-2.md`
- Status: completed (approved; production-grade readiness confirmed with low risk and no wave-0 remediations)
