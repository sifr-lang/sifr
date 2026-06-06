# Ad Hoc Phase Execution: Production Concurrency And Runtime Stdlib Parity

Phase contract: [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md)

Status: draft

## Scope Split

This ledger tracks:

- `queue`, `asyncio.Queue`
- `subprocess`, `asyncio.subprocess`
- `concurrent.futures`, `multiprocessing`
- `contextlib`, `warnings`, `signal`

Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate-execution.md](./ad-hoc-production-network-http-platform-substrate-execution.md). Text and internationalization parity remains in [ad-hoc-production-text-i18n-stdlib-parity-execution.md](./ad-hoc-production-text-i18n-stdlib-parity-execution.md).

## Milestone Checklist

- [ ] `milestone_concurrency_runtime_0`: CPython Inventory, Error Mapping, And Effect Classification
- [ ] `milestone_concurrency_runtime_1`: Asyncio Core, Queue, And Async Queue Parity
- [ ] `milestone_concurrency_runtime_2`: Subprocess, Popen, Async Subprocess, And Signals
- [ ] `milestone_concurrency_runtime_3`: Concurrent Futures And Thread Executors
- [ ] `milestone_concurrency_runtime_4`: Typed IPC, Process Pools, And Multiprocessing
- [ ] `milestone_concurrency_runtime_5`: Contextlib, Warnings, And Runtime Ergonomics
- [ ] `milestone_concurrency_runtime_6`: Integration, Documentation, And Production Gate

## Planning Reviews

- Inherited from the original combined stdlib parity planning review:
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-1d.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-2.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-3.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-4.md`
- Final combined review result before split: `PASS`.
- Split-phase Claude review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-1-constrained.md`
  - Result: `FAIL`; cross-phase dependency and ownership gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-2-constrained.md`
  - Result: `FAIL`; remaining ownership/disposition gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-3-constrained.md`
  - Result: `FAIL`; remaining sequencing/error-surface gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-4-constrained.md`
  - Result: `FAIL`; remaining async-context/file/default-encoding/thread-error gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-5-constrained.md`
  - Result: `FAIL`; remaining contextvars/future-cancellation/open-policy/worker-typing gaps were remediated across the split phase docs.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-6-constrained.md`
  - Result: `FAIL`; remaining executor map/timeout/cancellation/heterogeneous-future gaps and text-wrapper gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-7-constrained.md`
  - Result: `FAIL`; remaining executor state-machine, `StringIO`, `threading.local`, and codec error-handler gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-8-constrained.md`
  - Result: `FAIL`; remaining Future.cancel, wait partition, executor.map timeout, and text handler gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-9-constrained.md`
  - Result: `FAIL`; remaining executor deadline/cancellation/wait fallback and codec handler classification gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-10-constrained.md`
  - Result: `FAIL`; remaining handler enforcement, partial iteration, FIRST_EXCEPTION trigger, and shutdown pending/running gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-11-constrained.md`
  - Result: `FAIL`; remaining future ownership/lifecycle and shutdown observability gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-12-constrained.md`
  - Result: `FAIL`; remaining `wait()` ownership, cancelled result typing, and incremental codec finalization gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-13-constrained.md`
  - Result: `FAIL`; remaining `gather()` ownership/result typing, `as_completed()` timeout signaling, codec recoverable-error, and `TaskGroup` aggregation gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-14-constrained.md`
  - Result: `FAIL`; remaining network error hierarchy, TLS socket ownership, workload classification, handler model, concurrency gate, text decision, and review-gate gaps were remediated.
- Split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-15-constrained.md`
  - Result: `FAIL`; remaining TLS wrap failure-state, `signal.pause`, and text-i18n dependency milestone gaps were remediated.
- Final split-phase Claude follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-16-constrained.md`
  - Result: `PASS`; no material implementation-blocking gaps remained.
- Final implementation-readiness scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-17-final-readiness.md`
  - Result: `PASS`; all three phases were implementation-ready, with one editorial ledger-title mismatch remediated in this ledger.
- No-legacy readiness scans:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-18-no-legacy-readiness.md`
  - Result: `FAIL`; text/i18n stale dynamic-handler and implicit-open wording were remediated.
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-19-no-legacy-readiness.md`
  - Result: `PASS`; no remaining backward-compatibility, legacy-support, deprecated-behavior, shim, bridge-alias, or fallback decisions remained.
