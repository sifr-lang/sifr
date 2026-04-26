# Ad Hoc Phase Execution Checklist (Canonical Iteration Model and Lazy Parity Closure)

Status: completed (started 2026-03-20; `wave_psp_iter_fix_0` implementation merged with completion/production approvals; `wave_psp_iter_fix_1` implementation merged with completion + production-grade reviews approved after remediation; `wave_psp_iter_fix_2` implementation merged with completion + production-grade reviews approved; `wave_psp_iter_fix_3` implementation merged with completion + production-grade reviews approved after `filter(pred, iterator_variable)` regression + formatting remediation; `wave_psp_iter_fix_4` implementation/reviews merged with production-grade approval; `wave_psp_iter_fix_5` implementation/reviews merged with production-grade approval; `wave_psp_iter_fix_6` implementation/reviews merged with production-grade approval; `wave_psp_iter_fix_7` implementation/reviews merged with production-grade approval; `wave_psp_iter_fix_8` implementation/reviews merged with production-grade approval; wave-level extra completion + production-grade closure reviews approved; milestone-level completion + production-grade closure reviews approved; phase-level completion + production-grade closure reviews approved; post-closure CPython `itertools` related-test port add-on implementation + review cycles approved; closure notifications dispatched)
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
2. [x] `wave_psp_iter_fix_1`: type-system capability layer
3. [x] `wave_psp_iter_fix_2`: canonical iterator HIR
4. [x] `wave_psp_iter_fix_3`: concrete iterator codegen pipelines
5. [x] `wave_psp_iter_fix_4`: generator backend unification (completion and production-grade reviews approved)
6. [x] `wave_psp_iter_fix_5`: builtin surface cleanup (completion and production-grade reviews approved)
7. [x] `wave_psp_iter_fix_6`: `sifr.itertools` and iterator-returning stdlib closure (completion and production-grade reviews approved)
8. [x] `wave_psp_iter_fix_7`: user-defined iterable protocol participation (completion and production-grade reviews approved)
9. [x] `wave_psp_iter_fix_8`: downstream phase alignment and final closure
10. [x] wave-level extra completion review cycle done
11. [x] wave-level extra production-grade review cycle done
12. [x] milestone-level completion review cycle done
13. [x] milestone-level production-grade review cycle done
14. [x] phase-level completion review cycle done
15. [x] phase-level production-grade review cycle done
16. [x] closure telegram notification sent
17. [x] post-closure add-on: port all related CPython `itertools` tests for shipped Sifr surfaces
18. [x] post-closure add-on review pass 1 (completion/quality) approved
19. [x] post-closure add-on review pass 2 (production-grade) approved

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
  - `#1339` (merged): https://github.com/sifr-lang/sifr/pull/1339
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
- Status: completed (implementation + local validation + external completion/production reviews approved)
- Scope:
  - add reversible/capability-aware iteration typing
  - align tuple iterability and assignability with the frozen contract
- Implementation notes:
  - added type-system iteration capability metadata (`SinglePass`, `MultiPass`, `DoubleEnded`, `ExactSize`) and capability queries
  - added first-class `Reversible[T]` annotation support via canonical alias contract + assignability enforcement
  - `reversed(...)` now requires explicit reversible capability at type-check time
  - homogeneous tuple iteration now lowers through protocol entry (`iter(...)`), while heterogeneous tuple iteration remains an explicit type error
  - tuple iteration lowering in codegen now materializes a homogeneous tuple iterator path (no fallback shim)
  - wave-1 CPython traceability artifact added:
    - `verification/stdlib/wave_psp_iter_fix_1_cpython_traceability.md`
