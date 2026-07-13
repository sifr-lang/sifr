# M8 Design: Async Python Context Managers

## Objective

Activate `@python.context.aenter`, `@python.context.aexit`, and
`cleanup=async_context` on the M7 application-owned asyncio runtime without
reusing native `AsyncExitCause` as the Python cause authority. Every acquired
manager exits exactly once for fallthrough, return, loop control, typed error,
timeout, cancellation, and runtime-fault boundaries; body cancellation cannot
drop the async exit future.

M8 lands in two independently reviewed PR waves. Wave 1 completes and tests the
control-flow, cancellation, request, and lifecycle substrate behind the existing
`SIFR-PYRES-0002` public gate. Wave 2 atomically lifts the three M8 reservations,
adds compiled evidence, and updates every capability/documentation consumer.

## Existing Authorities To Reuse

- M5 synchronous Python contexts own the normative `python.ExitCause`, original
  `PythonError` replay, `SifrBoundaryError`, suppression, secondary-evidence,
  scoped-entered-borrow, and exact-once manager-consumption rules.
- M7 owns the one-loop submission registry, typed recursive conversion, exact
  asyncio-task cancellation, terminal waiting, shutdown drain, and native
  cancellation resumption.
- The compiler's native `HirAsyncWithKind::UserDefined` path is not the Python
  cause authority. It currently supports normal/return cleanup only and is an
  implementation substrate to generalize, not a semantic shortcut for Python.
- `PythonCleanupPolicy::AsyncContext` and the `AsyncContextOnly` must-use
  obligation already exist behind `SIFR-PYRES-0002`; no second ownership kind is
  introduced.

## Wave 1: Gated Async-Context Substrate

### Declaration Contract Behind The Gate

1. Parse and validate async opaque methods:
   - `@python.context.aenter(Self.__aenter__)` requires `async def`, borrowed
     `self`, no following parameters, and
     `Result[T, PythonError]`, where `T` has a direct async conversion;
   - `@python.context.aexit(Self.__aexit__)` requires `async def`, `own self`,
     exactly one compiler-constructed `python.ExitCause`, and
     `Result[python.ExitDecision, PythonError]`;
   - both retain `PythonInteropEffect::Async`, structured `Self` targets, and
     ordinary declaration metadata;
   - direct source calls to the declared exit remain rejected.
2. `cleanup=async_context` requires exactly one aenter and one consuming aexit.
   No sync enter/exit substitution is accepted. A distinct opaque entered
   result is valid only with `cleanup=drop`; self identity becomes a scoped
   borrow. Aggregates cannot hide semantic cleanup obligations.
3. Valid declarations still emit `SIFR-PYRES-0002` in Wave 1. Invalid shapes
   emit their stable `SIFR-PYCTX-*`/conversion diagnostic rather than being
   hidden by the reservation: lowering parses and validates first, emits the
   shape/obligation error alone when invalid, and emits the reservation only
   after a valid internal declaration has been retained. Wave 1 tests both
   diagnostic orders from real source. It therefore prepares one contract
   without exposing partial executable syntax.

### Dedicated HIR

Add a Python-specific async-with variant carrying:

- the manager expression and manager class;
- entered, enter-error, and exit-error types;
- whether the entered opaque result is the context-scoped borrow;
- enough source-level active-error type information to classify a concrete body
  `Err` before native cause erasure.

Lowering selects this variant only for an opaque class with
`cleanup=async_context`. It consumes the manager obligation at entry, installs
the same scoped-borrow restrictions as M5 for the entire async body, and rejects
unentered abandonment, duplicate entry, reuse, and invalid distinct entered
resources. Native user-defined async context managers retain their own
`AsyncExitCause` protocol and never provide Python classification.

### Cancellation-Unwind Substrate

Generated async-context execution uses a parent/child cancellation handoff
rather than a catchable sentinel or a second task representation.

1. Before running the acquired body, claim the current task's cancellation
   carrier for the context scope. The claim hook requests a fresh child carrier.
2. Run the body future under a task-local scope containing that child carrier.
   Race the pinned body future against a sticky private cancellation notification
   bound as the child's fallback, using a **biased select whose cancellation arm
   is always polled first**:
   - during an ordinary native await, cancellation notifies the race and drops
     only the body future;
   - during a Python await, the child carrier first cancels and terminally joins
     the exact asyncio task; M7 then resumes the child fallback, so the race does
     not select cancellation until Python `finally` is terminal. M7's
     `propagate()` then reaches its self-waking pending yield; control returns to
     the outer select, the already-sticky cancellation arm wins before the body
     is repolled, and dropping the body future prevents `propagate()` from ever
     returning its deliberate malformed-fallback runtime error. A regression
     test makes both branches ready together and proves the biased cancellation
     arm wins.
