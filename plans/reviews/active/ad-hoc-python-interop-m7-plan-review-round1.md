# M7 Owned Asyncio Runtime And Async Declarations — Plan Review, Round 1

## Scope

Review of the proposed five-wave implementation plan for milestone M7 added
to `plans/issues/active/ad-hoc-declaration-first-python-interop.md`. The
review compares each wave against M7's task list, acceptance criteria, and
validation matrix, against the durable contracts in
`internal_docs/python_interop_declaration_architecture.md` and
`internal_docs/python_interop_protocol_architecture.md`, and against the
current implementation surfaces the waves must touch. It is a plan review
only; no source files were modified.

## Reference Points Used

- Phase Delivery Rule
  (`plans/issues/active/ad-hoc-declaration-first-python-interop.md` around
  lines 114–128): "Internal substrate may land before its public syntax, but
  no PR may expose a temporary public grammar or a second runtime
  representation. Each milestone ends with one production path for the
  behavior it activates. When a milestone replaces an existing authority or
  representation, it updates every consumer, fixture, diagnostic, and
  document in the same merge unit."
- M6's own wave sequence (same file, lines 375–432) as the accepted pattern:
  substrate + gating first (Waves 1–2, syntax kept behind
  `SIFR-PYRES-0002`), atomic activation and target rewrite in one merge
  unit (Wave 3), deployment/evidence closure after.
- Async architecture: `internal_docs/python_interop_protocol_architecture.md`
  §"Async Python Calls", §"Cancellation And Shutdown", §"Cleanup Policies";
  `internal_docs/python_interop_declaration_architecture.md` §"Blocking And
  Async".
- Async concurrency contract: `internal_docs/async_concurrency_model.md`
  §"Cancellation Vocabulary" (lines 160–182).
- HIR/lowering surfaces:
  - `crates/sifr_ir/src/python_interop.rs` — `PythonInteropEffect::{BlockingIo,
    Async}` already exists but no code path constructs `Async`.
  - `crates/sifr_lowering/src/lower/python_interop.rs` —
    `PythonInteropStubBody::Bodyless` exists; every current declaration
    hard-codes `effect: PythonInteropEffect::BlockingIo`; `Coroutine` still
    routes through `SIFR-PYRES-0002`; `parse_opaque_class` gates
    `async_close`/`async_context` behind `SIFR-PYRES-0002`.
  - `crates/sifr_lowering/src/lower/async_await.rs:44-58` and
    `crates/sifr_lowering/src/lower/async_effects.rs:20-48` — the
    `NoSuspend` gate keyed off `collect_async_suspension_summaries` and
    `ctx.async_functions`.
  - `crates/sifr_lowering/src/lower/mod_impl.rs:216-231` — inserts every
    `async def` into `ctx.async_functions` and stamps `WorkloadKind::BlockingIo`
    on any function carrying a Python interop decorator (this stamps
    coroutines as `BlockingIo` today).
- Runtime surfaces:
  - `crates/sifr_runtime/src/python/coroutine_ops.rs` — the current
    `run_coroutine_blocking` calls `asyncio.run(...)` per invocation.
  - `crates/sifr_runtime/src/python.rs` — initialization, guard drop,
    `validate_shutdown`; no loop thread or submission registry yet.
- Task supervisor codegen (e2e cache group
  `crates/sifr/target/sifr_e2e_cache/groups/faf476fa59c49b20/src/main.rs`,
  lines ~393–658, representative of generated code today) — `Task::cancel`,
  `Task::cancel_and_join`, `Task::__sifr_timeout`, and
  `TaskScope::__sifr_join_all` implement cancellation exclusively via Tokio
  `abort_handle.abort()`. `abort()` drops the child future, so the child
  cannot itself `await` a Python task's terminal state before the parent
  scope proceeds. `crates/sifr_runtime/src/interop.rs:504` and
  `crates/sifr_runtime/src/http.rs:{227,271}` show the same abort-only
  pattern in runtime-owned tasks.

## Blocking Findings

### 1. Wave ordering conflicts with the phase Delivery Rule.

Wave 1 is written as an activation wave:

> "Activate `@python.coroutine(path)` only on bodyless `async def`
> functions… preserve `PythonInteropEffect::Async`… Route ellipsis through
> the interop `Bodyless` path before normal async body lowering and
> `NoSuspend` checking. Reject sync/async decorator substitution, borrowed
> async results, non-consuming async close, and unsupported cleanup shapes
> with stable diagnostics. Activate `cleanup=async_close` only when a
> consuming `@python.coroutine(Self.<member>)` close declaration satisfies
> the sealed opaque lifecycle contract."