- Validation evidence:
  - type-system unit lane:
    - `cargo test -p sifr_type_system --lib` -> PASS
  - HIR typing/lowering lane:
    - `cargo test -p sifr_hir expressions_tests -- --nocapture` -> PASS
  - positive e2e fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_1_type_capability_layer.sifr` -> PASS
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave1_type_capability_demo.sifr` -> PASS (prints `30`, `15`, `[6, 5, 4]`)
  - negative fixtures (expected compile failures):
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_reversed_iterator_not_reversible.sifr` -> PASS (expected failure)
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_iter_heterogeneous_tuple_unsupported.sifr` -> PASS (expected failure)
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_reversible_annotation_rejects_set.sifr` -> PASS (expected failure)
  - wave gate:
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`)
  - production-grade remediation gate:
    - `cargo clippy -p sifr_type_system -- -D warnings` -> PASS (2026-03-20)
    - `scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-20; report signature `e1bf653aaa770517`)

### wave_psp_iter_fix_2: Canonical Iterator HIR
- Status: completed (implementation + local validation complete; external completion/production review passes pending)
- Scope:
  - add dedicated iterator HIR nodes
  - lower `for`, protocol entry, iterator builtins, and generator-expression sources through canonical iterator IR
- Implementation notes:
  - introduced canonical `HirIteratorOp` and `HirExpr::IteratorCall` in `sifr_hir`
  - lowered protocol entry (`for` + comprehension/generator sources) and iterator builtin family (`iter`, `next`, `reversed`, `map`, `filter`, `zip`, `enumerate`) to `IteratorCall` instead of generic stringly `Call`
  - extended traversal/error-ref/plumbing layers in HIR/codegen to recurse through `IteratorCall`
  - updated codegen call/lowering dispatch to handle both `Call` and `IteratorCall` uniformly while preserving existing registry-driven lowering behavior
  - fixed bytes `for`-iteration regression exposed by full-lane validation by restoring explicit bytes iterator mapping (`u8` -> `int`) in simple for-lowering
  - added wave-2 fixture/demo/traceability artifacts:
    - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_2_canonical_iterator_hir.sifr`
    - `demos/ad_hoc_iter_fix_wave2_canonical_hir_demo.sifr`
    - `verification/stdlib/wave_psp_iter_fix_2_cpython_traceability.md`
- Validation evidence:
  - HIR canonical-lowering lane:
    - `cargo test -p sifr_hir expressions_tests -- --nocapture` -> PASS
  - structural contract assertion:
    - `crates/sifr_hir/src/lower/expressions_tests.rs::test_iterator_builtins_lower_to_canonical_iterator_call_nodes` -> PASS (covers builtin calls + comprehension/generator sources)
  - positive fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_2_canonical_iterator_hir.sifr` -> PASS
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave2_canonical_hir_demo.sifr` -> PASS (prints `[2, 3, 4, 5]`, `[4, 3, 2, 1]`, `[1, 2, 3, 4]`, `[1, 2, 3, 4]`)
  - regression closure check:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_base64_intrinsics.sifr` -> PASS
  - unit/non-pass lane:
    - `cargo test -p sifr -- --skip test_e2e_pass` -> PASS
  - wave gate:
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`; e2e pass `64 passed, 0 failed`; report signature `2161ea8c3fd4e3df`)

### wave_psp_iter_fix_3: Concrete Iterator Codegen Pipelines
- Status: completed (implementation merged; completion + production-grade reviews approved after `filter(pred, iterator_variable)` regression + formatting remediation)
- Scope:
  - emit concrete Rust iterator chains
  - centralize collection-to-iterator lowering
  - remove clone-based fake re-iteration of true iterators
