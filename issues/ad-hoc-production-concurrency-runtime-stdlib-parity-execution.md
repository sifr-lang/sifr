# Ad Hoc Phase Execution: Production Concurrency, Process, And Runtime Substrate

Phase contract: [ad-hoc-production-concurrency-runtime-stdlib-parity.md](./ad-hoc-production-concurrency-runtime-stdlib-parity.md)

Status: draft

## Scope Split

This ledger tracks the Sifr-native production runtime substrate for:

- `sifr.task`: structured tasks, deadlines, cancellation, typed task failures
- `sifr.sync`: channels, async channels, synchronization, backpressure
- `sifr.runtime`: blocking and CPU offload boundaries
- `sifr.parallel`: data-parallel CPU work
- `sifr.process`: subprocesses, async processes, owned pipes, supervision
- `sifr.signal`: structured shutdown and signal streams
- `sifr.resource`: deterministic cleanup scopes
- `sifr.ipc`: typed IPC foundation for future process workers

CPython-shaped modules such as `sifr.asyncio`, `sifr.queue`, `sifr.subprocess`, `sifr.concurrent.futures`, `sifr.multiprocessing`, `sifr.contextlib`, and `sifr.warnings` are evidence sources or possible adapters, not the production completion target.

Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate-execution.md](./ad-hoc-production-network-http-platform-substrate-execution.md). Text/Unicode/encoding/i18n runtime substrate remains in [ad-hoc-production-text-i18n-stdlib-parity-execution.md](./ad-hoc-production-text-i18n-stdlib-parity-execution.md).

Execution order: this is the second phase in the split production-stdlib sequence. Text/i18n runs first and provides encoding, Unicode, explicit text I/O, and locale/i18n gates; network/HTTP runs third and consumes this phase's task, cancellation, shutdown, offload, diagnostics, process, and lifecycle substrate.

## Milestone Checklist

- [ ] `milestone_concurrency_runtime_0`: Product Boundary And Rust Concurrency Contract
- [ ] `milestone_concurrency_runtime_1`: Structured Async Runtime
- [ ] `milestone_concurrency_runtime_2`: Synchronization, Channels, And Backpressure
- [ ] `milestone_concurrency_runtime_3`: Blocking And CPU Offload
- [ ] `milestone_concurrency_runtime_4`: Process And Subprocess Runtime
- [ ] `milestone_concurrency_runtime_5`: Shutdown, Signals, Cleanup, Context, And Diagnostics
- [ ] `milestone_concurrency_runtime_6`: Typed IPC And Future Process Workers
- [ ] `milestone_concurrency_runtime_7`: Integration, Documentation, And Production Gate

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
  - Result: `PASS`; no material implementation-blocking gaps remained for the previous CPython-shaped split.
- Final implementation-readiness scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-17-final-readiness.md`
  - Result: `PASS`; all three previous split phases were implementation-ready, with one editorial ledger-title mismatch remediated.
- No-legacy readiness scans:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-18-no-legacy-readiness.md`
  - Result: `FAIL`; text/i18n stale dynamic-handler and implicit-open wording were remediated.
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-19-no-legacy-readiness.md`
  - Result: `PASS`; no remaining backward-compatibility, legacy-support, deprecated-behavior, shim, bridge-alias, or fallback decisions remained.
- Namespace consistency scan:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-20-sifr-namespace-readiness.md`
  - Result: `PASS`; all three phase docs consistently use canonical `sifr.*` stdlib imports and reject bare CPython stdlib aliases.
- Current substrate reframing review:
  - Source: reviewer thoughts provided by the user in `/Users/yaseralnajjar/.codex/attachments/e5616ec2-7eb9-4106-b2e0-723a513a8993/pasted-text.txt`.
  - Result: accepted direction; phase reframed from CPython stdlib parity to Sifr production concurrency/runtime substrate.
- Substrate implementation-readiness Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-review-pass-1.md`
  - Result: `FAIL`; 12 implementation-readiness blockers were remediated across the phase and this ledger.
- Substrate implementation-readiness Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-review-pass-2.md`
  - Result: `FAIL`; remaining `sifr.parallel` pool-sizing and post-M0 review gate gaps were remediated.
