# M9 Design: Typed Callback Lifetimes And Dispatch

## Objective

Activate declaration-first Python callbacks with one typed contract spanning
`current`, `foreign`, and `asyncio` dispatch. Callback trampolines must never
outlive their declared call/result/receiver owner, must preserve checked Sifr
argument and result conversion, and must make close, failure, reentrancy, and
cancellation deterministic without blocking either async executor.

M9 lands in three independently reviewed implementation waves. Wave 1 adds the
complete source/HIR/type contract and a gated runtime lifecycle substrate behind
the existing `SIFR-PYRES-0002` public reservation. Wave 2 implements typed current and
foreign execution plus retained-owner shutdown while the public declaration is
still reserved. Wave 3 implements asyncio dispatch, atomically lifts the
reservation for all modes, and adds compiled evidence and documentation. No
dispatch mode becomes public before the full M9 contract is executable.

## Existing Authorities To Reuse

- M4 owns recursive checked value conversion, opaque identity, semantic close,
  poison-on-failed-close, and deterministic opaque cleanup obligations.
- M5 owns original Python-error replay, `SifrBoundaryError`, primary/secondary
  evidence ordering, and exact-once cleanup expectations.
- M7 owns the application Python loop, exact asyncio task registration,
  bidirectional cancellation, terminal waiting, and shutdown-before-loop-stop.
- The legacy `py_local_callback`/`py_threadsafe_callback` raw-handle helpers are
  compatibility/test substrate only. They do not satisfy typed conversion,
  declared ownership, error evidence, serial ordering, or asyncio semantics and
  are not the M9 public implementation.
- Existing nested-function capture discovery and task-boundary sendability
  analysis are the starting point for callback capture checks; M9 centralizes
  reusable sendability queries rather than introducing a second type policy.
- PyO3's `PyCFunction::new_closure` requires its Rust shell to be `Send + Sync`.
  Current-dispatch non-send captures therefore live in a thread-local target
  registry; the Python function shell captures only a copyable registry token.

## Declaration And HIR Contract

`@python.callback(...)` is an adjunct to exactly one ordinary `@python(...)` or
`@python.coroutine(...)` implementation declaration. It is not a second Python
implementation declaration and therefore does not participate in the existing
"one implementation declaration" truncation rule. Add a
`PythonCallbackDeclaration` record retained in
`PythonInteropDeclaration.callbacks` with:

- callback parameter name and source span;
- `PythonCallbackLifetime::{Call, Result, Receiver}`;
- `PythonCallbackDispatch::{Current, Foreign, Asyncio}`;
- optional `PythonCallbackConcurrency::{Serial, Parallel}`;
- normalized callback argument conventions/types, success type, optional
  handler-error type, and whether the callable is async;
- resolved retained-owner class/cleanup facts and the declaration span used by
  diagnostics.

Parsing is order-independent across decorators. Lowering first identifies the
single implementation decorator, parses every callback adjunct, then validates
the combined function signature. Duplicate callback metadata for one parameter,
unknown parameters, non-callable parameters, and callback metadata without an
implementation declaration are rejected.

The accepted surface is closed:

1. `lifetime=call | result | Self` and `dispatch=current | foreign | asyncio`
   are required literal policy values. Unknown, duplicated, or dynamically
   computed values are rejected.
2. `dispatch=current` accepts only synchronous `Callable` and only
   `lifetime=call`; `concurrency` is forbidden.
3. `dispatch=foreign` accepts only synchronous `Callable`; `concurrency=serial |
   parallel` is required.
4. `dispatch=asyncio` accepts only `AsyncCallable`; `concurrency=serial |
   parallel` is required. A synchronous `Callable` or `Type::AsyncFunction`
   annotation is not an implicit substitute.
5. `lifetime=result` requires a successful opaque result, directly or as the
   success member of `Result[Owner, E]`. `lifetime=Self` is valid only on an
   opaque receiver method. The selected owner must have `cleanup=close |
   async_close | context | async_context`; `cleanup=drop` is insufficient for a
   retained Python registration because it cannot perform unregister-first
   shutdown.
6. All callback argument and success types must have the corresponding M4
   recursive Python conversion. `foreign` additionally rejects Python opaque
   identity anywhere in arguments, results, handler errors, or captures and
   requires sendable boundary values. `asyncio` requires sendable Sifr-side
   values/captures but may convert ordinary Python values only on the owned loop.