- Implementation notes:
  - added explicit registry builtin lowering for `iter(...)` so canonical iterator HIR no longer falls back to unresolved plain-function calls during codegen
  - added registry builtin lowering for `filter(callable, iterable)` in iterator-input paths with owned-argument callable invocation inside Rust `filter` closures
  - rewired iterator consumers (`any`, `all`, `sum`, unary `min`/`max`) to consume `registry_iterable_to_owned_iter_expr(...)` instead of collection-only `.iter().cloned()` assumptions
  - generalized `sorted(...)` element-type derivation to `iterable_element_type()` so iterator-typed inputs close without unresolved-symbol fallback
  - remediated production-grade regression where `filter(pred, iterator_variable)` took an incorrect simple-lowering path and emitted clone calls on `Box<dyn Iterator<...>>`; iterator-typed filter inputs now bypass simple filter lowering and route through canonical registry lowering
  - added wave-3 fixture/demo/traceability artifacts:
    - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_3_concrete_iterator_codegen.sifr`
    - `demos/ad_hoc_iter_fix_wave3_codegen_demo.sifr`
    - `verification/stdlib/wave_psp_iter_fix_3_cpython_traceability.md`
- Validation evidence:
  - positive fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_3_concrete_iterator_codegen.sifr` -> PASS
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave3_codegen_demo.sifr` -> PASS (prints `true`, `[5, 3, 4]`, `[1, 3, 4, 5]`)
  - capability guard (negative):
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_reversed_iterator_not_reversible.sifr` -> expected compile failure (PASS)
  - generated Rust inspection:
    - `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/phase_psp_iter_fix_3_concrete_iterator_codegen.sifr` -> PASS (`iter`/`filter`/`sorted` unresolved-symbol fallback absent; no invalid `.iter()` usage on iterator values in emitted closure paths)
  - production-grade regression closure:
    - `cargo run -q -p sifr -- run /tmp/wave3_regression_filter_iterator_var.sifr` -> PASS (prints `[2, 3]`; confirms `filter(pred, iterator_variable)` no longer emits clone on boxed iterator)
  - unit/non-pass lane:
    - `cargo test -p sifr -- --skip test_e2e_pass` -> PASS
  - wave gate:
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`; e2e pass `64 passed, 0 failed`; report signature `2161ea8c3fd4e3df`)
  - production-grade remediation gate:
    - `cargo fmt --check` -> PASS (2026-03-20; remediates reviewer-noted formatting drift)
    - `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS (2026-03-20; post-remediation guardrail revalidation)
    - `cargo test -p sifr -- --skip test_e2e_pass` -> PASS (2026-03-20; post-remediation verification)
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; post-remediation full lane gate; report signature `2161ea8c3fd4e3df`)

### wave_psp_iter_fix_4: Generator Backend Unification
- Status: completed (implementation merged; local validation + demo complete; completion and production-grade reviews approved)
- Scope:
  - align generator functions and generator expressions with the canonical iterator backend
  - remove current narrow backend-shape dependence
  - retire the current single-top-level-`while` plus single-yield-site restriction as the default supported model