- Substrate implementation-readiness Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-review-pass-3.md`
  - Result: `FAIL`; remaining M2 channel/sync sendability enforcement gap was remediated.
- Final substrate implementation-readiness Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-review-pass-4.md`
  - Result: `PASS`; no blocking implementation-readiness gaps remained.

## Additional Planning Reviews

- Cross-phase implementation-readiness review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-21-phase-order-readiness.md`
  - Result: `FAIL`; network/runtime dependency matrix, network cancellation/shutdown provider wording, and legacy filename naming-note gaps were remediated across the split phase docs.
- Cross-phase implementation-readiness follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-22-phase-order-readiness.md`
  - Result: `PASS`; pass 21 remediations were verified, with one minor network state-vocabulary inconsistency remediated.
- Final cross-phase implementation-readiness verification:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-23-final-readiness.md`
  - Result: `PASS`; no material blockers, stale labels, or implementation-blocking contradictions remained.
- Rust ecosystem-first clarification:
  - Result: accepted; this phase now requires wrapping mature Rust runtime/concurrency crates where suitable, records preferred crate families, defers any required surface that the selected ecosystem stack cannot satisfy, and makes dependency decision records an M0 gate.
- Rust ecosystem-first expansion:
  - Result: accepted; preferred crate families now include Tokio/Tokio Util, futures utilities, Crossbeam/Tokio MPSC, std sync/Parking Lot/Once Cell/Scopeguard, Rayon, Tokio process/std process/Rustix, Tokio signal/Rustix, tracing/metrics, thiserror, and Serde/Postcard, all hidden behind Sifr APIs. Flume, Signal Hook, Nix, and Bincode are not used in this phase.
- Decision-completeness Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-1.md`
  - Result: `FAIL`; `JoinSet` drop, Rayon pool architecture, task context API slots, post-M0 review fallback, `sifr.asyncio` veneer disposition, and dependency-record timing gaps were remediated.
- Decision-completeness Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-2.md`
  - Result: `FAIL`; `JoinSet.join_all().await`, `JoinSet` submission API, and `Pool` instance API gaps were remediated.
- Decision-completeness Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-3.md`
  - Result: `FAIL`; `JoinSet` result ordering/`JoinItemId` role gap was remediated, and non-blocking `race`/`select`, `parallel.map`, and shell-effect details were tightened.
- Cross-phase decision-closure review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-24-decision-closure.md`
  - Result: `PASS`; all material product/API/dependency decisions across text/i18n, concurrency/runtime, and network/HTTP were clear enough for implementation. Reviewer noted `race`/`select` could be sharper, so the phase now explicitly records `race` as homogeneous collection competition and `select` as named-branch competition.
- Final cross-phase decision delta review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-25-final-delta.md`
  - Result: `PASS`; final `race`/`select` and no-bespoke-policy clarifications introduced no unmade or contradictory implementation decisions.
- Final decision-completeness Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-4.md`
  - Result: `PASS`; no blocking decision gaps remained.
- Final blocker-only decision review:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-5.md`
  - Result: `PASS`; no concrete implementation-blocking gaps remained. Non-blocking channel taxonomy, M5 entry-gate, and M5 typed-error index polish were applied.
