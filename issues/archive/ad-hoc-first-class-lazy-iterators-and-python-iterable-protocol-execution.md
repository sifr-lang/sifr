# Ad Hoc Phase Execution Checklist (First-Class Lazy Iterators and Python Iterable Protocol)

Status: completed (started 2026-03-18; closed 2026-03-18)
Owner: ad_hoc_iterator_protocol execution loop
Reference planning doc:
- `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 1
- [x] Scope remains constrained to active wave
- [x] Root cause is fixed without compatibility shims
- [x] Positive-path and negative-path validation recorded for each wave
- [x] Demo runs before opening each wave PR
- [x] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [x] PR opened/reviewed/merged before next wave starts
- [x] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_iter_1`: first-class `Iterable[T]` / `Iterator[T]` type-system + governance/doc contract
2. [x] `wave_iter_2`: `iter(...)` + `next(...)` builtin surfaces and protocol-driven `for` lowering
3. [x] `wave_iter_3`: generator rewrite to true lazy iterator semantics
4. [x] `wave_iter_4`: lazy builtin conversion for `zip`, `enumerate`, `reversed`
5. [x] `wave_iter_5`: lazy `itertools` subset (`chain`, `repeat`, `islice`, `count`) + explicit unsupported classification
6. [x] `wave_iter_6`: parity closure, dedicated demo, governance hardening
7. [x] wave-level extra completion review cycle done
8. [x] wave-level extra production-grade review cycle done
9. [x] milestone-level completion review cycle done
10. [x] milestone-level production-grade review cycle done
11. [x] phase-level completion review cycle done
12. [x] phase-level production-grade review cycle done
13. [x] closure telegram notification sent

## Entry Baseline Evidence (2026-03-18)

Baseline command:
- `$(pwd)/scripts/run_all_tests.sh --profile quick`

Observed baseline result before wave edits:
- HIR maintainability guardrails: PASS
- `sifr_driver` maintainability guardrails: PASS
- `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
- e2e fail/runtime/corpus lane: PASS (`25` tests)
- validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`): PASS (`7` rows)
- e2e pass suite quick profile: PASS (`24` fixtures, report signature `e1bf653aaa770517`)
- quick lane report: PASS (wall `37.06s`, max RSS `105.1MiB`, swaps `0`)

Required entry records:
- Initial generator-lowering strategy: move from buffered `_yields: Vec<T>` fallback to canonical iterator-returning lowering, while preserving deterministic no-panic behavior and explicit exhaustion via `Option[T]`.
- Concrete type-system spike target: introduce `Type::Iterable(Box<Type>)` and `Type::Iterator(Box<Type>)`, with `Iterator[T]` satisfying iterable semantics (`iterable_element_type` resolves to `T` for both).
- Initial CPython test-family inventory for wave 1:
  - `Lib/test/test_iter.py::test_iter_basic` -> adapted
  - `Lib/test/test_iter.py::test_iter_idempotency` -> adapted
  - `Lib/test/test_iter.py::test_iter_class_iter` -> waived (`unsupported`, user-defined dunder protocol surface not yet implemented)
  - `Lib/test/test_iter.py::test_iter_class_for` -> waived (`unsupported`, same reason)
- Initial borrow-safety example to enforce in wave 2:
  - compile-time rejection target: mutating a collection while a live iterator over it is borrowed in the same scope (`unsupported until borrow model for iterator aliases is explicit`, no silent eager fallback).

## Wave Progress