- Implementation notes:
  - lowered generator expressions with optional filters through a structured `filter_map(...)` iterator chain in codegen, including `Iterator[...]`-typed result boxing for protocol parity
  - replaced generator-function codegen’s single-top-level-while/single-yield-site specialized path with a unified iterator-producing backend:
    - generator body materializes into `_yields: Vec<T>` inside a `from_fn` closure initialization block
    - closure state (`__sifr_generator_initialized`, `__sifr_generator_iter`) now drives iterator return semantics
  - added structured `Yield` statement lowering in both top-level structured statement emission and nested block lowering paths so complex-yield expressions can lower without panic
  - cloned non-copy borrowed generator params into local owned shadows before closure capture to close returned `Box<dyn Iterator<...>>` lifetime constraints for loop-backed generator bodies
  - removed HIR generator-shape gate that previously rejected nested/trailing/composed generator forms before codegen
  - added wave-4 fixture/demo/traceability artifacts:
    - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_4_generator_backend_unification.sifr`
    - `demos/ad_hoc_iter_fix_wave4_generator_backend_demo.sifr`
    - `verification/stdlib/wave_psp_iter_fix_4_cpython_traceability.md`
- Validation evidence:
  - reproduced fracture closure:
    - `cargo run -q -p sifr -- run /tmp/w4_gen_expr_filter.sifr` -> PASS (prints `[2, 4]`)
    - `cargo run -q -p sifr -- run /tmp/w4_gen_fn_multi_yield.sifr` -> PASS (prints `[0, 1, 2, 3, 4]`)
    - `cargo run -q -p sifr -- run /tmp/w4_gen_fn_for_loop.sifr` -> PASS (prints `[2, 4]`)
  - positive fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_4_generator_backend_unification.sifr` -> PASS
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave4_generator_backend_demo.sifr` -> PASS (prints `[4, 16]`, `[0, 1, 2, 3, 4]`, `[2, 4]`)
  - HIR typing/lowering lane:
    - `cargo test -p sifr_hir expressions_tests -- --nocapture` -> PASS
  - wave gate:
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`; e2e pass `64 passed, 0 failed`; report signature `2161ea8c3fd4e3df`)
  - completion-review remediation gate:
    - `cargo test -p sifr_codegen test_generate_rust_generator_conditional_yield_preserves_else_branch -- --nocapture` -> PASS
    - `cargo test -p sifr_codegen test_generator_init_emission_is_structured_only -- --nocapture` -> PASS
    - `cargo test -p sifr_codegen test_generate_rust_generator_expression_without_filter_lowers_to_map_chain -- --nocapture` -> PASS
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`; report signature `2161ea8c3fd4e3df`)

### wave_psp_iter_fix_5: Builtin Surface Cleanup
- Status: completed (implementation merged; local validation + demo complete; completion and production-grade reviews approved)
- Scope:
  - make builtin lazy/eager boundaries match the final contract
  - convert `filter` to true lazy semantics
- Implementation notes:
  - aligned HIR builtin typing/lowering with final lazy/eager contract:
    - `filter(func, iterable)` now returns `Iterator[T]` (no eager list fallback)
    - unary `sum`, `min`, and `max` now accept general iterable inputs with statically-known element types
    - `filter` callable validation now enforces callable shape and bool-compatible return type
  - removed eager `Vec::from_iter(...)` fallback from registry builtin `filter` codegen path; `filter` now lowers to boxed lazy iterator chains for all iterable inputs
  - remediated simple call-lowering backend drift in `lower_expr.rs`:
    - simple `filter` lowering now emits lazy boxed iterator closure path
    - callable invocation from `filter` now normalizes borrowed item input to owned callable argument for named functions/lambdas
  - generalized builtin `sum` codegen type annotation to iterable-element types so `sum(iter(...))` no longer fails Rust type inference
  - updated impacted fixtures/demos that previously assumed eager `filter` results to explicit materialization via `list(filter(...))`
  - added wave-5 fixture/demo/traceability artifacts:
    - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_5_builtin_surface_cleanup.sifr`
    - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_5_filter_requires_explicit_materialization.sifr`
    - `demos/ad_hoc_iter_fix_wave5_builtin_surface_cleanup_demo.sifr`
    - `verification/stdlib/wave_psp_iter_fix_5_cpython_traceability.md`
- Validation evidence:
  - HIR typing/lowering lane:
    - `cargo test -p sifr_hir expressions_tests -- --nocapture` -> PASS
  - codegen regression lane:
    - `cargo test -p sifr_codegen test_generate_rust_filter_over_list_lowers_to_lazy_boxed_iterator -- --nocapture` -> PASS
  - positive fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_5_builtin_surface_cleanup.sifr` -> PASS
  - negative fixture (expected compile failure):
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_5_filter_requires_explicit_materialization.sifr` -> expected compile failure (PASS)
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave5_builtin_surface_cleanup_demo.sifr` -> PASS (prints `[2, 4]`, `[4, 3, 2, 1]`, `[(10, 1), (11, 2), (12, 3), (13, 4)]`, `10`, `[4, 3, 2, 1]`, `[2, 4]`)
  - wave gate:
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`; e2e pass `64 passed, 0 failed`; report signature `2161ea8c3fd4e3df`)

### wave_psp_iter_fix_6: `sifr.itertools` and Iterator-Returning Stdlib Closure
- Status: completed (implementation + local validation complete; completion and production-grade reviews approved)
- Implementation PRs:
  - `#1357` (merged): https://github.com/sifr-lang/sifr/pull/1357
  - `#1358` (merged): https://github.com/sifr-lang/sifr/pull/1358
- Scope:
  - rewrite iterable signatures and buffering semantics where required
  - align iterator-returning stdlib APIs with builtin iterator consumers
