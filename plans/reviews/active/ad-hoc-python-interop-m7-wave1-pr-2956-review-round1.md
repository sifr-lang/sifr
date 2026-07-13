## Review: PR #2956 — Gated Python coroutine + async-close frontend contracts

Direct verification of the diff against the working tree at `codex/python-interop-m7-wave1-frontend`.

**Reserved gate — every valid new form fails with SIFR-PYRES-0002 ✓**
- Function coroutine: `python_interop.rs:60-63` — `(Coroutine, true)` runs `reserved_declaration(...)` (emits `PYRES_UNIMPLEMENTED_DECLARATION`) before parsing.
- Method coroutine: `python_interop.rs:123-126` — same pattern.
- `cleanup=async_close`: `python_interop.rs:364-367` — emits `PYRES_UNIMPLEMENTED_DECLARATION` and now returns `Some(PythonCleanupPolicy::AsyncClose)` so the class body semantic check has metadata to operate on.

**No codegen/runtime/public activation leaked ✓**
- Repo-wide grep for `PythonInteropEffect::Async`, `PythonCleanupPolicy::AsyncClose`, and `PythonInteropDecoratorKind::Coroutine` outside `sifr_lowering`: only test fixtures in `sifr_codegen` reference `AsyncClose` inside a `Close | AsyncClose => CloseLike` obligation branch (`mod_context.rs:243`); no emitter or driver ever consumes `Async` effect or `Coroutine` kind.
- `python_interop_direct.rs:15` bounces any non-`Function` kind with `return None`; the receiver-consuming path also guards `kind != Function || ok_type != None || !params.is_empty()` (`python_interop_direct.rs:201`). Defense-in-depth is intact.

**Bodyless suspension + workload ✓**
- `stub_syntax.rs:24-29` — `is_bodyless_python_coroutine` requires `is_async && [ellipsis] && call decorator ["python","coroutine"]`.
- `async_effects.rs:23-39` seeds bodyless coroutine stubs as `Suspends`; `:52-54` short-circuits the fix-point loop so recomputation cannot downgrade them to `NoSuspend`. Non-bodyless async funcs still summarize normally.
- `mod_impl.rs:226-232` — the `BlockingIo` workload fallback now guards on `!func.is_async`, so `@python.coroutine` stubs never get stamped.

**async-close exactly-one Self.aclose ✓**
- `class_body_lowering.rs:614-640` counts `Coroutine + target=[Self, aclose] + own self + Result[None, _]` and requires exactly one. Verified with the exploration agent that `method.params` has `self` stripped at `class_body_lowering.rs:389-401`, so `.is_empty()` correctly means "no params beyond self". The `parse_method` interop-metadata side uses `parameter_metadata(...).skip(1)`; `receiver_is_owned` gates the entire predicate, so the `skip(1)` cannot silently misfire on a mis-decorated static method.
- `has_unmatched_consuming_method` at `:641-666` includes an explicit `AsyncClose` arm parallel to `Close`.

**Sync interop preserved ✓** — `parse_function`/`parse_method` are parametrized shells; the `(Function, false)` and `(Coroutine, true)` arms hit the same code with `PythonInteropEffect::BlockingIo` and produce byte-equivalent `PythonInteropDeclaration` shapes (same `kind`, `consumes_receiver`, `cleanup`, `parameters`, `required_import_root`). Diagnostic phrasing in `validate_python_interop_signature:459-491` picks "synchronous"/"asynchronous" from `declaration.effect`, so sync messages are unchanged.

**Diagnostics — method arm resolution ✓** — The round-1 fix at `python_interop.rs:157-170` adds an explicit `(Attribute|ContextEnter|ContextExit|Item, true)` arm above the `_ => reserved_declaration(...)` wildcard, restoring the "opaque Python methods must use synchronous `def`" message for M4/M5-active decorators on `async def`. Trailing wildcard only catches genuinely-reserved kinds (Callback/Buffer/Arrow/Dlpack/Context{Async}Enter/Exit).

**Tests + decomposition ✓** — 6 contract tests in `python_coroutine_contract_tests.rs` cover: gated valid coroutine, coroutine on sync `def`, sync decorator on `async def`, active sync method decorator on async, valid async-close, and async-close negatives (missing `own`, wrong target). The suspension summary has a dedicated test at `async_effects.rs:441-449`. `stub_syntax.rs` extraction brings `python_interop.rs` under the 900-line cap.

**Observations that do not block this PR**
- `parse_opaque_class` now returns `Some(AsyncClose)` (vs. the prior `None`), which registers the class in `ctx.python_opaque_classes`. On an empty `cleanup=async_close` class this fires *both* `PYRES_UNIMPLEMENTED_DECLARATION` (cleanup gate) and `PYCALL_INVALID_SHAPE` "cleanup=async_close requires exactly one…" — asymmetric with `async_context`, which drops the entry entirely. Also causes `must_use_obligation_for_type` (`mod_context.rs:236-246`) to emit `CloseLike` obligations for the class. Compilation still fails at PYRES so no runtime behavior leaks; diagnostic surface just gets noisier. No test covers the empty-class case, but the existing `close` path has the same shape.
- `method_consumes_receiver` (`python_interop.rs:696-714`) does not yet recognize `@python.coroutine(Self.aclose)` as consuming, so `python_consuming_methods` won't mark aclose calls as receiver-moves. Irrelevant while gated (compile always fails at PYRES), and a natural to pick up in the wave that activates the coroutine path.

Neither observation is a correctness bug in the current gated state.

SATISFIED
