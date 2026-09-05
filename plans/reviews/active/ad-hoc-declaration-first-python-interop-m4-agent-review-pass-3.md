Pass 2 flagged only the 900-line guardrail on `control_flow.rs`. Verifying the fix:

**Extraction correctness (`crates/sifr_lowering/src/lower/must_use_obligations.rs:6-48`)**
- Signature takes `branch_moved_states`, `branch_exits`, `saved_moved`, `has_else`, `range` and mutates `ctx`.
- Logic preserved verbatim: zips branch state with exit flags, adds `(saved_moved, false)` when `!has_else` (fall-through), iterates `live_must_use_bindings`, and emits `OWN_USE_AFTER_MOVE` when a binding is consumed on some but not all continuing branches. Same message text.
- Caller at `control_flow.rs:590-597` passes `else_body.is_some()` and the same `branch_exits` derived from `branch_const_integer_states`, matching pre-extraction inputs.
- `mod must_use_obligations;` registered at `mod.rs:85`; visibility `pub(in crate::lower)` matches other lowering helpers.
- Only caller: `lower_if`. No other join point relied on the inline logic, so the extraction is complete.

**Guardrails**
- `scripts/check_file_size_guardrails.py`: PASS (2501 files, limit 900) — `control_flow.rs` at 892.
- `scripts/check_hir_maintainability_guardrails.py`: PASS.
- `close_opaque_obligation_rejects_partial_branch_consumption` (regression for the extraction): PASS.

**Adjacent code untouched**
- Assign-site checks (`control_flow.rs:255-261`, `344-357`, `394`) and record-must-use bindings unchanged.
- Function-exit rejection (`annotations_and_function_lowering.rs:673-692, 821-838`) and class-body wiring (`class_body_lowering.rs:492-521`) still route through the same `ctx.live_must_use_bindings` surface.
- No new call sites needed; nothing else changed beyond the module move.

No semantic drift. No new actionable blockers.

Gate reason: Pass 2's only blocker (900-line guardrail) is resolved by a responsibility-based extraction that preserves inputs, outputs, error code, and message; guardrails and the extraction's regression test pass; no other code paths were altered.

SATISFIED