- Implementation notes:
  - generalized `sifr.itertools` helper inputs from list-only types to `Iterable[...]` where semantics are iterable-driven:
    - `take`, `flatten`, `pairwise`, `batched`, `islice`, `permutations`, `combinations`, `combinations_with_replacement`, `starmap`, `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest`, `cycle`
  - kept `chain` and `product` as list-vararg entry points in this wave because current vararg list-invariance in generic call checking blocks a safe `Iterable` vararg migration without wider type-checker refactoring
  - added `_collect_iterable[T](Iterable[T]) -> list[T]` to centralize explicit materialization for helpers that require indexed or buffered execution
  - generalized HIR generic inference to bind `TypeVar` from iterable-protocol parameters:
    - `Type::Iterable(...)` now infers from any argument with `iterable_element_type()`
    - `Type::Iterator(...)` now infers from iterator arguments
  - generalized codegen plain-call lowering for iterable-typed params by normalizing call arguments through `registry_iterable_to_vec_expr(...)` when signature expects `Iterable[...]`
  - added wave-6 fixture/demo/traceability artifacts:
    - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_6_itertools_iterable_stdlib_closure.sifr`
    - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_6_islice_non_iterable_input.sifr`
    - `demos/ad_hoc_iter_fix_wave6_itertools_iterable_closure_demo.sifr`
    - `verification/stdlib/wave_psp_iter_fix_6_cpython_traceability.md`