7. A callback returning `Result[R, HandlerError]` requires the enclosing
   declaration's error channel to contain `HandlerError`; a plain callback
   return has no handler-error channel. The compiler does not synthesize `Any`
   or widen the declared error union.
8. A call-scoped foreign/asyncio callback is dynamically invalidated as soon as
   the Python target call becomes terminal. The bridge/package contract promises
   it is not retained; an escaped call receives the stable closed-callback
   exception rather than reaching released state.

Wave 1 parses and validates this complete contract but emits `SIFR-PYRES-0002`
after a valid internal declaration is retained. Invalid declarations emit their
specific `SIFR-PYCB-0001`, conversion, owner, or sendability diagnostic without a
masking reservation. Thus the gated waves prepare one public surface rather
than expose incremental behavior.

## Net-New `AsyncCallable`

Add `Type::AsyncCallable(Vec<Type>, Vec<ParamConvention>, Box<Type>)` parallel
to `Type::Callable`. Source syntax is
`AsyncCallable[[Arg1, Arg2], Return]`; its return slot is the value produced by
awaiting the handler, including `Result[R, E]` when the handler can fail.

The type-system work is exhaustive rather than a display-only alias:

- annotation parsing and stable malformed-shape diagnostics;
- display/rust rendering, ownership, alias substitution, union ordering,
  traversal, generic substitution/inference, recursive `Any`/type queries, and
  schema/interop rejection switches;
- async call checking with ordinary callable arity/conventions, but expression
  calls produce `Coroutine[Return, Never]` and therefore require `await`;
- assignability from `AsyncFunction` to `AsyncCallable` when signatures match,
  and between matching `AsyncCallable` values;
- no assignability between `Function`/`Callable` and `AsyncCallable`, and no
  representation or semantic equivalence between `AsyncCallable` and
  `AsyncFunction`;
- function-parameter codegen using stable Rust `AsyncFn(...) -> R + Send + Sync`,
  which accepts ordinary `async fn` opaque futures without allocating each
  call. General class-field erasure uses an owned `Arc` adapter and boxed
  policy-neutral future because stable `AsyncFn` bounds cannot express that
  every call future is `Send`. Generated foreign/asyncio callback sidecars add
  the explicit boxed `Fn -> Pin<Box<dyn Future + Send>>` boundary after their
  sendability check. `AsyncFn` itself is not dyn-compatible, and callback
  codegen adds serial/parallel capture bounds rather than encoding dispatch
  policy into the source type.

Focused tests enumerate every exhaustive match family so future type additions
cannot silently omit async callable behavior.

## Typed Conversion And Failure Frames

Codegen creates one typed adapter per callback parameter. Runtime owns only a
policy-neutral invocation frame containing sealed Python argument identities,
entry sequence, owner generation, cancellation authority, and a terminal
sender. Generated code owns all Sifr types and performs M4 conversion. A typed
`CallbackFailureSlot<E>` is emitted beside each generated adapter; the generic
runtime sees only redacted exception metadata and a failure-present bit, never
an erased `HandlerError`. Call-scoped wrappers own these slots locally. Opaque
owner codegen collects every retained callback declaration that can attach to a
class and emits optional typed sidecar fields for those slots, so later owner
operations and cleanup recover the exact `E` without `Any` or downcasting.

- Python callback calls are positional-only because `Callable`/`AsyncCallable`
  annotations carry types but no keyword names. Keyword arguments or wrong
  arity raise stable Python `TypeError` before a handler is accepted.
- Python arguments are converted under the GIL/owned-loop authority before Sifr
  execution. Conversion failure is a Python callback-call error and does not
  enter the handler.
- Handler success is converted back to Python under the correct Python
  authority. Result-conversion failure is a Python callback-call error recorded
  against the same entry.
- `Result::Err(HandlerError)` records typed failure evidence and raises a
  registered `SifrCallbackError` whose Python object contains only stable
  redacted type/message metadata; the typed Sifr value stays in the call or
  owner evidence store.
- Entry sequence is allocated at acceptance. The evidence store keeps the
  lowest-sequence handler failure using a compare/update operation, so parallel
  completion order cannot change the selected typed failure.

