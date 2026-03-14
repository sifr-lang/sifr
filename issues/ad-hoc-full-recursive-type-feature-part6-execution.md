# Ad Hoc Recursive Type Feature: Part 6 Execution

Status: complete
Started: 2026-03-13
Completed: 2026-03-13
Part: `recursive_corpus_closure_tests_and_demo`
PR: `#1127`

## Goal

Close the recursive-type phase with the full regression matrix required for handoff into the Phase 31 tree-followup work.

This slice needed to do more than add fixtures. It had to prove that the recursive contracts established in parts 1 through 5 were stable across the real compiler surface:

- mutually recursive classes,
- recursive generic classes,
- recursive alias acceptance and rejection coverage,
- and a final runnable demo for the milestone.

## Root Cause

After part 5, the compiler could lower self-recursive tree structures to finite Rust, but the broader recursive corpus still had a gap:

- mutually recursive class fields were detected only as direct self-recursion, so same-SCC fields like `Expr.term: Term | None` and `Term.expr: Expr | None` emitted infinite-size Rust structs instead of boxed storage,
- recursive generic class instances lost their instantiated Rust type arguments once they appeared inside composite types such as `Node[int] | None`, because general signature and local-type rendering still fell back to `Type::rust_type()`,
- and the final regression matrix was missing concrete coverage for mutual recursion, recursive generic runtime behavior, and mutual naked-recursion rejection.

That meant the milestone had the core tree case working without yet closing the broader recursive feature boundary that later corpus work depends on.

## Implementation

- Reworked recursive field detection in codegen to analyze the full class dependency graph, compute same-SCC class sets, and box any field that references a class in the same recursive SCC rather than only direct self-fields.
- Stored the exact Rust storage type per recursive field so mutually recursive and generic recursive fields lower deterministically as `Box<T>` / `Option<Box<T>>` with the right instantiated type.
- Added generic-class template tracking to codegen and implemented recursive Rust type rendering that can recover instantiated type arguments from the concrete HIR class shape, including class types nested inside `Option`, `Result`, collections, and callable signatures.
- Routed free-function signatures, class-method signatures, constructor params, struct fields, recursive field storage fallback paths, and local type annotations through the shared generic-aware renderer.
- Bypassed the older simple-statement lowering fast path for let-bindings whose annotated type contains a generic class so those bindings cannot silently reintroduce erased generic arguments.
- Added codegen regressions for:
  - mutually recursive class boxing,
  - recursive generic node signature rendering,
  - and the earlier recursive tree traversal lowering contract.
- Added recursive-corpus fixtures for:
  - mutually recursive class runtime behavior,
  - recursive generic node runtime behavior,
  - and mutual naked recursive alias rejection.
- Added the part 6 demo showing the runnable recursive tree traversal plus a declared non-tree recursive alias surface.
- Fixed two pre-existing clippy blockers in `sifr_hir` and `sifr_codegen` so the required validation gate passes on the final part 6 branch.

## Validation

Targeted validation:

- `cargo test -p sifr_hir type_alias_tests -- --nocapture`
- `cargo test -p sifr_codegen test_generate_rust_recursive_generic_node_preserves_instantiated_type_arguments -- --nocapture`
- `cargo test -p sifr_codegen test_generate_rust_mutually_recursive_classes_box_same_scc_fields -- --nocapture`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/recursive_mutual_classes_runtime.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/recursive_generic_node_runtime.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/recursive_mutual_type_alias_missing_boundary.sifr`
- `cargo run -q -p sifr -- run demos/ad_hoc_recursive_type_part6_demo.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Coverage Added

- `crates/sifr_codegen/src/field_analysis_helpers.rs`
- `crates/sifr_codegen/src/generic_bounds_helpers.rs`
- `crates/sifr_codegen/src/function_emitter.rs`
- `crates/sifr_codegen/src/class_emitter.rs`
- `crates/sifr_codegen/src/class_method_emitter.rs`
- `crates/sifr_codegen/src/stmt_support_emitter.rs`
- `crates/sifr_codegen/src/lib_codegen_tests.rs`
- `crates/sifr_hir/src/lower/type_alias_tests.rs`
- `crates/sifr/tests/e2e/pass/recursive_mutual_classes_runtime.sifr`
- `crates/sifr/tests/e2e/pass/recursive_generic_node_runtime.sifr`
- `crates/sifr/tests/e2e/fail/recursive_mutual_type_alias_missing_boundary.sifr`
- `demos/ad_hoc_recursive_type_part6_demo.sifr`

## Closure Decision

Part 6 is complete because the recursive-type phase is now regression-locked at the feature boundary it set out to deliver:

- recursive aliases still resolve and validate deterministically,
- self-recursive and mutually recursive classes both lower to finite Rust layouts,
- recursive generic class signatures preserve instantiated type arguments through the emitted Rust surface,
- and the authoritative local validation gate passes end to end.

That closes the ad hoc recursive-type phase and leaves the remaining Phase 31 work as narrow corpus closure instead of prerequisite compiler plumbing.

## External Review Follow-up

- First external review pass (`reviews/phase-recursive-types-review-pass-1.md`) reported no actionable bugs.
- Second external review pass (`reviews/phase-recursive-types-production-grade-review-pass-2.md`) reported critical failures, but those findings were validated against a stale local checkout rather than merged `origin/main`.
- Validation against merged `origin/main` (`fbb99462`, PR `#1127`) showed:
  - the recursive e2e fixtures the review marked as missing are present in `crates/sifr/tests/e2e/pass/` and `crates/sifr/tests/e2e/fail/`,
  - the merged codegen contains same-SCC recursive field boxing in `crates/sifr_codegen/src/field_analysis_helpers.rs`,
  - and the merged codegen contains recursive generic Rust type rendering plus dedicated regressions in `crates/sifr_codegen/src/generic_bounds_helpers.rs` and `crates/sifr_codegen/src/lib_codegen_tests.rs`.
- Outcome: no additional code fix was required from production-grade review pass 2; the review was closed as stale-invalidated rather than patched.
- Additional production-grade re-review pass 4 (`reviews/phase-recursive-types-production-grade-review-pass-4.md`) was then run on a fresh branch at merged `origin/main` (`47108685`, PR `#1128`) and concluded the phase is ready for production with no further compiler changes required.