Yet the runtime pieces that make that grammar executable land later: the
owned loop and submission API in Wave 2, the generated async wrappers in
Wave 3, structured cancellation in Wave 4, and consuming-ownership transfer
plus poison-on-cleanup-failure in Wave 5. Between Wave 1 and Wave 5, a user
`@python.coroutine` declaration or `cleanup=async_close` opaque would parse
and lower but would have no end-to-end production path. That is exactly the
"temporary public grammar" the phase Delivery Rule forbids. M6's own wave
plan (same phase file) demonstrates the correct pattern: Waves 1–2 land
substrate while keeping `bridge.*` behind `SIFR-PYRES-0002`; Wave 3
atomically rewrites targets and lifts the gate in one merge unit.

Failure scenario: a PR at the Wave-1 boundary is merged in isolation.
`@python.coroutine(module.func)` on `async def` compiles through
`collect_python_interop_declarations` (Coroutine no longer hits
`PYRES_UNIMPLEMENTED_DECLARATION`), reaches codegen with
`PythonInteropEffect::Async`, and either produces no wrapper (there is no
async wrapper emitter yet in `sifr_codegen`) or emits code calling
symbols that Wave 2 has not yet added, breaking `cargo build`. Either way
Wave 1 fails the "one production path" invariant on its own.

Suggested resolution: keep `Coroutine` and the `async_close`/`async_context`
cleanup atoms behind `SIFR-PYRES-0002` through Waves 1–4 and lift the gate
atomically in the same merge unit as the wave that first produces working
generated wrappers (or fold Waves 1 + 2 + 3 into a single activation merge
unit, mirroring M6 Wave 3). The Wave-1 language of "activate" should be
replaced with "prepare frontend contracts and diagnostics behind the
existing PYRES-0002 gate".

### 2. Cancellation carrier vs. the current Tokio-abort task supervisor is
under-specified and may exceed one wave.

Wave 4 says:

> "Introduce a cancellation carrier understood by generated Sifr task
> supervisors and Python submissions; do not treat Tokio task abortion as
> a completed Python cancellation. Cancellation-before-start and
> cancellation-in-flight must request cancellation of the exact asyncio
> task and await its terminal state."

The current implementation of every Sifr cancellation site is Tokio
`abort_handle.abort()` (`Task::cancel`, `Task::cancel_and_join`,
`Task::__sifr_timeout`, `TaskScope::__sifr_join_all` fail-fast path, plus
runtime-owned tasks in `sifr_runtime/src/http.rs` and
`sifr_runtime/src/interop.rs`). `abort()` drops the child future
synchronously; the child cannot `await` for the Python task's terminal
state before the parent scope proceeds. Meeting the M7 acceptance criterion
"Sifr cancellation does not complete before Python `finally` cleanup" for
cancellation-in-flight during ordinary program flow (not only at process
shutdown) therefore requires either:

- (a) rewriting Sifr's task supervisor codegen so tasks holding Python
  coroutine futures use a cooperative cancellation token rather than
  `abort()`. That touches `task.spawn`, `task.spawn_blocking`,
  `task_scope`/`task_group` fail-fast, `task.timeout`, `task.race`,
  `task.select`, and every runtime-owned `.abort()` site above. It also
  interacts with the `async_concurrency_model.md` split between "active
  cancellation signal" and "materialized cancellation evidence", which the
  current codegen has not yet implemented.
- (b) redefining "await terminal state" to happen only in the shutdown
  registry, so at the awaiting Sifr task's completion point the Python
  task's terminal state has not necessarily been reached — which violates
  the M7 acceptance criterion outside shutdown.
- (c) some hybrid where the wrapper's `Drop` schedules
  `loop.call_soon_threadsafe(task.cancel)` and hands the future off to a
  runtime-owned pending set that shutdown drains, accepting (b) at the
  caller and (a) at shutdown.

Wave 4 does not identify which option is chosen. Option (a) is likely
larger than one wave and would benefit from being decomposed or paired with
an earlier substrate wave that lands the cancellation-carrier primitives in
the concurrency runtime before M7's wrappers depend on them. Options (b)
and (c) are consistent with a single M7 wave but weaken the acceptance
criterion and should be documented as the accepted semantics if chosen.