For call-scoped callbacks, the wrapper closes all trampolines after the Python
target reaches a terminal result and then inspects their aggregate evidence:

1. Python success plus any handler failure returns the lowest-entry
   `HandlerError`, even if Python caught `SifrCallbackError`.
2. Python failure remains primary; the lowest-entry handler failure is attached
   as secondary evidence.
3. Handler failure plus result-conversion or callback infrastructure failure
   uses the terminal callback error as primary and retains handler evidence as
   secondary unless the typed enclosing handler channel is the required primary
   outcome.

Retained evidence is stored on the owner callback group. Generated owner
operations inspect it before returning success; semantic shutdown also returns
it through the owner's declared error channel. If shutdown occurs during runtime
teardown with no Sifr observer, the stable shutdown diagnostics report it.

## Runtime Trampoline And Dispatch Engines

Replace the monolithic callback store with focused modules under
`sifr_runtime::python::callbacks`:

- `registry.rs`: opaque identities, nonce/generation validation, runtime-wide
  retained-owner registry, and shutdown ordering;
- `state.rs`: open/closing/closed state, accepted-call leases, active count,
  close joiners, capture release, evidence, and entry sequencing;
- `current.rs`: dynamic-call-scope trampoline and same-thread execution;
- `foreign.rs`: GIL-separated foreign serial/parallel execution;
- `asyncio.rs`: owned-loop coroutine/future bridge and cancellation;
- `errors.rs`: registered exception types and stable diagnostics.

The registry never holds its global mutex while invoking user code, acquiring a
serial permit, waiting for active calls, converting Python values, or dropping
captured/Python objects. An accepted-call RAII lease increments the active count
only after state/generation/reentrancy validation and decrements it on every
terminal path, waking close joiners exactly once when closing reaches zero.

### Current Dispatch

Current dispatch invokes a synchronous handler on the Python caller's thread and
is tied to a dynamic target-call scope. It may hold non-send captures. The
typed adapter and capture state are stored in a thread-local registry keyed by a
nonce/generation token. The `Send + Sync` PyO3 shell captures only that token,
checks that invocation occurs on the creating thread and active call generation,
and then resolves the local adapter. It never moves or shares the adapter.
Closing the call scope first rejects new entries, waits for accepted synchronous
invocations (normally already terminal on the same dynamic call tree), removes
the TLS target, releases captures on the creating thread, and invalidates the
callable.

### Foreign Dispatch

Foreign trampolines are `Send + Sync` and callable from arbitrary
Python-created threads. Argument extraction occurs under the calling thread's
GIL; the GIL is released before acquiring serial state or running Sifr code; the
result/error is converted after reacquiring the GIL. No sealed Python object or
opaque identity enters the detached Sifr phase.

- `serial` assigns the acceptance sequence before queuing and executes in FIFO
  acceptance order through generated mutex-protected handler state. A
  thread-local `(owner_id, callback_id)` invocation stack detects recursive
  entry before any queue/lock acquisition and raises
  `SifrCallbackReentrancyError`.
- `parallel` permits simultaneous calls and invokes immutable shared handler
  state. Static capture analysis requires sendable and shareable captures;
  generated Rust bounds remain a backstop, not the primary diagnostic.

Close from inside any accepted invocation of the same owner is rejected using
the same invocation stack before unregister or waiting begins. This applies even
when Python obscures the call chain.

### Asyncio Dispatch

An asyncio trampoline is installed and invoked only on M7's application-owned
Python loop. Calling it returns an `asyncio.Future` immediately. Runtime accepts
the entry and converts arguments on the loop. The generated adapter captures a
Tokio handle and a fresh Sifr cancellation carrier while it is still executing
on the Sifr executor; the Python shell sends a typed-erased job token back to
that generated adapter, which spawns the typed handler future on the captured
handle. Completion is sent through a compiler-private M7 loop request and sets
the Python future on the owned loop. `sifr_runtime` gains no production Tokio
dependency, and neither executor synchronously waits on the other.

Each entry owns one closed terminal state shared with M7 cancellation
infrastructure:

- cancellation of the Python future requests the exact Sifr handler carrier and
  waits asynchronously for its terminal acknowledgement before finalizing the
  Python future;