- Namespace consistency scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-20-sifr-namespace-readiness.md`
  - Result: `PASS`; all three phase docs consistently use canonical `sifr.*` stdlib imports and reject bare CPython stdlib aliases.
- Required follow-up: run a dedicated external review after M0 inventory and before M1 implementation, because this phase is now independently scoped.

## Planning Review Remediation Retained In This Phase

- [x] Define multiprocessing start-method, typed IPC, shared-memory, and ownership constraints.
- [x] Narrow unsafe signal handler registration to `unsupported` for this phase.
- [x] Add milestone dependency graph.
- [x] Add shared concurrency/runtime error mapping requirement.
- [x] Expand `concurrent.futures`, multiprocessing, warnings, and global-state requirements.
- [x] Name Tokio as the backing async runtime for this phase and require concrete feature expansion in M0.
- [x] Tie `multiprocessing.Pool` to the same typed IPC gate as `ProcessPoolExecutor`.
- [x] Clarify that class-based context managers are independent APIs, not fallback implementations for generator decorators.
- [x] Mark `contextmanager` and `asynccontextmanager` unsupported in this phase, with a revisit rule for a future CPython-compatible generator semantics phase.
- [x] Add explicit cross-phase dependency contract for text/i18n and network/web consumers.
- [x] Clarify that core `asyncio` scheduler/task helpers are prior async-model infrastructure and this phase owns only `asyncio.Queue`/`asyncio.subprocess` compatibility additions.
- [x] Assign private thread-pool substrate for `ThreadPoolExecutor` to this phase while keeping public `threading` module parity out of scope.
- [x] Add explicit terminal expectations for `signal.getsignal`, `signal.pause`, and `pthread_sigmask`.
- [x] Assign the `asyncio` task/wait/timeout/synchronization parity closure audit consumed by queues/subprocesses to `milestone_concurrency_runtime_1`.
- [x] Add concrete `ThreadPoolExecutor` behavior dispositions for `max_workers`, `submit`, `map`, `shutdown(cancel_futures)`, `thread_name_prefix`, `initializer`, `initargs`, and public worker-thread inspection.
- [x] Require M1's asyncio closure audit to finish before `asyncio.Queue` or `asyncio.subprocess` fixtures are marked conformant.
- [x] Mark `ThreadPoolExecutor` `initializer`/`initargs` unsupported until Sifr has a formal user-visible cross-thread callable sendability contract.
- [x] Add rationale that `TaskGroup` maps to Sifr's fixed structured-concurrency executor without exposing event-loop policies or callback transports.
- [x] Mark `contextvars` parity and implicit per-task context propagation out of scope, with M1 fixtures proving unsupported context propagation is diagnosed or waived rather than silently ignored.
- [x] Add ThreadPoolExecutor worker error propagation contract: worker failures return typed `ExecutorError` evidence and never panic or disappear.
- [x] Mark all `contextvars` usage unsupported in this phase, including single-task reads/writes and cross-task propagation.
- [x] Add `Future.cancel` and cancelled futures as a distinct typed terminal state.
- [x] Replace ambiguous worker failure wording with a compiler contract: worker callables must type-check as `Callable[..., T]` or `Callable[..., Result[T, E]]`; other user-visible fallible paths are compile-time errors and runtime boundary failures become typed `ExecutorError::WorkerRuntime`.
- [x] Specify `Executor.map` as an ordered iterator of typed item results for homogeneous callable result types.
- [x] Add typed timeout terminal state for `Future.result(timeout=...)`.
- [x] Specify `shutdown(cancel_futures=True)` marks pending futures with the same cancelled terminal state as `Future.cancel`.
- [x] Restrict `wait` and `as_completed` to homogeneous future collections unless users define an explicit sum type.
- [x] Add typed `Future.cancel()` return outcome for cancelled versus already-running futures.
- [x] Add typed timeout behavior for `Executor.map(..., timeout=...)` and `as_completed(..., timeout=...)`.
- [x] Define `wait(return_when=FIRST_EXCEPTION)` as first future with typed worker failure `Err(_)`.
- [x] Mark `threading.local` unsupported in this phase with diagnostics.
- [x] Add `Future.cancel()` `AlreadyDone` outcome.
- [x] Specify `wait(return_when=FIRST_EXCEPTION)` returns the full `(done, not_done)` partition.
- [x] Specify `Executor.map(..., timeout=...)` uses one absolute deadline from map-call time, not a per-item timeout reset.
- [x] Specify `as_completed(..., timeout=...)` uses one absolute deadline from call time, not a per-yield timeout reset.
- [x] Specify `wait(return_when=FIRST_EXCEPTION)` falls back to `ALL_COMPLETED` semantics when no worker failure occurs.
- [x] Specify `shutdown(cancel_futures=True)` routes pending futures through the same typed `Cancelled` outcome path as `Future.cancel`.
- [x] Specify `as_completed(..., timeout=...)` partial-result ownership: previously yielded futures remain valid, timeout is terminal, pending futures remain live/cancellable.
- [x] Define `FIRST_EXCEPTION` trigger as ordinary typed `Result::Err` worker outcomes plus executor/runtime worker failures.
- [x] Specify `shutdown(cancel_futures=True)` cancels only pending not-yet-started futures; running futures continue to typed completion.
- [x] Specify `as_completed` borrows or clones observation handles and does not consume caller-owned future handles.
- [x] Define `not_done` futures from `wait(FIRST_EXCEPTION)` as still scheduled; caller is responsible for cancellation or executor shutdown.
- [x] Specify `shutdown(wait=True)` stores running-future results before returning and `shutdown(wait=False)` keeps result channels alive for future observation.
- [x] Specify `wait()` borrows or clones observation handles and returns valid `done`/`not_done` views without consuming caller-owned futures.
- [x] Define future result observation as `Result[T, FutureError[E]]` or equivalent with dedicated `Cancelled`, `TimedOut`, `Worker(E)`, and `WorkerRuntime` variants.
- [x] Specify `gather()` borrows or clones task/future observation handles and does not consume caller-owned cancellation handles.
- [x] Define `gather(return_exceptions=True)` as per-element typed results and `gather(return_exceptions=False)` as Sifr structured typed aggregate/cancellation behavior rather than exceptions.
- [x] Specify `as_completed()` yields typed completed-future results, reports timeout as a terminal typed error, and leaves pending futures live/cancellable through original handles.
- [x] Define `TaskGroup` child failure aggregation as `TaskGroupError[E]` or equivalent containing all observed child `FutureError[E]` values.
- [x] Add binary pass/fail criteria for the M1 asyncio closure audit and make failed audit items block M1/M2 conformance claims.
- [x] Convert `contextmanager` and `asynccontextmanager` from vague generator-phase blockers into formal unsupported/waived surfaces with a future-generator-semantics revisit rule.
- [x] Make typed IPC design explicitly owned by M4, with no unnamed external prerequisite for `ProcessPoolExecutor` or `multiprocessing.Pool`.
- [x] Replace the conditional `signal.pause()` adoption path with an explicit unsupported/waived terminal state for this phase.
- [x] Resolve `signal.pause()` to unsupported/waived in this phase with diagnostics and a future safe signal-handler or structured signal-stream revisit rule.
- [x] Add external-review owner and five-working-day fallback rule.
- [x] Pin subprocess text mode, warning text encoding, and text-open demos to text/i18n `milestone_text_i18n_1` completion.
- [x] Add no-backward-compatibility policy: current-CPython API shape under canonical `sifr.*` imports only, no bare CPython stdlib aliases, no legacy aliases, no deprecated behavior, no pickle-style fallbacks, and no compatibility shims; only inventory-recorded current adapters with Sifr-safe semantics are allowed.
- [x] Align the phase with the stdlib namespace cleanup: `sifr.*` remains the permanent public stdlib namespace and bare CPython stdlib import attempts get namespace-contract diagnostics.
- [x] Mark `subprocess.getoutput` and `subprocess.getstatusoutput` unsupported as legacy shell-invocation helpers; typed `run(..., shell=...)` remains the explicit shell path where allowed.

## Implementation PRs

- M0: pending.
- M1: pending.
- M2: pending.
- M3: pending.
- M4: pending.
- M5: pending.
- M6: pending.

## Validation Evidence

Record local validation for each milestone before opening its PR.

Required baseline commands:

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
python3 scripts/check_hir_maintainability_guardrails.py
scripts/run_all_tests.sh --profile create-pr
```

Required merge-gate command before milestone closure:

```bash
scripts/run_all_tests.sh
```

## CPython Scan Evidence

Each milestone must record:

- CPython source files scanned.
- CPython docs files scanned.
- CPython tests scanned.
- Public APIs adopted, adapted, waived.
- Unsupported/intentional-diff/host-limited surfaces.
- Sifr e2e pass/fail fixtures added.

## Waiver Index

No waivers recorded yet.

Every waiver must include:

- surface
- terminal state: `intentional-diff`, `unsupported`, or `host-limited`
- rationale
- revisit rule
- CPython evidence
- Sifr regression fixture
