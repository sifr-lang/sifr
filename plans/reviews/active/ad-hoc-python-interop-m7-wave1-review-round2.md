I've re-reviewed the complete M7 Wave 1 diff and the round-1 remediation.

## Verification of round-1 remediation

**Finding 1 (misleading diagnostic) — RESOLVED.** `collect_python_method_declarations` now has an explicit `(Attribute | ContextEnter | ContextExit | Item, true)` arm at python_interop.rs:157-170 that emits `PYCALL_INVALID_SHAPE "opaque Python methods must use synchronous 'def'"` above the `_ => reserved_declaration(...)` wildcard. Bare `@python.item` async is still caught earlier at 96-105 with the item-specific message, and Coroutine/Function async substitutions retain their own targeted messages. Coverage: `active_sync_method_decorator_reports_shape_not_reserved_on_async_def` in python_coroutine_contract_tests.rs:66-76 asserts PYCALL_INVALID_SHAPE fires and PYRES_UNIMPLEMENTED_DECLARATION does not.

**Finding 2 (headroom) — RESOLVED.** `PythonInteropStubBody`, `skips_normal_body_lowering`, `has_python_interop_decorator_syntax`, `classify_python_interop_stub_body`, and the new `is_bodyless_python_coroutine` moved to `python_interop/stub_syntax.rs` (66 lines) with `pub(in crate::lower) use` re-exports at python_interop.rs:19-22. `python_interop.rs` now 853 lines (47-line headroom).

## Fresh correctness sweep

- **Bodyless coroutine suspension** — `is_bodyless_python_coroutine` requires `is_async && [ellipsis] && call decorator with path == ["python", "coroutine"]`; seeded as `Suspends` at async_effects.rs:31-35 and skipped in the fix-point loop (52-54) so it can't be overwritten. Non-bodyless async funcs correctly recompute via `summarize_stmts`. New test at async_effects.rs:441-449 covers the stub-suspension path.
- **AsyncClose class contract** — python_interop.rs:364-367 emits `PYRES_UNIMPLEMENTED_DECLARATION` and returns `Some(AsyncClose)`, so `ctx.python_opaque_classes` retains cleanup metadata. class_body_lowering.rs:614-640 counts methods matching `Coroutine + Self.aclose + own self + Result[None, _]` and requires exactly one; 654-658 rejects any other consuming coroutine when cleanup is AsyncClose. `async_close_requires_one_consuming_aclose_coroutine` covers both `self` (missing `own`) and `Self.shutdown` (wrong name) negatives.
- **Sync/async substitution errors** — `(Function, true)`/`(Coroutine, false)` for both functions (48-72) and methods (122-145) hard-error with `PYCALL_INVALID_SHAPE` and the right suggestion. Coroutine on `async def` still runs `parse_function`/`parse_method` after `reserved_declaration` so shape/conversion checks fire while the reserved gate blocks compile.
- **BlockingIo workload stamp** — `mod_impl.rs:226-232` gates on `!func.is_async`, so `@python.coroutine` async defs are not stamped as blocking. Sync interop functions still get the fallback.
- **Async-flavored diagnostics** — `validate_python_interop_signature` picks "an asynchronous"/"a synchronous" phrasing from `declaration.effect` for both Result-shape and direct-conversion messages (python_interop.rs:459-492), matching the coroutine contract tests' expectations.
- **Bridge authority preserved** — `parse_function` unchanged in bridge-target rewriting logic; sync callers still resolve `bridge.*` via `ctx.python_bridge_authorities`.
- **No public syntax / codegen surface** — diff limited to lowering + new lowering tests + review artifacts. `sifr_codegen`, `sifr_driver`, `sifr_runtime`, `sifr_package`, grammar unchanged.
- **File sizes** — python_interop.rs 853, class_body_lowering.rs 836, async_effects.rs 453, stub_syntax.rs 66, python_coroutine_contract_tests.rs 109. All under the 900-line cap.
- **Wildcard reachability** — after the explicit `(Attribute|ContextEnter|ContextExit|Item, true)` arm, the trailing `_ => reserved_declaration(...)` only catches Opaque/Callback/Buffer/Arrow/Dlpack/ContextAsyncEnter/ContextAsyncExit, which are genuinely reserved for later waves.

No new findings surfaced.

SATISFIED