- cancellation of the Sifr handler or owner/runtime shutdown schedules
  cancellation of the exact Python future and terminally joins both sides;
- completion and cancellation race through one compare/exchange terminal, so a
  late result is released and cannot overwrite cancellation;
- `serial` uses an async FIFO permit. The task-local invocation stack is checked
  before awaiting that permit, rejecting recursive self-await deterministically;
- `parallel` submits independent futures with immutable `Send + Sync` captures.

Asyncio call-scoped callbacks are drained before the enclosing Python coroutine
request becomes terminal. Retained callbacks are drained by owner shutdown.

## Retained Owner Aggregation And Shutdown

A successful `lifetime=result` call transfers the callback group into the newly
constructed opaque result before that result is exposed to Sifr. A failed call
closes the provisional group. `lifetime=Self` registers into the receiver's
existing group only after Python target registration succeeds; failed
registration rolls back the new trampoline. Multiple callbacks aggregate under
one owner in declaration order, while failure selection uses global entry
sequence.

Generated opaque wrappers carry a compiler-private callback-owner identity next
to their Python object identity. The opaque class's already-declared consuming
cleanup operation is the unregister authority; M9 adds no undeclared Python
method or callback-specific unregister target. All consuming cleanup paths
compose callback shutdown with the M4 semantic Python close:

Before invoking that semantic cleanup, the owner claims a joinable unregister
guard shared with runtime shutdown. Exactly one path runs unregister; dropping
the guard records completion and wakes the other path. Closing and capture
release wait for unregister completion, so concurrent semantic/runtime teardown
cannot double-unregister or release a callable while unregister is still using
it.

1. enter M4 semantic close and invoke the owner's declared `close`, `aclose`,
   `__exit__`, or `__aexit__` while the callback group remains open, because
   that operation is contractually responsible for unregistering callbacks;
2. immediately after that operation becomes terminal, transition the group
   `open -> closing` and reject new callback entries; an entry accepted before
   the transition remains owned by the drain;
3. request cancellation of accepted asyncio entries and wait asynchronously for
   their terminal acknowledgements; synchronous close drains foreign accepted
   calls without holding Python/store locks;
4. surface retained failure evidence through the close operation's typed error
   channel;
5. release generated captures and Python callable objects;
6. transition to `closed`, wake idempotent internal joiners, and finish M4
   semantic close success. If the Python cleanup/unregister operation fails,
   M4 poisons the owner; the callback group still transitions to closing and
   drains/releases locally so no Sifr capture remains reachable from a Python
   callable. The cleanup error is primary and retained handler failure is
   secondary evidence.

The package binding contract for a retained declaration promises that the
opaque owner's declared cleanup operation unregisters the callback. Runtime
closure after cleanup is still authoritative: a package that invokes an escaped
callable later receives the stable closed-callback exception and cannot reach
released Sifr state. A retained owner with `cleanup=drop` remains rejected
because it has no unregister authority.

Because Sifr owners are affine, source can consume an owner only once. Runtime
joinability handles concurrent Python/runtime teardown races. Runtime shutdown
snapshots retained groups, runs unregister/closing/drain in stable owner-id order
before M7 loop shutdown, reports aggregate failures, and then verifies that the
registry is empty. No capture or Python callable is released while an accepted
lease exists.

## Static Close-From-Callback Rejection

When a callback argument is a named nested/local handler and the retained owner
is statically identifiable, lowering inspects the handler body and rejects any
direct or resolved call that consumes/closes that owner. The analysis follows
existing nested-function capture identities and errs conservatively for aliases
that preserve the same affine owner. Indirect/dynamic paths remain protected by
the runtime owner invocation stack. Static analysis is a diagnostic improvement;
runtime rejection is the semantic authority.

## Wave 1: Gated Contract And Lifecycle Substrate

1. Add callback policy HIR, order-independent adjunct parsing, owner/error/
   conversion/sendability validation, and diagnostic registry entries. Retain
   the public reservation after successful validation.
2. Add exhaustive `AsyncCallable` annotation, type-system, lowering, and
   codegen support with no callback activation.
3. Split the legacy callback runtime by responsibility and add the closed
   owner-state/lease/entry/evidence/exception/registry substrate. Preserve raw
   helper behavior through compatibility adapters.
