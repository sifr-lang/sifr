I've read the phase plan, the runtime async_runtime/async_terminal/foreign_object/object_ops/recursive_ops modules, the codegen direct wrapper, and the lowering gate. Here is the design review.

## What Wave 5 requires vs. what the design delivers

| Requirement | Design coverage |
|-------------|----------------|
| Gate remains SIFR-PYRES-0002 | ✔ Design keeps `reserved_declaration` on `Coroutine`; wrapper is exercised only by unit + runtime tests, no lowering escape hatch |
| Compiler-private owned inputs/identity cross to owned loop | ✔ `PythonAsyncValue` transport is Rust-owned; opaque/Object inputs carry a sealed identity lease (Arc over `ForeignObject`), not a `Py<PyAny>` |
| All Python work on loop thread | ✔ Setup callback resolves target/receiver, materializes args, invokes, calls `loop.create_task` (awaitability check), registers exact task; done callback extracts `task.result`, applies schema conversion, converts PyErr — all under the loop-thread GIL |
| Functions / factories / borrowed & consuming methods | ✔ Codegen branches on `PythonInteropDecoratorKind`; consuming non-cleanup methods transfer the owned handle by move (distinct from sync semantic-close, which the current `python_interop_direct.rs` already rejects for non-Function/non-`()`/non-empty-args, so the async path adds the case rather than colliding with it) |
| Positional / keyword-only / variadic / omit / recursive values / opaque results / bridges / failures | ✔ Reuses existing per-parameter shape logic; bridge targets are already rewritten in `parse_function` before the declaration is packed into HIR; opaque result path performs `isinstance` + `store_object` on the loop thread |
| Non-send public opaque, receiver frozen across await | ✔ Wrapper is `async fn(&self, …)`; the source `&self` borrow lives for the entire `.await`, and the lease adds defense-in-depth over `ForeignObject::close`. No raw `Py<PyAny>` moves — only the Arc-wrapped `ForeignObject` and Rust-owned transport values |
| No alternate coroutine execution path | ✔ Raw `run_coroutine_blocking` and typed wrappers both go through the one `PythonTerminal` + `reserve/register/finish_submission` + cancellation-carrier machinery; only the setup/done callbacks and terminal payload variant differ |
| No test-only bypass | ✔ Design explicitly forbids env flag / private lowering escape hatch |

## Root-cause correctness

- `submit_typed(..., __sifr_current_task_cancellation().as_ref()).await` composes cleanly with the existing `Option<&CancellationCarrier>` claim path from Wave 3; `CancelledBeforeClaim`, `AlreadyClaimed`, and `StateUnavailable` map to the same runtime errors as the raw path.
- Duplicate-kwarg validation on the caller side (before queueing) mirrors the sync `call_object_owned` check and does not require GIL; correct.
- Shutdown handles the typed variant by construction — `drain_outstanding_submissions` and `cancel_registered_submissions` operate on `RegisteredSubmission` identity, not payload shape.

## Concurrency/identity safety

- The transport is `Send` because `ForeignObject` (Arc<Mutex<Py<PyAny>>>) is `Send`, but no bare `Py<PyAny>` escapes the sealed wrapper. The `Py<PyAny>` is only materialized under the GIL on the loop thread, matching the phase contract.
- The lease/freeze prevents `close()` from racing a live request; combined with the source `&self` borrow this is belt-and-braces. The mechanism is under-specified (blocking behavior vs. error on close-with-live-lease) but this is a Wave 5 implementation choice rather than a design defect — Sifr's borrow checker already precludes the practical race for borrowed receivers, and consuming methods move the handle in exclusively.
- `PyCFunction::new_closure` requires `Send + 'static`; all transport variants (`PythonAsyncValue`, output schema, `PythonError`) are `Send`.

## Practical / non-blocking observations (not verdict-changing)

1. **File-size guardrail pressure.** `crates/sifr_codegen/src/python_interop_direct.rs` is 899 lines. Adding a Coroutine branch plus construction/extraction helpers pushes it over 900 unless the emitter is split (sync/async or by helper family). The design mentions "a focused runtime module" for the runtime side but does not call out the codegen split.
2. **Terminal payload generalization is glossed.** Moving `PythonTerminalOutcome` from `Result<Py<PyAny>, …>` to a `Raw|Typed` payload enum touches all raw-path tests. The design should name whether it is one enum with two success variants or parameterizes `PythonTerminal` — either works, but the raw callsites in `async_runtime.rs`/`coroutine_ops.rs` need a small refactor.
3. **Consuming non-close method surface.** Design correctly covers this, but note that today `python_interop_method_body` bails out on any consuming non-close case (`declaration.consumes_receiver` branch demands `Function` + `None` return + no params). The async emitter must not reuse that branch — it needs its own consuming-async path that transfers `self.__sifr_python_object` into the descriptor and then returns the schema-destructured result.
4. **Every typed wrapper re-resolves the target on the loop thread.** Not a correctness bug — matches the "loop-thread-only" contract — but is worth a per-declaration cache in a later wave if it shows up in profiles.

## Blocking findings

None. The design does not create a second execution path, does not weaken the SIFR-PYRES-0002 gate, does not leak `Py<PyAny>` across threads, does not introduce a test-only compiler bypass, and covers every listed Wave 5 shape and failure mode by extending the single existing terminal/registry.

VERDICT: SATISFIED
