# Ad Hoc Phase Execution Checklist (First-Class Lazy Iterators and Python Iterable Protocol)

Status: in_progress (started 2026-03-18)
Owner: ad_hoc_iterator_protocol execution loop
Reference planning doc:
- `issues/ad-hoc-first-class-lazy-iterators-and-python-iterable-protocol.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 1
- [ ] Scope remains constrained to active wave
- [ ] Root cause is fixed without compatibility shims
- [ ] Positive-path and negative-path validation recorded for each wave
- [ ] Demo runs before opening each wave PR
- [ ] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [ ] PR opened/reviewed/merged before next wave starts
- [ ] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_iter_1`: first-class `Iterable[T]` / `Iterator[T]` type-system + governance/doc contract
2. [ ] `wave_iter_2`: `iter(...)` + `next(...)` builtin surfaces and protocol-driven `for` lowering
3. [ ] `wave_iter_3`: generator rewrite to true lazy iterator semantics
4. [ ] `wave_iter_4`: lazy builtin conversion for `zip`, `enumerate`, `reversed`
5. [ ] `wave_iter_5`: lazy `itertools` subset (`chain`, `repeat`, `islice`, `count`) + explicit unsupported classification
6. [ ] `wave_iter_6`: parity closure, dedicated demo, governance hardening
7. [ ] wave-level extra completion review cycle done
8. [ ] wave-level extra production-grade review cycle done
9. [ ] milestone-level completion review cycle done
10. [ ] milestone-level production-grade review cycle done
11. [ ] phase-level completion review cycle done
12. [ ] phase-level production-grade review cycle done
13. [ ] closure telegram notification sent

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
  - `https://github.com/yaseralnajjar/sifr/pull/1241` (merged)
- Validation:
  - positive path: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave1_type_protocol_demo.sifr` -> `no errors found`
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave1_type_protocol_demo.sifr` -> `12`
  - negative path: `cargo test -p sifr_hir -- test_iterator_annotation_rejects_plain_list_argument` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_iter_2: Builtin Protocol Entry and `for` Lowering
- Status: ready_for_pr
- Implementation PR:
  - branch: `wave-iter-2-builtin-protocol-entry-and-for-lowering`
- Validation:
  - positive path: `cargo test -p sifr_hir -- test_for_loop_lowers_through_iter_protocol_call` -> PASS
  - positive path: `cargo test -p sifr_hir -- test_iter_and_next_builtin_protocol_calls_lower` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_next_rejects_plain_iterable_argument` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_for_rejects_mutation_of_collection_with_live_iterator` -> PASS
  - demo check: `cargo run -q -p sifr -- check demos/ad_hoc_iter_wave2_protocol_entry_demo.sifr` -> `no errors found`
  - demo run: `cargo run -q -p sifr -- run demos/ad_hoc_iter_wave2_protocol_entry_demo.sifr` -> `1`, `9`, `16`
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_iter_3: Generator Rewrite
- Status: pending
- Implementation PR:
  - pending

### wave_iter_4: Core Builtin Lazy Parity
- Status: pending
- Implementation PR:
  - pending

### wave_iter_5: Initial `itertools` Lazy Subset
- Status: pending
- Implementation PR:
  - pending

### wave_iter_6: Parity Closure, Demo, Governance
- Status: pending
- Implementation PR:
  - pending

## External Review Passes

### review_pass_1 (completion-gap)
- Reviewer artifact: pending
- Status: pending

### review_pass_2 (production-grade)
- Reviewer artifact: pending
- Status: pending

### closure review cycles
- wave closure completion review: pending
- wave closure production-grade review: pending
- milestone closure completion review: pending
- milestone closure production-grade review: pending
- phase closure completion review: pending
- phase closure production-grade review: pending