4. Add call/result/receiver attachment metadata to codegen plans behind the
   gate; no source declaration emits a trampoline yet.

Validation covers valid gated declarations, every malformed policy combination,
decorator ordering, owner/error eligibility, recursive conversion, foreign/
asyncio sendability and opaque rejection, `AsyncCallable` call/await/
assignability/exhaustive switches, owner state transitions, lease draining,
lowest-sequence evidence, exception registration, shutdown ordering, raw-helper
compatibility, file-size/maintainability guardrails, and the authoritative
create-PR profile.

## Wave 2: Gated Current And Foreign Execution

1. Generate typed current call-scoped trampolines and call-terminal evidence
   reconciliation.
2. Generate foreign serial/parallel trampolines, GIL separation, static capture
   validation, FIFO serial execution, deterministic pre-lock reentrancy, and
   checked conversions.
3. Attach provisional groups to returned/receiver owners, integrate unregister-
   first close with sync owner cleanup, and prove runtime shutdown ordering.
4. Keep `SIFR-PYRES-0002` for every otherwise-valid source declaration; exercise
   execution with constructed HIR/codegen plans and compiler-private runtime
   fixtures only.

Validation covers current non-send capture and escape rejection; foreign serial
FIFO, parallel overlap, wrong argument/result, handler error, Python error,
swallowed callback error, concurrent lowest-entry failure, reentrancy before
lock, close during active calls, close from invocation, callback after close,
capture/callable exact-once release, retained error observation, runtime
shutdown, and generated Rust compilation tests.

## Wave 3: Asyncio, Atomic Activation, And Evidence

1. Implement owned-loop asyncio future creation/completion, Sifr handler task
   submission, serial/parallel execution, pre-await reentrancy, bidirectional
   exact-task cancellation, and terminal drain.
2. Remove only the `@python.callback` reservation once current, foreign, and
   asyncio dispatch all pass their matrices. Retain M10-M12 reservations.
3. Add compiled offline examples:
   - CFFI invokes current and foreign callbacks from native/background threads;
   - Kafka uses a deterministic in-process fake producer/consumer callback API
     with the same public callback shape and no broker/network dependency;
   - Pub/Sub uses a deterministic in-process subscriber/future callback API and
     no cloud credentials or network dependency.
   Each fixture exercises the real compiled Sifr wrapper and Python package
   dependency where practical; service behavior is locally simulated so all
   profiles remain hermetic.
4. Register the callback suite unconditionally in create-PR, merge, nightly,
   and release profiles; add `demos/m9_demo` that executes compiled evidence and
   prints a hardened marker derived from binary output.
5. Activate callback capability rows and update public docs, both durable Python
   interop architecture documents, runtime architecture, verification README/
   exit evidence, roadmap, tracker, and review records.

The executable matrix includes current, foreign serial, foreign parallel, and
asyncio; captures/sendability; wrong args/results; handler/Python/combined
errors; serial reentrancy; concurrent close; after-close; runtime shutdown;
call-scoped first failure; swallowed `SifrCallbackError`; Python primary plus
secondary handler evidence; owner failure observation; cancellation in both
directions; and proof that neither executor blocks.

## Likely Implementation Surfaces

- `crates/sifr_type_system/src/types/`, `union.rs`
- `crates/sifr_ir/src/python_interop.rs`
- `crates/sifr_lowering/src/lower/python_interop/` and callable expression/type
  lowering
- reusable sendability/capture analysis under `crates/sifr_lowering/src/lower/`
- `crates/sifr_codegen/src/python_interop_*`, function scope/callable emission,
  opaque wrapper emission, and `PythonInteropPlan`
- `crates/sifr_runtime/src/python/callbacks/`, shutdown hooks, async runtime,
  object/semantic-close integration, and registered exceptions
- `crates/sifr_stdlib/src/python.rs` compatibility adapters
- `verification/areas/python_interop/`, `demos/m9_demo/`, capability ledger,
  public/internal docs, roadmap, tracker, and review records

## Review And Closure

Each wave receives design alignment, focused tests, both authoritative local
gates, a frozen-diff manual review, merge, and tracker status/link updates.
After Wave 3, the complete M9 implementation receives one comprehensive Fable
High milestone review across all merged wave diffs; actionable findings are
fixed and re-reviewed before its tracker checkbox is closed.
