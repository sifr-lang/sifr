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

Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate-execution.md](./ad-hoc-production-network-http-platform-substrate-execution.md). Text and internationalization parity remains in [ad-hoc-production-text-i18n-stdlib-parity-execution.md](./ad-hoc-production-text-i18n-stdlib-parity-execution.md).

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
- Required follow-up: run a dedicated external review after M0 inventory and before M1 implementation, because this phase is now independently scoped around native runtime APIs.

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
- [x] Shrink `warnings` from Python global filter parity to explicit structured diagnostics unless M0 proves a narrow adapter is justified.

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
- one traceability document per milestone domain under `verification/stdlib/`

## API Tier Decision Index

No decisions recorded yet.

Every decision must include:

- surface
- support tier: `production-substrate`, `production-public`, `internal-runtime`, `adapter-later`, `deferred`, or `rejected`
- terminal state
- rationale
- CPython evidence, when applicable
- Sifr fixture or design artifact

## Waiver Index

No waivers recorded yet.

Every waiver must include:

- surface
- terminal state: `intentional-diff`, `unsupported`, `host-limited`, `deferred`, or `rejected`
- rationale
- revisit rule
- CPython evidence
- Sifr regression fixture
