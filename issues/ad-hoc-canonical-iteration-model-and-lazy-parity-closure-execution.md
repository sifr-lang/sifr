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
8. [ ] `wave_psp_iter_fix_7`: user-defined iterable protocol participation and final closure
9. [ ] wave-level extra completion review cycle done
10. [ ] wave-level extra production-grade review cycle done
11. [ ] milestone-level completion review cycle done
12. [ ] milestone-level production-grade review cycle done
13. [ ] phase-level completion review cycle done
14. [ ] phase-level production-grade review cycle done
15. [ ] closure telegram notification sent

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
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_hir/src/lower/statements.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
  - `crates/sifr_codegen/src/function_emitter.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
- lock the final language contract for:
  - `Iterable[T]`
  - `Iterator[T]`
  - `Reversible[T]`
  - tuple iterability
  - lazy vs eager builtin boundaries

## Wave Progress

### wave_psp_iter_fix_0: Contract Freeze and Governance Lock
- Status: planned
- Scope:
  - freeze canonical iteration semantics and permanent divergences
  - update architecture and governance docs before implementation waves begin
- Validation target:
  - architecture + waiver artifacts updated
  - explicit baseline repro cases recorded

### wave_psp_iter_fix_1: Type-System Capability Layer
- Status: planned
- Scope:
  - add reversible/capability-aware iteration typing
  - align tuple iterability and assignability with the frozen contract
- Validation target:
  - positive typing tests for iterable/iterator/reversible capability use
  - negative typing tests for invalid reversibility and tuple misuse

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
  - end-to-end closure for current known failing iterator cases
  - generated Rust inspection confirms no invalid `.iter()` / `.clone()` assumptions on iterator values

### wave_psp_iter_fix_4: Generator Backend Unification
- Status: planned
- Scope:
  - align generator functions and generator expressions with the canonical iterator backend
  - remove current narrow backend-shape dependence
- Validation target:
  - positive generator-function and filtered generator-expression coverage
  - negative unsupported-shape diagnostics remain precise

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

### wave_psp_iter_fix_7: User-Defined Iterable Protocol Participation and Final Closure
- Status: planned
- Scope:
  - add user-defined iterable participation
  - close remaining parity/governance gaps and land closure demo
- Validation target:
  - user-defined iterable positive/negative fixtures
  - full phase gate via `scripts/run_all_tests.sh`