### wave_iter_1: Iterator Protocol and Type-System Contract
- Status: merged
- Goal:
  - add first-class `Iterable[T]` / `Iterator[T]` in type model
  - wire typing helpers so iterator element extraction is protocol-based
  - update architecture/governance docs to replace eager-lazy waiver baseline
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1241` (merged)
- Validation:
  - positive path: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave1_type_protocol_demo.sifr` -> `no errors found`
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave1_type_protocol_demo.sifr` -> `12`
  - negative path: `cargo test -p sifr_hir -- test_iterator_annotation_rejects_plain_list_argument` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_iter_2: Builtin Protocol Entry and `for` Lowering
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1242` (merged)
- Validation:
  - positive path: `cargo test -p sifr_hir -- test_for_loop_lowers_through_iter_protocol_call` -> PASS
  - positive path: `cargo test -p sifr_hir -- test_iter_and_next_builtin_protocol_calls_lower` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_next_rejects_plain_iterable_argument` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_for_rejects_mutation_of_collection_with_live_iterator` -> PASS
  - demo check: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave2_protocol_entry_demo.sifr` -> `no errors found`
  - demo run: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave2_protocol_entry_demo.sifr` -> `1`, `9`, `16`
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_iter_3: Generator Rewrite
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1243` (merged)
- Validation:
  - positive path: `cargo test -p sifr_hir -- test_generator_function_infers_iterator_return_type --nocapture` -> PASS
  - positive path: `cargo test -p sifr_hir -- test_generator_expression_is_typed_as_iterator --nocapture` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_generator_function_rejects_non_iterator_annotation --nocapture` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_generator_rejects_nested_yield_shape --nocapture` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_generator_rejects_trailing_statements_after_loop --nocapture` -> PASS
  - demo check: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave3_generator_rewrite_demo.sifr` -> `no errors found`
  - demo run: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave3_generator_rewrite_demo.sifr` -> `3`, `2`, `[1]`, `[4, 3, 2, 1]`
  - milestone demo run: `cargo run -q -p sifr -- run demos/milestone_generators_demo.sifr` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_iter_4: Core Builtin Lazy Parity
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1244` (merged)
- Validation:
  - positive path: `cargo test -p sifr_hir -- test_reversed_enumerate_zip_are_typed_as_iterators --nocapture` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/builtin_enumerate_zip.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr` -> PASS
  - negative path: compile-time behavior requires explicit materialization when assigning lazy builtin outputs to `list[...]`-typed values; updated fixtures now use `list(...)` at the eager boundary
  - demo check: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave4_builtin_lazy_parity_demo.sifr` -> `no errors found`
  - demo run: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave4_builtin_lazy_parity_demo.sifr` -> `2`, `[1, 3]`, `[(5, "a"), (6, "b")]`, `[(1, "x", true), (2, "y", false)]`
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_iter_5: Initial `itertools` Lazy Subset
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1245` (merged)
- Validation:
  - positive path: `cargo test -p sifr_codegen -- test_generate_rust_generator_conditional_yield_preserves_else_branch --nocapture` -> PASS
  - positive path: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave5_itertools_lazy_subset_demo.sifr` -> `no errors found`
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave5_itertools_lazy_subset_demo.sifr` -> `[1, 2, 3]`, `[7, 7, 7]`, `[20, 40]`, `5`, `7`, `9`, `11`
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_itertools.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_itertools_consolidated.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_chain_float.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_chain_str.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/itertools_chain_own.sifr` -> PASS
  - negative path: `islice(..., step <= 0)` now yields an empty iterator; fixtures assert explicit `[]` materialization through `list(...)`
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_iter_6: Parity Closure, Demo, Governance
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1247` (merged)
- Validation:
  - positive path: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave6_parity_closure_demo.sifr` -> `no errors found`
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave6_parity_closure_demo.sifr` -> `ad_hoc_iter_wave6_parity_closure_demo: ok`
  - positive path: `cargo run -q -p sifr -- run demos/milestone_lazy_iterators_demo.sifr` -> PASS
  - negative path: retained advanced `itertools` combinators stay explicitly eager/list-backed and are now governed as `intentional-diff` instead of `phase-tracked`
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### CPython-derived accounting refresh (phase closure)
- `Lib/test/test_iter.py::test_iter_basic` -> adapted (covered by wave 1 protocol demo + iterator annotation tests)
- `Lib/test/test_iter.py::test_iter_idempotency` -> adapted (covered by iterator protocol lowering/tests in wave 2)
- `Lib/test/test_iter.py::test_iter_for_loop` -> adapted (covered by `test_for_loop_lowers_through_iter_protocol_call` + wave demos)
- `Lib/test/test_iter.py::test_iter_independence` -> adapted (collection-backed iterable reuse validated by protocol demos)
- `Lib/test/test_iter.py::test_nested_comprehensions_iter` -> adapted (generator/comprehension iterator typing and runtime tests in wave 3)
- `Lib/test/test_iter.py::test_iter_class_for` -> waived (`unsupported`, user-defined dunder iterator protocol surface remains out of scope)
- `Lib/test/test_iter.py::test_iter_class_iter` -> waived (`unsupported`, same boundary as above)

## External Review Passes

### review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-review-pass-1.md`
- Status: completed (validated; no additional code defects identified beyond merged phase scope)

### review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-review-pass-2.md`
- Status: completed (validated; production-grade review reported no additional defects)

### closure review cycles
- wave closure completion review: completed (`reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-wave-closure-completion-review.md`)
- wave closure production-grade review: completed (`reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-wave-closure-production-grade-review.md`)
- milestone closure completion review: completed (`reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-milestone-closure-completion-review.md`)
- milestone closure production-grade review: completed (`reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-milestone-closure-production-grade-review.md`)
- phase closure completion review: completed (`reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-phase-closure-completion-review.md`)
- phase closure production-grade review: completed (`reviews/phase-ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol-phase-closure-production-grade-review.md`)

### Closure notification
- phase closure telegram notification: sent (`message_id=117`, 2026-03-18)
