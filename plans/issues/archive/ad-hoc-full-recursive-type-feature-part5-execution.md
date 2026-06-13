# Ad Hoc Recursive Type Feature: Part 5 Execution

Status: complete
Started: 2026-03-13
Completed: 2026-03-13
Part: `recursive_rust_lowering_and_codegen`
PR: `#1126`

## Goal

Lower supported recursive tree-style code to finite, runnable Rust without resorting to `TreeNode`/`ListNode` special cases.

This slice takes the recursive HIR behavior from part 4 and closes the next boundary: the emitted Rust must preserve the source-level recursive semantics even though recursive fields are represented with `Box<T>` internally at the struct boundary.

## Root Cause

After part 4, recursive tree traversals type-checked correctly, but code generation still failed for two concrete reasons:

- option truthiness in structured lowering still emitted raw unary `!value` for `TreeNode | None` instead of option-aware Rust such as `is_none()` / `let-else`,
- and recursive field reads still emitted the raw boxed Rust field shape (`Option<Box<TreeNode>>`) instead of converting it back into the source-level recursive value shape (`TreeNode | None`) expected by later expressions and calls.

That left runnable recursive traversals broken even though the compiler had already proven them valid in HIR.

## Implementation

- Lower option truthiness and negated option truthiness in structured expression/condition emission through `is_some()` / `is_none()` rather than raw unary `!`.
- Lower early-exit recursive option guards like `if not node: return ...` and `if not p or not q: return ...` through `let-else` unwrapping so the surviving path keeps the unwrapped recursive-node bindings in scope.
- Reuse borrowed-option bindings via `.as_ref()` when the narrowed value originated from a borrowed parameter, preserving borrow-by-default behavior at the Rust layer.
- Lower recursive boxed field reads back to source-level recursive values using cloned borrowed projections (`as_deref().cloned()` / `as_ref().clone()`) so later recursive calls and locals see the expected type shape.
- Add a codegen regression test that asserts the emitted Rust contains the key recursive lowering constructs.
- Add a runnable e2e pass fixture for recursive tree traversal over boxed recursive fields.
- Add a runnable part 5 demo covering recursive tree traversal and paired recursive descent.

## Validation

Targeted validation:

- `cargo test -p sifr_codegen --lib --no-run`
- `cargo test -p sifr_codegen test_generate_rust_recursive_tree_traversal_uses_option_let_else_and_cloned_box_reads -- --nocapture`
- `cargo run -q -p sifr -- emit demos/ad_hoc_recursive_type_part4_demo.sifr`
- `cargo run -q -p sifr -- run demos/ad_hoc_recursive_type_part4_demo.sifr`
- `cargo run -q -p sifr -- run demos/ad_hoc_recursive_type_part5_demo.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/recursive_tree_traversal_runtime.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/recursive_tree_attribute_without_narrowing.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

## Coverage Added

- `crates/sifr_codegen/src/helpers.rs`
- `crates/sifr_codegen/src/expr_render_helpers.rs`
- `crates/sifr_codegen/src/stmt_support_emitter.rs`
- `crates/sifr_codegen/src/lib_codegen_tests.rs`
- `crates/sifr/tests/e2e/pass/recursive_tree_traversal_runtime.sifr`
- `demos/ad_hoc_recursive_type_part5_demo.sifr`

## Closure Decision

Part 5 is complete because supported recursive tree traversals now lower to finite, valid, deterministic Rust:

- recursive fields still use boxed storage at the struct boundary,
- guarded recursive-node values are unwrapped into stable Rust bindings when control flow proves them present,
- and recursive field reads round-trip back into the source-level recursive value shape expected by later expressions.

The remaining work is now the final closure slice: broadening the regression matrix and documenting the full recursive-type feature handoff in part 6.
