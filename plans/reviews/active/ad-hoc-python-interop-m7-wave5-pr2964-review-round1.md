## Review of PR #2964 — Add typed async Python declaration wrappers

### Scope match

Diff touches only Wave 5 surfaces: async runtime split (`async_declaration.rs`, `async_value.rs`, `async_terminal.rs`, `foreign_object.rs`, `async_runtime.rs`), codegen split (`python_interop_async.rs`, `python_interop_common.rs`, `python_interop_direct.rs` re-routing + async plan bit), coverage tests, and three review artifacts under `plans/reviews/active/`. No unrelated files.

### Wave 5 requirements — verified

- **Public gate closed:** `reserved_declaration(ctx, kind, span)` still fires `PYRES_UNIMPLEMENTED_DECLARATION` at `Severity::Error` for `Coroutine` on both function (python_interop.rs:60–63) and method (python_interop.rs:122–127) paths.
- **Loop-thread confinement:** `submit_typed` queues one setup via `call_soon_threadsafe`; setup resolves callable, materializes args, calls, `inspect.isawaitable`, `asyncio.ensure_future(..., loop=…)`, `add_done_callback`, `register_submission`, and `publish`/cancel entirely on the loop thread under the same GIL (async_declaration.rs:50–250). Done-callback runs `task.result()` + `convert_output` (including `resolve_target`/`is_instance`/`store_object` for `Opaque`) on the same thread.
- **Send transport, no raw `Py<PyAny>` across threads:** `PythonAsyncValue::Object(PythonAsyncObject)` fields (`lease`, `owner`) are `pub(super)`; `Py<PyAny>` is only materialized via `ForeignObjectLease::clone_ref(py)` under the loop-thread GIL (async_value.rs:64–70, 143–145; foreign_object.rs:144–155). Transport is Send via `Arc<Mutex<...>>` identity.
- **Identity pinning across await:** `ForeignObjectLease` clones the Arc; `ForeignObject::close()` defers Py release while `active_leases > 0` (foreign_object.rs:82–116, 162–176). Borrowed methods emit `&self.__sifr_python_object`; consuming methods use `RustParam::SelfValue` (class_method_emitter.rs:664–666) and `PythonAsyncRequest::owned_method` (python_interop_async.rs:69–91).
- **One terminal/registry/cancellation:** raw and typed both funnel through shared `terminal_for_submission`/`reserve_submission`/`register_submission`/`finish_submission`; `PythonTerminalValue` is `Raw|Typed` with cross-rejection (async_runtime.rs:177–184, async_declaration.rs:29–37).
- **Panic safety:** `catch_unwind(AssertUnwindSafe(...))` wraps both setup and done bodies; panics degrade to `AsyncRuntimeFailed` (async_declaration.rs:109, 161). No user-triggerable panic in the runtime or codegen paths.
- **Shape coverage:** positional/keyword-only/positional-variadic/keyword-variadic/`python.omit`, recursive list/tuple/dict[str,T]/record/`Option`/opaque/`Object`, bridge segments (verbatim), and `async_input_conversion` / `output_schema` / `async_output_value` all lower symmetrically.
- **Method-only async plan:** `python_interop_plan.rs:93–98` ORs `class.methods` async effect into `requires_async_loop`; `method_only_async_python_declaration_requires_owned_loop` test covers it. Codegen preamble emits `build_task_cancellation_items` for async-declaration-only modules without dragging in task-scope/join-set (entrypoints.rs:58–79, lib_modules_and_codegen.rs:540–553).
- **Concurrent one-loop identity:** `typed_failures_and_concurrent_calls_use_one_terminal_registry_and_loop` verifies two concurrent typed calls see the same `loop_id:thread_id` and diagnostics report `loop_threads: 1`.

### Guardrails

All touched files under 900 lines (max: `python_interop_direct.rs` 895, `python_interop_async.rs` 826, `async_runtime.rs` 812, `python.rs` 887).

### Review artifact accuracy

Round1 and round2 implementation reviews and the design review are all faithful to the committed code — file paths, line numbers, and identifiers match. PR body's Validation section correctly lists the four gates and both prior review verdicts.

### Non-blocking observations (not verdict-changing)

- `close_object(client)` in `typed_factory_and_borrowed_method_preserve_sealed_identity_across_await` (async_declaration_tests.rs:96–99) is comment-labeled as closing the "public identity" but actually just drops the `ObjectHandle` wrapper (object_ops.rs:306–309). The lease-across-cleanup invariant it exercises is a strict superset of an explicit `.close()` (leases also survive `ForeignObject::close()` by construction). Cosmetic.
- `async_from_owned_object` is exported but not yet emitted by codegen — reserved for later waves' owned-parameter transfer.
- No runtime test exercises consuming async methods or omitted-argument runtime materialization; both are codegen-covered and Wave 6/7 will activate them.
- Every typed wrapper re-resolves opaque `expected` types on the loop thread; consistent with the "loop-thread-only" contract, per-declaration caching is a later profile-driven concern.
- PR state is `DRAFT` — expected for pre-merge review.

VERDICT: SATISFIED