- Validation evidence:
  - HIR inference lane:
    - `cargo test -p sifr_hir generic_inference -- --nocapture` -> PASS
  - stdlib parity checks:
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/stdlib_itertools_consolidated.sifr` -> PASS
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr` -> PASS
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/cpython_itertools.sifr` -> PASS
  - positive fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_6_itertools_iterable_stdlib_closure.sifr` -> PASS
  - negative fixture (expected compile failure):
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_6_islice_non_iterable_input.sifr` -> expected compile failure (PASS; first diagnostic `SIFR-TYPE-0001` for non-iterable input)
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave6_itertools_iterable_closure_demo.sifr` -> PASS (prints `[2, 4]`, `[1, 3, 6, 10]`, `[1, 3]`, `[1, 2]`, `1`)
  - cross-phase/demo parity check:
    - `cargo run -q -p sifr -- run demos/m30_1d_itertools_parity_demo/main.sifr` -> PASS (`pairwise(iter(...))` closure validated after pass-1 remediation)
  - wave gate:
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`; e2e pass `64 passed, 0 failed`; report signature `2161ea8c3fd4e3df`; hardening `variants=18, failures=0`)
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20 pass-1 remediation gate; profile `pr`; e2e pass `64 passed, 0 failed`; report signature `2161ea8c3fd4e3df`; hardening `variants=18, failures=0`)

### wave_psp_iter_fix_7: User-Defined Iterable Protocol Participation
- Status: completed (implementation merged; completion and production-grade reviews approved)
- Scope:
  - add user-defined iterable participation
  - validate user-defined iterable protocol conformance across typing, lowering, and codegen
- Implementation:
  - type-system:
    - `crates/sifr_type_system/src/types.rs` extends class protocol participation for iterable/iterator/reversible inference and assignability (`__iter__`, `__next__`, `__reversed__`)
  - HIR:
    - `crates/sifr_hir/src/lower/classes.rs` adds protocol-conformance diagnostics for malformed class iteration dunder methods
    - `crates/sifr_hir/src/lower/expressions.rs` widens `next(...)` acceptance to user-defined iterator protocol classes
    - `crates/sifr_hir/src/lower/expressions_tests.rs` adds positive and negative protocol coverage
  - codegen:
    - `crates/sifr_codegen/src/intrinsic_method_emitters.rs` supports user-class iterator protocol in builtin iterator lowering
    - `crates/sifr_codegen/src/stmt_support_emitter.rs` adds protocol-aware `for`-loop iterator lowering for class/iterable inputs
    - `crates/sifr_codegen/src/lower_stmt.rs` aligns simple-lowering fallback behavior and mutability forcing for protocol iterators
    - `crates/sifr_codegen/src/hir_analysis/queries.rs` marks `next(name)` as iterator-state mutation for mutable-binding inference
  - fixtures and demo:
    - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_7_user_defined_iterable_protocol.sifr`
    - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_iter_signature.sifr`
    - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_next_signature.sifr`
    - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_reversed_signature.sifr`
    - `demos/ad_hoc_iter_fix_wave7_user_defined_iterable_protocol_demo.sifr`
  - traceability:
    - `verification/stdlib/wave_psp_iter_fix_7_cpython_traceability.md`
- Validation:
  - targeted compiler tests:
    - `cargo test -p sifr_type_system --lib` -> PASS
    - `cargo test -p sifr_hir expressions_tests -- --nocapture` -> PASS
    - `cargo test -p sifr_codegen hir_analysis::queries::tests::collect_mutated_vars_marks_iterator_next_argument -- --nocapture` -> PASS
  - wave fixtures:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_7_user_defined_iterable_protocol.sifr` -> PASS
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_iter_signature.sifr` -> expected compile failure (PASS; `SIFR-TYPE-0001` with protocol signature diagnostic)
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_next_signature.sifr` -> expected compile failure (PASS; `SIFR-TYPE-0001` with protocol signature diagnostic)
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_7_invalid_reversed_signature.sifr` -> expected compile failure (PASS; `SIFR-TYPE-0001` with protocol signature diagnostic)
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave7_user_defined_iterable_protocol_demo.sifr` -> PASS (prints `[4, 3, 2, 1]`, `[1, 2, 3, 4]`, `10`, `2`, `1`, `None`)
  - wave gate:
    - `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-20; profile `pr`; e2e pass `64 passed, 0 failed`; report signature `2161ea8c3fd4e3df`; hardening `variants=18, failures=0`)

### wave_psp_iter_fix_8: Downstream Phase Alignment and Final Closure
- Status: completed (implementation/reviews merged with production-grade approval)
- Scope:
  - audit inherited iterator-sensitive surfaces from the earlier implemented ad hoc phases
  - revalidate bytes, runtime/file, and earlier stdlib iterator-returning APIs against the canonical iteration contract
  - land closure demo, final negative-case coverage, and parity-governance alignment without rewriting earlier historical phase claims
- Implementation notes:
  - closed iterable coercion mismatch in structured top-level and nested stmt lowering:
    - `Iterable[T]`-typed local bindings now use the same iterable-to-collection coercion path already used by plain-call signature lowering
    - `Iterable[T]` return sites now use the same coercion path to prevent `Iterator[T] -> Iterable[T]` Rust type mismatches
    - simple stmt lowering now bypasses `let`/`return` for iterable targets so coercion is always handled by structured lowering
  - added focused codegen regressions for local/return iterable coercion:
    - `test_generate_rust_iterable_binding_from_iterator_materializes_once`
    - `test_generate_rust_iterable_return_from_iterator_materializes_for_signature`
  - added wave-8 fixture/demo/traceability artifacts:
    - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_8_downstream_alignment_closure.sifr`
    - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_8_reversed_runtime_iterator_not_reversible.sifr`
    - `demos/ad_hoc_iter_fix_wave8_downstream_alignment_demo.sifr`
    - `verification/stdlib/wave_psp_iter_fix_8_cpython_traceability.md`
- Validation target:
  - inherited-surface regression fixtures and demos pass under the final iterator model
  - residual differences are documented as intentional divergences
  - full phase gate via `scripts/run_all_tests.sh`
- Validation:
  - focused codegen tests:
    - `cargo test -p sifr_codegen iterable_binding_from_iterator -- --nocapture` -> PASS
    - `cargo test -p sifr_codegen iterable_return_from_iterator -- --nocapture` -> PASS
  - wave fixture:
    - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_8_downstream_alignment_closure.sifr` -> PASS
  - negative fixture:
    - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_8_reversed_runtime_iterator_not_reversible.sifr` -> expected compile failure (PASS; `reversed() argument must be reversible, got 'Iterator[str]'`)
  - demo:
    - `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave8_downstream_alignment_demo.sifr` -> PASS (prints `[3, 4, 5]`, `[7, 8]`, `[66, 91]`, `[1, 4]`, `2`, `2`)

## External Review Passes

### wave_psp_iter_fix_0 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-0-review-pass-1.md`
- Status: completed (approved; no contract-lock, governance, or validation omissions found)

### wave_psp_iter_fix_0 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-0-review-pass-2.md`
- Status: completed (approved; production-grade readiness confirmed with low risk and no wave-0 remediations)

### wave_psp_iter_fix_1 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-1-review-pass-1.md`
- Status: completed (approved; no implementation gaps found across wave scope, validation evidence, and governance alignment)

### wave_psp_iter_fix_1 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-1-review-pass-2.md`
- Status: completed (approved after remediation; clippy or-pattern style updated in `sifr_type_system/src/types.rs` and validation rerun)