Failure scenario without resolution: the Wave-4 PR either lands (b)/(c)
silently — presenting to reviewers as "terminal-state waiting" while in
practice cancellation-in-flight can still return `TaskResult::Cancelled`
before Python `finally` runs — or attempts (a) and grows into a
cross-cutting task-supervisor rewrite whose Rust diff dwarfs the M7
substrate diff, violating the "small, reviewable" workflow rule and
`AGENTS.md` "keep changes focused on the requested milestone/issue".

### 3. `cleanup=async_close` behavior is split across Waves 1 and 5.

Wave 1 activates `cleanup=async_close` at the frontend as soon as the
opaque class declares a consuming `@python.coroutine(Self.<member>)` close.
Wave 5 completes the runtime contract: "Transfer consuming ownership
before submission; close exactly once, poison on cleanup failure, and
reject reuse, duplicate close, or abandonment of an `async_close`
obligation." Between Wave 1 and Wave 5, `cleanup=async_close` is either
non-executable (finding 1) or partially enforced: the linear must-use
obligation from M4 exists at the frontend but the runtime does not poison
on failure or reject double close. Because the whole reason to introduce
async declarations is to make `async_close` sound, the plan should either
fold Wave 1's `cleanup=async_close` activation into the same merge unit as
Wave 5's completion (preferred), or explicitly document that Wave 1 lands
`cleanup=async_close` behind `SIFR-PYRES-0002` and that Wave 5 lifts the
gate. The current wording of Wave 1 ("Activate `cleanup=async_close` only
when …") reads as public activation.

Failure scenario: a user program with `cleanup=async_close` compiles after
Wave 1 lands but before Wave 5, calls the declared `aclose()`, encounters a
Python exception, and either continues to use the handle (no poison) or
suffers a runtime panic where the ownership contract expected but the
runtime did not enforce the closed/poisoned transition.

## Non-Blocking Suggestions

### 4. Conditional loop startup vs. sync-only applications.

`internal_docs/python_interop_protocol_architecture.md` line 59: "A
generated application containing any async Python declaration owns
exactly one CPython asyncio loop on one dedicated OS thread." Wave 2
states "Start one loop on one named OS thread after CPython and
bridge-loader setup" with no gating on the target's declaration graph.
`crates/sifr_driver/src/build/python_interop.rs` currently constructs a
single `PythonInteropPlan` and does not distinguish async-bearing targets.
Wave 2 (or Wave 3) should explicitly state that the driver only wires the
loop bootstrap when the resolved package graph contains at least one
`PythonInteropEffect::Async` declaration (function, method, factory, or
async context enter/exit), so sync-only apps do not pay the OS-thread
cost.

### 5. `NoSuspend` bypass mechanism is implicit.

`collect_async_suspension_summaries`
(`crates/sifr_lowering/src/lower/async_effects.rs:20-48`) walks every
async top-level function collected by `mod_impl.rs:216-220` and would
classify a `@python.coroutine async def` with an ellipsis body as
`NoSuspend`, triggering `ASYNC_AWAIT_NO_SUSPEND` on any awaiting caller
via `lower_await` (`async_await.rs:44-58`). Wave 1 says "Route ellipsis
through the interop `Bodyless` path before normal async body lowering and
`NoSuspend` checking" but does not name the mechanism. Options: (i)
exclude Bodyless-stub coroutine functions from `ctx.async_functions`; (ii)
teach the summarizer to treat Bodyless async stubs as `Suspends`; (iii)
suppress the `NoSuspend` diagnostic for functions with a
`PythonInteropEffect::Async` declaration. Wave 1 should pick one and
name it, so reviewers can check the correct site.

### 6. Concurrent one-loop identity proof for typed wrappers is deferred to
Wave 5.

Wave 2 proves "repeated and concurrent raw calls use one loop/thread
identity" for the raw submission path. Wave 3 activates typed wrappers but
does not repeat the identity check. Wave 5 collects a "concurrent one-loop
identity proof". This is acceptable but Wave 3 should include a small
end-to-end identity assertion (two typed `@python.coroutine` calls
concurrently observing the same loop id) so that the Wave-3 PR itself does
not require the reviewer to trust the raw-path proof.

### 7. Callback-shutdown insertion point in Wave 4.

M7's phase-intro task list requires the loop to "stop it after registered
async cleanup and callback shutdown." Callbacks are M9; Wave 4 defines the
M7 shutdown as "stops admission, cancels all registered work, awaits
Python `finally` and registered async cleanup, stops the loop, and joins
the OS thread." Wave 4 should explicitly note where callback shutdown
(M9) will insert in this sequence so that M9 does not have to reshape the
shutdown ordering later.

### 8. Raw-API `blocking_io` semantics after Wave 2 rewiring.

`stdlib/_sifr/python.sifr:380` and `stdlib/sifr/python.sifr:779` expose
`py_run_coroutine_blocking` as `blocking_io`. Wave 2 replaces the current
`asyncio.run` implementation with owned-loop submission. The `blocking_io`
classification remains correct — the sync caller still waits for a Python
future's terminal state — but Wave 2 should state that the classification
is preserved and that async Sifr code continues to require explicit
offload (per the M3 acceptance criterion "Sync declarations cannot be
called directly from async code without explicit Sifr offload"). This is
mostly a doc/diagnostic note but avoids ambiguity when the intrinsic is
rewired.

## Coverage Cross-Check

| M7 Task / Acceptance / Validation                                            | Wave                             | Status                                   |
| ---------------------------------------------------------------------------- | -------------------------------- | ---------------------------------------- |
| One generated-application-owned loop on a dedicated OS thread                | Wave 2                           | Covered; conditional startup gap (§4)    |
| Start after CPython/bridge init; stop after async cleanup + callback shutdown | Wave 2 (start), Wave 4 (stop)    | Covered; callback ordering hook (§7)     |
| `@python.coroutine(path)` on async def for functions/factories/methods       | Wave 1 (frontend), Wave 3 (codegen) | Covered; ordering finding (§1)          |
| Convert inputs, invoke, await, convert outputs on loop thread                | Wave 3                           | Covered                                  |
| Structured bidirectional cancellation; terminal-state waiting; `CancelledError` mapping; suppression | Wave 4 | Under-specified vs. Tokio abort (§2) |
| Bodyless routing before body lowering / `NoSuspend`                          | Wave 1                           | Covered but mechanism implicit (§5)      |
| Raw coroutine API on owned loop; remove per-call `asyncio.run`               | Wave 2                           | Covered; classification note (§8)        |
| Opaque results non-send                                                      | Wave 3                           | Covered                                  |
| Consuming `cleanup=async_close`; poison-on-cleanup-failure                   | Wave 1 (frontend), Wave 5 (runtime) | Split across waves — activation split (§3) |
| Sync/async substitution rejected                                             | Wave 1                           | Covered                                  |
| Async success / Python failure / conversion failure matrices                 | Wave 5                           | Covered                                  |
| Cancellation-before-start / in-flight / suppression matrices                 | Wave 5                           | Covered (semantics depend on §2)         |
| Shutdown matrices                                                            | Wave 4 (diagnostics), Wave 5 (matrix) | Covered                              |
| Async-close success/failure/poison/use-after-close                           | Wave 5                           | Covered                                  |
| Reject abandonment of `cleanup=async_close`                                  | Wave 5                           | Covered                                  |
| Concurrent one-loop identity                                                 | Wave 2 (raw), Wave 5 (typed)     | Wave 3 identity assertion missing (§6)   |
| Compiled httpx-style example + demos/m7_demo                                 | Wave 5                           | Covered                                  |
| Doc/roadmap/review/checkbox/PR-link updates                                  | Wave 5                           | Covered                                  |

## Verdict

The five-wave decomposition covers every task, acceptance criterion, and
validation item in the M7 spec at the level of stated intent. However
findings §1, §2, and §3 above are actionable plan-level defects that
should be resolved before the plan is executed:

- §1: reorder or reword so no wave lands public activation of
  `@python.coroutine`/`cleanup=async_close` ahead of the substrate that
  makes those forms end-to-end executable, matching M6's substrate-first
  pattern and the phase Delivery Rule.
- §2: specify the reconciliation between the M7 cancellation carrier and
  the current `abort()`-based task supervisor, or decompose Wave 4 to land
  the required supervisor primitives explicitly.
- §3: fold `cleanup=async_close` activation into the same merge unit as
  its runtime completion, or state clearly that Wave 1 leaves it gated
  behind `SIFR-PYRES-0002`.

Findings §4–§8 are non-blocking refinements.

NOT SATISFIED
