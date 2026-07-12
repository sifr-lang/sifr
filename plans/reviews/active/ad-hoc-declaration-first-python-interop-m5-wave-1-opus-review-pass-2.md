I have enough to conclude. Let me trace the fix against the three checks.

**Recursive entered-value hole closure.** In `context.rs:172-193`, after `Result` unwrap the code now falls through the non-`Type::Class` else branch and calls `ctx.must_use_obligation_for_type(entered_type)`. That helper (`mod_context.rs:220-254`) recursively descends `List → item`, `Tuple/Union → any`, `Dict → key or value`, `Result → ok`. So `list[Session]`, `tuple[Session, ...]`, `dict[str, Session]`, `Option[Session]` — every aggregate containing a cleanup-bearing opaque — now emits `PYCTX_INVALID_DECLARATION` with the "entered aggregates cannot hide semantic cleanup obligations" message. Regression test `context_enter_rejects_aggregate_hiding_close_required_opaque_result` exercises exactly the pass-1 example (`Result[list[Session], PythonError]`) and asserts on the message.

**No false positives on ordinary aggregates.** `must_use_obligation_for_type` returns `None` for `list[int]`, `dict[str, str]`, `tuple[int, str]`, `Option[str]`, etc. — anything whose leaves aren't opaque classes with cleanup ≠ Drop. Those cases silently pass the new else branch. The `is_direct_type` gate remains the sole authority for shape acceptance.

**Wave-1 behavior preserved.**
- Bare `Type::Class` entered type: unchanged path — `entered_name == class_name` short-circuits Self, otherwise the existing `python_opaque_classes` lookup rejects any non-Drop cleanup ("distinct opaque" message). The prior tests including `context_enter_rejects_distinct_close_required_opaque_result` still hit that branch.
- Non-class entered type: previously silently accepted; now rejected only when a hidden obligation exists. Nothing that used to compile now fails unless it was the exact hole flagged in pass 1.
- The `MustUseObligation` refactor is a pure return-shape change (was `String`, now a struct that `Display`s to the same label via `Debug`-formatted cleanup); the `format!("{obligation}")` call site prints identically to the prior string form.

The follow-up closes the recursive hole precisely, keeps ordinary aggregates uncontested, and preserves the pass-1 outcomes.

SATISFIED