- Final post-polish verification:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-6.md`
  - Result: `PASS`; no blockers, non-blocking polish, stale pending-review labels, Python legacy leakage, missing Rust ecosystem decisions, or cross-document contradictions remained.

## Pending Reviews

- Post-M0 external review: run a dedicated external review after M0 inventory and before M1 implementation. M1 cannot start until this review has a `PASS` result recorded in `Planning Reviews`. If review output is unavailable for five working days after the review artifact is posted, the phase owner may proceed only by recording the attempted review, open questions, and a conservative self-review in this ledger.

## Planning Review Remediation Retained In This Phase

- [x] Define multiprocessing start-method, typed IPC, shared-memory, and ownership constraints.
- [x] Narrow unsafe signal handler registration to `unsupported` for this phase.
- [x] Add milestone dependency graph.
- [x] Add shared concurrency/runtime error mapping requirement.
- [x] Name Tokio as the backing async runtime for internal lowering and require concrete feature expansion in M0.
- [x] Tie any future process-pool API to the same typed IPC gate.
- [x] Clarify that class-based cleanup/context APIs are independent production APIs, not fallback implementations for generator decorators.
- [x] Mark `contextmanager` and `asynccontextmanager` unsupported in this phase, with a revisit rule for a future generator semantics phase.
- [x] Add explicit cross-phase dependency contract for text/i18n and network/web consumers.
- [x] Clarify that core scheduler/task helpers are native runtime infrastructure, not CPython module parity work.
- [x] Assign private blocking/CPU offload substrate to this phase while keeping public `threading` module parity out of scope.
- [x] Add explicit terminal expectations for `signal.getsignal`, `signal.pause`, and `pthread_sigmask`.
- [x] Require M0 classification before any `sifr.asyncio`, `sifr.queue`, `sifr.subprocess`, `sifr.concurrent.futures`, or `sifr.multiprocessing` adapter claims production scope.
- [x] Mark `contextvars` parity and implicit per-task context propagation out of scope.
- [x] Add a Sifr-native task/request context planning item for tracing, deadlines, cancellation metadata, and web observability.
- [x] Add sendability/shareability as a phase-wide gate before task/thread/process boundary captures.
- [x] Preserve typed future/task cancellation and worker error propagation requirements as Sifr-native `sifr.task`/`sifr.runtime` behavior rather than `concurrent.futures` parity.
- [x] Preserve homogeneous worker result/cancellation/deadline fixture requirements for any accepted future/offload adapter.
- [x] Define `TaskGroup` child failure aggregation as typed aggregate evidence.
- [x] Make typed IPC design explicitly owned by M6, with no unnamed external prerequisite for future process workers.
- [x] Resolve `signal.pause()` to unsupported/waived in this phase with diagnostics and a future safe signal-handler or structured signal-stream revisit rule.
- [x] Add external-review owner and five-working-day fallback rule.
- [x] Pin subprocess text mode, warning text encoding, and text-open demos to text/i18n `milestone_text_i18n_1` completion.
- [x] Add no-backward-compatibility policy: current adapters under canonical `sifr.*` imports only, no bare CPython stdlib aliases, no legacy aliases, no deprecated behavior, no pickle-style fallbacks, and no compatibility shims.
- [x] Align the phase with the stdlib namespace cleanup: `sifr.*` remains the permanent public stdlib namespace and bare CPython stdlib import attempts get namespace-contract diagnostics.
- [x] Mark `subprocess.getoutput` and `subprocess.getstatusoutput` unsupported as legacy shell-invocation helpers.
- [x] Add support tiers: production substrate, production public API, internal runtime adapter, compatibility adapter, deferred, and rejected.
- [x] Add no-toy-concurrency gate rejecting public partial modules that exist only because CPython has them.
- [x] Demote `multiprocessing` and `ProcessPoolExecutor` from baseline CPU parallelism; Sifr uses typed offload and data parallelism for CPU work.
- [x] Reject Python global `warnings` filter parity in this phase; use explicit structured diagnostics only.
- [x] Add M0 gates for the resolved decision register, import-resolution tests, host matrix, workload database, task typing, detached-task policy, task context, adapter migration value, and reviewer designation.
- [x] Assign sendability/shareability compiler enforcement to M1 with M3/M4/M6 verification extensions.
- [x] Resolve `TaskGroup`/`JoinSet` distinction: `TaskGroup` is scoped structured concurrency with failure cancellation; `JoinSet` is a dynamically-growable homogeneous completed-work collection.
- [x] Resolve task collection typing: homogeneous by default, heterogeneous only with an explicit user sum/enum type.
- [x] Resolve detached-task policy: stable public tasks are structured by default; handle drop before failure observation is a diagnostic; detached tasks are rejected in this phase.
- [x] Assign subprocess text-mode ownership to M4 once text/i18n M1 is complete, or require explicit `milestone_concurrency_runtime_text_subprocess_integration` deferral.
- [x] Require M0 cleanup classification for the existing `sifr.asyncio` compatibility veneer.
- [x] Require signal-to-host matrix and supported-host matrix recording before host-limited subprocess/signal adoption.
- [x] Require typed IPC design approval to be recorded before serialization dependency selection.
- [x] Add explicit M0 decision and DoD gate for `sifr.parallel` pool sizing.
- [x] Add a post-M0 external review `PASS` gate before M1 starts.
- [x] Assign channel and sync-primitive value-type sendability/shareability enforcement to M2.
- [x] Add explicit entry gates for M3 pool sizing and M6 typed IPC design approval.
- [x] Require per-milestone traceability documents to be created in the first implementation PR and closed before milestone completion.
- [x] Expand Rust ecosystem-first policy to prefer mature crates for async utilities, channels, locking/once/cleanup, signals, diagnostics, typed errors, and IPC; if the selected ecosystem stack cannot satisfy a required surface, defer that surface with evidence instead of adding bespoke infrastructure in this phase.
- [x] Resolve `JoinSet` drop, submission, observation, and cancellation semantics.
- [x] Resolve configured `sifr.parallel.Pool` architecture and instance API.
- [x] Reserve M1 task context API slots for M5 explicit propagation.
- [x] Freeze existing `sifr.asyncio` veneer as adapter-later and ensure new native APIs do not depend on it.
- [x] Resolve `JoinSet` result ordering and `JoinItemId` role.
- [x] Clarify `parallel.map`/`Pool.map` async calling convention.
- [x] Name shell subprocess usage as the `@shell_exec` security effect.
- [x] Resolve `race` versus `select`: `race` is homogeneous collection competition returning index plus typed outcome, while `select` is named-branch competition returning branch tag plus typed outcome; both cancel losers with typed evidence.

## Implementation PRs

- M0: pending.
- M1: pending.
- M2: pending.
- M3: pending.
- M4: pending.
- M5: pending.
- M6: pending.
- M7: pending.

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
- Public APIs adopted, adapted, waived, adapter-later, deferred, or rejected.
- Unsupported/intentional-diff/host-limited surfaces.
- Sifr e2e pass/fail fixtures added.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-concurrency-runtime-stdlib-parity-execution.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.json`
- `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md`
- `verification/stdlib/concurrency_runtime_workload_database.md`
- `verification/stdlib/concurrency_runtime_supported_host_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

## Review Ownership

- Designated compiler/runtime reviewer role: compiler/runtime reviewer assigned by the phase owner in M0; M1 cannot start until the reviewer identity is recorded here.
- Typed IPC design approval process: M6 requires a named design artifact reviewed by the phase owner and designated compiler/runtime reviewer, then recorded here before any serialization crate is selected.

## API Tier Decision Index

Pre-M0 phase-level decisions:

| Surface | Support tier | Terminal state | Rationale | CPython evidence | Sifr fixture or design artifact |
| --- | --- | --- | --- | --- | --- |
| `sifr.task` | `production-public` | `done-through-M1` | Structured task API is the recommended async model. | `Lib/test/test_asyncio/test_tasks.py`, `Lib/test/test_asyncio/test_taskgroups.py` | M1 task traceability document |
| `sifr.sync` | `production-public` | `done-through-M2` | Channels and synchronization are the recommended queue/backpressure model. | `Lib/test/test_queue.py`, `Lib/test/test_asyncio/test_queues.py`, `Lib/test/test_asyncio/test_locks.py` | M2 sync traceability document |
| `sifr.runtime` / `sifr.parallel` | `production-public` | `done-through-M3` | Explicit blocking and CPU offload replace executor parity as the production model. | `Lib/test/test_concurrent_futures/` | M3 offload traceability document |
| `sifr.process` | `production-public` | `done-through-M4` | Native process supervision and owned pipes replace `subprocess` parity as the production model. | `Lib/test/test_subprocess.py`, `Lib/test/test_asyncio/test_subprocess.py` | M4 process traceability document |
| `sifr.signal` / `sifr.resource` / diagnostics / context | `production-public` | `done-through-M5` | Structured shutdown, cleanup, diagnostics, and explicit context are production ergonomics. `ContextError` and `DiagnosticError` are owned by this milestone. | `Lib/test/test_signal.py`, `Lib/test/test_contextlib.py`, `Lib/test/test_warnings/` | M5 ergonomics traceability document |
| `sifr.ipc` | `production-substrate` | `done-through-M6` | Typed IPC is the foundation for future supervised process workers. | `Lib/test/_test_multiprocessing.py`, `Lib/test/test_multiprocessing_spawn/` | M6 IPC design artifact |
| `sifr.asyncio`, `sifr.queue`, `sifr.subprocess`, `sifr.concurrent.futures`, `sifr.multiprocessing` | `adapter-later` / `deferred` | `not-implemented-this-phase` | CPython-shaped modules remain evidence sources and later migration adapters only. | CPython module tests listed in phase source of truth | Negative import/adapter fixtures |
| Python global `warnings` filter model | `rejected` | `rejected` | Runtime diagnostics use tracing/metrics and typed Sifr diagnostics, not global Python warning filters. | `Lib/test/test_warnings/` | M5 warning-global rejection fixture |
| Rust ecosystem choices | `internal-runtime` | `accepted` | Use Tokio/Tokio Util, Futures Util, Crossbeam Channel, Tokio MPSC, std sync/Parking Lot/Once Cell/Scopeguard, Rayon, Tokio process/std process/Rustix, Tokio signal/Rustix, tracing/tracing-subscriber/metrics, thiserror, and Serde/Postcard. Do not use Flume, Signal Hook, Nix, Bincode, or bespoke replacements. | N/A | M0 dependency decision records |
| `JoinSet` drop | `production-public` | `done-through-M3` | Live/non-empty `JoinSet` values must be consumed by `join_all()` or `cancel_all().await`; unobserved drop is a compile-time diagnostic. | `Lib/test/test_concurrent_futures/` | M3 JoinSet drop diagnostic fixture |
| Rayon pool architecture | `internal-runtime` | `accepted` | Top-level `sifr.parallel` uses a private lazy default Rayon pool; configured parallelism uses explicit `Pool(config)` private Rayon pools. Rayon's global pool is never configured. | N/A | M3 pool architecture decision record |
| Existing `sifr.asyncio` veneer | `adapter-later` | `frozen-this-phase` | Existing supported veneer entry points remain frozen; M1 does not build on, extend, or remove them. New runtime APIs use `sifr.task`, `sifr.sync`, and `sifr.process`. | `Lib/test/test_asyncio/` | M1 veneer-free implementation fixture |
| `JoinSet` result ordering | `production-public` | `done-through-M3` | `join_all().await` returns results in submission order, `cancel_all().await` returns cancellation evidence in submission order, and `JoinItemId` is an opaque user-side correlation token with no query API. | `Lib/test/test_concurrent_futures/` | M3 JoinSet ordering fixture |
| Shell subprocess effect | `production-substrate` | `done-through-M4` | Shell subprocess usage is marked with `@shell_exec` in addition to `@blocking_io`; shell APIs require explicit shell selection and async/offload diagnostics. | `Lib/test/test_subprocess.py` | M4 shell effect fixture |

Every decision must include:

- surface
- support tier: `production-substrate`, `production-public`, `internal-runtime`, `adapter-later`, `deferred`, or `rejected`
- terminal state
- rationale
- CPython evidence, when applicable
- Sifr fixture or design artifact

## Waiver Index

This index is populated during M0 inventory. All non-goal, deferred, rejected, unsupported, intentional-diff, and host-limited surfaces in the phase must have waiver/decision entries by M0 close.

| Surface | Terminal state | Rationale | Revisit rule | CPython evidence | Sifr regression fixture |
| --- | --- | --- | --- | --- | --- |
| `signal.pause` | `unsupported` | Safe arbitrary signal-handler wakeup is not adopted in this phase; production shutdown uses structured signal streams instead. | Revisit only in a future safe signal-handler or structured signal-stream expansion that proves deterministic cancellation/wakeup behavior across the supported host matrix. | `Lib/test/test_signal.py`, `Doc/library/signal.rst` | M5 must add a negative diagnostic fixture for static `sifr.signal.pause` use before closure. |
| `sifr.asyncio` new APIs | `adapter-later` | Existing veneer frozen; no new `sifr.asyncio` APIs ship in this phase. | Revisit only in a later migration-adapter issue that proves value over `sifr.task`/`sifr.sync`/`sifr.process`. | `Lib/test/test_asyncio/` | M1/M2/M4 fixtures prove native APIs do not depend on the veneer. |

Every waiver must include:

- surface
- terminal state: `intentional-diff`, `unsupported`, `host-limited`, `deferred`, or `rejected`
- rationale
- revisit rule
- CPython evidence
- Sifr regression fixture
