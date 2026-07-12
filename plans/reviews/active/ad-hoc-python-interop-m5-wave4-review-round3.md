All three round-2 blockers are resolved and no new actionable findings surfaced. Summary of verification:

- **Clippy:** `cargo clippy -p sifr_codegen -- -D warnings` completes cleanly.
- **File size:** `stmt_block.rs` is 893 lines. `stmt_block_helpers.rs` (601), `loops_try_finally.rs` (578), and `python_context.rs` (675) are all comfortably under 900.
- **Extraction is behavior-preserving:**
  - `try_lower_simple_block_stmt_for_ir` (in `stmt_block_helpers.rs:476-489`) is a straight De Morgan rewrite of the inlined gate — same effect.
  - `lower_loop_control_stmt_for_ir` (in `loops_try_finally.rs:6-21`) mirrors the `_broke`/`RustStmt::Break`/`RustStmt::Continue` emission in `lower_stmt/simple_dispatch_and_bindings.rs:295-308`, gated on `loop_else_stack.last() == Some(true)`. It only fires when the previous code path would have returned `Ok(None)` (i.e. `try_closure_depth > 0`), so it expands IR coverage rather than changing existing lowering — which is exactly what the sqlite `break_from_transaction`/`continue_from_transaction` cases need.
  - Both `loop_else_stack.push` sites (while at `stmt_block.rs:714`, for at `stmt_block.rs:789`) pop on the None-fallout path, so the extraction does not leak stack state on failure.
- **Focused tests:** All 7 `stmt_support_emitter::python_context::tests::*` cases pass; the full `sifr_codegen` unit suite (758 tests) still passes.
- **Round-1 findings all still resolved** (LetElse recursion at `python_context.rs:558-566`, typed depth-0 arm at `python_context.rs:212-223`, canonical `classify_cause_kind` at `python_context.rs:609-620`, type-driven `active_is_python_error` at `python_context.rs:99-103`, mutable entered binding at `python_context.rs:309-318`, defence-in-depth cleanup evidence in `python_error_exit_body` at `python_context.rs:412-420`, sqlite fixture wired into `library_examples.py`, per-function `python_context_counter` reset at `class_method_emitter.rs:606,732`, `generator_bodies.rs:276,418`, `scope_and_function_types.rs:528,595`).

VERDICT: SATISFIED