### wave_psp_iter_fix_2 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-2-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_iter_fix_2 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-2-review-pass-2.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_iter_fix_3 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-3-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_iter_fix_3 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-3-review-pass-2.md`
- Status: completed (approved after remediation; fixed `filter(pred, iterator_variable)` regression, applied `cargo fmt`, revalidated guardrails and full lane)

### wave_psp_iter_fix_4 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-4-review-pass-1.md`
- Status: completed (approved after remediation; replaced brittle whitespace/pattern assertions in generator codegen tests with semantic checks and added generator-expression-without-filter lowering coverage)

### wave_psp_iter_fix_4 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-4-review-pass-2.md`
- Status: completed (approved; no functional remediation required, only status/doc alignment)

### wave_psp_iter_fix_5 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-5-review-pass-1.md`
- Status: completed (approved; no functional remediation required)

### wave_psp_iter_fix_5 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-5-review-pass-2.md`
- Status: completed (approved; no functional remediation required)

### wave_psp_iter_fix_6 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-6-review-pass-1.md`
- Status: completed (approved after remediation; replaced invalid Option-state assignment in `pairwise` with explicit prior-element state, added iterator-input coverage for `pairwise`/`batched`/`cycle`, and revalidated m30 parity demo + full lane gate)

### wave_psp_iter_fix_6 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-6-review-pass-2.md`
- Status: completed (approved; production-grade readiness confirmed with no additional remediation required)

### wave_psp_iter_fix_7 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-7-review-pass-1.md`
- Status: completed (approved after remediation; class iteration protocol validation now runs only in the post-alias class pass, eliminating duplicate diagnostics in invalid protocol fixtures; remediation revalidated with full lane gate)

### wave_psp_iter_fix_7 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-7-review-pass-2.md`
- Status: completed (approved; production readiness confirmed with low regression risk and no additional remediation required)

### wave_psp_iter_fix_8 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-8-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_iter_fix_8 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-psp-iter-fix-8-review-pass-2.md`
- Status: completed (approved; production-grade readiness confirmed with no remediation changes required)

### wave_closure review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-closure-review-pass-1.md`
- Status: completed (approved; no wave-level completion gaps found)

### wave_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-wave-closure-review-pass-2.md`
- Status: completed (approved; production-grade readiness confirmed and milestone-closure progression recommended)

### milestone_closure review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-milestone-closure-review-pass-1.md`
- Status: completed (approved; no milestone-level completion gaps found)

### milestone_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-milestone-closure-review-pass-2.md`
- Status: completed (approved; production-grade readiness confirmed with no blocking findings)

### phase_closure review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-phase-closure-review-pass-1.md`
- Status: completed (approved; no phase-level completion gaps found)

### phase_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-phase-closure-review-pass-2.md`
- Status: completed (approved after remediation; addressed clippy findings in `sifr_hir` lowering paths and revalidated with `cargo clippy -p sifr_hir -- -D warnings` + `scripts/run_all_tests.sh --profile quick`)

## Post-Closure Add-On

### cpython_itertools_related_port
- Goal:
  - close remaining CPython `test_itertools.py` parity-port coverage gaps for all shipped `sifr.itertools` helpers while preserving documented Sifr intentional diffs
- Artifacts:
  - `crates/sifr/tests/e2e/pass/cpython_itertools.sifr` (expanded CPython-derived fixture for shipped helper family)
  - `lib/sifr/itertools.sifr` (`count` root-cause remediation: replace unsupported unbounded `yield` loop with bounded-prefix delegation to `count_from`)
  - `verification/stdlib/wave_psp_iter_fix_6_cpython_traceability.md` (expanded mapping and intentional-diff boundary record)
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1373` (merged)
- Validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_itertools.sifr` -> PASS
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr` -> PASS
  - `scripts/run_all_tests.sh --profile quick` -> PASS

### cpython_itertools_related_port review_pass_1 (completion/quality)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-post-closure-itertools-review-pass-1.md`
- Status: completed (approved; no actionable findings)

### cpython_itertools_related_port review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-canonical-iteration-model-and-lazy-parity-closure-post-closure-itertools-review-pass-2.md`
- Status: completed (approved; no actionable findings)