3. The enclosing task is not aborted while the parent claim is held. After body
   cancellation wins, install a fresh, unrequested cleanup carrier as the
   **task-local** and also pass it to the `__aexit__` submission. This masks the
   original request for the exit and for any generated nested Python operation
   reached while producing that exit. Shutdown remains independent of the
   carrier: the M7 registry directly cancels and terminally drains the exact
   Python task before stopping the loop.
4. After exit and evidence recording, release the parent claim and resume the
   parent's already-bound native fallback through the M7 idempotent resume API.
   Cancellation/timeout remains primary; an exit error is secondary evidence.
   Only an unrecoverable runtime fault can replace that cause under the existing
   task boundary rules. A missing/no-op fallback yields the same explicit
   bounded runtime failure as M7, never a hang.
5. Nested async contexts create nested child carriers. Every claim lease is
   released in lexical LIFO order. The innermost cleanup completes, its claim on
   the enclosing body carrier is dropped, and that carrier's fallback is resumed
   while the next outer claim is still held; the outer biased select then drops
   the completed inner future and performs its cleanup. This repeats outward and
   prevents a cancellation request from skipping or delaying an outer exit.
   Mixed sync/async nesting keeps the same lexical LIFO ordering.
6. Every generated async entrypoint that may contain this construct has an
   ambient carrier. Existing Sifr-spawned tasks already install one. Wave 1 wraps
   generated async `main` in a task-local root carrier before its user body, so
   direct top-level use cannot observe `None`. Direct Tokio handles are not a
   Sifr-language surface; all user-visible cancellation authority flows through
   the carrier. A missing carrier after this invariant is an explicit internal
   runtime error before acquisition, never an unmasked fallback path.

The carrier state remains closed and generation-checked. Add no Tokio
dependency to `sifr_runtime` production features; Tokio-specific task-local and
select glue remains generated-application code.

### Owned-Loop Enter And Exit Operations

Extend compiler-private typed async requests instead of calling synchronous
context adapters or `asyncio.run`:

- generated code retains the sole manager `ObjectHandle`; async enter leases it
  through a borrowed receiver request and never consumes it. The existing
  recursive converter may succeed or fail, but the manager remains available
  for the mandatory exit after a successful `__aenter__` invocation;
- async exit consumes that `ObjectHandle` through the existing
  `PythonAsyncObject::semantic_close` ownership channel. Its request input is a
  closed private cause enum: normal/control flow, an Arc-backed cloned original
  `PythonError` replay capability, or redacted `SifrExitCause`;
- on the loop thread, exit materializes `(None, None, None)`, the original live
  Python exception triple, or a registered `SifrBoundaryError`, invokes
  `__aexit__`, requires an awaitable, and converts Python truthiness into the
  closed `PythonExitDecision`;
- the terminal has a distinct `PythonTerminalValue::ExitDecision` variant, not
  a bool encoded as `PythonAsyncValue`. A hidden
  `submit_async_context_exit(...) -> Result<PythonExitDecision, PythonError>`
  shares the M7 setup/terminal engine but rejects every other terminal variant;
- terminal success calls `finish_semantic_close(true)` and closes the manager;
  setup failure, callback panic, await/conversion failure, cancellation,
  shutdown, request drop, or any wrong terminal variant calls
  `finish_semantic_close(false)` and poisons it. The Arc-held request's existing
  idempotent completion/drop path owns this exact-once transition.

The request owns or leases only compiler-owned sealed handles. No raw Python
pointer crosses threads. `PythonError::clone` clones the replay
`ForeignObject`'s Arc, so the primary and request are co-owners of one pinned
triple rather than two releases. The request clone drops after exit; suppression
drops the primary at the block boundary, propagation returns it to the caller,
and nested exits may clone it again. The final Arc owner alone releases the
triple. Runtime initialization registers the shared `SifrBoundaryError` type
before the owned loop starts, and async exit resolves that same `OnceLock` type
under the loop-thread GIL.

### Concrete Body Outcome And Suppression

Codegen executes the async body in a pinned async closure. The closure returns
the same typed body shape M5 uses,
`Result<Result<Option<ReturnValue>, LoopControl>, ActiveError>`, where
`LoopControl` distinguishes break and continue. The surrounding biased select
adds one outer `Cancelled` branch. `?` and explicit error propagation terminate
only the async closure; after the select, generated code invokes exit and then
replays the stored return/loop/error action in the enclosing function. The
shared rewriter handles nested statements but never rewrites control flow owned
by an inner loop/context.

- normal/control-flow outcomes call exit with `None`; truthiness cannot alter
  the pending control action;
- an originating `PythonError` with replay calls exit with that exact triple;
  `Suppress` continues after the block, while `Propagate` preserves the error;
- ordinary Sifr errors, timeout, cancellation, and runtime fault use
  `SifrBoundaryError`; truthy decisions are recorded and ignored;
- exit failure on a normal/control-flow outcome is primary;
- exit failure with an active body error, timeout, cancellation, or fault is
  attached as secondary evidence and the original cause remains primary;
