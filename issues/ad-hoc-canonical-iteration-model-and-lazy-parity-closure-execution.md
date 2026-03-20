# Ad Hoc Phase Execution Checklist (Canonical Iteration Model and Lazy Parity Closure)

Status: planned (documented 2026-03-20)
Owner: ad_hoc_canonical_iteration execution loop
Reference planning doc:
- `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [ ] Entry baseline validated before wave 0
- [ ] Scope remains constrained to active wave
- [ ] Root cause is fixed without compatibility shims
- [ ] Positive-path and negative-path validation recorded for each wave
- [ ] Demo runs before opening each wave PR
- [ ] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [ ] PR opened/reviewed/merged before next wave starts
- [ ] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [ ] `wave_psp_iter_fix_0`: contract freeze and governance lock
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

## Entry Baseline Evidence (pending)

Baseline command:
- `scripts/run_all_tests.sh --profile quick`

Required entry records:
- document the exact currently failing iterator cases that motivate the phase, including:
  - `any(iter(xs))`
  - `filter(pred, iter(xs))`
  - `reversed(iter(xs))`
  - `sorted(iter(xs))`
  - tuple iteration mismatch where applicable
- record the current iterator codegen fracture points in:
  - `crates/sifr_type_system/src/types.rs`
  - `crates/sifr_hir/src/hir_nodes.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_hir/src/lower/statements.rs`
  - `crates/sifr_hir/src/lower/builtin_calls.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/function_emitter.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/operator_protocol_emitters.rs`
- lock the final language contract for:
  - `Iterable[T]`
  - `Iterator[T]`
  - `Reversible[T]`
  - tuple iterability
  - lazy vs eager builtin boundaries
  - single-pass reuse rejection vs multi-pass re-iteration rules

## Wave Progress

### wave_psp_iter_fix_0: Contract Freeze and Governance Lock
- Status: planned
- Scope:
  - freeze canonical iteration semantics and permanent divergences
  - update architecture and governance docs before implementation waves begin
- Validation target:
  - architecture + waiver artifacts updated
  - explicit baseline repro cases recorded
  - CPython-family mapping recorded for `test_iter`, `test_generators`, `test_itertools`, and tuple-iteration coverage

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