- if entered-value conversion fails after successful aenter, aexit still runs
  exactly once with an ordinary Sifr conversion cause.

Lowering discharges `AsyncContextOnly` when it consumes the manager into the
dedicated async-with HIR. Generated exit is then present on every terminal HIR
outcome—fallthrough, return, break, continue, typed error, timeout,
cancellation, runtime fault, and post-enter conversion failure—and runtime
semantic-close completion proves the single dynamic discharge. Focused tests
cover each branch rather than inferring coverage from the normal path.

Share control-flow rewriting helpers with M5 by responsibility rather than
copying the synchronous emitter. Native `HirAsyncWithKind::UserDefined` and its
`AsyncExitCause` remain out of scope for semantic generalization in M8; the
Python variant does not route through them. Shared helpers are syntax-neutral
rewriters only. Mixed-order tests use the native path's already-supported
normal/return cases and the complete dedicated Python path.

Split before implementation:

- lowering: move the current context module to
  `lower/python_interop/context/{mod.rs,borrows.rs,declarations.rs}` and put M8
  source selection/obligation logic in `context/async_with.rs`;
- codegen: replace the 675-line emitter with
  `stmt_support_emitter/python_context/{mod.rs,outcome.rs,sync.rs,async_context.rs,cancellation_scope.rs}`;
- runtime: keep shared boundary evidence in `context_ops.rs`, put specialized
  request/cause/terminal work in `python/async_context.rs`, and factor only the
  minimum common submission engine from `async_declaration.rs`.

### Wave 1 Validation

- declaration shape/substitution/cleanup/obligation lowering tests while the
  valid public form remains reserved;
- real-source gated tests proving invalid shape and unentered obligation errors
  are emitted without being masked by the reservation;
- constructed-HIR codegen tests for every body outcome, nested ordering,
  original Python replay, ignored truthy Sifr causes, conversion-after-enter,
  and exact-once consumption;
- runtime tests for enter/exit truthiness, replay, SifrBoundaryError,
  success/poison, shutdown, and typed conversion;
- generated cancellation-scope tests for pre-body, native-await, Python-await,
  nested cleanup, exit failure, and missing/no-op fallback behavior;
- package formatting, file-size, HIR maintainability, and authoritative
  create-PR validation before review and merge.

## Wave 2: Atomic Activation And Evidence

1. Remove only the `ContextAsyncEnter`, `ContextAsyncExit`, and
   `cleanup=async_context` reservations. Retain M9-M12 reservations and every
   M8 shape, conversion, ownership, and direct-call diagnostic.
2. Wire source `async with` to the dedicated Python variant and replace gated
   positives with executable lowering/codegen/runtime coverage.
3. Add `aiosqlite>=0.20,<1` to the locked Python interop environment and compile
   an offline database/session package against real `aiosqlite` over a local
   temporary or in-memory SQLite database. It must prove one loop identity,
   normal value conversion,
   originating Python suppression, ordinary Sifr unsuppressed truthiness,
   cancellation/finally/exit ordering, exit failure evidence, nested sync/async
   LIFO order, and exact-once manager exit. No external service or network is
   permitted.
4. Register the suite unconditionally in create-PR, merge, nightly, and release
   profiles. Measure and record any required blocking budget increase; no
   manual or conditional evidence path is accepted.
5. Add `demos/m8_demo` that invokes the compiled suite and prints real binary
   output. Activate the existing `async-context` capability row with positive,
   negative, cleanup, cancellation, and live compiled owners.
6. Update public docs, both durable Python interop architecture documents,
   runtime architecture, verification README/exit evidence, roadmap, and review
   records. The tracker-only PR after merge records the final PR link and M8
   checkbox.

## Likely Implementation Surfaces

- `crates/sifr_ir/src/hir_nodes.rs`
- `crates/sifr_lowering/src/lower/async_with.rs`
- `crates/sifr_lowering/src/lower/python_interop.rs` and
  `lower/python_interop/context/{mod.rs,borrows.rs,declarations.rs,async_with.rs}`
- `crates/sifr_codegen/src/stmt_support_emitter/python_context/` with
  `mod.rs`, `outcome.rs`, `sync.rs`, `async_context.rs`, and
  `cancellation_scope.rs`
- `crates/sifr_codegen/src/preamble/task_cancellation_runtime.rs`
- `crates/sifr_runtime/src/cancellation.rs`
- focused modules under `crates/sifr_runtime/src/python/` for async-context
  request setup/terminal conversion rather than growing `async_declaration.rs`
- `verification/areas/python_interop/`, `demos/m8_demo/`, public/internal docs,
  roadmap, and capability ledger

## Review And Closure

Each wave receives design review, focused tests, the authoritative local
create-PR gate, a frozen-diff Opus review until satisfied, an actual PR review
until satisfied, merge, and a tracker update with its merged link. After Wave 2
the full M8 implementation receives a milestone-level review before the M8
tracker checkbox is closed.
