# Ad Hoc Phase Execution: Production Concurrency, Process, And Runtime Substrate

Phase contract: [ad-hoc-production-concurrency-runtime-platform-substrate.md](./ad-hoc-production-concurrency-runtime-platform-substrate.md)

Status: completed on 2026-06-09

## Scope Split

This ledger tracks the Sifr-native production runtime substrate for:

- `sifr.task`: structured scopes/tasks, deadlines, cancellation, typed task failures, and affine task observation handles
- `sifr.sync`: channels, async channels, synchronization, backpressure
- `sifr.runtime`: blocking and CPU offload boundaries
- `sifr.parallel`: data-parallel CPU work
- `sifr.process`: subprocesses, async processes, owned pipes, supervision
- `sifr.signal`: structured shutdown and signal streams
- `sifr.resource`: deterministic cleanup scopes
- `sifr.ipc`: typed IPC foundation for future process workers

CPython-shaped modules such as `sifr.asyncio`, `sifr.queue`, `sifr.subprocess`, `sifr.concurrent.futures`, `sifr.multiprocessing`, `sifr.contextlib`, and `sifr.warnings` are evidence sources or legacy implementation debt to remove/diagnose, not production targets or compatibility adapters.

This ledger also records the structured runtime work decision: async tasks, blocking offload, CPU offload, long-running child processes, and future typed workers are all scoped work units with typed handles and typed observation/cancellation evidence. Threads and processes are execution substrates, not separate public concurrency worlds.

Network/HTTP platform substrate remains in [ad-hoc-production-network-http-platform-substrate-execution.md](./ad-hoc-production-network-http-platform-substrate-execution.md). Text/Unicode/encoding/i18n runtime substrate remains in [ad-hoc-production-text-i18n-platform-substrate-execution.md](./ad-hoc-production-text-i18n-platform-substrate-execution.md).

Execution order: this is the second phase in the split production-stdlib sequence. Text/i18n runs first and provides encoding, Unicode, explicit text I/O, and locale/i18n gates; network/HTTP runs third and consumes this phase's task, cancellation, shutdown, offload, diagnostics, process, and lifecycle substrate.

## Milestone Checklist

- [x] `milestone_concurrency_runtime_0`: Product Boundary And Rust Concurrency Contract
- [x] `milestone_concurrency_runtime_0a`: Legacy CPython-Shaped Surface Removal Gate
- [x] `milestone_concurrency_runtime_1`: Structured Async Runtime
- [x] `milestone_concurrency_runtime_2`: Synchronization, Channels, And Backpressure
- [x] `milestone_concurrency_runtime_3`: Blocking And CPU Offload
- [x] `milestone_concurrency_runtime_4`: Process Runtime
- [x] `milestone_concurrency_runtime_5`: Shutdown, Signals, Cleanup, Context, And Diagnostics
- [x] `milestone_concurrency_runtime_6`: Typed IPC And Future Process Workers
- [x] `milestone_concurrency_runtime_7`: Integration, Documentation, And Production Gate

## Planning Reviews

- Inherited from the original combined stdlib parity planning review:
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-1d.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-2.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-3.md`
  - `reviews/ad-hoc-production-stdlib-platform-parity-planning-review-pass-4.md`
- Final combined review result before split: `PASS`.
- Split-phase agent review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-1-constrained.md`
  - Result: `FAIL`; cross-phase dependency and ownership gaps were remediated across the split phase docs.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-2-constrained.md`
  - Result: `FAIL`; remaining ownership/disposition gaps were remediated across the split phase docs.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-3-constrained.md`
  - Result: `FAIL`; remaining sequencing/error-surface gaps were remediated across the split phase docs.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-4-constrained.md`
  - Result: `FAIL`; remaining async-context/file/default-encoding/thread-error gaps were remediated across the split phase docs.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-5-constrained.md`
  - Result: `FAIL`; remaining contextvars/future-cancellation/open-policy/worker-typing gaps were remediated across the split phase docs.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-6-constrained.md`
  - Result: `FAIL`; remaining executor map/timeout/cancellation/heterogeneous-future gaps and text-wrapper gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-7-constrained.md`
  - Result: `FAIL`; remaining executor state-machine, `StringIO`, `threading.local`, and codec error-handler gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-8-constrained.md`
  - Result: `FAIL`; remaining Future.cancel, wait partition, executor.map timeout, and text handler gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-9-constrained.md`
  - Result: `FAIL`; remaining executor deadline/cancellation/wait fallback and codec handler classification gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-10-constrained.md`
  - Result: `FAIL`; remaining handler enforcement, partial iteration, FIRST_EXCEPTION trigger, and shutdown pending/running gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-11-constrained.md`
  - Result: `FAIL`; remaining future ownership/lifecycle and shutdown observability gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-12-constrained.md`
  - Result: `FAIL`; remaining `wait()` ownership, cancelled result typing, and incremental codec finalization gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-13-constrained.md`
  - Result: `FAIL`; remaining `gather()` ownership/result typing, `as_completed()` timeout signaling, codec recoverable-error, and `TaskGroup` aggregation gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-14-constrained.md`
  - Result: `FAIL`; remaining network error hierarchy, TLS socket ownership, workload classification, handler model, concurrency gate, text decision, and review-gate gaps were remediated.
- Split-phase agent follow-up:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-15-constrained.md`
  - Result: `FAIL`; remaining TLS wrap failure-state, `signal.pause`, and text-i18n dependency milestone gaps were remediated.
- Final split-phase agent follow-up:
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
- Substrate implementation-readiness agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-review-pass-1.md`
  - Result: `FAIL`; 12 implementation-readiness blockers were remediated across the phase and this ledger.
- Substrate implementation-readiness agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-review-pass-2.md`
  - Result: `FAIL`; remaining `sifr.parallel` pool-sizing and post-M0 review gate gaps were remediated.
- Substrate implementation-readiness agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-review-pass-3.md`
  - Result: `FAIL`; remaining M2 channel/sync sendability enforcement gap was remediated.
- Final substrate implementation-readiness agent follow-up:
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
  - Result: accepted; this phase now requires wrapping mature Rust runtime/concurrency crates where suitable, locks accepted/rejected crate choices in the phase doc, defers any required surface that the selected ecosystem stack cannot satisfy, and forbids crate-family discovery during implementation.
- Rust ecosystem-first expansion:
  - Result: accepted; locked crate decisions cover Tokio, Tokio Util, conditional Futures Util, Crossbeam Channel, Rayon, conditional targeted Rustix, tracing, metrics, thiserror, Serde, and Postcard, all hidden behind Sifr APIs with exact version/feature plans in the phase doc. Tokio remains `current_thread`; blocking I/O parallelism uses Tokio's blocking pool and CPU parallelism uses Rayon. Flume, async-channel, futures-channel, direct Parking Lot, new Once Cell, Scopeguard, production tracing-subscriber, IPC Serde JSON, Bincode, Signal Hook, Nix, direct Mio/Bytes/DashMap, runtime/language-facing Anyhow/Eyre, and bespoke replacements are not used in this phase.
- Rust ecosystem dependency-lock agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-rust-ecosystem-decisions-review-pass-1.md`
  - Result: `FAIL`; review findings were remediated by explicitly recording the `current_thread` Tokio runtime invariant, documenting that tokio-util 0.7.18 exposes `tokio_util::sync::CancellationToken` through `rt` rather than a nonexistent `sync` feature, aligning no-public-type lists, making tracing attribute macros unavailable, and clarifying Tokio `sync` wrappers in the ledger.
- Rust ecosystem dependency-lock agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-rust-ecosystem-decisions-review-pass-2.md`
  - Result: `PASS`; no blockers remained after the dependency-lock remediation.
- Rust ecosystem dependency-lock elegance review:
  - `reviews/ad-hoc-production-concurrency-runtime-rust-ecosystem-decisions-review-pass-3.md`
  - Result: `PASS`; no blockers remained, and material polish around `metrics` default features, concrete `futures-util` exclusions, and direct `tracing` dependency wording was applied.
- Rust ecosystem dependency-lock final follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-rust-ecosystem-decisions-review-pass-4.md`
  - Result: `PASS`; all pass-3 polish was verified and no further meaningful dependency-lock polish remained.
- General dependency policy addition:
  - Result: accepted; `internal_docs/dependency_policy.md` now defines dependency rings for compiler/tooling-only, generated-runtime core, stdlib feature-gated substrate, feature-specific protocol/data substrate, dev/test/demo-only, and rejected direct dependencies. The concurrency/runtime phase now applies those rings to its crate decisions, keeps Serde/Postcard M6 typed-IPC-only, allows `anyhow`/`eyre` only as contained compiler/tooling dependencies, and rejects `bincode` because Postcard is the selected typed IPC codec rather than because Bincode is pickle-like.
- General dependency policy agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-dependency-policy-review-pass-1.md`
  - Result: `PASS`; no blockers found. Non-blocking polish to include `futures-util` in Ring 2 examples was applied.
- General dependency policy agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-dependency-policy-review-pass-2.md`
  - Result: `PASS`; the `futures-util` Ring 2 policy addition stayed consistent with the phase ring table and introduced no drift.
- Conditional dependency tightening review:
  - Result: accepted; `futures-util` is now conditional and added only if M1 proves `join_all`, `race`, `select`, or stream adapters would otherwise require substantial custom `Future`/`poll` code. `rustix` now requires a documented `std`/Tokio capability gap plus supported-host matrix rows and deterministic host-specific fixtures before use.
- Conditional dependency tightening agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-dependency-policy-review-pass-3.md`
  - Result: `PASS`; `futures-util` and `rustix` conditionality is consistent across dependency policy, phase table, resolved decision register, and execution ledger.
- Decision-completeness agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-1.md`
  - Result: `FAIL`; `JoinSet` drop, Rayon pool architecture, task context API slots, post-M0 review fallback, `sifr.asyncio` veneer disposition, and dependency-record timing gaps were remediated.
- Decision-completeness agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-2.md`
  - Result: `FAIL`; `JoinSet.join_all().await`, `JoinSet` submission API, and `Pool` instance API gaps were remediated.
- Decision-completeness agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-3.md`
  - Result: `FAIL`; `JoinSet` result ordering/`JoinItemId` role gap was remediated, and non-blocking `race`/`select`, `parallel.map`, and shell-effect details were tightened.
- Cross-phase decision-closure review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-24-decision-closure.md`
  - Result: `PASS`; all material product/API/dependency decisions across text/i18n, concurrency/runtime, and network/HTTP were clear enough for implementation. Reviewer noted `race`/`select` could be sharper, so the phase now explicitly records `race` as homogeneous collection competition and `select` as named-branch competition.
- Final cross-phase decision delta review:
  - `reviews/ad-hoc-production-split-stdlib-phases-review-pass-25-final-delta.md`
  - Result: `PASS`; final `race`/`select` and no-bespoke-policy clarifications introduced no unmade or contradictory implementation decisions.
- Final decision-completeness agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-4.md`
  - Result: `PASS`; no blocking decision gaps remained.
- Final blocker-only decision review:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-5.md`
  - Result: `PASS`; no concrete implementation-blocking gaps remained. Non-blocking channel taxonomy, M5 entry-gate, and M5 typed-error index polish were applied.
- Final post-polish verification:
  - `reviews/ad-hoc-production-concurrency-runtime-substrate-decision-completeness-pass-6.md`
  - Result: `PASS`; no blockers, non-blocking polish, stale pending-review labels, Python legacy leakage, missing Rust ecosystem decisions, or cross-document contradictions remained.
- Structured runtime work review:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-1.md`
  - Result: `FAIL`; reviewer reported boundary-table, M4/M6 IPC placement, and M1/M2 DoD concerns. Local source verification showed the findings were review-packet artifacts rather than source defects.
- Structured runtime work follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-2.md`
  - Result: `PASS`; exact source snippets verified one process-boundary row, M4-only subprocess tests, M6-owned IPC design, and separate M1/M2 definitions of done.
- Final structured runtime work verification:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-3.md`
  - Result: `PASS`; no blocker remained. The structured work model, milestone boundaries, and M0 implementation-audit decisions were implementation-ready.
- Post-review structured runtime work verification:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-4.md`
  - Result: `PASS`; `TaskGroup[E]`, `TaskHandle`, scope/group split, timeout evidence, Sifr-owned cancellation scope option, `CancelOutcome`, sync/async lock split, IPC frame families, shell effect behavior, and M0-justified `Barrier`/`Once` decisions were implementation-ready.
- Final contract-level structured runtime review:
  - Source: reviewer notes provided by the user on 2026-06-06.
  - Result: accepted; post-M0 review fallback, observed `TaskGroup` failure semantics, `race`/`select` result containers, minimum `CancelOutcome` states, scoped process handle shape, IPC schema compatibility, `IpcSerializable` strictness, `sifr.subprocess` freeze status, async lock guard await rules, and offload error mapping were recorded in the phase contract.
- Structured runtime work agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-5.md`
  - Result: `FAIL`; remaining `TaskGroup`/scope canonical owner, M1 sibling-cancellation DoD, process handle decision-register row, `sifr.subprocess` freeze wording, `select` call syntax, TaskGroup offload error binding, lock guard wording, no-public-Rust-types model wording, and `Task`/`BlockingTask` audit gaps were remediated.
- Structured runtime work agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-6.md`
  - Result: `FAIL`; remaining `cancel_scope` stable-vs-optional contradiction was remediated, and non-blocking polish for supervised process examples, `spawn_scoped` orientation, and `race`/`select` loser evidence type was applied.
- Structured runtime work agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-7.md`
  - Result: `FAIL`; remaining TaskGroup offload error binding versus `JoinSet.join_all()` wrapper alignment gap was remediated, and non-blocking polish for cancellation scope naming, process example pipe-access intent, and `JoinSet.join_all()` resolved-decision return type was applied.
- Final structured runtime work agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-8.md`
  - Result: `PASS`; TaskGroup offload error binding and `JoinSet.join_all()` wrapper alignment were verified, with only non-blocking wording/ledger polish applied.
- Final blocker-only agent verification:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-9.md`
  - Result: `PASS`; no material blockers, contradictions, stale state vocabulary, missing binding decisions, or ambiguous contracts remained.
- No-subprocess-compatibility agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-no-subprocess-compat-review-pass-1.md`
  - Result: `PASS`; docs were clean under the no-backward-compatibility, no-CPython-adapter, `sifr.process`-only decision, with only non-blocking wording/waiver-index polish applied.
- Final no-subprocess-compatibility agent verification:
  - `reviews/ad-hoc-production-concurrency-runtime-no-subprocess-compat-review-pass-2.md`
  - Result: `PASS`; no backward-compatibility or CPython-shaped adapter commitment remained, and `sifr.subprocess` was verified as legacy implementation debt to remove, keep internal-test-only, or route to unsupported diagnostics.
- M0 implementation agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-m0-implementation-review-pass-1.md`
  - Result: `PASS`; CPython scan, inventory, evidence matrix, workload database, platform contract, host matrix, golden manifest entries, native namespace diagnostics, and M0/M0a gates met M0 requirements. Non-blocking polish for `sifr.contextlib`/`sifr.warnings` disposition and warnings diagnostic steering was applied.
- M0a legacy-surface agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-1.md`
  - Result: `FAIL`; local validation recording, duplicate legacy-import fail fixtures, empty review artifact, and dead `sifr.asyncio` veneer lowering blockers were remediated.
- M0a legacy-surface agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-2.md`
  - Result: `PASS`; public legacy modules were verified unreachable, `SIFR-IMPORT-0009` replacement diagnostics were verified, native task lowering was verified free of `sifr.asyncio` compatibility paths, demos/manifests/goldens were clean, validation evidence was recorded, and no blocker remained.
- M0a final legacy-surface agent confirmation:
  - `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-3.md`
  - Result: `PASS`; pass-1 blockers remained remediated in the current working tree, create-pr validation artifacts were verified with `70 passed`, `0 failed` e2e pass coverage and platform golden `pass=5`, `skip=2`, and the implementation was confirmed ready for the M0a PR.
- Post-M0 external review gate:
  - `reviews/ad-hoc-production-concurrency-runtime-post-m0-external-review-pass-1.md`
  - Result: `PASS`; M0 substrate inventory, CPython scan evidence, workload database, platform contract, dependency decisions, M0a legacy surface removal, validation evidence, and M1 entry gates were verified. M1 may start.
- M1 structured-async implementation agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-1.md`
  - Result: `PASS`; M1 structured task APIs, reserved `ctx` slots, named `select`, and shared spawn enforcement were verified. Non-blocking polish for arbitrary select-branch signature wording, `async_with.rs` decomposition, and sequential same-name task-owner cleanup was applied.
- M1 structured-async implementation agent review:
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-2.md`
  - Result: `PASS`; `TaskGroup(ctx=None)`, `task.spawn_scoped(..., ctx=None)`, named-branch `task.select(first=..., second=...)`, existing task-boundary enforcement, traceability, manifests, and create-pr validation evidence were verified. Non-blocking demo/select and `spawn_scoped` model-doc polish was applied.
- M1 final post-polish agent follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-5.md`
  - Result: `PASS`; demo select syntax, `spawn_scoped` and placeholder select docs, `task_owner_scope_state` extraction, same-name `TaskGroup` cleanup tests, line-cap status, ledger, traceability, and post-polish validation were verified. Reviewer is satisfied and M1 is ready to PR/merge.

## Pending Reviews

- M0/M0a/post-M0 reviews are complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m0-implementation-review-pass-1.md`, `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-2.md`, `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-3.md`, and `reviews/ad-hoc-production-concurrency-runtime-post-m0-external-review-pass-1.md`. M1 may start.
- M1 structured-async implementation reviews are complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-1.md`, `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-2.md`, and `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-5.md`. M1 is ready to PR/merge.
- M2 sync/channel implementation review is complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m2-sync-review-pass-1.md`. M2 is locally validated and ready to PR/merge.
- M3 JoinSet implementation review is complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m3-joinset-review-pass-2.md` and `reviews/ad-hoc-production-concurrency-runtime-m3-joinset-review-pass-3.md`; PR #2320 is merged.
- M3 scoped owner offload implementation review is complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m3-scoped-offload-review-pass-1.md`; PR #2323 is merged.
- M3 default parallel pool closure review is complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m3-default-pool-review-pass-1.md`; PR #2326 is merged.
- M3 closeout implementation review is complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-4.md`; PR #2325 is merged and no strict M3 closure blockers remain.
- M4 closeout classification review is complete: `PASS` in `reviews/ad-hoc-production-concurrency-runtime-m4-closeout-review-pass-1.md`; M4 is locally validated and ready to PR/merge.

## M1 Implementation Ledger

Current M1 wave: structured task API public-shape closure.

- PR: [#2313](https://github.com/sifr-lang/sifr/pull/2313), merged at `605d47d1397272dac45ae9634525f97d011f5805`.
- Reserved context shape:
  - `task.TaskGroup(ctx=None)` lowers and rejects non-`None` context values until M5 context propagation.
  - `task.spawn_scoped(..., ctx=None)` lowers through the active named structured owner and rejects non-`None` context values until M5.
- Scoped spawn proof:
  - `task.spawn_scoped` requires an active structured task owner and a named `async with task.TaskGroup() as group` / `task.scope() as scope` owner.
  - The helper reuses `scope.spawn` / `TaskGroup.spawn` task-boundary checks for direct coroutine calls, borrowed captures, non-send captures, homogeneous `TaskGroup` error typing, and affine handles.
- Named select syntax:
  - `task.select(first=..., second=...)` is the accepted M1 call shape.
  - Positional `task.select(a, b)` is rejected.
- Traceability:
  - `verification/stdlib/concurrency_runtime_m1_traceability.md`
- Local validation before M1 review:
  - `cargo fmt --check`: pass.
  - `cargo clippy --workspace -- -D warnings`: pass.
  - `cargo test -p sifr_lowering task_runtime_m1 -- --nocapture`: pass, 10 passed.
  - `cargo test -p sifr_codegen task_select -- --nocapture`: pass, 2 passed.
  - `cargo test -p sifr test_e2e_fail -- --nocapture`: pass.
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/task_spawn_scoped_named_owner.sifr`: pass.
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_spawn_scoped_without_owner_rejected.sifr`: expected fail with `SIFR-TYPE-0002`.
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/task_select_first_completion.sifr`: pass.
  - `scripts/run_e2e_pass.sh --profile create-pr`: pass; create-pr e2e reported 71 passed, 0 failed.
  - `python3 scripts/check_hir_maintainability_guardrails.py`: pass.
  - `python3 scripts/check_file_size_guardrails.py`: pass, 2115 files under the 900-line limit.
  - `scripts/run_all_tests.sh --profile create-pr`: pass; create-pr e2e reported 71 passed, 0 failed; platform golden reported pass=5, skip=2; advisory: warm wall-time budget exceeded.
- Review-polish validation:
  - `cargo fmt --check`: pass.
  - `cargo clippy --workspace -- -D warnings`: pass.
  - `cargo test -p sifr_lowering task_runtime_m1 -- --nocapture`: pass, 10 passed.
  - `python3 scripts/check_file_size_guardrails.py`: pass, 2115 files under the 900-line limit.
  - `cargo run -q -p sifr -- check demos/structured_concurrency_demo/main.sifr`: pass.
  - `scripts/run_all_tests.sh --profile create-pr`: pass; create-pr e2e reported 71 passed, 0 failed; platform golden reported pass=5, skip=2; advisory: warm wall-time budget exceeded.
- M1 review loop:
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-1.md`: `PASS`; non-blocking select signature wording, async-with decomposition, and sequential same-name task-owner cleanup polish was applied.
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-2.md`: `PASS`; non-blocking demo/select and `spawn_scoped` model-doc polish was applied.
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-5.md`: `PASS`; reviewer verified the post-polish tree and is satisfied.

## M2 Implementation Ledger

Current M2 wave: synchronization, channels, and backpressure closure.

- Channel runtime hardening:
  - Generated `sifr.sync` channel runtime now uses explicit `tokio::sync::Notify` wakeups for sender capacity, receiver availability, close, and endpoint-drop events instead of full/empty yield-loop polling.
  - Sender and receiver wait loops enable their `Notified` future before checking channel state, preserving Tokio's documented no-lost-wake pattern for multi-waiter backpressure.
  - Existing sender/receiver close, FIFO, bounded backpressure, cancellation, async iteration, and drop semantics remain on the Sifr-owned public endpoint API.
- Synchronization guard policy:
  - `SemaphorePermit` is classified as a guard-like resource.
  - Live semaphore permits cannot cross `await` and cannot escape through return values.
  - Sync `Lock`/`RwLock` guard diagnostics remain intact.
- Surface disposition:
  - `Notify` is the accepted edge-triggered event primitive in M2.
  - Level-triggered `Event` behavior uses explicit state plus `Notify` in the first model.
  - `sync.AsyncMutex[T]`, `sync.AsyncRwLock[T]`, public `Barrier`, public `Once`, and Python-shaped queue accounting remain deferred/internal-only as recorded in the M2 traceability artifact.
- Traceability:
  - `verification/stdlib/concurrency_runtime_m2_sync_traceability.md`
  - `verification/stdlib/concurrency_runtime_substrate_inventory.md`
  - `verification/stdlib/concurrency_runtime_substrate_inventory.json`
  - `internal_docs/async_concurrency_model.md`
- Focused local validation before M2 review:
  - `cargo fmt --check`: pass.
  - `cargo clippy --workspace -- -D warnings`: pass.
  - `python3 scripts/check_file_size_guardrails.py`: pass, 2117 files under the 900-line limit.
  - `python3 scripts/check_hir_maintainability_guardrails.py`: pass.
  - `python3 -m json.tool verification/stdlib/concurrency_runtime_substrate_inventory.json`: pass.
  - `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json`: pass.
  - `cargo test -p sifr_lowering semaphore_permit -- --nocapture`: pass, 2 passed.
  - `cargo test -p sifr_lowering lock_guard -- --nocapture`: pass, 3 passed.
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/channel_backpressure.sifr`: pass.
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/channel_cancel_receive_no_loss.sifr`: pass.
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/notify_basic.sifr`: pass.
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/semaphore_basic.sifr`: pass.
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/semaphore_permit_across_await_rejected.sifr`: expected fail with `SIFR-OWN-0009`.
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/semaphore_permit_escape_rejected.sifr`: expected fail with `SIFR-OWN-0003`.
  - `cargo test -p sifr -- test_e2e_fail -- --nocapture`: pass; fail harness completed 399 fail fixtures.
  - `scripts/run_all_tests.sh --profile create-pr`: pass; report `target/validation_lane_reports/create-pr.latest.json`; platform golden reported pass=5, skip=2; create-pr e2e pass suite reported 71 passed, 0 failed; advisory: warm wall-time budget exceeded.
- M2 review loop:
  - `reviews/ad-hoc-production-concurrency-runtime-m2-sync-review-pass-1.md`: `PASS`; channel Notify wakeups, semaphore permit guard diagnostics, surface disposition, sendability/shareability coverage, traceability, host matrix, inventory, merge manifest, and validation evidence were verified.
  - `reviews/ad-hoc-production-concurrency-runtime-m2-sync-review-pass-2.md`: `NOT PASS`; technical implementation was verified, but reviewer required housekeeping so empty scratch artifacts and unrelated network/http work would not contaminate the M2 PR.
  - `reviews/ad-hoc-production-concurrency-runtime-m2-sync-review-pass-3.md`: `PASS`; staged M2 diff was verified after housekeeping, including `Notified::enable()` no-lost-wake behavior and the unstaged network/http work exclusion.

## Planning Review Remediation Retained In This Phase

- [x] Define multiprocessing start-method, typed IPC, shared-memory, and ownership constraints.
- [x] Narrow unsafe signal handler registration to `unsupported-with-diagnostic` for this phase.
- [x] Add milestone dependency graph.
- [x] Add shared concurrency/runtime error mapping requirement.
- [x] Name Tokio as the backing async runtime for internal lowering and require concrete feature expansion in M0.
- [x] Tie any future process-pool API to the same typed IPC gate.
- [x] Clarify that class-based cleanup/context APIs are independent production APIs, not fallback implementations for generator decorators.
- [x] Mark `contextmanager` and `asynccontextmanager` `unsupported-with-diagnostic` in this phase, with a revisit rule for a future generator semantics phase.
- [x] Add explicit cross-phase dependency contract for text/i18n and network/web consumers.
- [x] Clarify that core scheduler/task helpers are native runtime infrastructure, not CPython module parity work.
- [x] Assign private blocking/CPU offload substrate to this phase while keeping public `threading` module parity out of scope.
- [x] Add explicit terminal expectations for `signal.getsignal`, `signal.pause`, and `pthread_sigmask`.
- [x] Require M0 classification before any `sifr.asyncio`, `sifr.queue`, `sifr.subprocess`, `sifr.concurrent.futures`, or `sifr.multiprocessing` surface can remain visible; production scope is rejected for CPython-shaped adapters.
- [x] Mark `contextvars` parity and implicit per-task context propagation out of scope.
- [x] Add a Sifr-native task/request context planning item for tracing, deadlines, cancellation metadata, and web observability.
- [x] Add sendability/shareability as a phase-wide gate before task/thread/process boundary captures.
- [x] Preserve typed future/task cancellation and worker error propagation requirements as Sifr-native `sifr.task`/`sifr.runtime` behavior rather than `concurrent.futures` parity.
- [x] Preserve homogeneous worker result/cancellation/deadline fixture requirements for accepted native offload APIs.
- [x] Define `TaskGroup` child failure aggregation as typed aggregate evidence.
- [x] Make typed IPC design explicitly owned by M6, with no unnamed external prerequisite for future process workers.
- [x] Resolve `signal.pause()` to `unsupported-with-diagnostic` or `waived-with-rationale` in this phase with diagnostics and a future safe signal-handler or structured signal-stream revisit rule.
- [x] Add external-review owner and five-working-day fallback rule.
- [x] Pin subprocess text mode, warning text encoding, and text-open demos to text/i18n `milestone_text_i18n_1` completion.
- [x] Add no-backward-compatibility policy: no public CPython-shaped adapters, no bare CPython stdlib aliases, no legacy aliases, no deprecated behavior, no pickle-style fallbacks, and no compatibility shims.
- [x] Align the phase with the stdlib namespace cleanup: `sifr.*` remains the permanent public stdlib namespace and bare CPython stdlib import attempts get namespace-contract diagnostics.
- [x] Mark `subprocess.getoutput` and `subprocess.getstatusoutput` `unsupported-with-diagnostic` as legacy shell-invocation helpers.
- [x] Add shared support states: `production-substrate`, `production-public`, `internal-only`, `compat-adapter`, `deferred-to-phase-X`, `unsupported-with-diagnostic`, `host-limited`, and `rejected`.
- [x] Add no-toy-concurrency gate rejecting public partial modules that exist only because CPython has them.
- [x] Demote `multiprocessing` and `ProcessPoolExecutor` from baseline CPU parallelism; Sifr uses typed offload and data parallelism for CPU work.
- [x] Reject Python global `warnings` filter parity in this phase; use explicit structured diagnostics only.
- [x] Add M0 gates for the resolved decision register, import-resolution tests, host matrix, workload database, task typing, detached-task policy, task context, rejected CPython-shaped surface disposition, and reviewer designation.
- [x] Assign sendability/shareability compiler enforcement to M1 with M3/M4/M6 verification extensions.
- [x] Resolve `TaskGroup`/`JoinSet` distinction: `TaskGroup` is scoped structured concurrency with failure cancellation; `JoinSet` is a dynamically-growable homogeneous completed-work collection.
- [x] Resolve task collection typing: homogeneous by default, heterogeneous only with an explicit user sum/enum type.
- [x] Resolve detached-task policy: stable public tasks are structured by default; handle drop before failure observation is a diagnostic; detached tasks are rejected in this phase.
- [x] Assign subprocess text-mode ownership to M4 once text/i18n M1 is complete, or require explicit `milestone_concurrency_runtime_text_subprocess_integration` deferral.
- [x] Require M0 cleanup classification for existing `sifr.asyncio` veneer implementation debt.
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
- [x] Treat existing `sifr.asyncio` veneer as legacy implementation debt and ensure new native APIs do not depend on it.
- [x] Add a dedicated post-M0/pre-M1 legacy CPython-shaped surface removal gate so `sifr.subprocess`, `sifr.asyncio` new APIs, `sifr.queue`, `sifr.concurrent.futures`, and `sifr.multiprocessing` cannot remain public adapters while production APIs are implemented.
- [x] Tighten final implementation-readiness contract: M6 rejects Python-shaped process pools, M0 defines static handled-failure proof, M0a is included in implementation backlog scope, M0 tests the TaskGroup-only owner model, M0a updates async model docs if needed, M2 owns semaphore permit await policy, M4 owns expected/unexpected process-exit semantics, and M6 IPC compatibility is schema-generated.
- [x] Resolve `JoinSet` result ordering and `JoinItemId` role.
- [x] Clarify `parallel.map`/`Pool.map` async calling convention.
- [x] Name shell subprocess usage as the `@shell_exec` security effect.
- [x] Resolve `race` versus `select`: `race` is homogeneous collection competition returning index plus typed outcome, while `select` is named-branch competition returning branch tag plus typed outcome; both cancel losers with typed evidence.
- [x] Add structured runtime work model: scopes own async tasks, blocking offload, CPU offload, supervised child processes, and future typed process workers; raw threads/process pools are not public runtime worlds.
- [x] Record current implementation reality as M0 input: existing `TaskScope`/`TaskGroup` lowering, affine task/blocking handles, abort-based cancellation, generated task preamble, primitive generated channels, placeholder threading/concurrent surfaces, and sync shell-style subprocess intrinsics.
- [x] Separate boundary and communication models: send/share gates, same-process channels, process pipes, and typed IPC frames have distinct contracts.

## Implementation PRs

- M0: https://github.com/sifr-lang/sifr/pull/2310
- M0a: https://github.com/sifr-lang/sifr/pull/2311
- M1: https://github.com/sifr-lang/sifr/pull/2313
- M2: https://github.com/sifr-lang/sifr/pull/2315
- M3 first wave: https://github.com/sifr-lang/sifr/pull/2316
- M3 `task.spawn_cpu` wave: https://github.com/sifr-lang/sifr/pull/2318
- M3 `JoinSet` wave: https://github.com/sifr-lang/sifr/pull/2320
- M3 scoped owner offload wave: https://github.com/sifr-lang/sifr/pull/2323
- M3 default parallel pool closure: https://github.com/sifr-lang/sifr/pull/2326
- M3 closeout: https://github.com/sifr-lang/sifr/pull/2325
- M3: complete.
- M4 sync process foundation: https://github.com/sifr-lang/sifr/pull/2331
- M4 sync child wait: https://github.com/sifr-lang/sifr/pull/2334
- M4 timeout status evidence: https://github.com/sifr-lang/sifr/pull/2336
- M4 sync child kill: https://github.com/sifr-lang/sifr/pull/2337
- M4 signal status evidence: https://github.com/sifr-lang/sifr/pull/2341
- M4 async process run/output loopback: https://github.com/sifr-lang/sifr/pull/2345
- M4 sync stdout/stderr pipe readers: https://github.com/sifr-lang/sifr/pull/2352
- M4 async process run timeout: https://github.com/sifr-lang/sifr/pull/2354
- M4 sync stdin pipe writer: https://github.com/sifr-lang/sifr/pull/2357
- M4 async process output timeout: https://github.com/sifr-lang/sifr/pull/2362
- M4 async stdin-byte communicate: https://github.com/sifr-lang/sifr/pull/2365
- M4 sync process terminate: https://github.com/sifr-lang/sifr/pull/2367
- M4 async process spawn/wait: https://github.com/sifr-lang/sifr/pull/2369
- M4 method-form async child kill/terminate: https://github.com/sifr-lang/sifr/pull/2372
- M4 async process runtime split: https://github.com/sifr-lang/sifr/pull/2375
- M4 sync PipeReader streaming reads: https://github.com/sifr-lang/sifr/pull/2377
- M4 top-level async child kill/terminate: https://github.com/sifr-lang/sifr/pull/2378
- M4 async owned process pipes: https://github.com/sifr-lang/sifr/pull/2381
- M4 process handle boundary diagnostics: https://github.com/sifr-lang/sifr/pull/2382
- M4 async wait cancellation-safe observation: https://github.com/sifr-lang/sifr/pull/2386
- M4 subprocess strict text encoding: https://github.com/sifr-lang/sifr/pull/2390
- M4 async shell process APIs: https://github.com/sifr-lang/sifr/pull/2393
- M4 scoped process supervision: https://github.com/sifr-lang/sifr/pull/2392
- M4 timeout process-group cleanup: https://github.com/sifr-lang/sifr/pull/2396
- M4 sync child drop cleanup: https://github.com/sifr-lang/sifr/pull/2398
- M4 scoped parent cancellation evidence: https://github.com/sifr-lang/sifr/pull/2400
- M4 closeout: https://github.com/sifr-lang/sifr/pull/2403
- M4: complete.
- M5 signal value-model foundation: https://github.com/sifr-lang/sifr/pull/2405
- M5 warnings global-filter rejection: https://github.com/sifr-lang/sifr/pull/2407
- M5 resource nullcontext foundation: https://github.com/sifr-lang/sifr/pull/2409
- M5 signal `strsignal` value-helper: https://github.com/sifr-lang/sifr/pull/2412
- M5 task context value-model foundation: https://github.com/sifr-lang/sifr/pull/2414
- M5 signal constants: https://github.com/sifr-lang/sifr/pull/2416
- M5 resource value-carrying nullcontext: https://github.com/sifr-lang/sifr/pull/2419
- M5 signal stream shape and lowering: https://github.com/sifr-lang/sifr/pull/2418
- M5 resource cleanup helper diagnostics: https://github.com/sifr-lang/sifr/pull/2423
- M5 signal stream Unix delivery harness: https://github.com/sifr-lang/sifr/pull/2426
- M5 structured runtime diagnostics: https://github.com/sifr-lang/sifr/pull/2428
- M5 explicit task context propagation: https://github.com/sifr-lang/sifr/pull/2431
- M5 runtime diagnostic metrics policy: https://github.com/sifr-lang/sifr/pull/2433
- M5 cancellation cleanup traceability addendum: https://github.com/sifr-lang/sifr/pull/2430
- M5: complete.
- M6 typed IPC design gate: https://github.com/sifr-lang/sifr/pull/2437
- M6 typed IPC dependency metadata: https://github.com/sifr-lang/sifr/pull/2439
- M6 typed IPC value model: https://github.com/sifr-lang/sifr/pull/2441
- M6 typed IPC schema hash: https://github.com/sifr-lang/sifr/pull/2443
- M6 typed IPC frame codec: https://github.com/sifr-lang/sifr/pull/2445
- M6 typed IPC stream read/write: https://github.com/sifr-lang/sifr/pull/2447
- M6 typed IPC request tracker: https://github.com/sifr-lang/sifr/pull/2450
- M6 typed IPC connection state: https://github.com/sifr-lang/sifr/pull/2452
- M6 typed IPC payload eligibility: https://github.com/sifr-lang/sifr/pull/2454
- M6 typed IPC Unix process-pipe fixture: https://github.com/sifr-lang/sifr/pull/2455
- M6 typed IPC process-pipe backpressure and unsupported-payload evidence: https://github.com/sifr-lang/sifr/pull/2458
- M6 typed IPC payload diagnostics: https://github.com/sifr-lang/sifr/pull/2460
- M6 typed IPC CPython-shaped multiprocessing diagnostics: https://github.com/sifr-lang/sifr/pull/2462
- M6 typed IPC compiler-internal schema extraction: https://github.com/sifr-lang/sifr/pull/2464
- M6 typed IPC generated worker-boundary compose proof: https://github.com/sifr-lang/sifr/pull/2470
- M6 typed IPC closeout classification: https://github.com/sifr-lang/sifr/pull/2467
- M6: complete.
- M7 traceability scaffold: https://github.com/sifr-lang/sifr/pull/2469
- M7 public documentation: https://github.com/sifr-lang/sifr/pull/2473
- M7 internal architecture audit: https://github.com/sifr-lang/sifr/pull/2476
- M7 demo closure: https://github.com/sifr-lang/sifr/pull/2479
- M7 generated dependency and panic-scan evidence: https://github.com/sifr-lang/sifr/pull/2482
- M7 validation lane and inventory closure: https://github.com/sifr-lang/sifr/pull/2485
- M7 final review and validation gate: https://github.com/sifr-lang/sifr/pull/2488
- M7: complete.

## Validation Evidence

Record local validation for each milestone before opening its PR.

M4 async wait cancellation-safe observation merge ledger:

- Merged as PR #2386 (`d54d2c11497e54ca5db3061d8e026ee2afb09154`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`134.60s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`108 passed`, `0 failed`, `cache_hits=26/27`, `report_signature=df97adcd1a958b0c`).

M4 scoped process supervision implementation:

- Added `sifr.process.ProcessHandle`, with `wait()`, `kill()`, `terminate()`, `stdin()`, `stdout()`, and `stderr()` methods backed by the existing Tokio async child/pipe handle tables.
- Added `scope.spawn_process(command)` / `TaskGroup.spawn_process(command)` lowering to a generated `__SifrTaskScope` method returning `Result[ProcessHandle, ProcessError]`.
- Added delayed scope-exit process observation: scoped process observers start only during scope cleanup so owned pipe extraction remains available inside the scope body. Explicit `ProcessHandle.wait()` marks the process as observed; unobserved successful processes are joined by scope cleanup; TaskGroup fail-fast cancellation triggers a process kill hook and keeps the observer alive to reap.
- Split async process preamble needs so `ProcessHandle` pipe/wait users emit child table and pipe helpers without requiring the public `AsyncChild` spawn function.
- Added `process_scoped_spawn_handle` fixture coverage to create-pr and merge manifests.
- Updated M4 process traceability and supported-host matrix with scoped process supervision evidence and host-limited Windows status.

M4 scoped process supervision targeted local validation:

- `cargo check -q -p sifr_codegen -p sifr_lowering -p sifr_stdlib` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_scoped_spawn_handle.sifr` -> PASS.

M4 scoped process supervision merge ledger:

- Merged as PR #2392 (`db1872c550be86ad35cf8050f0ed0286ad5cfa62`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`230.49s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`111 passed`, `0 failed`, `cache_hits=22/29`, `report_signature=7564a7fcb0791ad4`).

M4 sync child drop cleanup implementation:

- Added module-aware stdlib codegen so `sifr.process` resource wrappers can opt out of generated `Clone` without changing ordinary user classes named `Child`.
- Removed auto-clone/equality derives from `sifr.process` child/pipe handle wrappers and added a generated `Drop` implementation for sync `Child`.
- `Child` drop now removes an unwaited child handle from the generated sync child table, dropping the underlying `std::process::Child` handle without pretending to kill, wait, or recursively supervise the host process.
- Updated M4 traceability and supported-host matrix evidence for deterministic sync child handle cleanup while keeping termination escalation and non-Unix signal semantics as remaining lifecycle work.

M4 sync child drop cleanup targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo check -q -p sifr_codegen -p sifr_driver` -> PASS.
- `cargo test -p sifr_codegen process_child_resource_derives_are_module_scoped -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_pipe_readers.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_pipe_writer.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_pipes.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_scoped_spawn_handle.sifr` -> PASS.
- Emission check for `process_spawn_wait_status` -> PASS; emitted Rust shows `sifr.process` resource wrappers derive `Debug` only and `Child` has an `impl Drop` that removes unwaited handles from `__SIFR_PROCESS_CHILDREN`.
- Post-`origin/main` rebase rerun after the scoped process supervision and timeout process-group cleanup merges: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`131.14s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`113 passed`, `0 failed`, `cache_hits=30/30`, `report_signature=5cbbb189c83d1068`).

M4 sync child drop cleanup review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-sync-child-drop-cleanup-review-pass-1.md`: `PASS`; reviewer verified the module-scoped resource derive suppression, generated sync `Child` drop cleanup, non-clone process handle wrappers, targeted codegen coverage, traceability/host matrix honesty, and no file-size or panic-safety blockers. Non-blocking follow-ups were kept out of the implementation scope for top-level wait `_waited` bookkeeping and future async handle/table cleanup.

M4 sync child drop cleanup merge ledger:

- Merged as PR #2398 (`f84f854a84f08d7d2c39ab66d200ac1d53b15039`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; no advisories (`118.68s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`113 passed`, `0 failed`, `cache_hits=30/30`, `report_signature=5cbbb189c83d1068`).

M4 scoped parent cancellation evidence implementation:

- Added `process_scoped_parent_cancel` fixture coverage for `TaskGroup.spawn_process(...)` fail-fast cancellation stopping a scoped child before delayed marker side effects can escape.
- Added the fixture to create-pr and merge e2e manifests.
- Updated M4 process traceability and supported-host matrix to close the parent-cancellation evidence follow-up while preserving non-Unix status/termination semantics as intentionally host-limited.

M4 scoped parent cancellation evidence targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `git diff --check` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_scoped_parent_cancel.sifr` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`137.24s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`114 passed`, `0 failed`, `cache_hits=29/30`, `report_signature=b11e218d104a7820`).

M4 scoped parent cancellation evidence review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-parent-cancel-review-pass-3.md`: `PASS`; reviewer verified the dedicated fixture's PID-scoped marker cleanup, fail-fast `TaskGroup.spawn_process(...)` cancellation evidence, Unix shell timing guard, manifest entries, validation evidence, and documentation honesty around immediate-child cancellation only. Non-Unix signal/status/termination and process-group/descendant claims remain intentionally open.

M4 scoped parent cancellation evidence merge ledger:

- Merged as PR #2400 (`13e98095d2b35d5a91259b3667285a7af0c208f3`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`173.75s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`114 passed`, `0 failed`, `cache_hits=25/30`, `report_signature=b11e218d104a7820`).

M4 closeout classification implementation:

- Marked `milestone_concurrency_runtime_4` complete in this ledger.
- Closed the M4 process traceability document by replacing stale pending lifecycle wording with supported macOS/Linux evidence and explicit host-limited non-Unix/Windows classification.
- Promoted the supported-host matrix subprocess umbrella row to `supported` on macOS/Linux while preserving Windows as `host-limited` and pointing to the dedicated process rows.
- Kept post-M4 future work limited to optional strict-text error-handler expansion, future stdlib re-export workload metadata, and explicitly host-limited non-Unix status/termination fixture work.

M4 closeout classification targeted local validation:

- `git diff --check` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`145.47s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`114 passed`, `0 failed`, `cache_hits=28/30`, `report_signature=b11e218d104a7820`).

M4 closeout classification review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-closeout-audit-pass-1.md`: `PASS`; reviewer verified all blocking M4 DoD items have merged runtime and test evidence on macOS/Linux, and identified stale docs-only status/classification wording to remediate before closeout.
- `reviews/ad-hoc-production-concurrency-runtime-m4-closeout-review-pass-1.md`: `PASS`; reviewer verified the docs-only closeout diff correctly closes M4, removes stale pending lifecycle wording from active M4 surfaces, keeps non-Unix/Windows semantics host-limited, records validation evidence, and leaves M5 as the next pending entry.

M4 closeout classification merge ledger:

- Merged as PR #2403 (`3f4512625a3eec3206276b8e96bd7bf915f0b172`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`123.41s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`114 passed`, `0 failed`, `cache_hits=28/30`, `report_signature=b11e218d104a7820`).

M5 signal value-model foundation implementation:

- Added the first `sifr.signal` embedded stdlib module with `Signal`, `SignalError`, and portable `sigint()` / `sigterm()` value helpers.
- Added `signal_value_model_basic` pass coverage and create-pr/merge manifest entries.
- Added unsupported signal API import diagnostics for `pause`, arbitrary handler registration through `signal`, `getsignal`, `raise_signal`, and `pthread_sigmask`.
- Added the M5 shutdown traceability artifact with signal host matrix, signal follow-up boundaries, and cleanup/context/diagnostics follow-up slots.
- Updated the supported-host matrix to mark the signal value model supported on macOS/Linux/Windows while keeping structured signal streams in-progress and Windows delivery semantics host-limited.

M5 signal value-model foundation targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo test -p sifr_stdlib stdlib_source_inventory_contains_user_modules -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_value_model_basic.sifr` -> PASS.
- New signal unsupported fixtures `signal_pause_unsupported`, `signal_handler_registration_unsupported`, `signal_getsignal_unsupported`, `signal_raise_signal_unsupported`, and `signal_pthread_sigmask_host_limited` -> expected `SIFR-NAME-0004`.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail suite reported `439 fail tests completed`.
- `cargo fmt --check` and `git diff --check` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`195.32s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`115 passed`, `0 failed`, `cache_hits=22/31`, `report_signature=fa75f7f525acd21c`).

M5 signal value-model foundation review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-foundation-review-pass-1.md`: `PASS`; reviewer verified the embedded `sifr.signal` registration, `Signal`/`sigint()`/`sigterm()` value-model coverage, unsupported CPython-style signal API missing-member diagnostics, manifest entries, host-matrix scope, and validation evidence. Non-blocking follow-ups requested clearer `SignalError`, `pthread_sigmask`, and Windows-by-inspection wording.
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-foundation-review-pass-2.md`: `PASS`; reviewer verified the docs-only follow-up wording does not overclaim implementation, keeps structured streams/constants/signal delivery as follow-up, and accurately describes the current missing-member diagnostics.

M5 signal value-model foundation merge ledger:

- Merged as PR #2405 (`98d858f0057e3bab9cab74a1d90e45f3c278566b`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`231.98s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`115 passed`, `0 failed`, `cache_hits=23/31`, `report_signature=fa75f7f525acd21c`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR #2405 merge commit, foundation-only scope wording, validation metrics, advisory wording, guardrails, and lane-step coverage for this docs-only merge-ledger PR.

M5 signal stream shape and lowering implementation:

- Added `_sifr.signal` intrinsic typing and codegen lowering for awaitable `signal_ctrl_c`, `signal_terminate`, and `signal_shutdown` backed by Tokio signal APIs.
- Extended public `sifr.signal` with `strsignal(...)`, `ctrl_c()`, `terminate()`, `ShutdownStream.next()`, and `shutdown_stream()` while preserving the existing `Signal(name, number)` value model.
- Added Tokio's `signal` feature to the generated Tokio dependency bundle so signal-backed awaitables compile in generated projects.
- Added `signal_stream_shape_strsignal` pass coverage for `strsignal(...)` and the public awaitable stream shapes without polling host signals, plus codegen registry coverage for the actual Tokio Ctrl-C/SIGTERM lowerings.
- Updated M5 shutdown traceability and the supported-host matrix to keep deterministic external signal delivery as follow-up while marking stream shape/lowering in progress on macOS/Linux and host-limited for Windows SIGTERM.

M5 signal stream shape and lowering targeted local validation:

- `cargo fmt --check` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo check -q -p sifr_stdlib -p sifr_codegen` -> PASS.
- `cargo test -p sifr_codegen lowers_signal_intrinsics_via_registry -- --nocapture` -> PASS.
- `cargo test -p sifr_stdlib stdlib_source_inventory_contains_user_modules -- --nocapture` -> PASS.
- `cargo test -p sifr_driver test_generate_test_runner_cargo_toml_includes_required_features -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_stream_shape_strsignal.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_value_model_basic.sifr` -> PASS.
- Emission check for `signal_stream_shape_strsignal` -> PASS; emitted Rust includes `ShutdownStream`, public `ctrl_c()` / `terminate()` wrappers, `tokio::signal::ctrl_c().await`, Unix `tokio::signal::unix::SignalKind::terminate()`, and typed non-Unix `SIGTERM is unsupported on this host`.
- Post-review blocker remediation updated the grouped e2e harness Tokio dependency spec and matching assertions to include Tokio's `signal` feature, matching the generated dependency bundle used by normal project/run paths.
- Post-remediation `cargo test -p sifr_codegen test_generate_project_emits_tokio_dependency_when_required -- --nocapture` -> PASS.
- Post-remediation `cargo test -p sifr test_generate_cargo_toml_required_tokio_uses_runtime_features -- --nocapture` -> PASS.
- Post-remediation `scripts/run_e2e_pass.sh --profile create-pr` -> PASS; create-pr e2e pass suite covered `117` fixtures with `117 passed`, `0 failed`, `cache_hits=3/26`, `report_signature=ded105ad58090608`.
- Broad non-profiled probe `cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` still exposed unrelated existing text/I/O and bytes conversion failures (`cpython_io_subset`, `stdlib_io_consolidated`, `open_context_manager`, `open_read`, `open_readline`, `open_write`, `bytes_conversion_errors`) and is not the accepted gate for this wave; the authoritative profiled create-pr e2e lane above passed.
- Post-remediation `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; e2e pass suite `117 passed`, `0 failed`, `cache_hits=22/33`, `report_signature=ded105ad58090608`; platform golden `pass=6`, `skip=1`. Advisories: warm wall-time budget exceeded (`1041.25s`, warm target `<=2m`) and warm-cache hit rate below advisory target (`67%`, target `>=90%`).
- Post-rebase `scripts/run_all_tests.sh --profile create-pr` -> PASS after resolving docs against PR #2419; report `target/validation_lane_reports/create-pr.latest.json`; e2e pass suite `120 passed`, `0 failed`, `cache_hits=27/34`, `report_signature=293aaf3695dc42f8`; platform golden `pass=6`, `skip=1`. Advisories: warm wall-time budget exceeded (`959.65s`, warm target `<=2m`) and warm-cache hit rate below advisory target (`79%`, target `>=90%`).

M5 signal stream shape and lowering review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-stream-review-pass-1.md`: `FAIL`; reviewer verified the API/lowering shape and host semantics but blocked the PR because the grouped e2e harness and two assertions still hard-coded Tokio without the `signal` feature. The blocker was remediated by updating the harness dependency spec, harness contract assertion, and codegen dependency assertion; the shape-only fixture also now documents that signal futures are intentionally not awaited.
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-stream-review-pass-2.md`: `PASS`; reviewer verified the Tokio `signal` feature rollout is now consistent across generated projects, grouped e2e harness Cargo generation, harness contract tests, and codegen dependency tests, and that the full create-pr lane passed with the new signal fixture.
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-stream-review-pass-3.md`: `PASS`; post-rebase reviewer verified PR #2418's merged code, the conflict-resolved docs against PR #2419, the post-rebase create-pr validation metrics, generated Tokio `signal` feature consistency, typed `SignalError` host paths, and no overclaim beyond stream shape/lowering.

M5 signal stream shape and lowering merge ledger:

- Merged as PR #2418 (`abdd8674b9a51dc88260782283b6f47c4c7791ff`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`1373.86s`, warm target `<=2m`) and warm-cache hit rate below advisory target (`53%`, target `>=90%`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`120 passed`, `0 failed`, `cache_hits=18/34`, `report_signature=293aaf3695dc42f8`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-stream-ledger-review-pass-2.md`: `PASS`; reviewer verified PR #2418 metadata, merge SHA/date, the earlier warm-cache validation snapshot, review artifact references, branch hygiene, and no overclaim beyond stream shape/lowering.
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-stream-ledger-review-pass-3.md`: `FAIL`; reviewer verified the corrected validation metrics and PR metadata but blocked because the pass-2 ledger bullet still claimed to verify the final metrics after the final cache-hit/advisory wording changed. This bullet was corrected before the next review pass.
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-stream-ledger-review-pass-4.md`: `PASS`; reviewer verified the pass-2 overclaim correction, honest pass-3 failure record, corrected validation metrics, and no new signal-stream scope overclaim.

M5 warnings global-filter rejection implementation:

- Added `warnings_filter_global_rejected` to pin Python `warnings.filterwarnings` global filter parity as a static unsupported CPython import diagnostic.
- Updated the M5 shutdown traceability artifact to close the warning-global rejection surface while keeping structured diagnostics/tracing runtime work separate.

M5 warnings global-filter rejection targeted local validation:

- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/warnings_filter_global_rejected.sifr` -> expected `SIFR-IMPORT-0009`.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail suite reported `440 fail tests completed`.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`138.24s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`115 passed`, `0 failed`, `cache_hits=31/31`, `report_signature=fa75f7f525acd21c`).

M5 warnings global-filter rejection review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-warnings-filter-review-pass-1.md`: `PASS`; reviewer verified the `filterwarnings` fixture pins the legacy `sifr.warnings` rejection path, the traceability artifact closes only warning-filter parity while keeping diagnostics/tracing runtime work open, the ledger keeps M5 in progress, and the validation evidence is accurate.

M5 warnings global-filter rejection merge ledger:

- Merged as PR #2407 (`58813d6edb620abd3bd6f1461d616fa67bff86f4`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`160.63s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`115 passed`, `0 failed`, `cache_hits=27/31`, `report_signature=fa75f7f525acd21c`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-warnings-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR #2407 merge commit, validation metrics, advisory wording, lane-step coverage, and in-progress M5 scope for this docs-only merge-ledger PR.

M5 resource nullcontext foundation implementation:

- Added the embedded `sifr.resource` module with `NullContext` and no-value `nullcontext()`.
- Added `resource_nullcontext_basic` pass coverage and create-pr/merge manifest entries.
- Added unsupported `sifr.resource` import diagnostics for `redirect_stdout`, `redirect_stderr`, `chdir`, `suppress`, `contextmanager`, and `asynccontextmanager`.
- Updated the M5 shutdown traceability artifact and supported-host matrix to mark no-value no-op `nullcontext()` as supported while leaving value-carrying generic nullcontext, cleanup stacks, owned closing helpers, cancellation cleanup reports, and async cleanup as M5 follow-up work.

M5 resource nullcontext foundation targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo test -p sifr_stdlib stdlib_source_inventory_contains_user_modules -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/resource_nullcontext_basic.sifr` -> PASS.
- New resource unsupported fixtures `resource_redirect_stdout_unsupported`, `resource_redirect_stderr_unsupported`, `resource_chdir_unsupported`, `resource_suppress_unsupported`, `resource_contextmanager_unsupported`, and `resource_asynccontextmanager_unsupported` -> expected `SIFR-NAME-0004`.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail suite reported `446 fail tests completed`.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`204.80s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`116 passed`, `0 failed`, `cache_hits=31/32`, `report_signature=6dd646fdf4fc2cb4`).

M5 resource nullcontext foundation review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-nullcontext-review-pass-1.md`: `PASS`; reviewer verified `sifr.resource` registration, no-value `nullcontext()` with the synchronous `with` protocol, missing-member diagnostics for rejected CPython contextlib helpers, manifest entries, traceability and supported-host matrix honesty, and validation evidence. Non-blocking feedback requested removing the premature unexercised `ResourceError` symbol.
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-nullcontext-review-pass-2.md`: `PASS`; reviewer verified the `ResourceError` cleanup, final validation metrics, and absence of overclaims for `ResourceError`, `ExitStack`, `AsyncExitStack`, `closing`/`aclosing`, generic value-carrying nullcontext, or async cleanup.

M5 resource nullcontext foundation merge ledger:

- Merged as PR #2409 (`5001b0985838a240a7adeb01adf6fa343970cb36`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`132.53s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`116 passed`, `0 failed`, `cache_hits=31/32`, `report_signature=6dd646fdf4fc2cb4`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-ledger-review-pass-1.md`: `FAIL`; reviewer found the top-level M5 status block had been promoted from `in progress.` to the PR URL, inconsistent with the accepted M5 foundation-slice ledger convention. The status line was restored before the second review pass.
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-ledger-review-pass-2.md`: `PASS`; reviewer verified the status-block convention fix, PR #2409 merge commit/date, validation metrics and advisories, lane-step coverage, and no overclaim beyond the no-value `nullcontext()` slice.
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-ledger-review-pass-3.md`: `PASS`; reviewer verified the final create-pr rerun metrics, single advisory, status-block convention, review-loop bullets, branch scope, and no overclaim beyond the no-value `nullcontext()` slice.

M5 resource value-carrying nullcontext implementation:

- Extended `sifr.resource.NullContext` to `NullContext[T]` with a carried `value: T`.
- Kept `nullcontext()` available through a default `None` argument and added `nullcontext(value)` support so the `with` binding receives the carried value type.
- Updated generated synchronous `with` guards to render concrete generic context-manager types, preventing bare generic guard fields such as `NullContext` when the source expression has type `NullContext[int]`.
- Added narrow generated-code handling for `None` literals passed to Sifr `None` or generic type parameters so `nullcontext()` lowers to unit instead of Rust `Option`.
- Relaxed generated bounds for `NullContext[T]` and `nullcontext[T]` to the actual `Clone` requirement.
- Updated `resource_nullcontext_basic` to cover no-value, integer, and string payload forms.
- Updated M5 shutdown traceability, supported-host matrix, and substrate inventory docs to mark no-value and value-carrying generic `nullcontext(...)` as supported while leaving cleanup stacks, owned closing helpers, async cleanup, cancellation cleanup ordering, and task context propagation as M5 follow-up work.

M5 resource value-carrying nullcontext targeted local validation:

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/resource_nullcontext_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/resource_nullcontext_basic.sifr` -> PASS; emitted guards included `NullContext<()>`, `NullContext<i64>`, and `NullContext<String>`.
- `cargo check -p sifr_codegen` -> PASS.
- `cargo check -p sifr_lowering -p sifr_codegen -p sifr_driver` -> PASS.
- `cargo fmt --check`, touched-file `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`196.47s`, warm target `<=2m`) and warm-cache hit rate below advisory target (`85%`, target `>=90%`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`119 passed`, `0 failed`, `cache_hits=28/33`, `report_signature=0df4819d3daf7aa4`).

M5 resource value-carrying nullcontext review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-nullcontext-value-review-pass-1.md`: `PASS`; reviewer verified the generic `NullContext[T]` source surface, no-value and value-carrying `nullcontext(...)` behavior, generated guards with `NullContext<()>`, `NullContext<i64>`, and `NullContext<String>`, no generated user-path panic/fallback/runtime leak, narrow `None`-to-unit lowering for `Type::None`/`TypeVar` without changing `Option` parameters, scoped synchronous `with` guard codegen, and docs honesty for cleanup-stack/async-cleanup follow-ups.

M5 resource value-carrying nullcontext merge ledger:

- Merged as PR #2419 (`4c67a99ecdba74d4d8693b1643b9c98a9e823de7`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`142.67s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`119 passed`, `0 failed`, `cache_hits=33/33`, `report_signature=0df4819d3daf7aa4`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-nullcontext-value-ledger-review-pass-1.md`: `PASS`; reviewer verified PR #2419, merge SHA/date, validation metrics and single advisory, status-block convention, cache-hit advisory removal after the fully warm rerun, and no overclaim beyond no-value plus value-carrying generic `nullcontext(...)`.

M5 resource cleanup helper diagnostics implementation:

- Added negative fixtures for `sifr.resource.ExitStack`, `AsyncExitStack`, `closing`, and `aclosing`, pinning each unsupported helper to the stable missing-member diagnostic.
- Updated M5 shutdown traceability, supported-host matrix, and substrate inventory docs to close cleanup stacks and owned closing helpers as diagnostics for this phase while preserving `nullcontext(...)` support.
- Recorded the implementation blocker for future support: cleanup stacks need typed cleanup-error aggregation, and `closing`/`aclosing` require an owned-close protocol that can honestly preserve mutating and fallible close behavior.

M5 resource cleanup helper diagnostics targeted local validation:

- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/resource_exitstack_unsupported.sifr` -> expected `SIFR-NAME-0004` for missing member `ExitStack`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/resource_async_exitstack_unsupported.sifr` -> expected `SIFR-NAME-0004` for missing member `AsyncExitStack`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/resource_closing_unsupported.sifr` -> expected `SIFR-NAME-0004` for missing member `closing`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/resource_aclosing_unsupported.sifr` -> expected `SIFR-NAME-0004` for missing member `aclosing`.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; `451 fail tests completed` and harness result `1 passed`, `0 failed`. Two pre-existing fail-suite ICE diagnostic captures were printed by the harness and did not fail the suite.
- `python3 -m json.tool verification/stdlib/concurrency_runtime_substrate_inventory.json`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`339.84s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`120 passed`, `0 failed`, `cache_hits=34/34`, `report_signature=293aaf3695dc42f8`).

M5 resource cleanup helper diagnostics review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-cleanup-diagnostics-review-pass-1.md`: `PASS`; reviewer verified the four unsupported fixtures, true missing-member behavior from the `sifr.resource` surface, preserved `nullcontext(...)` scope, future blockers for typed cleanup-error aggregation and owned-close protocol support, host-matrix and traceability consistency, and the targeted validation evidence.

M5 resource cleanup helper diagnostics merge ledger:

- Merged as PR #2423 (`efaf92ed58bc85e92a7f4f6aef2ed4488ae59e47`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`973.46s`, warm target `<=2m`) and warm-cache hit rate below advisory target (`68%`, target `>=90%`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`120 passed`, `0 failed`, `cache_hits=23/34`, `report_signature=293aaf3695dc42f8`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-cleanup-diagnostics-ledger-review-pass-1.md`: `FAIL`; reviewer verified PR #2423 metadata and scope but blocked because the merge-ledger validation row copied PR #2418 cache-hit/advisory metrics instead of the cited PR #2423 report.
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-cleanup-diagnostics-ledger-review-pass-2.md`: `FAIL`; reviewer verified the same blocker remained after the first attempted correction targeted the wrong row.
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-cleanup-diagnostics-ledger-review-pass-3.md`: `PASS`; reviewer verified the corrected PR #2423 validation metrics, merge SHA/date, honest failure history, and no scope overclaim beyond diagnostic closure for unsupported cleanup helpers.
- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-cleanup-diagnostics-ledger-review-pass-4.md`: `PASS`; reviewer verified the post-rerun PR #2423 validation metrics, dual advisory, `cache_hits=23/34`, `report_signature=293aaf3695dc42f8`, merge SHA/date, honest failure history, and no scope overclaim beyond diagnostic closure for unsupported cleanup helpers.

M5 signal stream Unix delivery harness implementation:

- Added `signal_stream_delivery_unix` pass coverage for deterministic child-sent signal delivery to the current Sifr process.
- The fixture awaits `ctrl_c()`, `terminate()`, and `shutdown_stream().next()` while a delayed child shell sends `SIGINT` or `SIGTERM`, then waits for the child command to exit successfully.
- The fixture is explicitly host-gated with `sifr.platform.system() == "Windows"` so Windows does not claim Unix signal delivery semantics.
- Updated M5 shutdown traceability and the supported-host matrix to mark Unix signal stream delivery supported on macOS/Linux while keeping non-Unix delivery and Unix-only constants host-limited.

M5 signal stream Unix delivery harness targeted local validation:

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_stream_delivery_unix.sifr` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_e2e_pass.sh --profile create-pr` -> PASS; create-pr e2e pass suite covered `121` fixtures with `121 passed`, `0 failed`, `cache_hits=23/28`, `report_signature=d760194c89dbc954`.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded and warm-cache hit rate below target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`121 passed`, `0 failed`, `cache_hits=28/35`, `report_signature=d760194c89dbc954`). Wall time was `1699.18s`; slowest step was `crate_tests` at `1265945ms`.

M5 signal stream Unix delivery harness review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-delivery-review-pass-1.md`: `PASS`; reviewer verified deterministic child-sent Unix signal delivery for `ctrl_c()`, `terminate()`, and `shutdown_stream().next()`, Windows gating, child wait observation, traceability/host-matrix honesty, no public API addition, and no overclaim for Unix-only constants or non-Unix semantics.

M5 signal stream Unix delivery harness merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2426
- Merge commit: `1f04c697dccd358384de73eeb09aceda7417563e`
- Merged at: `2026-06-08T20:14:56Z`
- Scope: deterministic Unix signal delivery pass coverage for `ctrl_c()`, `terminate()`, and `shutdown_stream().next()`, with Windows host gating and updated traceability/host-matrix boundaries.
- Merge-ledger validation: docs-only ledger update; `git diff --check` -> PASS.

M5 signal stream Unix delivery harness merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-delivery-ledger-review-pass-1.md`: `PASS`; reviewer verified PR #2426 URL, merge commit/date, review-loop citation, local validation evidence, Windows host gating, and no scope overclaim for Unix-only constants or non-Unix delivery semantics.

M5 structured runtime diagnostics implementation:

- Added public `sifr.runtime` diagnostic value types: `DiagnosticLevel`, `DiagnosticEvent`, and `DiagnosticError`, plus `diagnostic_event(...)` and `emit_diagnostic(...) -> Result[None, DiagnosticError]`.
- Added `_sifr.runtime.runtime_emit_diagnostic(...)` intrinsic typing and codegen lowering to structured `tracing::event!` calls with fixed internal target `sifr.runtime` and structured `diagnostic_target`, `diagnostic_name`, and `diagnostic_message` fields.
- Added locked `tracing = { version = "0.1.44", default-features = false, features = ["std"] }` dependency wiring for generated projects and grouped e2e batch crates without exposing tracing types in Sifr source.
- Added `runtime_diagnostics_tracing` pass coverage to create-pr and merge manifests, plus codegen and grouped Cargo.toml contract tests.
- Updated M5 shutdown traceability to close structured runtime diagnostic events while deferring metrics until concrete metric names, label/cardinality policy, emission points, redaction policy, and deterministic tests are approved.

M5 structured runtime diagnostics targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_codegen runtime_diagnostic -- --nocapture` -> PASS.
- `cargo test -p sifr_codegen runtime_module_dependency_metadata_includes_tracing_only -- --nocapture` -> PASS.
- `cargo test -p sifr_stdlib runtime -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/runtime_diagnostics_tracing.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/runtime_diagnostics_tracing.sifr` -> PASS.
- `SIFR_E2E_FIXTURE_MANIFEST=/tmp/sifr-m5-diagnostics-next/verification/validation_lanes/create_pr_e2e_manifest.json cargo test -p sifr --test e2e test_e2e_pass -- --nocapture` -> PASS; `121` pass tests completed, `0` failed, `report_signature=d760194c89dbc954`.
- Post-review and post-rebase `scripts/run_all_tests.sh --profile create-pr` on the final PR base -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`136.44s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`122 passed`, `0 failed`, `cache_hits=36/36`, `report_signature=e04a8b6c2c420820`). The final total is one fixture higher than the mid-review manifest rerun because PR #2426's `signal_stream_delivery_unix` fixture landed on `main` before the final rebase.

M5 structured runtime diagnostics review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-runtime-diagnostics-review-pass-1.md`: `FAIL`; reviewer found the traceability document claimed `runtime_diagnostics_tracing` coverage in create-pr and merge lanes before the fixture was listed in either lane manifest. The blocker was remediated by adding the fixture to both manifests and rerunning the lane.
- `reviews/ad-hoc-production-concurrency-runtime-m5-runtime-diagnostics-review-pass-2.md`: `PASS`; reviewer verified the manifest blocker was fixed, the grouped e2e batch Cargo.toml `tracing` dependency gap was closed with inference/spec/contract coverage, the public diagnostic value surface stays Sifr-owned, lowering is panic-free, and docs truthfully defer metrics policy.

M5 structured runtime diagnostics merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2428
- Merge commit: `134963a2b27359a624346dcf357e33519e18156e`
- Merged at: `2026-06-08T20:24:48Z`
- Scope: Sifr-owned runtime diagnostic events, tracing-backed lowering, generated-project and grouped-e2e dependency metadata, lane coverage, and M5 traceability updates.
- Merge-ledger validation: docs-only ledger update; `git diff --check` -> PASS.

M5 structured runtime diagnostics merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-runtime-diagnostics-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR #2428 URL, merge commit/date, implementation summary, validation evidence, review-loop citations, docs-only scope, and no overclaim of metrics emission or M5 closure.

M5 explicit task context propagation implementation:

- Added `sifr.task.current_context()` backed by the `_sifr.task.task_current_context` intrinsic and a generated Tokio task-local label helper.
- Changed `sifr.task.Context` from a marker value to a Sifr-owned value with `name: str`, default `"Context"`, and `__str__` returning the label.
- Extended `task.TaskGroup(ctx=Context(...))` lowering, HIR, and codegen so group-spawned children inherit the explicit context label.
- Extended `task.spawn_scoped(..., ctx=Context(...))` lowering and runtime helpers so the child gets an explicit override while the active group context is restored for later spawns.
- Kept `ctx=None` valid and changed invalid non-`Context` values to a stable `SIFR-TYPE-0002` diagnostic.
- Added `task_context_propagation_basic` pass coverage, updated `task_context_propagation_rejected`, and listed the pass fixture in create-pr and merge E2E manifests.
- Updated the M5 shutdown traceability artifact and supported-host matrix to mark explicit task context propagation supported while continuing to reject Python `contextvars` dynamic mutation semantics.

M5 explicit task context propagation targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_lowering task_runtime_m1 -- --nocapture` -> PASS; `11 passed`.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_context_propagation_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_context_value_model_basic.sifr` -> PASS.
- `! cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_context_propagation_rejected.sifr` -> PASS by expected `SIFR-TYPE-0002`.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2246 files`, `900` line limit, and touched `task_runtime.rs` remained at `892` lines.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`483.76s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`123 passed`, `0 failed`, `cache_hits=0/36`, `report_signature=4a74179bcdf2ba0c`).

M5 explicit task context propagation review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-context-propagation-review-pass-1.md`: `PASS`; reviewer verified HIR/walker coverage for `TaskGroup { context }`, duplicate task-local/helper emission gates, honest structural context checking for the current type-system shape, runtime override restoration, tests, manifests, traceability, and the absence of Python `contextvars` overclaim.

M5 explicit task context propagation merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2431
- Merge commit: `262c052c9c5c2215f9df20d10ee3f85ff5e79fa3`
- Merged at: `2026-06-08T21:28:58Z`.
- Scope: explicit `sifr.task.Context` propagation for `TaskGroup(ctx=...)` and `task.spawn_scoped(..., ctx=...)`, `current_context()` intrinsic/runtime support, fixtures, lane manifests, review artifact, and M5 traceability updates.
- Merge-ledger validation: docs-only ledger update; `git diff --check` -> PASS.

M5 runtime diagnostic metrics policy implementation:

- Added `metrics = "0.24.6"` as a stable `StdlibFeature::Metrics` generated dependency for `sifr.runtime`, `_sifr.runtime`, and explicit required-crate inference.
- Extended `_sifr.runtime.runtime_emit_diagnostic(...)` dependency metadata so generated diagnostic code requires both `metrics` and `tracing`.
- Emitted fixed-schema metrics counters beside the existing `tracing::event!` calls: `sifr.runtime.diagnostic.emitted` for accepted diagnostic levels and `sifr.runtime.diagnostic.rejected` before returning `DiagnosticError` for unsupported levels.
- Kept metric labels low-cardinality and redacted: accepted emissions use only `surface="runtime"` plus fixed `level` values, rejected emissions use only `surface="runtime"` plus `reason="unsupported_level"`, and no diagnostic target, diagnostic name, diagnostic message, or rejected level text is used as a label.
- Updated grouped e2e and fixture generated Cargo.toml dependency inference so generated Rust containing `metrics::` receives the locked metrics facade dependency.
- Updated M5 shutdown traceability, supported-host matrix, and phase dependency notes with concrete metric names, label/cardinality policy, redaction policy, emission points, deterministic tests, and duration-histogram deferral.

M5 runtime diagnostic metrics policy targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_codegen runtime_diagnostic -- --nocapture` -> PASS.
- `cargo test -p sifr_codegen runtime_module_dependency_metadata_includes_observability_facades -- --nocapture` -> PASS.
- `cargo test -p sifr --test e2e test_generate_cargo_toml_runtime_diagnostics_use_locked_observability_specs -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/runtime_diagnostics_tracing.sifr` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2246 files` and the `900` line limit.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`467.90s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`123 passed`, `0 failed`, `cache_hits=0/36`, `report_signature=4a74179bcdf2ba0c`).

M5 runtime diagnostic metrics policy review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-metrics-policy-review-pass-1.md`: `PASS`; reviewer verified the stable metrics feature/dependency wiring, `runtime_emit_diagnostic` tracing+metrics requirements, accepted/rejected counter emission points, low-cardinality/redacted labels, grouped fixture dependency inference, deterministic tests, and honest histogram deferral.

M5 runtime diagnostic metrics policy merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2433
- Merge commit: `a13950d34a70313100f35a2a5f5240d713a5c3d9`
- Merged at: `2026-06-08T21:56:31Z`
- Scope: fixed-schema runtime diagnostic metrics counters, generated metrics dependency metadata, fixture dependency inference, review artifact, M5 traceability, supported-host matrix, and phase dependency policy updates.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M5 closeout classification implementation:

- Marked `milestone_concurrency_runtime_5` complete in this ledger after the diagnostics metrics policy wave merged.
- Strengthened `lowers_signal_intrinsics_via_registry` so the codegen unit test explicitly pins both `#[cfg(unix)]` and `#[cfg(not(unix))]` branches for `terminate()` and `shutdown_stream().next()`.
- Closed M5 shutdown traceability with non-Unix signal delivery classified as future host-limited evidence instead of a local M5 blocker; future support must run on a non-Unix host and deliver a real host console-control event before the host matrix can mark it supported.
- Updated the supported-host matrix so deterministic cleanup scopes are closed for accepted `nullcontext(...)` coverage and unsupported cleanup-stack/owned-closing diagnostics, with no accepted cleanup-stack surface remaining pending in M5.

M5 closeout classification targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_codegen lowers_signal_intrinsics_via_registry -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_stream_delivery_unix.sifr` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2246 files` and the `900` line limit.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`745.69s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`123 passed`, `0 failed`, `cache_hits=0/36`, `report_signature=4a74179bcdf2ba0c`).

M5 closeout classification review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-closeout-review-pass-2.md`: `PASS`; reviewer verified that M5 completion is valid after merged work through PR #2433/#2434, non-Unix signal delivery is not overclaimed, signal codegen tests pin both Unix and non-Unix branches for `terminate()` and `shutdown_stream().next()`, cleanup scope wording is honest, M5 artifacts are consistent, and M6/M7 remain pending. The reviewer noted overlapping cleanup wording in open PR #2430 as merge-sequence context, not a blocker for this closeout.

M5 closeout classification merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2435
- Merge commit: `a87cb2f279530f7d245f1601252e32f782b70297`
- Merged at: `2026-06-08T22:36:25Z`
- Scope: M5 completion classification, signal codegen cfg-branch evidence, non-Unix host-limited signal delivery boundary, cleanup-scope support classification, closeout validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M5 cancellation cleanup traceability addendum implementation:

- Credited the existing `cancellation_cleanup_runs` pass fixture as M5 cleanup-ordering evidence: timeout cancellation runs Sifr `finally` cleanup before the timeout error is observed.
- Added `cancellation_cleanup_runs` to the merge e2e manifest so cancellation cleanup evidence remains covered in both create-pr and merge lanes after M5 closeout.
- Updated closed M5 shutdown traceability and the supported-host matrix to record deterministic language cleanup scope evidence without reopening M5 or changing the unsupported diagnostics for `ExitStack`, `AsyncExitStack`, `closing`, and `aclosing`.

M5 cancellation cleanup traceability addendum targeted local validation:

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cancellation_cleanup_runs.sifr` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2253 files` and the `900` line limit.
- Post-`origin/main` rebase rerun after the M6 IPC stream helpers merge: `scripts/run_e2e_pass.sh --profile merge` -> PASS; merge e2e pass suite covered `136` fixtures with `136 passed`, `0 failed`, `cache_hits=41/41`, `report_signature=dc77a4a9bb841f30`.
- Post-`origin/main` rebase rerun after the M6 IPC stream helpers merge: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`143.52s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=530c89bb7012eeb0`). Slowest step was `platform_golden` at `39543ms`.

M5 cancellation cleanup traceability addendum review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-review-pass-1.md`: `PASS`; reviewer verified `cancellation_cleanup_runs` honestly proves timeout-cancellation `finally` cleanup ordering before timeout observation, merge-manifest inclusion, host-matrix and traceability wording, unsupported cleanup helper diagnostics, and targeted validation metrics. Non-blocking follow-up for the full create-pr gate was completed above.
- `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-review-pass-2.md`: `PASS`; reviewer verified the post-closeout addendum preserves closed M5 status, records `cancellation_cleanup_runs` as merge-lane traceability evidence, keeps unsupported cleanup helper diagnostics bounded, and cites the final post-closeout validation metrics accurately.
- `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-review-pass-3.md`: `PASS`; reviewer verified the latest M6-base addendum keeps M5 closed, preserves the M6 sections already on main, records `cancellation_cleanup_runs` honestly in create-pr and merge evidence, keeps unsupported cleanup helpers as unsupported diagnostics, and matches the final post-M6 validation metrics.
- `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-review-pass-4.md`: `FAIL`; reviewer found the branch had fallen behind the latest M6 frame-codec merge ledger and that the committed validation metrics were stale versus the refreshed post-rebase runs. Addressed by rebasing onto the M6 IPC stream helpers merge and committing refreshed validation metrics before the next review pass.
- `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-review-pass-5.md`: `FAIL`; reviewer found the branch had fallen behind the M6 IPC stream-helper merge ledger after the previous rebase. Addressed by rebasing onto the stream-helper merge ledger commit before the next review pass.
- `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-review-pass-6.md`: `PASS`; reviewer verified the branch is rebased onto the latest M6 stream-helper merge ledger, M5 remains closed, M6 docs and ledgers from main are preserved, `cancellation_cleanup_runs` is honestly recorded, unsupported cleanup helpers remain diagnostic-only, final validation metrics match the committed ledger, and pass-4/pass-5 failures are documented as addressed.

M5 cancellation cleanup traceability addendum merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2430
- Merge commit: `41e376fc27963e4e3bfd0550487e213a9647f293`
- Merged at: `2026-06-09T00:35:01Z`
- Scope: cancellation cleanup traceability addendum, merge-lane fixture coverage, closed M5 traceability/host-matrix wording, and reviewer artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- Merge-ledger review: `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-ledger-review-pass-1.md` -> `PASS`; reviewer verified the PR link, merge commit, merged timestamp, scope, docs-only validation claim, and unchanged M5/M6 status.
- Merge-ledger review: `reviews/ad-hoc-production-concurrency-runtime-m5-cancellation-cleanup-traceability-ledger-review-pass-2.md` -> `PASS`; reviewer verified the current ledger diff, expanded validation line, review artifact link, and unchanged M5/M6 status.

M6 typed IPC design gate implementation:

- Added `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` as the named M6 design approval artifact required before serialization dependency wiring or process-worker implementation.
- Defined typed IPC as a Sifr-owned production substrate layered above M4 process pipes, with `sifr.process` retaining process lifecycle, termination, timeout, and supervision ownership.
- Recorded the initial wire format, schema identity/hash policy, compatible-version negotiation, bootstrap/work/control/health/protocol-error frame families, bounded in-flight backpressure, cancellation/close behavior, malformed-frame handling, payload eligibility, CPython-shaped API classifications, observability redaction policy, and implementation wave order.
- Updated the supported-host matrix to reference the design artifact while keeping runtime implementation and host evidence blocked on M6 follow-up work.
- This PR intentionally does not add Serde/Postcard dependency wiring or public process-worker APIs; dependency wiring is allowed only after this design artifact is reviewed and recorded.

M6 typed IPC design gate targeted local validation:

- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2246 files` and the `900` line limit.
- `wc -l verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` -> `232` lines.

M6 typed IPC design gate review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-design-review-pass-1.md`: `PASS`; reviewer verified the named design artifact is sufficient for the M6 entry gate, covers typed payload eligibility, serialization format, schema/version negotiation, process-pipe layering, bootstrap, result/error frames, cancellation, backpressure, close, malformed-frame behavior, CPython-shaped API classification, and makes no implementation-support, dependency-wiring, or process-worker pool overclaim.

M6 typed IPC design gate merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2437
- Merge commit: `624248d058f166562148749243f5140358cde4e1`
- Merged at: `2026-06-08T22:46:03Z`
- Scope: named M6 typed IPC design artifact, process-pipe layering, wire format, schema/version negotiation, frame families, payload eligibility, backpressure, cancellation/close, malformed-frame behavior, CPython-shaped API classification, host-matrix boundary, validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC dependency metadata implementation:

- Added the internal `StdlibFeature::Ipc` feature that emits the locked Ring 4 typed-IPC dependencies: `postcard = { version = "1.1.3", default-features = false, features = ["use-std"] }` plus `serde = { version = "1.0.228", features = ["derive"] }`.
- Mapped `sifr.ipc`, `_sifr.ipc`, and explicit `ipc` / `postcard` codegen requirements to the IPC feature without adding public process-worker APIs.
- Updated grouped e2e generated Cargo.toml dependency inference so generated Rust containing `postcard::` receives the locked IPC dependency pair, and so `sifr.ipc` module metadata uses Postcard/Serde without pulling `serde_json`.
- Kept `crates/sifr/tests/e2e_support/fixture_compilation.rs` at the 900-line file-size cap after the change.

M6 typed IPC dependency metadata targeted local validation:

- `cargo test -p sifr_stdlib ipc_feature_renders_locked_postcard_specs_without_json -- --nocapture` -> PASS.
- `cargo test -p sifr --test e2e test_generate_cargo_toml_ipc_uses_locked_postcard_specs -- --nocapture` -> PASS.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2246 files` and the `900` line limit.
- Touched file line counts after formatting: `crates/sifr_stdlib/src/features.rs` `894`, `crates/sifr/tests/e2e_support/fixture_compilation.rs` `900`, `crates/sifr/tests/e2e_support/harness_model.rs` `790`, and `crates/sifr/tests/e2e_support/harness_behavior_tests.rs` `872`.

M6 typed IPC dependency metadata review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-dependency-metadata-review-pass-1.md`: `PASS`; reviewer verified the change is limited to Ring 4 typed IPC dependency metadata and grouped e2e Cargo.toml inference, locked Postcard/Serde specs match the approved design, IPC does not pull `serde_json`, runtime diagnostics wiring is unchanged, file-size guardrails are respected, and the ledger does not overclaim M6 completion.

M6 typed IPC dependency metadata merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2439
- Merge commit: `5d00e813691b7cae62f2ef7fc280b3bb0c6ebd2d`
- Merged at: `2026-06-08T22:59:11Z`
- Scope: internal typed IPC stdlib feature metadata, locked Postcard/Serde dependency specs, `sifr.ipc` / `_sifr.ipc` / `ipc` / `postcard` requirement mapping, grouped e2e generated Cargo.toml inference, validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC value model implementation:

- Added the public `sifr.ipc` value-model module with `SchemaId`, `ProtocolVersion`, `FrameKind`, frame-family constants, `BackpressurePolicy`, and helpers for schema ids, protocol version bounds, default backpressure, and exact schema matching.
- Added `ipc_value_model_basic` pass coverage and create-pr/merge manifest entries for the host-independent value model.
- Added `ipc_process_pool_executor_unsupported` and `ipc_multiprocessing_process_unsupported` fail fixtures to pin missing-member diagnostics for CPython-shaped process-pool and multiprocessing names under the native IPC module.
- Updated M6 typed IPC traceability and the supported-host matrix to mark only the schema/frame/backpressure value model as supported; frame encoding, process-pipe transport, runtime backpressure, payload eligibility enforcement, cancellation, close, and malformed-frame behavior remain M6 follow-up work.

M6 typed IPC value model targeted local validation:

- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/ipc_value_model_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/ipc_process_pool_executor_unsupported.sifr` -> produced expected `SIFR-NAME-0004` at column `22`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/ipc_multiprocessing_process_unsupported.sifr` -> produced expected `SIFR-NAME-0004` at column `22`.
- `cargo test -p sifr_stdlib stdlib_source_inventory_contains_user_modules -- --nocapture` -> PASS.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; the fail harness reported `453` fail tests completed and still prints existing unrelated internal-compiler-error lines for expected-fail cases.

M6 typed IPC value model review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-value-model-review-pass-1.md`: `PASS`; reviewer verified the new `sifr.ipc` module is limited to host-independent schema/frame/backpressure values and helpers, reuses prior dependency metadata, covers value behavior and CPython-shaped missing-member diagnostics, updates both e2e manifests, keeps frame transport blocked, and does not overclaim M6 completion.

M6 typed IPC value model merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2441
- Merge commit: `d696146d0ccad063e0e5c4213bec7b3e25f4709d`
- Merged at: `2026-06-08T23:12:23Z`
- Scope: `sifr.ipc` schema/frame/backpressure value model, stdlib source registration, pass/fail fixtures, create-pr/merge manifest entries, M6 traceability, supported-host matrix, validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC schema hash implementation:

- Added internal `sifr_stdlib::ipc_schema` descriptor types for generated IPC schemas, fields, variants, and eligible type shapes.
- Added deterministic canonical descriptor rendering for module path, schema name, compatible version range, request/response/error types, record fields, enum variants, and nested container types.
- Added dependency-free FNV-1a-128 schema hash v1 helpers (`schema_hash_v1`, `schema_hash_hex_v1`) for compatibility evidence, not cryptographic trust.
- Updated M6 typed IPC traceability to record schema descriptor/hash evidence while keeping compiler integration and generated schema extraction as follow-up work.

M6 typed IPC schema hash targeted local validation:

- `cargo test -p sifr_stdlib ipc_schema -- --nocapture` -> PASS.
- `cargo fmt` and `cargo fmt --check` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2250 files` and the `900` line limit.
- Touched file line counts after formatting: `crates/sifr_stdlib/src/ipc_schema.rs` `265`, `crates/sifr_stdlib/src/lib.rs` `430`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `243`, and this ledger `1964`.

M6 typed IPC schema hash review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-schema-hash-review-pass-1.md`: `PASS` with minor advisories; reviewer verified the internal-only schema descriptor/hash scope, deterministic canonical rendering, dependency-free FNV-1a-128 hash, traceability framing, and file-size guardrail, while asking to strengthen the sensitivity test and refresh stale line counts.
- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-schema-hash-review-pass-2.md`: `PASS`; reviewer verified the sensitivity test now mutates an actual request-record field type, ledger line counts match current files, scope remains internal-only, traceability still leaves compiler integration as follow-up work, and validation stayed green.

M6 typed IPC schema hash merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2443
- Merge commit: `f5a03ee0b646b2e04624cc0377066ad20f8913b4`
- Merged at: `2026-06-08T23:28:56Z`
- Scope: internal IPC schema descriptor types, canonical descriptor rendering, dependency-free FNV-1a-128 schema hash v1 helpers, schema hash tests, M6 traceability, validation evidence, and two review artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC frame codec implementation:

- Added `postcard = 1.1.3` to workspace dependencies with `default-features = false` and `use-std`, matching the M6 Ring 4 dependency decision; `sifr_stdlib` now depends on workspace `serde` and `postcard` for internal IPC frame helpers.
- Added internal `sifr_stdlib::ipc_frame` envelope, schema, frame-kind, shutdown, worker-status, protocol-error, and encode/decode helpers for the M6 `u32` little-endian length-prefixed Postcard wire format.
- Added typed `IpcFrameError` outcomes for encode/decode failures, truncated length prefixes, truncated payloads, oversize frames, unsupported lengths, and trailing bytes without rendering payload bytes in error text.
- Added unit coverage for bootstrap/work/control/health/protocol-error frame-family round trips plus malformed-frame cases. This wave does not claim process-pipe transport, connection-state handling, payload eligibility enforcement, cancellation, close, or runtime backpressure support.
- Updated M6 typed IPC traceability and the supported-host matrix to mark only host-independent frame codec helpers as supported; `Typed IPC frames over process pipes` remains blocked on follow-up M6 transport evidence.

M6 typed IPC frame codec targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_stdlib ipc_frame -- --nocapture` -> PASS; 9 frame codec tests covered bootstrap round trip, all frame-family round trips, negotiated max-frame enforcement, truncated length prefixes, truncated payloads, oversize frames, Postcard decode errors, trailing bytes, and redacted error text.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2251 files` and the `900` line limit.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`612.60s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=0/37`, `report_signature=530c89bb7012eeb0`).
- Touched file line counts after formatting: `crates/sifr_stdlib/src/ipc_frame.rs` `487`, `crates/sifr_stdlib/src/lib.rs` `436`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `244`, and this ledger `1999`.

M6 typed IPC frame codec review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-frame-codec-review-pass-1.md`: `PASS`; reviewer verified the Ring 4 Postcard/Serde dependency scope, internal-only frame helper surface, length-prefix encode/decode correctness, oversize-before-decode handling, typed malformed-frame errors, redacted error text, no runtime unwrap/expect/panic path for malformed input, frame-family test coverage, host-matrix and traceability honesty, and no overclaim beyond host-independent codec helpers.

M6 typed IPC frame codec merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2445
- Merge commit: `fb06126a0ad1239a54bfbc73125b7b04b77510a7`
- Merged at: `2026-06-08T23:57:31Z`
- Scope: Ring 4 workspace Postcard dependency, `sifr_stdlib` Serde/Postcard dependency wiring, internal IPC envelope and length-prefixed Postcard encode/decode helpers, malformed-frame tests, M6 traceability, supported-host matrix, validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC stream read/write implementation:

- Added internal `sifr_stdlib::ipc_transport` read/write helpers on top of the frame codec for `std::io::Read`/`Write` pipe-shaped byte streams.
- `read_frame(...)` treats clean EOF before a length prefix as `Ok(None)` close evidence, reports partial prefixes and payloads as typed frame errors, rejects oversize frames before reading payload bytes, decodes valid frames through the existing Postcard codec, and drops raw I/O error details.
- `write_frame(...)` encodes through the existing frame codec, writes the length-prefixed frame, flushes the stream, and maps writer failures to typed transport errors without rendering payload bytes.
- Added unit coverage for stream round trips, clean EOF, truncated prefixes, oversize prefixes, truncated payloads, encode-limit failures, writer failures, and bootstrap frames. This wave does not claim child-process fixture transport, connection-state handling, payload eligibility enforcement, cancellation, close protocol, or runtime backpressure support.
- Updated M6 typed IPC traceability and the supported-host matrix to mark only host-independent stream helpers as supported; `Typed IPC frames over process pipes` remains blocked on child-process fixture evidence.

M6 typed IPC stream read/write targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_stdlib ipc_transport -- --nocapture` -> PASS; 9 stream helper tests covered pipe-shaped stream round trip, clean EOF before prefix, truncated prefix, oversize prefix rejection before payload read, truncated payload, encode-limit failure, writer failure, bootstrap frame read/write, and length-prefix constant shape.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2252 files` and the `900` line limit.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`655.41s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=0/37`, `report_signature=530c89bb7012eeb0`).
- Touched file line counts after formatting: `crates/sifr_stdlib/src/ipc_transport.rs` `261`, `crates/sifr_stdlib/src/lib.rs` `438`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `245`, and this ledger `2029`.

M6 typed IPC stream read/write review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-transport-review-pass-1.md`: `PASS`; reviewer verified clean EOF versus partial-prefix behavior, oversize-prefix rejection before payload allocation, truncated-payload handling, opaque read/write I/O error mapping, write/flush error mapping, no unwrap/expect/panic path, stream helper scope discipline, and traceability/host-matrix honesty. Non-blocking follow-ups remain for read-error, flush-error, interrupted-read, zero-length-frame, multi-frame-stream, copy-avoidance, `Display`/`Error`, and unreachable length-sentinel hardening.

M6 typed IPC stream read/write merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2447
- Merge commit: `019fd05a55dd5c1631021086aad50f89842c39a0`
- Merged at: `2026-06-09T00:22:23Z`
- Scope: internal `sifr_stdlib::ipc_transport` read/write helpers over `std::io::Read`/`Write` pipe-shaped byte streams, typed clean-EOF/truncated-prefix/truncated-payload/oversize/read/write error handling, M6 traceability, supported-host matrix, validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC request tracker implementation:

- Added internal `sifr_stdlib::ipc_request_tracker` for request-id lifecycle tracking and bounded in-flight request windows on top of the frame model.
- The tracker reserves `Run` request IDs, rejects duplicates, rejects new runs after drain/close, enforces `max_in_flight`, validates `Started`, `Cancel`, `Completed`, and `Failed` request IDs, releases capacity on terminal frames, and clears state on `Terminating`.
- `Shutdown(Drain)` stops new runs while preserving in-flight work; `Shutdown(CancelInFlight)` enters draining state and clears in-flight work for cancellation evidence.
- Added unit coverage for duplicate IDs, unknown terminal/cancel IDs, full-window backpressure, capacity release, drain shutdown, cancel-in-flight shutdown, terminating close, shutdown-after-close terminal behavior, non-request frame pass-through, and redacted error text. This wave does not claim child-process fixture transport, full connection negotiation, payload eligibility enforcement, or generated worker integration.
- Updated M6 typed IPC traceability and the supported-host matrix to mark only host-independent request tracking/backpressure state as supported; `Typed IPC frames over process pipes` remains blocked on child-process fixture evidence.

M6 typed IPC request tracker targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_stdlib ipc_request_tracker -- --nocapture` -> PASS; 12 request tracker tests covered in-flight reservation, `Run` dispatch through `apply_frame`, duplicate request IDs, full-window backpressure, capacity release on completed/failed frames, unknown terminal/cancel IDs, started/cancel non-terminal behavior, drain shutdown, cancel-in-flight shutdown, terminating close, shutdown-after-close terminal behavior, non-request frame pass-through, and redacted error text.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2253 files` and the `900` line limit.
- Touched file line counts after formatting: `crates/sifr_stdlib/src/ipc_request_tracker.rs` `332`, `crates/sifr_stdlib/src/lib.rs` `440`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `246`, and this ledger `2060`.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`552.38s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=0/37`, `report_signature=530c89bb7012eeb0`).

M6 typed IPC request tracker review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-request-tracker-review-pass-1.md`: `PASS`; reviewer verified request-id lifecycle dispatch, duplicate-before-capacity evidence ordering, unknown request-id handling, started/cancel non-terminal behavior, drain versus cancel-in-flight shutdown, terminating close, redacted error text, re-export wiring, and traceability/host-matrix honesty. Non-blocking hardening requested closed-state terminal behavior and additional dispatch-boundary tests.
- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-request-tracker-review-pass-2.md`: `PASS`; reviewer verified `begin_shutdown(...)` now preserves `Closed`, `shutdown_after_terminating_keeps_tracker_closed`, `Run` dispatch through `apply_frame`, non-request frame pass-through coverage, reconciled validation evidence and line counts, and no new blockers.

M6 typed IPC request tracker merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2450
- Merge commit: `5bba51a72acf7ab264035c9a6a4e68dddcae31d0`
- Merged at: `2026-06-09T00:47:12Z`
- Scope: internal `sifr_stdlib::ipc_request_tracker` request-id lifecycle state machine, bounded in-flight backpressure, typed duplicate/unknown/full/draining/closed errors, M6 traceability, supported-host matrix, validation evidence, and two reviewer artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC connection-state implementation:

- Added internal `sifr_stdlib::ipc_connection` for fixture-oriented bootstrap and established-frame state management on top of the frame codec and request tracker.
- The helper builds parent `Hello` frames, accepts worker `Ready`/`Reject`, validates worker-side parent `Hello`, chooses the highest overlapping protocol version, enforces exact schema identity plus compatible-version range overlap, negotiates max-frame byte limits, and closes on bootstrap rejection/error evidence.
- Established-frame handling rejects bootstrap frames after readiness, routes `Run`/`Started`/`Cancel`/`Completed`/`Failed` frames through the request tracker, transitions to draining on `Shutdown`, closes on `Terminating` and protocol-error frames, and exposes code-only malformed-frame construction without rendering payload bytes.
- Updated M6 typed IPC traceability and the supported-host matrix to mark only host-independent connection-state and bootstrap negotiation as supported. This wave does not claim child-process fixture transport, payload eligibility diagnostics, or generated worker integration.

M6 typed IPC connection-state targeted local validation:

- `cargo fmt --check` -> PASS after formatting.
- `cargo test -p sifr_stdlib ipc_connection -- --nocapture` -> PASS; 14 connection-state tests covered protocol overlap selection, exact schema identity/range checks, parent hello emission, worker ready/reject decisions, parent ready acceptance, forged-ready schema rejection, pre-ready frame rejection, request-tracker integration, duplicate request propagation, shutdown drain transition, terminating close, and protocol-error frame redaction.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2254 files` and the `900` line limit.
- Touched file line counts after formatting: `crates/sifr_stdlib/src/ipc_connection.rs` `705`, `crates/sifr_stdlib/src/lib.rs` `445`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `248`, `verification/platform/supported_host_matrix.md` `46`, and this ledger `2123`.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`674.35s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=0/37`, `report_signature=530c89bb7012eeb0`).

M6 typed IPC connection-state review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-connection-state-review-pass-1.md`: `PASS`; reviewer verified bootstrap negotiation, parent/worker state gates, protocol overlap and schema identity checks, max-frame negotiation, established-frame phase handling, request-tracker routing, shutdown/terminating/protocol-error close behavior, redacted error text, honest traceability/host-matrix scope, and focused test/validation evidence. No blockers remain for this wave.

M6 typed IPC connection-state merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2452
- Merge commit: `9c4a1229342b3776554f148afb987b1e4e649ae7`
- Merged at: `2026-06-09T01:16:16Z`
- Scope: internal `sifr_stdlib::ipc_connection` parent/worker bootstrap negotiation, protocol/schema/max-frame negotiation, established-frame state gating, request-tracker integration, shutdown/terminating/protocol-error close behavior, M6 traceability, supported-host matrix, validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC payload eligibility implementation:

- Added internal `sifr_stdlib::ipc_payload::validate_ipc_payload_type(...)` for host-independent `IpcSerializable` schema-shape validation before generated worker integration.
- Extended `IpcSchemaType` with an explicit `Unsupported { type_name }` sentinel so generated schema extraction and tests can carry rejected process/task/resource-like payload evidence without pretending the type is encodable.
- The validator accepts the initial primitive, option, result, tuple, list, `dict[str, T]`, record, and enum schema families, recursively rejects unsupported nested payload shapes, and returns typed `UnsupportedPayload` evidence without rendering payload values.
- Updated M6 typed IPC traceability and the supported-host matrix to mark only host-independent payload eligibility validation as supported; compiler diagnostics, child-process fixture transport, generated schema extraction, and public connection/worker APIs remain M6 follow-up work.

M6 typed IPC payload eligibility targeted local validation:

- `cargo test -p sifr_stdlib ipc_payload -- --nocapture` -> PASS; 5 tests covered accepted initial `IpcSerializable` families, unsupported process resource payloads inside records, unsupported task payloads inside enum variants, recursive unsupported payload rejection through every container dispatch path, and redacted eligibility error text.
- `cargo test -p sifr_stdlib ipc_schema -- --nocapture` -> PASS; existing descriptor/hash tests stayed stable after adding the unsupported payload sentinel.
- `cargo fmt`, `cargo fmt --check`, `cargo clippy -p sifr_stdlib -- -D warnings`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2256 files` and the `900` line limit.
- Touched file line counts after formatting: `crates/sifr_stdlib/src/ipc_payload.rs` `203`, `crates/sifr_stdlib/src/ipc_schema.rs` `273`, `crates/sifr_stdlib/src/lib.rs` `447`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `252`, and `verification/platform/supported_host_matrix.md` `47`.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS on the rebased final-base tree; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`172.38s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=530c89bb7012eeb0`; slowest step `crate_tests` `52752ms`).

M6 typed IPC payload eligibility review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-eligibility-review-pass-1.md`: `PASS`; reviewer verified scope alignment, recursive payload validation, `Unsupported` sentinel honesty, redacted error text, panic-freedom, module wiring, documentation/host-matrix honesty, and local validation evidence. Non-blocking polish requested explicit `None` unit-type wording, direct recursive-container rejection coverage, and descriptor evidence wording.
- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-eligibility-review-pass-2.md`: `PASS`; reviewer verified the post-pass-1 polish addressed every non-blocking item without changing the validator contract or overclaiming compiler diagnostics, generated schema extraction, child-process transport, public APIs, or wire-compatible `unsupported(...)` payloads.
- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-eligibility-review-pass-3.md`: `PASS`; reviewer verified the final rebase preserved both connection-state and payload-eligibility traceability, kept deferred compiler diagnostics, generated schema extraction, child-process fixture transport, and public worker/connection APIs honest, and matched final-base validation metrics exactly.

M6 typed IPC payload eligibility merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2454
- Merge commit: `ff71edd1f81fa7cb49a9c407434390d261e7a7ef`
- Merged at: `2026-06-09T01:50:32Z`
- Scope: internal `sifr_stdlib::ipc_payload` host-independent payload eligibility validator, explicit `IpcSchemaType::Unsupported` rejected-type evidence sentinel, recursive unsupported payload coverage, M6 traceability, supported-host matrix, final-base validation evidence, and three reviewer artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC payload eligibility merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-eligibility-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, GitHub merged-at timestamp, status-list replacement, docs-only validation evidence, and no M6 completion or deferred-surface overclaim.

M6 typed IPC Unix process-pipe fixture implementation:

- Added an internal `sifr_stdlib` fixture worker binary gated behind the `__test_fixture` feature so normal workspace builds do not build a public or production worker executable.
- Added Unix integration coverage that spawns the fixture worker as a real child process and exchanges length-prefixed Postcard IPC frames over child stdin/stdout using the existing transport, connection-state, and request-tracker helpers.
- The fixture covers parent `Hello` / worker `Ready` bootstrap, `Run` / `Started` / `Completed` request completion, in-flight `Cancel` producing a typed `Failed` terminal frame, `Shutdown` / `Terminating` close, and truncated-frame reporting through a redacted `MalformedFrame` protocol error.
- Updated M6 typed IPC traceability and the supported-host matrix to mark Unix child-process pipe transport evidence as supported while keeping Windows fixtures, compiler diagnostics for payload eligibility/generated extraction, and generated worker integration as follow-up work.

M6 typed IPC Unix process-pipe fixture targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo build -p sifr_stdlib` -> PASS; verifies the fixture worker remains gated out of ordinary `sifr_stdlib` builds.
- `cargo test -p sifr_stdlib --test ipc_process_pipe_fixture -- --nocapture` -> PASS; 3 Unix child-process pipe tests covered request completion plus shutdown, in-flight cancellation plus shutdown, and malformed truncated-frame reporting over real child stdin/stdout pipes.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2257 files` and the `900` line limit on the final branch after merging `origin/main`.
- Touched file line counts after formatting on the final branch: `crates/sifr_stdlib/Cargo.toml` `27`, `crates/sifr_stdlib/tests/fixtures/ipc_pipe_fixture_worker.rs` `134`, `crates/sifr_stdlib/tests/ipc_process_pipe_fixture.rs` `248`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `254`, `verification/platform/supported_host_matrix.md` `47`, and this ledger `2195`.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS on the conflict-resolved final branch; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`139.46s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests including the Unix process-pipe integration fixture, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=530c89bb7012eeb0`; slowest step `crate_tests` `39271ms`).

M6 typed IPC Unix process-pipe fixture review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-process-pipe-fixture-review-pass-1.md`: `PASS`; reviewer verified the fixture worker is gated behind the internal `__test_fixture` feature, ordinary `cargo build -p sifr_stdlib` does not build a production worker binary, tests use real Unix child stdin/stdout pipes plus existing IPC helpers, coverage includes bootstrap, completion, cancellation, shutdown close, and malformed truncated-frame reporting, docs honestly scope support to Unix process-pipe evidence, and touched files remain below the guardrail.
- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-process-pipe-fixture-review-pass-2.md`: `PASS`; post-`origin/main` merge reviewer verified payload eligibility evidence was preserved, process-pipe evidence remains after it, design and host matrix honestly combine payload validation plus Unix pipe support, fixture gating and Unix coverage remain intact, final create-pr metrics match the conflict-resolved branch, and all touched files remain below the guardrail.

M6 typed IPC Unix process-pipe fixture merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2455
- Merge commit: `ed3fe513ece009e326d6b2a94aadc7ac1f8ce778`
- Merged at: `2026-06-09T02:10:57Z`
- Scope: internal test-gated Unix child-process pipe fixture worker, real stdin/stdout IPC frame transport, bootstrap/request completion/cancellation/shutdown/malformed evidence, M6 traceability, supported-host matrix, validation evidence, and two reviewer artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC process-pipe backpressure and unsupported-payload evidence implementation:

- Extended the internal `__test_fixture` worker to run with `max_in_flight: 1` and report request-tracker backpressure over the real child stdin/stdout pipe as a redacted `MalformedFrame(RequestId, "backpressure_full")` protocol error.
- Extended the same Unix process-pipe fixture to emit typed `UnsupportedPayload { type_name }` evidence for an unsupported-payload sentinel without echoing payload bytes.
- Added Unix integration coverage for the backpressure and unsupported-payload cases while preserving the existing bootstrap, request completion, cancellation, shutdown close, and malformed-frame tests.
- Updated M6 typed IPC traceability and the supported-host matrix to include Unix process-pipe evidence for bounded backpressure and unsupported payloads while keeping Windows fixtures, compiler diagnostics for payload eligibility/generated extraction, and generated worker integration as follow-up work.

M6 typed IPC process-pipe backpressure and unsupported-payload evidence targeted local validation:

- `cargo test -p sifr_stdlib --test ipc_process_pipe_fixture -- --nocapture` -> PASS; 5 Unix child-process pipe tests covered request completion plus shutdown, in-flight cancellation plus shutdown, backpressure-full protocol-error reporting, unsupported-payload evidence, and malformed truncated-frame reporting over real child stdin/stdout pipes.
- `cargo clippy -p sifr_stdlib --features __test_fixture --bin sifr-stdlib-ipc-pipe-fixture-worker -- -D warnings` -> PASS.
- `cargo fmt --check` -> PASS.
- `cargo build -p sifr_stdlib` -> PASS; verifies the fixture worker remains gated out of ordinary `sifr_stdlib` builds.
- `cargo clippy -p sifr_stdlib -- -D warnings` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail reported `2258 files` and the `900` line limit before final create-pr validation.
- Touched file line counts after final create-pr validation ledger update: `crates/sifr_stdlib/tests/fixtures/ipc_pipe_fixture_worker.rs` `183`, `crates/sifr_stdlib/tests/ipc_process_pipe_fixture.rs` `320`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `254`, `verification/platform/supported_host_matrix.md` `47`, and this ledger `2229`.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`152.33s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests including the Unix process-pipe integration fixture, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`124 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=530c89bb7012eeb0`; slowest step `crate_tests` `51346ms`).
- Post-ledger `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC process-pipe backpressure and unsupported-payload evidence review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-process-pipe-edge-evidence-review-pass-1.md`: `PASS`; reviewer verified the internal fixture-worker gate, real Unix child-process pipe backpressure evidence, redacted `MalformedFrame(RequestId, "backpressure_full")` reporting, unsupported-payload type-name-only evidence, parent/worker connection-state symmetry, honest Unix-only host-matrix/docs scope, validation evidence, and touched source/test file sizes. Non-blocking follow-ups remain for a future invalid UTF-8 unsupported sentinel fixture case and future symmetric parent-side pre-wire backpressure coverage.

M6 typed IPC process-pipe backpressure and unsupported-payload evidence merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2458
- Merge commit: `4af2e423cdee04499b93a3c6948d9bd78f330c2b`
- Merged at: `2026-06-09T03:02:21Z`
- Scope: internal Unix child-process pipe fixture edge evidence for bounded backpressure and unsupported payloads, fixture-worker redacted connection-error reporting, type-name-only unsupported-payload evidence, M6 traceability, supported-host matrix, validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC process-pipe backpressure and unsupported-payload evidence merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-process-pipe-edge-evidence-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged-at timestamp, one-line pending-status replacement, docs-only validation evidence, diff containment, and no M6 completion or deferred-surface overclaim.

M6 typed IPC payload diagnostics implementation:

- Added `sifr.ipc.require_serializable(...)` as a compiler-erased marker for representative concrete IPC payload eligibility diagnostics. The marker is not a public connection or worker API and does not add runtime serialization behavior.
- Added `SIFR-OWN-0013` for typed IPC payloads that try to cross process boundaries with process-local resources, synchronization endpoints/guards, task/runtime handles, callables, iterators, unknown types, or non-initial-schema container shapes.
- Added recursive lowering-side eligibility checks that accept the initial primitive, `None`, option, list, `dict[str, T]`, tuple, record/class, enum, and `Result[T, E]` payload families while rejecting concrete resource-like values before generated schema extraction exists.
- Added e2e fixtures for accepted marker usage plus rejected process-pipe and channel-endpoint payloads, and added the accepted fixture to both create-pr and merge e2e manifests.
- Updated M6 typed IPC traceability and the supported-host matrix to mark host-independent payload eligibility diagnostics as supported while keeping generated schema extraction, public connection/worker APIs, generated worker integration, and Windows process-pipe fixtures as follow-up work.

M6 typed IPC payload diagnostics targeted local validation:

- `cargo fmt --check` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo test -p sifr_diagnostics registry -- --nocapture` -> PASS; 3 registry tests verified the active diagnostic registry and generated docs page for `SIFR-OWN-0013`.
- `cargo test -p sifr_lowering ipc_payload_calls -- --nocapture` -> PASS; 2 tests covered accepted initial payload families plus rejected process-local/callable payloads.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/ipc_payload_require_serializable_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/ipc_payload_process_resource_rejected.sifr` -> expected `SIFR-OWN-0013` for `PipeReader`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/ipc_payload_sync_endpoint_rejected.sifr` -> expected `SIFR-OWN-0013` for `ChannelSender`.
- `cargo clippy -p sifr_lowering -p sifr_diagnostics -- -D warnings` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`481.24s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`125 passed`, `0 failed`, `cache_hits=0/37`, `report_signature=50edc954137c87b4`).
- Conflict-resolved final branch validation after rebasing on PR #2458/#2459: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`153.93s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`125 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=50edc954137c87b4`).
- Reviewer pass 1: `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-diagnostics-review-pass-1.md` -> PASS; non-blocking enum/erasure clarity follow-ups addressed with an enum unit case and compiler-erased marker comment.
- Post-review focused validation: `cargo fmt --check` and `cargo test -p sifr_lowering ipc_payload_calls -- --nocapture` -> PASS.
- Reviewer pass 2: `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-diagnostics-review-pass-2.md` -> PASS; verified the post-review enum/erasure clarifications and final docs scope.
- Touched file line counts after formatting: `crates/sifr_lowering/src/lower/ipc_payload_calls.rs` `236`, `crates/sifr_lowering/src/lower/expressions/regular_calls.rs` `468`, `crates/sifr_lowering/src/lower/ownership_diagnostics.rs` `256`, `crates/sifr_lowering/src/lower/nested_function_inference/capture_collection.rs` `223`, `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs` `751`, `crates/sifr_diagnostics/src/codes/registry.rs` `739`, `crates/sifr_diagnostics/src/codes/registry/registry_entries/calls_flow_and_protocols.rs` `571`, `docs/errors/SIFR-OWN-0013.md` `16`, `docs/errors/diagnostic-codes.md` `251`, `internal_docs/diagnostic_codes.md` `253`, `lib/sifr/ipc.sifr` `85`, `crates/sifr/tests/e2e/pass/ipc_payload_require_serializable_basic.sifr` `26`, `crates/sifr/tests/e2e/fail/ipc_payload_process_resource_rejected.sifr` `8`, `crates/sifr/tests/e2e/fail/ipc_payload_sync_endpoint_rejected.sifr` `9`, `verification/validation_lanes/create_pr_e2e_manifest.json` `131`, `verification/validation_lanes/merge_e2e_manifest.json` `143`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `255`, `verification/platform/supported_host_matrix.md` `48`, `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-diagnostics-review-pass-1.md` `16`, `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-diagnostics-review-pass-2.md` `23`, and this ledger `2279`.

M6 typed IPC payload diagnostics merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2460
- Merge commit: `c319b3a0600cc355eda9cb559cdfb7559d53f533`
- Merged at: `2026-06-09T03:14:10Z`
- Scope: compiler-erased `sifr.ipc.require_serializable(...)` marker, `SIFR-OWN-0013` registry/docs, recursive lowering-side IPC payload eligibility diagnostics, representative pass/fail e2e fixtures, validation manifest entries, M6 traceability, supported-host matrix, execution-ledger evidence, and two reviewer artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC payload diagnostics merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-diagnostics-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged-at timestamp, pending M6 status, docs-only validation evidence, line-count tally, and diff containment.

M6 typed IPC CPython-shaped multiprocessing diagnostics implementation:

- Added focused `sifr.ipc` missing-member fixtures for the remaining CPython-shaped process-pool and multiprocessing names called out by the M6 design: `Queue`, `Pipe`, `Pool`, `fork`, `forkserver`, and `shared_memory`.
- Updated the M6 typed IPC design evidence row to list the full focused fixture family alongside the existing `ProcessPoolExecutor` and `Process` fixtures, while preserving the boundary that these are diagnostics only and not public IPC worker APIs.

M6 typed IPC CPython-shaped multiprocessing diagnostics targeted local validation:

- Direct `cargo run -q -p sifr -- check ...` checks for `ipc_multiprocessing_queue_unsupported`, `ipc_multiprocessing_pipe_unsupported`, `ipc_multiprocessing_pool_unsupported`, `ipc_multiprocessing_fork_unsupported`, `ipc_multiprocessing_forkserver_unsupported`, and `ipc_multiprocessing_shared_memory_unsupported` -> expected `SIFR-NAME-0004` for each missing `sifr.ipc` member.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail harness reported `461 fail tests completed`.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`530.61s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`125 passed`, `0 failed`, `cache_hits=0/37`, `report_signature=50edc954137c87b4`).
- Reviewer pass 1: `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-cpython-shape-diagnostics-review-pass-1.md` -> PASS; reviewer verified fixture column markers, the full focused fixture family in the M6 design row, missing-member diagnostic scope, pending M6 status, validation evidence, and diff containment.
- Touched file line counts: `crates/sifr/tests/e2e/fail/ipc_multiprocessing_queue_unsupported.sifr` `6`, `crates/sifr/tests/e2e/fail/ipc_multiprocessing_pipe_unsupported.sifr` `6`, `crates/sifr/tests/e2e/fail/ipc_multiprocessing_pool_unsupported.sifr` `6`, `crates/sifr/tests/e2e/fail/ipc_multiprocessing_fork_unsupported.sifr` `6`, `crates/sifr/tests/e2e/fail/ipc_multiprocessing_forkserver_unsupported.sifr` `6`, `crates/sifr/tests/e2e/fail/ipc_multiprocessing_shared_memory_unsupported.sifr` `6`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `255`, `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-cpython-shape-diagnostics-review-pass-1.md` `8`, and this ledger `2306`.

M6 typed IPC CPython-shaped multiprocessing diagnostics merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2462
- Merge commit: `e9f49b0e82d7f7e00facc6b3fe72c15567685112`
- Merged at: `2026-06-09T03:35:23Z`
- Scope: focused `sifr.ipc` missing-member fixtures for CPython-shaped `Queue`, `Pipe`, `Pool`, `fork`, `forkserver`, and `shared_memory`, M6 typed IPC design evidence update, execution-ledger validation evidence, and reviewer artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC CPython-shaped multiprocessing diagnostics merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-cpython-shape-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged timestamp, M6 pending status, docs-only diff scope, validation claims, and scope summary for the merge-ledger packet.

M6 typed IPC compiler-internal schema extraction implementation:

- Added a lowering-owned `ipc_schema_extraction` helper that maps accepted concrete Sifr payload type graphs into `sifr_stdlib::IpcSchemaType` descriptors for the initial primitive, `None`, option, list, `dict[str, T]`, tuple, record/class, enum, and `Result[T, E]` families.
- Kept rejected concrete payload evidence explicit by mapping process-local or otherwise unsupported type graphs to `IpcSchemaType::Unsupported { type_name }`, without treating `unsupported(...)` as a wire-compatible payload.
- Wired the compiler-erased `sifr.ipc.require_serializable(...)` marker path to compute the schema type after payload eligibility succeeds, while preserving the marker's erased runtime behavior and without adding public worker/connection APIs.
- Updated M6 typed IPC traceability and the supported-host matrix to mark compiler-internal schema extraction as supported while keeping generated worker integration, public connection/worker APIs, runtime peer schema exchange, and Windows process-pipe fixtures as follow-up work.

M6 typed IPC compiler-internal schema extraction targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_lowering ipc_schema_extraction -- --nocapture` -> PASS; 2 tests covered accepted schema-family extraction and unsupported payload evidence.
- `cargo test -p sifr_lowering ipc_payload_calls -- --nocapture` -> PASS; 2 tests verified the existing payload marker eligibility behavior after schema extraction wiring.
- `cargo clippy -p sifr_lowering -- -D warnings` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS; file-size guardrail covered `2269` files after rebasing on the M6 CPython-shaped diagnostics slice.
- Rebased final branch validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`173.46s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`125 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=50edc954137c87b4`). Slowest step was `crate_tests` at `63804ms`.

M6 typed IPC compiler-internal schema extraction review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-schema-extraction-review-pass-1.md`: `PASS`; reviewer found no blocking issues, verified the compiler-internal scope, and confirmed the docs honestly exclude public worker/connection APIs, generated worker integration, runtime peer schema exchange, and Windows process-pipe fixture support.
- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-schema-extraction-review-pass-2.md`: `PASS`; reviewer verified the final rebased diff preserves the M6 CPython-shaped diagnostics merge ledger, introduces no scope drift, and keeps schema-extraction claims honest.

M6 typed IPC compiler-internal schema extraction merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2464
- Merge commit: `08e0a3b821a165e072eabd5dbe8330c21f3b056b`
- Merged at: `2026-06-09T03:53:24Z`
- Scope: lowering-owned IPC schema type extraction into `IpcSchemaType`, erased `sifr.ipc.require_serializable(...)` marker-path schema computation after eligibility succeeds, M6 traceability and supported-host matrix updates, rebased create-pr validation evidence, and two reviewer artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC compiler-internal schema extraction merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-schema-extraction-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged timestamp, docs-only scope, validation claim, pending M6 status, and no overclaim of generated worker integration or Windows process-pipe fixture support.

M6 typed IPC generated worker-boundary compose proof implementation:

- Extended the internal Unix IPC fixture worker to accept a test-provided schema name and schema hash through environment variables while preserving the existing hard-coded default schema for all current process-pipe fixture tests.
- Added a lowering-owned Unix compose test that builds a representative compiler-internal IPC schema from concrete Sifr `Type` graphs, computes the stable schema hash through `sifr_stdlib::schema_hash_v1`, passes that identity to the fixture worker, completes `Hello`/`Ready` bootstrap over child stdin/stdout, round-trips `Run`/`Completed`, and closes with `Shutdown`/`Terminating`.
- Updated M6 typed IPC traceability and the supported-host matrix so compiler-extracted schema identity is proven over the Unix fixture worker without exposing a public worker pool or public `ipc.Connection` API. Windows process-pipe fixture evidence remains host-limited follow-up work, and public generated worker integration remains `deferred-to-phase-X`.

M6 typed IPC generated worker-boundary compose proof targeted local validation:

- `cargo test -p sifr_lowering generated_schema_drives_unix_fixture_worker_bootstrap_and_round_trip -- --nocapture` -> PASS.
- `cargo test -p sifr_lowering ipc_schema_extraction -- --nocapture` -> PASS; 3 tests covered accepted schema-family extraction, unsupported payload evidence, and generated-schema worker-boundary composition.
- `cargo test -p sifr_stdlib --test ipc_process_pipe_fixture -- --nocapture` -> PASS; 5 existing Unix process-pipe fixture tests stayed stable with the default fixture schema.
- `cargo clippy -p sifr_lowering -p sifr_stdlib -- -D warnings` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS after rebase onto current `origin/main`; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`122.43s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`125 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=50edc954137c87b4`; slowest step `crate_tests` `32511ms`).
- Scope review: `reviews/ad-hoc-production-concurrency-runtime-m6-remaining-scope-review-pass-1.md` -> FAIL-on-closeout without this compose proof; reviewer confirmed generated worker-boundary composition is a true M6 blocker, but not a public process-worker API requirement.
- Reviewer pass 1: `reviews/ad-hoc-production-concurrency-runtime-m6-generated-worker-boundary-review-pass-1.md` -> PASS; reviewer verified the real compose proof, existing fixture compatibility, no public API overclaim, fixture-only panic surface, and docs/status accuracy.
- Touched file line counts after formatting: `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs` `359`, `crates/sifr_stdlib/tests/fixtures/ipc_pipe_fixture_worker.rs` `201`, `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md` `257`, `verification/platform/supported_host_matrix.md` `50`, `reviews/ad-hoc-production-concurrency-runtime-m6-remaining-scope-review-pass-1.md` `46`, `reviews/ad-hoc-production-concurrency-runtime-m6-generated-worker-boundary-review-pass-1.md` `13`, and this ledger `2404`.

M6 typed IPC generated worker-boundary compose proof merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2470
- Merge commit: `912b50d250e97f4a3fac3d7526469149b1719f5e`
- Merged at: `2026-06-09T04:32:18Z`
- Scope: lowering-owned compose proof that compiler-extracted IPC schema identity drives Unix fixture-worker `Hello`/`Ready`, `Run`/`Completed`, and `Shutdown`/`Terminating`; fixture-worker test schema environment override with default compatibility preserved; M6 design/host-matrix evidence updates; rebased create-pr validation evidence; and two reviewer artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC generated worker-boundary compose proof merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-generated-worker-boundary-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged timestamp, docs-only scope, validation claim, M6 complete/M7 in-progress status preservation, and no overclaim of public generated worker/API support.

M6 closeout classification implementation:

- Marked `milestone_concurrency_runtime_6` complete after the typed IPC substrate waves and schema-extraction ledger merged.
- Closed stale M6 design wording by classifying public worker-pool APIs and generated worker integration as `deferred-to-phase-X` over the M6 substrate rather than remaining M6 implementation work.
- Preserved Windows process-pipe fixture evidence as host-limited future work; the M6 DoD is satisfied by host-independent typed IPC helpers plus Unix real-process pipe fixture evidence.
- Updated the supported-host matrix so generated worker integration is future deferred work and not an M6-owned follow-up blocker.

M6 closeout classification targeted local validation:

- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`135.84s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`125 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=50edc954137c87b4`; slowest step `crate_tests` `49190ms`).

M6 closeout classification review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-closeout-readiness-review-pass-1.md`: `FAIL`; reviewer found the substantive M6 DoD met but identified docs-only blockers in stale design/host wording that still claimed generated worker integration was M6 implementation work. This closeout slice addresses those blockers.
- `reviews/ad-hoc-production-concurrency-runtime-m6-closeout-review-pass-1.md`: `PASS`; reviewer verified the stale M6 design and host-matrix blockers are resolved, generated worker/public worker APIs are consistently `deferred-to-phase-X`, Windows process-pipe fixture evidence remains host-limited, validation evidence is recorded, M6 can be considered closed, and M7 remains pending.

M6 typed IPC closeout classification merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2467
- Merge commit: `1606e5d0817af1cb6c0f05b56bf4e5636dfd7775`
- Merged at: `2026-06-09T04:11:16Z`
- Scope: docs-only M6 closeout classification, stale generated-worker wording cleanup, host-matrix classification update, roadmap/phase issue status update, validation evidence, and agent closeout review artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M6 typed IPC closeout classification merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m6-closeout-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged timestamp with expected one-second GitHub/commit timestamp skew, docs-only scope, validation claim, and M6 complete/M7 pending status.

M7 traceability scaffold implementation:

- Created `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` as the required M7 milestone traceability artifact.
- Recorded M7 closeout gates for public docs, internal architecture docs, demos, generated Cargo dependency snapshots, generated-code panic/emitted-quality coverage, validation lane manifests, inventory closure, and final external review.
- Preserved M7 as in-progress rather than complete; this scaffold records the remaining closeout PR slices and does not satisfy the final phase gate.

M7 traceability scaffold targeted local validation:

- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`222.51s`, warm target `<=2m`) after package-cache lock waits. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`125 passed`, `0 failed`, `cache_hits=37/37`, `report_signature=50edc954137c87b4`; slowest step `crate_tests` `72699ms`).

M7 traceability scaffold review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-traceability-review-pass-1.md`: `PASS`; reviewer verified the required M7 traceability artifact exists, M7 remains in progress rather than complete, open gates are tracked with correct state semantics, validation evidence is recorded, and the scaffold does not overclaim phase completion.

M7 traceability scaffold merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2469
- Merge commit: `9b72f3f151cf5e241f3050e9debbadb633a7461d`
- Merged at: `2026-06-09T04:25:38Z`
- Scope: required M7 traceability artifact creation, M7 in-progress ledger status, open closeout gate tracking, M0-M6 closure input summary, and agent scaffold review artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M7 traceability scaffold merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-traceability-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged timestamp, docs-only scope, validation claim, and M7 in-progress status without phase-completion overclaim.

M7 public documentation implementation:

- Added `docs/concurrency_runtime.md` as the public M7 docs entry for `sifr.task`, `sifr.sync`, `sifr.runtime`, `sifr.parallel`, `sifr.process`, `sifr.signal`, `sifr.resource`, and `sifr.ipc`.
- Documented accepted API families, ownership/sendability/shareability boundaries, typed error surfaces, process/signal host limits, IPC schema/frame substrate boundaries, and intentional divergences from CPython event-loop, queue/threading, subprocess, signal-handler, cleanup-stack, multiprocessing, and process-pool APIs.
- Updated `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` so every public-doc gate points at the new public docs artifact as `pending-pr` while preserving M7 as open/in-progress until the remaining internal architecture, demos, generated dependency/panic-scan, validation inventory, and final review gates close.

M7 public documentation targeted local validation:

- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- Touched file line counts: `docs/concurrency_runtime.md` `240`, `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` `65`, and this ledger `2444`.

M7 public documentation review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-public-docs-review-pass-1.md`: `PASS`; reviewer verified public docs coverage for all eight modules, intentional CPython divergence and public API boundaries, M7 in-progress status, non-public-doc gates left open, validation claims, and touched-file line counts.

M7 public documentation merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2473
- Merge commit: `9a17a5fd76a701761b91604bd45ac7e58ecdf7bc`
- Merged at: `2026-06-09T04:41:12Z`
- Scope: public `docs/concurrency_runtime.md` coverage for `sifr.task`, `sifr.sync`, `sifr.runtime`, `sifr.parallel`, `sifr.process`, `sifr.signal`, `sifr.resource`, and `sifr.ipc`; M7 traceability public-doc rows marked complete after merge; validation evidence; and agent public-docs review artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M7 public documentation merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-public-docs-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged timestamp, docs-only scope, validation claim, public-doc gates closed, remaining M7 gates still open/partial/pending, and no M7 completion overclaim.

M7 internal architecture audit implementation:

- Added the M7 Production Closure Audit table to `internal_docs/structured_runtime_work_model.md`, locking terminal contracts for task ownership, process ownership, channels/synchronization, blocking and CPU offload, sendability/shareability, task/request context, diagnostics and signal global state, typed IPC policy, and the rejected CPython-shaped surface index.
- Added an `internal_docs/architecture.md` concurrency-safety pointer to the M7 audit so the main architecture contract references the terminal production runtime closeout surface.
- Updated `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` so the internal architecture docs gate and required PR-slice row point at the audit as `pending-pr` while leaving demos, generated dependency/panic-scan, validation inventory, and final review gates open.

M7 internal architecture audit targeted local validation:

- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- Touched file line counts: `internal_docs/structured_runtime_work_model.md` `266`, `internal_docs/architecture.md` `1361`, `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` `65`, and this ledger `2472`.

M7 internal architecture audit review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-architecture-audit-review-pass-1.md`: `PASS`; reviewer verified coverage of task/process/channel/offload/runtime boundaries, typed IPC policy, blocking/offload policy, sendability/shareability, task/request context, diagnostics/signal global-state policy, rejected CPython-shaped surface index, M7 open/in-progress status, remaining non-architecture gates left open, validation claims, and touched-file line counts.

M7 internal architecture audit merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2476
- Merge commit: `d21d4da4e4e05c227fc0165ac719bde94ba3c0ec`
- Merged at: `2026-06-09T04:48:39Z`
- Scope: M7 production closure audit table in `internal_docs/structured_runtime_work_model.md`, main architecture pointer in `internal_docs/architecture.md`, M7 traceability architecture gate marked complete after merge, validation evidence, and agent architecture-audit review artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.

M7 internal architecture audit merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-architecture-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR URL, merge commit, merged timestamp, docs-only scope, validation claim, architecture gate closed, remaining gates still open/partial/pending, and no M7 completion overclaim.

M7 demo closure implementation:

- Added `demos/parallel_map_demo/main.sifr`, `demos/async_subprocess_pipeline_demo/main.sifr`, `demos/structured_shutdown_demo/main.sifr`, and `demos/cancellation_cleanup_demo/main.sifr` for the remaining required M7 demo categories.
- Reused existing `demos/structured_concurrency_demo/main.sifr`, `demos/sync_channel_demo/main.sifr`, and `demos/blocking_offload_demo/main.sifr` for structured task group, producer/consumer channel pipeline, and blocking offload coverage.
- Updated `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` so the required demos gate and demo-closure slice point at all seven demo commands as `pending-pr` while leaving generated dependency/panic-scan, validation inventory, and final review gates open.

M7 demo closure targeted local validation:

- `cargo run -q -p sifr -- run demos/structured_concurrency_demo/main.sifr` -> PASS.
- `cargo run -q -p sifr -- run demos/sync_channel_demo/main.sifr` -> PASS.
- `cargo run -q -p sifr -- run demos/blocking_offload_demo/main.sifr` -> PASS.
- `cargo run -q -p sifr -- run demos/parallel_map_demo/main.sifr` -> PASS.
- `cargo run -q -p sifr -- run demos/async_subprocess_pipeline_demo/main.sifr` -> PASS.
- `cargo run -q -p sifr -- run demos/structured_shutdown_demo/main.sifr` -> PASS.
- `cargo run -q -p sifr -- run demos/cancellation_cleanup_demo/main.sifr` -> PASS.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- Touched file line counts: `demos/parallel_map_demo/main.sifr` `36`, `demos/async_subprocess_pipeline_demo/main.sifr` `33`, `demos/structured_shutdown_demo/main.sifr` `49`, `demos/cancellation_cleanup_demo/main.sifr` `37`, `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` `65`, and this ledger `2507`.

M7 demo closure review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-demo-closure-review-pass-1.md`: `PASS`; reviewer verified all seven required demo categories have concrete evidence, new demos are valid and scoped, M7 remains open/in progress, non-demo gates remain unclosed, and validation claims and line counts are plausible.

M7 demo closure merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2479
- Merge commit: `040dfa81138b2e4a8ccf97a7e825dd894c93eead`
- Merged at: `2026-06-09T05:00:20Z`
- Scope: four new concurrency runtime demos for CPU parallel map, async subprocess pipeline, structured shutdown, and cleanup under cancellation; validation of existing structured task group, producer/consumer channel pipeline, and blocking offload demos; M7 traceability demo gate marked complete after merge; validation evidence; and agent demo-closure review artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` -> PASS; `python3 scripts/check_file_size_guardrails.py` -> PASS.

M7 demo closure merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-demo-ledger-review-pass-1.md`: `PASS`; reviewer verified the ledger does not overclaim M7 or phase completion, only closes the demo gate, keeps non-demo gates open or partial as appropriate, and records the PR URL, merge commit, timestamp, and scope clearly enough for phase closeout traceability.
- `reviews/ad-hoc-production-concurrency-runtime-m7-demo-ledger-review-pass-2.md`: `PASS`; reviewer verified the cleaned-up ledger records final PASS validation evidence, references the populated pass-1 review, matches the merge commit timestamp exactly, and needs no further review rounds before commit.

M7 generated dependency and panic-scan evidence implementation:

- Added `verification/stdlib/concurrency_runtime_dependency_snapshots.json` as the resolver-backed dependency snapshot artifact for accepted concurrency/runtime feature combinations: Tokio task/sync/process/signal/offload, Rayon parallel map, runtime diagnostics metrics/tracing, IPC Postcard/Serde serialization, and `sifr_runtime` path emission where generated code requires runtime helpers.
- Added `crates/sifr_stdlib/tests/concurrency_runtime_dependency_snapshots.rs` to parse the snapshot artifact and compare every row against `sifr_stdlib::generated_cargo_dependencies(...)`, including normalized `sifr_runtime` path placeholders and sorted unique snapshot ids.
- Extended `verification/generated_code_quality/manifest.json` with a dedicated `concurrency-runtime-m7` group covering the seven required M7 demos, and updated `verification/generated_code_quality/generated_code_quality.py` so manifest loading fails if any M7 demo entry is missing.
- Updated M7 closeout traceability to mark generated dependency snapshots and generated-code panic/emitted-code quality coverage as `pending-pr`, leaving validation lanes, inventory closure, final review, M7, and the phase open.

M7 generated dependency and panic-scan evidence validation:

- `cargo check -p sifr_codegen -p sifr_stdlib` -> PASS.
- `cargo test -p sifr_stdlib concurrency_runtime_dependency_snapshots -- --nocapture` -> PASS (`1` integration test passed).
- `cargo run -q -p sifr -- run demos/parallel_map_demo/main.sifr` -> PASS.
- `SIFR_GCQ_SHARED_ROOT=target/sifr_m7_gcq python3 verification/generated_code_quality/generated_code_quality.py corpus --group concurrency-runtime-m7` -> PASS; evidence `target/sifr_generated_code_quality/evidence/corpus-1780982450-70347.json`.
- `SIFR_GCQ_SHARED_ROOT=target/sifr_m7_gcq python3 verification/generated_code_quality/generated_code_quality.py panic-scan --group concurrency-runtime-m7` -> PASS; evidence `target/sifr_generated_code_quality/evidence/panic-scan-1780982529-75206.json`.
- `SIFR_GCQ_SHARED_ROOT=target/sifr_m7_gcq python3 verification/generated_code_quality/generated_code_quality.py rustfmt --group concurrency-runtime-m7` -> PASS; evidence `target/sifr_generated_code_quality/evidence/rustfmt-1780982529-75244.json`.
- `SIFR_GCQ_SHARED_ROOT=target/sifr_m7_gcq python3 verification/generated_code_quality/generated_code_quality.py clippy --group concurrency-runtime-m7` -> PASS; evidence `target/sifr_generated_code_quality/evidence/clippy-1780982531-75768.json`.
- `SIFR_GCQ_SHARED_ROOT=target/sifr_m7_gcq python3 verification/generated_code_quality/generated_code_quality.py determinism --group concurrency-runtime-m7` -> PASS; evidence `target/sifr_generated_code_quality/evidence/determinism-1780982533-69991.json`.
- `python3 -m json.tool verification/stdlib/concurrency_runtime_dependency_snapshots.json` and `python3 -m json.tool verification/generated_code_quality/manifest.json` -> PASS.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- Touched file line counts: `crates/sifr_codegen/src/preamble/parallel_runtime.rs` `234`, `crates/sifr_stdlib/tests/concurrency_runtime_dependency_snapshots.rs` `82`, `verification/generated_code_quality/generated_code_quality.py` `797`, `verification/generated_code_quality/manifest.json` `100`, `verification/stdlib/concurrency_runtime_dependency_snapshots.json` `90`, `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` `65`, and this ledger `2546`.

M7 generated dependency and panic-scan evidence review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-generated-evidence-review-pass-1.md`: `PASS`; reviewer verified the dependency snapshot schema and resolver equivalence, the M7 generated-code quality manifest group and harness enforcement, the generated parallel `try_map` bound fix, scoped `pending-pr` traceability, local validation evidence shape, and no overclaim of M7 or phase completion.

M7 generated dependency and panic-scan evidence merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2482
- Merge commit: `727f234511427e4dafa1644b39af4712a9a8c30b`
- Merged at: `2026-06-09T05:37:13Z`
- Scope: resolver-backed concurrency runtime dependency snapshots with an integration test, dedicated M7 generated-code quality manifest coverage for all seven required demos, generated parallel `try_map` bound cleanup required by the new clippy lane, M7 traceability for generated dependency and panic/emitted-code quality coverage, validation evidence, and agent review artifact.
- Merge-ledger validation: docs-only ledger update; `git diff --check` -> PASS; `python3 scripts/check_file_size_guardrails.py` -> PASS.

M7 generated dependency and panic-scan evidence merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-generated-ledger-review-pass-1.md`: `PASS`; reviewer verified the ledger does not overclaim M7 or phase completion, only closes the generated dependency snapshot and panic/emitted-code quality coverage gates, keeps validation lane manifests partial and inventory closure plus final external review open, records the PR URL, merge commit, timestamp, scope, and docs-only `git diff --check` plus `python3 scripts/check_file_size_guardrails.py` PASS evidence accurately for phase closeout traceability.
- `reviews/ad-hoc-production-concurrency-runtime-m7-generated-ledger-review-pass-2.md`: `PASS`; reviewer verified the final ledger references the populated pass-1 artifact, closes only the generated-evidence gates, keeps remaining M7 gates open or partial, and needs no further review rounds before commit.

M7 validation lane and inventory closure implementation:

- Added `verification/stdlib/concurrency_runtime_m7_inventory_closure.md` to audit create-pr and merge validation lane coverage across task, sync, offload, parallel, process, signal/resource/runtime, and IPC families.
- Added direct `spawn_blocking_basic` to `verification/validation_lanes/merge_e2e_manifest.json` so both create-pr and merge lanes carry direct blocking-offload evidence in addition to existing `join_set_spawn_blocking` coverage.
- Updated `scripts/generate_concurrency_runtime_inventory.py` so regenerated inventory, CPython evidence, and workload artifacts carry M7 inventory-audited status text instead of stale M0/M3 active labels.
- Regenerated `verification/stdlib/concurrency_runtime_substrate_inventory.md`, `concurrency_runtime_substrate_inventory.json`, `concurrency_runtime_cpython_evidence_matrix.md`, and `concurrency_runtime_workload_database.md`.
- Updated M7 closeout traceability to mark validation lane manifests and inventory closure as `pending-pr`, leaving final external review and final merge-gate validation pending and M7/phase open.

M7 validation lane and inventory closure validation:

- `python3 scripts/generate_concurrency_runtime_inventory.py` -> PASS (`generated 135 CPython evidence entries`).
- `python3 -m json.tool verification/stdlib/concurrency_runtime_substrate_inventory.json`, `verification/validation_lanes/create_pr_e2e_manifest.json`, `verification/validation_lanes/merge_e2e_manifest.json`, `verification/platform/golden/manifest.json`, and `verification/platform/platform_contract.json` -> PASS.
- Validation-lane and inventory audit assertions -> PASS: create-pr lane has `125` fixtures, merge lane has `138` fixtures, both lanes cover task, sync, offload, parallel, process, signal/resource/runtime, and IPC fixture families; inventory status is `milestone_concurrency_runtime_7-inventory-audited`; every legacy Python-shaped surface has a revisit rule; M2 semaphore permit policy is preserved as guard-like and await/return-forbidden; M3 `spawn_cpu`, scoped offload, `JoinSet`, and `parallel.map/try_map` workload rows remain present.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_blocking_basic.sifr` -> PASS.
- `git diff --check` and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- Touched hand-maintained file line counts: `scripts/generate_concurrency_runtime_inventory.py` `749`, `verification/stdlib/concurrency_runtime_substrate_inventory.md` `76`, `verification/stdlib/concurrency_runtime_workload_database.md` `27`, `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md` `147`, `verification/stdlib/concurrency_runtime_m7_inventory_closure.md` `62`, `verification/stdlib/concurrency_runtime_m7_closeout_traceability.md` `65`, `verification/validation_lanes/merge_e2e_manifest.json` `144`, and this ledger `2582`; generated `verification/stdlib/concurrency_runtime_substrate_inventory.json` is excluded from the 900-line hand-maintained guardrail.

M7 validation lane and inventory closure review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-inventory-closure-review-pass-1.md`: `FINDINGS`; reviewer found that regenerating inventory artifacts reverted closed M2 semaphore-permit policy, M3 workload evidence rows, and M5 production-surface notes.
- `reviews/ad-hoc-production-concurrency-runtime-m7-inventory-closure-review-pass-2.md`: `PASS`; reviewer verified the generator and regenerated artifacts now preserve the M2 semaphore policy, M3 `spawn_cpu`/scoped-offload/`JoinSet`/parallel workload evidence, and M5 signal/resource/context notes; validation lanes, inventory audit, traceability, and ledger remain scoped with final review still open.

M7 validation lane and inventory closure merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2485
- Merge commit: `525f5695075ac42c2b71ac90d754ac750284ee56`
- Merged at: `2026-06-09T06:12:51Z`
- Scope: M7 inventory closure audit for validation lanes, inventory, platform golden, supported-host rows, and waiver/quarantine state; direct `spawn_blocking_basic` merge-lane coverage; generator and regenerated inventory artifacts updated to M7 inventory-audited status while preserving M2/M3/M5 closed evidence; M7 traceability for validation lane and inventory closure; validation evidence; and agent review artifacts.
- Merge-ledger validation: docs-only ledger update; `git diff --check` -> PASS; `python3 scripts/check_file_size_guardrails.py` -> PASS.

M7 validation lane and inventory closure merge-ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-inventory-ledger-review-pass-1.md`: `PASS`; reviewer verified PR #2485, merge commit `525f5695075ac42c2b71ac90d754ac750284ee56`, merge timestamp `2026-06-09T06:12:51Z`, validation PASS evidence, closed validation-lane/inventory traceability rows, and preservation of the final open/pending M7 gates with no phase overclaim.
- `reviews/ad-hoc-production-concurrency-runtime-m7-inventory-ledger-review-pass-2.md`: `FINDINGS`; reviewer verified the ledger semantics but flagged the live pass-2 output file as an unrelated empty untracked artifact while the review command was still writing it, requiring a follow-up review after recording the artifact trail.
- `reviews/ad-hoc-production-concurrency-runtime-m7-inventory-ledger-review-pass-3.md`: `PASS`; reviewer verified the final ledger state, pass-1/pass-2 artifact trail, closed validation-lane/inventory traceability rows, and preservation of all final open/pending M7 gates with no phase overclaim.

M7 final review and validation gate implementation:

- Reworked generated process-async preamble plumbing so codegen passes the existing `SharedPreludeProcessAsyncNeeds` struct through `build_process_async_items(...)` instead of ten boolean parameters, and grouped child-table booleans into `ProcessAsyncChildTableNeeds`.
- Removed redundant named `format!` arguments in runtime diagnostic intrinsic lowering so workspace clippy stays clean under `-D warnings`.
- Fixed `verification/performance/run_benchmarks.py` command benchmarks to build `target/debug/sifr` once and invoke that binary directly instead of measuring Cargo front-end overhead in every sample.
- Reused the same build output directory for build-mode command benchmark samples so performance budgets measure representative warm rebuild behavior instead of forcing a fresh project build for each measured sample.
- Updated M7 closeout traceability to mark the final review and merge gate as `pending-pr`, leaving final completion and roadmap closure for the post-merge ledger PR.

M7 final review and validation gate validation:

- Initial `scripts/run_all_tests.sh` on this branch failed only in `performance_budget_checks`; a pristine `origin/main` probe showed the same command-benchmark failure shape, with `check-single-file-001-arithmetic` median `1951.153ms` versus threshold `1334.139ms` and p95 `2487.280ms` versus threshold `1419.542ms`, proving the failing budget was pre-existing benchmark harness overhead rather than this branch's codegen cleanup.
- After the benchmark harness fix, representative performance probes passed.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS (`2273` files under the 900-line hand-maintained source limit).
- `cargo clippy -p sifr_codegen -p sifr_stdlib -- -D warnings` -> PASS after the process-async and runtime diagnostic codegen cleanup.
- `cargo clippy --workspace -- -D warnings` -> PASS.
- `cargo test -p sifr_stdlib` -> PASS; stdlib unit tests, concurrency runtime dependency snapshot tests, and IPC process-pipe fixtures passed.
- `cargo test -p sifr -- stdlib` -> PASS.
- `scripts/run_e2e_pass.sh` -> PASS; merge profile reported `138` passed, `0` failed, `report_signature=4ede7c71d86f381c`.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; create-pr profile reported `125` e2e pass fixtures, `0` failed, platform golden `pass=6`, `skip=1`, and `report_signature=50edc954137c87b4`; advisory: warm wall-time exceeded the create-pr target.
- `scripts/run_all_tests.sh` -> PASS; merge profile reported `wall_time=853.82s`, `budget_ok=yes`, `138` e2e pass fixtures, `0` failed, `report_signature=4ede7c71d86f381c`, platform golden `pass=6`, `skip=1`, hardening `variants=34`, `failures=0`, and advisory-only group skew.

M7 final review and validation gate review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-final-closeout-review-pass-1.md`: `PASS`; reviewer verified the process-async preamble struct cleanup, runtime diagnostic `format!` cleanup, benchmark harness correction, M7 traceability status discipline, validation evidence, file-size guardrail, and no phase-completion overclaim. Final closeout implementation is ready to PR/merge, with M7 and roadmap completion left for the post-merge ledger PR.

M7 final review and validation gate merge ledger:

- PR: https://github.com/sifr-lang/sifr/pull/2488
- Merge commit: `9a271d64b1e62b36a5365f0831cb990d83f8d4e9`
- Merged at: `2026-06-09T07:29:51Z`
- Scope: final generated process-async preamble and runtime diagnostic clippy cleanup, performance benchmark harness correction for direct `sifr` binary measurement and warm build-mode samples, full create-pr and merge validation evidence, final agent implementation review `PASS`, and M7 closeout traceability final-gate status.
- Merge-ledger validation: docs-only final ledger update; `git diff --check` -> PASS; `python3 scripts/check_file_size_guardrails.py` -> PASS (`2273` files under the 900-line hand-maintained source limit).

M7 final phase ledger review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m7-final-ledger-review-pass-1.md`: `PASS`; reviewer verified PR #2488's merge commit and timestamp, final implementation agent `PASS` evidence, docs-only validation scope, M7 and roadmap completion status flips, closed final external review and merge-gate traceability rows, and no stale status contradiction. Final ledger is ready to PR/merge, and phase 36.4 is complete and audited once this ledger PR merges.

Post-closure agent host-matrix remediation:

- `reviews/ad-hoc-production-concurrency-runtime-agent-final-review-pass-1.md`: `FAIL`; agent reviewer verified the closure chain but found stale active supported-host matrix rows for `Blocking I/O offload` and `CPU parallelism` still marked `blocked-on-concurrency-runtime-m3` after M3 and M7 closure.
- Remediation: flipped those M3-owned rows to `supported` using existing `spawn_blocking_basic`, `join_set_spawn_blocking`, `spawn_cpu_basic`, `join_set_spawn_cpu_join_all_ordered`, `parallel_map_basic`, `parallel_try_map_basic`, and `parallel_pool_map_basic` evidence; refreshed `concurrency_runtime_m7_inventory_closure.md` stale pending wording.
- `reviews/ad-hoc-production-concurrency-runtime-agent-final-review-pass-2.md`: `PASS`; agent reviewer verified the pass-1 blocker was fully remediated, the inventory audit no longer contradicts closed M7 status, and the closure record is internally consistent after this docs-only correction.
- Docs-only validation: `git diff --check` -> PASS; `python3 scripts/check_file_size_guardrails.py` -> PASS (`2274` files under the 900-line hand-maintained source limit).

Post-closure cancellation-model provider record:

- PR #2493: https://github.com/sifr-lang/sifr/pull/2493
- Scope: clarified the completed provider cancellation model as abort-backed task-handle cancellation with typed observation, compiler-recognized `async with task.timeout(duration)` same-task timeout scopes, no public `cancel_scope` / `CancelScope` / cancellation-token surface, and conditional-only `tokio-util` dependency records.
- `reviews/ad-hoc-production-concurrency-runtime-cancellation-agent-review-pass-2.md`: `FAIL`; agent reviewer verified the stale `cancel_scope` public-API records were fixed but found one remaining unconditional `tokio-util` dependency summary.
- `reviews/ad-hoc-production-concurrency-runtime-cancellation-agent-review-pass-3.md`: `PASS`; agent reviewer verified the remaining blocker and wording notes were fixed, all changed files were staged, and no blocking findings remained.
- Local validation: `git diff --cached --check` -> PASS; `python3 -m json.tool verification/stdlib/concurrency_runtime_substrate_inventory.json` -> PASS; `python3 scripts/check_file_size_guardrails.py` -> PASS (`2274` files under the 900-line hand-maintained source limit); `scripts/run_all_tests.sh --profile create-pr` -> PASS with wall-time advisory only (`125` e2e pass fixtures, `0` failed, `report_signature=50edc954137c87b4`).

M5 signal `strsignal` value-helper implementation:

- Added `sifr.signal.strsignal(signal)` as a pure Sifr value helper that returns the signal name without consulting process-global host signal state or claiming stream delivery.
- Added `signal_strsignal_basic` pass coverage and create-pr/merge manifest entries.
- Updated the M5 shutdown traceability artifact and supported-host matrix to mark `strsignal(signal)` as host-independent value-model support while keeping `ctrl_c`, `terminate`, `shutdown_stream`, importable constants, Unix-only constants, and signal delivery semantics in progress.

M5 signal `strsignal` value-helper targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo test -p sifr_stdlib stdlib_source_inventory_contains_user_modules -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_strsignal_basic.sifr` -> PASS.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`177.28s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`117 passed`, `0 failed`, `cache_hits=29/32`, `report_signature=ded105ad58090608`).

M5 signal `strsignal` value-helper review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-strsignal-review-pass-1.md`: `PASS`; reviewer verified the panic-free value-helper implementation, public fixture coverage, symmetric manifest entries, traceability and host-matrix scope boundaries, validation metrics, and in-progress M5 status discipline. Non-blocking follow-ups remain for repeated-use ownership coverage and future stream/constants work.

M5 signal `strsignal` value-helper merge ledger:

- Merged as PR #2412 (`d5b618199f38762a90ab988f5c20aee296d5120b`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`152.76s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`117 passed`, `0 failed`, `cache_hits=30/32`, `report_signature=ded105ad58090608`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-strsignal-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR #2412 merge commit/date, final validation metrics and advisory wording, lane-step coverage, in-progress M5 status convention, docs-only scope, and no overclaim beyond the `strsignal` value-helper slice.

M5 task context value-model foundation implementation:

- Added the embedded `sifr.task` value-model module with `Context`, `ContextKey[T]`, and `empty_context()` without changing compiler-recognized `task.TaskGroup`, `task.scope`, or `task.spawn_scoped` lowering.
- `ContextKey[T]` carries a typed default marker so the key's value type is represented in generated code without adding dynamic Python `contextvars` behavior.
- Added `task_context_value_model_basic` pass coverage and create-pr/merge manifest entries.
- Added `task_context_propagation_rejected` fail coverage to pin non-`None` `ctx` rejection until explicit task propagation semantics are implemented.
- Updated the M5 shutdown traceability artifact and supported-host matrix to mark task context values as host-independent support while keeping propagation and request handoff in progress.

M5 task context value-model foundation targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json`, `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json`, and `python3 -m json.tool verification/stdlib/concurrency_runtime_substrate_inventory.json` -> PASS.
- `cargo test -p sifr_stdlib stdlib_source_inventory_contains_user_modules -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_context_value_model_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_context_propagation_rejected.sifr` -> expected `SIFR-TYPE-0002`.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail suite reported `447 fail tests completed`.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`205.84s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`118 passed`, `0 failed`, `cache_hits=24/33`, `report_signature=8826f5b3144352b0`).

M5 task context value-model foundation review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-task-context-foundation-review-pass-1.md`: `FAIL`; reviewer could not verify the untracked new `sifr.task` source or pass/fail fixture contents from the initial patch packet.
- `reviews/ad-hoc-production-concurrency-runtime-m5-task-context-foundation-review-pass-2.md`: `PASS`; reviewer verified the `sifr.task` value model, typed `ContextKey[T]` default marker, pass/fail fixture boundary, create-pr/merge manifest entries, in-progress propagation/request-handoff status, no `contextvars` overclaim, and recorded local validation evidence.

M5 task context value-model foundation merge ledger:

- Merged as PR #2414 (`521ffced5834ca56802237105e8e57de947f8fcd`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`294.95s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`118 passed`, `0 failed`, `cache_hits=21/33`, `report_signature=8826f5b3144352b0`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-task-context-foundation-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR #2414 merge commit/date, validation metrics and advisories, docs-only scope, and no overclaim that non-`None` task context propagation is complete.

M5 signal constants implementation:

- Added portable module-level `sifr.signal.SIGINT` and `sifr.signal.SIGTERM` values backed by annotated object-valued stdlib module constants.
- Extended annotated module-constant lowering to record constructor-call constants and extended module-constant codegen to emit non-primitive constants through private factory functions using the existing expression lowering path.
- Added `signal_constants_basic` pass coverage and create-pr/merge manifest entries.
- Updated M5 shutdown traceability, supported-host matrix, and substrate inventory to mark portable constants supported while keeping structured signal streams, Unix-only constants, and delivery semantics in progress or host-limited.

M5 signal constants targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json`, `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json`, and `python3 -m json.tool verification/stdlib/concurrency_runtime_substrate_inventory.json` -> PASS.
- `cargo check -p sifr_lowering -p sifr_codegen -p sifr_driver` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/signal_constants_basic.sifr` -> PASS.
- `cargo fmt --check`, `git diff --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`216.65s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`119 passed`, `0 failed`, `cache_hits=31/33`, `report_signature=0df4819d3daf7aa4`).

M5 signal constants review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-constants-review-pass-1.md`: `PASS`; reviewer verified the constructor-only object constant lowering, private factory codegen, portable `SIGINT`/`SIGTERM` value-model scope, ownership-aware fixture, symmetric manifests, no stream/delivery overclaim, and recorded validation metrics.

M5 signal constants merge ledger:

- Merged as PR #2416 (`634a552f6e6bdf23a8a358544d45d571562dfbb7`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`143.66s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`119 passed`, `0 failed`, `cache_hits=32/33`, `report_signature=0df4819d3daf7aa4`).
- `reviews/ad-hoc-production-concurrency-runtime-m5-signal-constants-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR #2416 merge commit/date, merge-ledger validation metrics, docs-only scope, and no overclaim beyond the portable `SIGINT`/`SIGTERM` constants wave.

M3 first-wave targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo check -p sifr_lowering -p sifr_codegen -p sifr_stdlib` -> PASS.
- `cargo clippy --workspace -- -D warnings` -> PASS.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2126 files checked, 900-line limit.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_map_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_try_map_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_pool_map_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_map_worker_panic_typed.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_try_map_user_error_typed.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/parallel_map_async_direct_rejected.sifr` -> expected fail with `SIFR-ASYNC-0004`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/parallel_map_non_send_item_rejected.sifr` -> expected fail with `SIFR-TYPE-0002`.
- `SIFR_E2E_PROFILE=create-pr SIFR_E2E_MANIFEST=verification/validation_lanes/create_pr_e2e_manifest.json SIFR_E2E_DISABLE_CACHE=1 cargo test -p sifr --test e2e parallel_ -- --nocapture` -> PASS; verified the five new parallel pass fixtures as a grouped batch after adding the grouped-test Rayon dependency path.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`76 passed`, `0 failed`, `cache_hits=20/21`).

M3 first-wave review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m3-parallel-design-review-pass-1.md`: `FAIL`; reviewer blocked `std::process::abort()`, missing typed pool/worker runtime failure channels, and unwrapped Rayon worker panic behavior. The implementation was remediated by changing `parallel.map`/`Pool.map` to `Result[..., WorkerRuntimeError]`, changing `parallel.try_map`/`Pool.try_map` to `Result[..., WorkerError]`, deleting the abort path, and wrapping Rayon worker calls in typed catch boundaries.
- `reviews/ad-hoc-production-concurrency-runtime-m3-parallel-design-review-pass-2.md`: `PASS`; reviewer verified the pass-1 blockers are closed and accepted the first-wave boundary, with non-blocking follow-ups retained for later M3 work.
- `reviews/ad-hoc-production-concurrency-runtime-m3-parallel-review-pass-2.md`: `PASS`; reviewer verified typed pool-construction errors, typed worker panic conversion, result-shaped `sifr.parallel`/`Pool` APIs, validation fixtures, manifest entries, and traceability. Non-blocking follow-ups were recorded for global panic-hook suppression, future `WorkerError[E]`, and lazy private default pool shutdown design.
- `reviews/ad-hoc-production-concurrency-runtime-m3-parallel-review-pass-3.md`: `PASS`; reviewer verified the post-pass-2 grouped e2e Rayon dependency fix, confirmed it does not hide a runtime feature-gating issue, re-verified typed pool/worker failure handling, sendability/type diagnostics, validation manifests, traceability, and PR readiness.

M3 `task.spawn_cpu` wave targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo check -p sifr_lowering -p sifr_codegen -p sifr_stdlib -p sifr` -> PASS.
- `cargo clippy --workspace -- -D warnings` -> PASS.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2134 files checked, 900-line limit.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_cpu_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_cpu_user_error_typed.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_cpu_worker_panic_typed.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_map_basic.sifr` -> PASS after shared worker-error preamble extraction.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_try_map_user_error_typed.sifr` -> PASS after shared worker-error preamble extraction.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/spawn_cpu_unannotated_rejected.sifr` -> expected fail with `SIFR-ASYNC-0005`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/spawn_cpu_blocking_io_rejected.sifr` -> expected fail with `SIFR-ASYNC-0005`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/spawn_cpu_non_send_rejected.sifr` -> expected fail with `SIFR-TYPE-0002`.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail suite reported 404 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`79 passed`, `0 failed`, `cache_hits=22/22`).

M3 `task.spawn_cpu` wave review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m3-spawn-cpu-review-pass-2.md`: `PASS`; reviewer verified async-only `@cpu_heavy` validation, rejection of blocking-I/O and unannotated workers, `BlockingTask[T, WorkerRuntimeError]` / `BlockingTask[T, WorkerError]` result shapes, typed worker panic and Rayon pool construction evidence, Rayon/Tokio feature gating, sendability checks, no `sifr.parallel` regression from shared worker-error extraction, and honest remaining-M3 traceability. Non-blocking follow-ups were retained for global panic-hook cleanup, OS thread creation failure handling if the bridge changes, and minor emitted-error redundancy.

M3 `JoinSet` wave targeted local validation:

- `cargo check -p sifr_lowering -p sifr_codegen -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/join_set_spawn_cpu_join_all_ordered.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/join_set_add_task_join_all.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/join_set_cancel_all_evidence.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/join_set_cancel_all_task_cancelled.sifr` -> PASS; proves `cancel_all()` aborts an added task and reports `Cancelled` evidence instead of merely counting outcomes.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/join_set_spawn_blocking.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/join_set_bound_terminal_await.sifr` -> PASS; verifies a `pending = joins.join_all(); results = await pending` terminal await consumes the live set.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/join_set_unconsumed_rejected.sifr` -> expected fail with `SIFR-OWN-0001`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/join_set_terminal_must_be_awaited_rejected.sifr` -> expected fail with `SIFR-OWN-0001`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/join_set_reassign_live_rejected.sifr` -> expected fail with `SIFR-OWN-0001`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/join_set_spawn_cpu_worker_error_required.sifr` -> expected fail with `SIFR-TYPE-0002`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/join_set_add_type_mismatch_rejected.sifr` -> expected fail with `SIFR-TYPE-0002`.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/join_set_spawn_blocking.sifr | rg -n "rayon|__sifr_spawn_cpu|ThreadPoolBuilder|__sifr_with_silent_join_set_panic_hook"` -> no matches; non-CPU JoinSet usage emits no Rayon references.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/join_set_spawn_cpu_join_all_ordered.sifr | rg -n "rayon|__sifr_spawn_cpu|ThreadPoolBuilder"` -> PASS; CPU JoinSet usage emits the expected CPU bridge and Rayon references.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail suite reported 409 fail tests completed.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2147 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; platform golden reported pass=5, skip=2; create-pr e2e pass suite reported 85 passed, 0 failed, cache_hits=20/23; advisory: warm wall-time budget exceeded.

M3 `JoinSet` wave review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m3-joinset-review-pass-1.md`: `CHANGES_REQUESTED`; reviewer flagged union sort key uniqueness, live JoinSet rebinding, added-task cancellation, generic type-var collection/inference, deterministic diagnostics, bound terminal awaitables, cancel evidence strength, finished-cancelled outcome mapping, and non-CPU Rayon feature leakage. The current wave remediated those blockers with JoinSet generic arms, deterministic live-set diagnostics, rebinding rejection, pending-terminal awaitable tracking, underlying abort-handle preservation, stronger cancel fixtures, `Cancelled` outcome mapping, and a split CPU-only JoinSet preamble.
- `reviews/ad-hoc-production-concurrency-runtime-m3-joinset-review-pass-2.md`: `PASS`; reviewer verified all ten pass-1 blockers were remediated, re-ran `cargo test -p sifr -- --skip test_e2e_pass`, the six JoinSet pass fixtures, and non-CPU JoinSet emit gating. Non-blocking follow-ups remain for Sifr-owned diagnostics after binding a terminal awaitable, future sort-key uniqueness polish, and optional retirement/strengthening of the older length-only cancel evidence fixture now superseded by `join_set_cancel_all_task_cancelled.sifr`.

M3 scoped owner offload wave targeted local validation:

- `cargo check -p sifr_lowering -p sifr_codegen` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_scope_spawn_blocking.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_group_spawn_cpu.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_group_spawn_cpu_user_error.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_scope_spawn_cpu_unannotated_rejected.sifr` -> expected fail with `SIFR-ASYNC-0005`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_group_spawn_blocking_error_mismatch_rejected.sifr` -> expected fail with `SIFR-TYPE-0002`.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/task_scope_spawn_blocking.sifr | rg -n "rayon|__sifr_scope_spawn_cpu|__sifr_with_silent_scope_cpu_panic_hook"` -> no matches; scoped blocking-only usage emits no Rayon references.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/task_group_spawn_cpu.sifr | rg -n "rayon|__sifr_scope_spawn_cpu|__sifr_with_silent_scope_cpu_panic_hook"` -> PASS; scoped CPU usage emits the expected CPU bridge and Rayon references.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2155 files checked, 900-line limit.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; platform golden reported pass=5, skip=2; create-pr e2e pass suite reported 88 passed, 0 failed, cache_hits=20/23; advisory: warm wall-time budget exceeded.

M3 scoped owner offload wave review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m3-scoped-offload-review-pass-1.md`: `PASS`; reviewer independently re-ran the three pass fixtures, the two fail fixtures, generated Cargo/Rust dependency gating checks for scoped blocking and scoped CPU usage, and verified scoped `Task[T, E]` observation semantics, TaskGroup open/error-homogeneity reuse, typed CPU worker failure mapping, manifests, traceability, and docs. Non-blocking follow-ups remain for receiver-specific diagnostic wording, optional runtime-emission split polish, validator return-shape cleanup, symmetric fixture expansion, and user-facing cancellation wording for already-started blocking work.
- PR #2323 merged at `2768218fa27118d0c6b7f6d019002a7309eeb0d7`.
- `reviews/ad-hoc-production-concurrency-runtime-m3-joinset-review-pass-3.md`: `PASS`; independent retry review confirmed the same blocker closure and PR readiness.

M3 default parallel pool closure targeted local validation:

- `cargo fmt --check` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_default_pool_reused.sifr` -> PASS.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/parallel_default_pool_reused.sifr | rg -n "OnceLock|__SIFR_DEFAULT_PARALLEL_POOL|__sifr_default_parallel_pool|__sifr_build_parallel_pool\\(__sifr_default_parallel_worker_count"` -> PASS; emitted Rust uses the process-local `OnceLock` default pool path and no top-level fresh default-pool build call remains.
- `cargo check -p sifr_codegen` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2156 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_map_basic.sifr`, `parallel_try_map_basic.sifr`, and `parallel_pool_map_basic.sifr` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; platform golden reported pass=5, skip=2; create-pr e2e pass suite reported 89 passed, 0 failed, cache_hits=19/23; advisory: warm wall-time budget exceeded.

M3 default parallel pool closure review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m3-default-pool-review-pass-1.md`: `PASS`; reviewer verified top-level `parallel.map`/`try_map` now use one private process-local `OnceLock` Rayon pool, typed default-pool construction failures remain `WorkerRuntimeError`/`WorkerError`, configured `Pool(config)` semantics remain unchanged, manifests and traceability are honest, and no Rayon global pool mutation is introduced. Non-blocking traceability wording for cached construction failure was applied before PR validation.
- PR #2326 merged at `69f7a06ad12948dcd071de21c991c650ae062672`.

M3 worker capture-sendability closure targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo check -p sifr_lowering` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2162 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr_lowering nested_function -- --nocapture` -> PASS; 18 passed.
- `cargo test -p sifr_lowering ownership_and_async -- --nocapture` -> PASS; 58 passed, 1 ignored.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/spawn_blocking_non_send_capture_rejected.sifr` -> expected fail with `SIFR-OWN-0010`.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` -> PASS; fail suite reported 415 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; platform golden reported pass=5, skip=2; create-pr e2e pass suite reported 89 passed, 0 failed, cache_hits=23/23; advisory: warm wall-time budget exceeded.
- Post-`origin/main` merge rerun: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; platform golden reported pass=5, skip=2; create-pr e2e pass suite reported 89 passed, 0 failed, cache_hits=20/23; advisories: warm wall-time budget exceeded and warm-cache hit rate below advisory target.

M3 worker capture-sendability closure review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m3-capture-sendability-review-pass-1.md`: `PASS`; reviewer verified capture-summary scoping, non-send capture diagnostics, sendable nested-capture deferral, unchanged top-level worker behavior, worker-boundary coverage, and honest docs. Non-blocking feedback requested symmetric `task.spawn_blocking()` validator coverage and fixture.
- `reviews/ad-hoc-production-concurrency-runtime-m3-capture-sendability-review-pass-2.md`: `PASS`; reviewer verified the `task.spawn_blocking()` symmetry fix, `spawn_blocking_non_send_capture_rejected` fixture, unchanged capture-summary scoping, full named-worker boundary coverage, and no docs overclaim.
- PR #2329 merged at `21e66d84ea81e3609762140f57a6b76fb7c90926`.

M3 closeout wave targeted local validation:

- `cargo check -p sifr_codegen -p sifr --quiet` -> PASS after initializing the Ruff submodule in the auxiliary clean worktree.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_map_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_default_pool_reused.sifr` -> PASS after rebasing over PR #2326; two top-level `sifr.parallel.map` calls in one process exercise generated default-pool reuse.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_try_map_user_error_typed.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_map_worker_panic_typed.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/parallel_pool_map_basic.sifr` -> PASS.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/parallel_map_basic.sifr | rg -n "OnceLock|__SIFR_DEFAULT_PARALLEL_POOL|__sifr_default_parallel_pool|__SIFR_WORKER_PANIC_HOOK_LOCK"` -> PASS; top-level `sifr.parallel` now emits a lazy private default pool and shared serialized hook-suppression guard.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2156 files checked, 900-line limit after rebasing over the scoped owner offload wave.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo check -p sifr_lowering -p sifr_codegen -p sifr --quiet` -> PASS after shared hook and clippy remediation.
- `cargo clippy --workspace -- -D warnings` -> PASS after fixing `JoinSet` doc comments, borrowing JoinSet type parameters in lowering helpers, and sharing one generated worker panic-hook guard across `sifr.parallel`, `task.spawn_cpu`, and `JoinSet.spawn_cpu`.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/parallel_map_basic.sifr | rg -n "OnceLock|__SIFR_DEFAULT_PARALLEL_POOL|__sifr_default_parallel_pool|__SIFR_WORKER_PANIC_HOOK_LOCK|__sifr_with_silent_worker_panic_hook|__sifr_with_silent_parallel_panic_hook|__sifr_with_silent_cpu_panic_hook|__sifr_with_silent_join_set_panic_hook"` -> PASS; emitted code contains the lazy default pool and shared worker hook guard and no old per-surface hook helpers.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/spawn_cpu_worker_panic_typed.sifr | rg -n "__SIFR_WORKER_PANIC_HOOK_LOCK|__sifr_with_silent_worker_panic_hook|__sifr_with_silent_cpu_panic_hook"` -> PASS; emitted CPU offload code uses the shared hook guard and no old CPU-only helper.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/task_group_spawn_cpu.sifr | rg -n "__SIFR_WORKER_PANIC_HOOK_LOCK|__sifr_with_silent_worker_panic_hook|__sifr_with_silent_scope_cpu_panic_hook"` -> PASS after rebasing over PR #2323; emitted scoped owner CPU offload code uses the shared hook guard and no old scoped CPU-only helper.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/join_set_spawn_cpu_join_all_ordered.sifr | rg -n "__SIFR_WORKER_PANIC_HOOK_LOCK|__sifr_with_silent_worker_panic_hook|__sifr_with_silent_join_set_panic_hook"` -> PASS; emitted JoinSet CPU code uses the shared hook guard and no old JoinSet-only helper.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_cpu_worker_panic_typed.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/task_group_spawn_cpu.sifr` -> PASS after rebasing over PR #2323.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/join_set_spawn_cpu_join_all_ordered.sifr` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS after rebasing over the scoped owner offload and default parallel pool closure waves; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`89 passed`, `0 failed`, `cache_hits=21/23`).

M3 closeout wave review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-1.md`: `CHANGES_REQUESTED`; reviewer found missing full baseline validation, inherited clippy failures, a cross-surface panic-hook race because only `sifr.parallel` used the new mutex, and doc precision gaps. The current wave remediates those blockers with full workspace clippy, create-pr validation, borrowed JoinSet type parameters/doc-comment fixes, one shared generated worker panic-hook guard used by `sifr.parallel`, configured `Pool` work, scoped owner CPU offload, `task.spawn_cpu`, and `JoinSet.spawn_cpu`, a two-call default-pool reuse fixture, and explicit documentation of cached default-pool construction failures and serialized configured-Pool hook suppression.
- `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-2.md`: `PASS`; reviewer verified all pass-1 blockers and low-severity findings were remediated, re-ran workspace clippy, confirmed the refreshed create-pr report (`86 passed`, `0 failed`, platform golden `pass=5`, `skip=2`, no advisories), and confirmed no strict M3 closure blockers remain after this closeout PR merges and the ledger is updated.
- `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-3.md`: `PASS`; post-rebase reviewer verified the PR #2323 scoped owner CPU offload surface is folded into the shared worker panic-hook guard, re-checked emission predicates and ordering, confirmed post-rebase validation (`89 passed`, `0 failed`, platform golden `pass=5`, `skip=2`), and confirmed no strict M3 closure blockers remain after this PR and the final ledger update merge.
- `reviews/ad-hoc-production-concurrency-runtime-m3-closeout-review-pass-4.md`: `PASS`; final post-PR-#2326 reviewer verified the branch preserves the canonical lazy default pool fixture and default-pool implementation, removes the duplicate closeout fixture, routes all five CPU/Rayon surfaces through the shared worker panic-hook guard, and is ready to force-push and merge with no strict M3 closure blockers.
- PR #2325 merged at `9edf51988475ce6711bb42e79b01e96c8e34e9b5`; M3 is closed.

M4 sync process foundation targeted local validation:

- `cargo check -p sifr_stdlib -p sifr_codegen -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_output_text.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_bytes_env_cwd_stdin.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_shell_exec_output.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_blocking_direct_async_rejected.sifr` -> expected FAIL with `SIFR-ASYNC-0003`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_shell_exec_direct_async_rejected.sifr` -> expected FAIL with `SIFR-ASYNC-0007`.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_sync_output_text.sifr | rg "std::process::Command|process_output_text|subprocess|sh|split_once|Stdio::piped"` -> PASS; emitted ordinary process path uses `std::process::Command`, env splitting, and piped stdio, with no legacy subprocess helper.
- `cargo fmt --check` -> PASS after formatting the new M4 Rust modules.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2163 files checked, 900-line limit.
- `python3 scripts/check_diagnostic_docs_sync.py` -> PASS after generating `SIFR-ASYNC-0007` docs.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 413 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`92 passed`, `0 failed`, `cache_hits=23/25`).

M4 sync process foundation review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-sync-process-review-pass-1.md`: `PASS`; reviewer verified ordinary argv APIs lower to `std::process::Command` without shell or legacy subprocess helpers, shell APIs are explicit and classified with `SIFR-ASYNC-0007`, env/cwd/stdin/output/text behavior is typed without data-dependent panics, imported workload metadata covers stdlib imports and local re-exports, traceability honestly preserves remaining M4 process lifecycle work, and the wave is ready to PR. Non-blocking follow-ups were recorded for stdin setter semantics, deletion of unused legacy `_sifr.sys.subprocess_*` intrinsic paths, future stdlib re-export workload metadata, and later signal/timeout/cancellation/text-mode completion.
- PR #2331 merged at `b473c763ae3e92d614e7799af956bafcf4d60cb8`.

M4 sync child wait targeted local validation:

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr` -> PASS.
- Existing sync process regressions: `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`, and `process_spawn_wait_status` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_wait_direct_async_rejected.sifr` -> expected FAIL with `SIFR-ASYNC-0003`.
- Existing process async-diagnostic regressions: `process_blocking_direct_async_rejected`, `process_shell_exec_direct_async_rejected`, and `process_wait_direct_async_rejected` -> expected FAIL with `SIFR-ASYNC-0003` / `SIFR-ASYNC-0007` / `SIFR-ASYNC-0003`.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr | rg "__SIFR_PROCESS_CHILDREN|__sifr_next_process_child_id|std::process::Child|process_spawn|process_wait|std::process::Command"` -> PASS; emitted spawn/wait path includes the private process-child table and `std::process::Command`.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_sync_output_text.sifr | rg "__SIFR_PROCESS_CHILDREN|__sifr_next_process_child_id|std::process::Child"` -> expected no matches; ordinary output path does not emit child-handle runtime state.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2172 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 418 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`93 passed`, `0 failed`, `cache_hits=24/25`).

M4 sync child wait review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-child-wait-review-pass-1.md`: `PASS`; reviewer verified the wave is sync-only and does not overclaim pipes/async process support, process-child runtime state is gated to spawn/wait users, `process_wait` is one-shot and typed without data-dependent panics, top-level `wait(child)` triggers imported `@blocking_io` direct-async diagnostics, top-level/method wait asymmetry is safe for this wave, and traceability preserves remaining lifecycle work. Non-blocking follow-ups were recorded for unified wait-observation wording, explicit unwaited-child leak/drop cleanup documentation, possible tighter `Child`-only import preamble gating, and legacy `_sifr.sys.subprocess_*` cleanup.
- PR #2334 merged at `314e7f9ff7b300f7a333655fa2ad3ed756b29442`.

M4 timeout status evidence wave targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo check -p sifr_stdlib -p sifr_codegen -p sifr --quiet` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo test -p sifr_codegen lowers_process_timeout_intrinsics_via_registry -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_timeout_status.sifr` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 418 fail tests completed.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2171 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_timeout_status.sifr | rg "try_wait|kill\\(\\)|try_from_secs_f64|checked_add|is_finite|__timed_out"` -> PASS; emitted timeout paths guard invalid timeout values, use checked duration conversion and checked host-clock deadline construction for out-of-range values, poll with `try_wait`, kill timed-out children, and return typed timeout evidence.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`93 passed`, `0 failed`, `cache_hits=22/25`, `report_signature=91dc84a36565dad4`).
- Post-`origin/main` merge rerun: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`94 passed`, `0 failed`, `cache_hits=24/25`, `report_signature=e656d8db94f60742`).

M4 timeout status evidence wave review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-timeout-status-review-pass-1.md`: `CHANGES_REQUESTED`; reviewer found a user-triggerable panic for positive finite timeout values too large for `Duration::from_secs_f64`. The wave was remediated by switching generated timeout conversion to checked `Duration::try_from_secs_f64`, adding an overflow `ProcessError` regression fixture, and tightening traceability.
- `reviews/ad-hoc-production-concurrency-runtime-m4-timeout-status-review-pass-2.md`: `PASS`; reviewer verified the pass-1 overflow blocker was fixed and noted a non-blocking theoretical host-clock overflow band in `Instant + Duration`.
- `reviews/ad-hoc-production-concurrency-runtime-m4-timeout-status-review-pass-3.md`: `PASS`; reviewer verified the additional `Instant::checked_add(...).ok_or_else(ProcessError)?` hardening closes the host-clock deadline overflow path and no timeout-path data-dependent panic blocker remains.
- Merged as PR #2336: https://github.com/sifr-lang/sifr/pull/2336

M4 sync child kill targeted local validation:

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_child_kill_wait.sifr` -> PASS.
- Existing process pass regressions: `process_sync_output_text`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`, `process_spawn_wait_status`, and `process_child_kill_wait` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_kill_direct_async_rejected.sifr` -> expected FAIL with `SIFR-ASYNC-0003`.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2174 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 419 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`129.98s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`94 passed`, `0 failed`, `cache_hits=22/25`).
- Post-`origin/main` rebase rerun over the timeout status evidence wave: `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet`, `process_child_kill_wait`, `process_timeout_status`, expected `process_kill_direct_async_rejected` `SIFR-ASYNC-0003`, `cargo fmt --check`, file-size guardrails (`2176` files), HIR guardrails, and `cargo test -p sifr test_e2e_fail -- --nocapture` (`420` fail tests) -> PASS.
- Post-`origin/main` rebase rerun: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`152.56s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`95 passed`, `0 failed`, `cache_hits=24/25`, `report_signature=d8d730bd5475756c`).

M4 sync child kill review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-child-kill-review-pass-1.md`: `PASS`; reviewer verified the wave is an honest sync forceful-kill slice, `process_kill` returns typed `ProcessError` for closed/unknown handles without data-dependent panics, kill preserves the child handle for later `wait`, top-level `kill(child)` triggers `SIFR-ASYNC-0003`, process-child runtime gating remains intact, and docs do not overclaim graceful termination, timeout escalation, structured cancellation, or signal evidence. Non-blocking feedback was applied before PR by changing the fixture from `sh -c "sleep 5"` to direct `sleep 30`, documenting that kill targets only the immediate child handle, and tracking method-form `@blocking_io` enforcement for `Child.wait()` / `Child.kill()` as later compiler work.
- Merged as PR #2337: https://github.com/sifr-lang/sifr/pull/2337 (`2c6addfc2d67cc3fca15aa88d3e3956218fd106d`).

M4 signal status evidence targeted local validation:

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_signal_status.sifr` -> PASS; Unix `SIGTERM` exits are surfaced as `Status(kind="signal", signal=15)`.
- Existing process regressions `process_sync_output_text`, `process_spawn_wait_status`, `process_timeout_status`, and `process_child_kill_wait` -> PASS after updating killed-child observation to expect signal status on Unix.
- Expected async diagnostics `process_blocking_direct_async_rejected`, `process_shell_timeout_direct_async_rejected`, and `process_kill_direct_async_rejected` -> expected FAIL with `SIFR-ASYNC-0003` / `SIFR-ASYNC-0007` / `SIFR-ASYNC-0003`.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_sync_output_text.sifr | rg "__sifr_process_exit_signal|ExitStatusExt|__SIFR_PROCESS_CHILDREN"` -> PASS; ordinary process status users emit the signal helper without the child-handle table.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr | rg "__sifr_process_exit_signal|__SIFR_PROCESS_CHILDREN|__sifr_next_process_child_id"` -> PASS; spawn/wait users emit both the signal helper and private child-handle table.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2177 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 420 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`166.98s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`96 passed`, `0 failed`, `cache_hits=21/25`, `report_signature=f84374f7aa32a96e`).

M4 signal status evidence review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-signal-status-review-pass-1.md`: `PASS`; reviewer verified tuple-shaped raw exit status is coherent across stdlib typing, lowering, and Sifr wrappers; Unix signal exits surface as `Status(kind="signal", signal=N, success=False)` while timeout evidence retains precedence; the cfg-gated `__sifr_process_exit_signal` helper is portable by inspection; ordinary process APIs emit the status helper without the child table; manifests, traceability, supported-host matrix, and execution ledger are honest about Unix-only signal evidence and remaining M4 process lifecycle work. Non-blocking follow-ups were applied by adding the omitted child-kill PR link and documenting that `signal` carries the meaningful status when `kind == "signal"`.
- Merged as PR #2341: https://github.com/sifr-lang/sifr/pull/2341 (`56b3aadeb65b63fc589c2530b0c02031b0e9596a7`).

M4 legacy subprocess intrinsic cleanup implementation:

- Removed the unused `_sifr.sys.subprocess_run`, `_sifr.sys.subprocess_run_with_input`, and `_sifr.sys.subprocess_run_structured` intrinsic signatures now that production process behavior is routed through `sifr.process`.
- Deleted the matching codegen registry dispatch arms and legacy shell-shaped lowerer module, so no generated code path can bypass the production `sifr.process` process/status model through the old private intrinsic names.
- Added stdlib and codegen negative guards proving the deleted private intrinsic names are neither registered nor lowered.
- Updated M4 process traceability by closing the legacy `_sifr.sys.subprocess_*` cleanup follow-up; public `sifr.subprocess` and bare `subprocess` diagnostics remain intact as namespace-contract behavior.

M4 legacy subprocess intrinsic cleanup targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_stdlib legacy_subprocess_intrinsics_are_not_registered -- --nocapture` -> PASS.
- `cargo test -p sifr_codegen legacy_subprocess_intrinsics_are_not_lowered -- --nocapture` -> PASS.
- `cargo check -p sifr_stdlib -p sifr_codegen -p sifr --quiet` -> PASS.
- Process regressions `process_sync_output_text` and `process_signal_status` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2176 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`868.13s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`96 passed`, `0 failed`, `cache_hits=7/25`, `report_signature=f84374f7aa32a96e`).

M4 legacy subprocess intrinsic cleanup review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-legacy-subprocess-intrinsic-cleanup-review-pass-1.md`: `PASS`; reviewer verified the cleanup is surgical, no live consumer of the removed private intrinsic names remains outside negative guards and historical notes, public `sifr.subprocess` / bare `subprocess` diagnostics still point to `sifr.process`, production `process_*` dispatch is untouched, and the full create-pr gate passed. Non-blocking note: keep unrelated network-phase files out of this PR.
- Merged as PR #2344: https://github.com/sifr-lang/sifr/pull/2344 (`6f4c0fe56cc7c9f7348bf73a0d8c6349df99b9b8`).
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`340.13s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`96 passed`, `0 failed`, `cache_hits=25/25`, `report_signature=f84374f7aa32a96e`).

M4 async process run/output targeted local validation:

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_output.sifr` -> PASS; `async_run` and `async_output` return typed process `Status`/`Output` through `Awaitable[Result[...]]`, and async output with stdin bytes returns typed `ProcessError` while owned pipe support remains deferred.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_async_run_output.sifr | rg "__sifr_process_async_run|__sifr_process_async_output|tokio::process::Command|Box::pin|std::process::Command|__SIFR_PROCESS_CHILDREN|__sifr_process_status_from_exit"` -> PASS; emitted code includes boxed owned async futures, the Tokio process helper, and the private status conversion helper, with no sync child table.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_sync_output_text.sifr | rg "__sifr_process_async_run|tokio::process::Command|std::process::Command|__SIFR_PROCESS_CHILDREN"` -> PASS; ordinary sync output emits `std::process::Command` and no async helper or child table.
- Existing M4 regressions `process_sync_output_text`, `process_spawn_wait_status`, `process_timeout_status`, `process_signal_status`, and `process_child_kill_wait` -> PASS.
- Expected async diagnostics `process_blocking_direct_async_rejected`, `process_shell_timeout_direct_async_rejected`, and `process_kill_direct_async_rejected` -> expected FAIL with `SIFR-ASYNC-0003` / `SIFR-ASYNC-0007` / `SIFR-ASYNC-0003`.
- `cargo test -p sifr_codegen test_generate_project_emits_tokio_dependency_when_required -- --nocapture` -> PASS.
- `cargo test -p sifr test_generate_cargo_toml_required_tokio_uses_runtime_features -- --nocapture` -> PASS; generated e2e harness Cargo specs include Tokio `process`.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2178 files checked, 900-line limit after rebasing over the legacy subprocess intrinsic cleanup wave.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 420 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS after rebasing over the legacy subprocess intrinsic cleanup wave and current `origin/main`; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`201.89s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`97 passed`, `0 failed`, `cache_hits=26/26`, `report_signature=36054c952f8fafec`).
- Broad non-lane probe `cargo test -p sifr test_e2e_pass -- --nocapture` failed in unrelated existing I/O/encoding fixtures (`cpython_io_subset`, `stdlib_io_consolidated`, `open_*`, `bytes_conversion_errors`) with no subprocess or async-process failures; the authoritative create-pr lane above passed after adding Tokio `process` to the generated harness dependency spec.

M4 async process run/output review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-process-review-pass-1.md`: `PASS`; reviewer verified the wave is scoped to async argv run/output loopback, public APIs consume `Command` and return typed awaitables, generated futures own cloned command fields, Tokio `process` is wired through generated projects and grouped e2e harnesses, stdin bytes are rejected with a typed owned-pipe deferral error, sync process paths and child-table gating are not regressed, and traceability/host matrix/manifests do not overclaim spawn/wait/communicate, pipes, timeout, cancellation, shell async APIs, scoped supervision, or Windows support.
- Merged as PR #2345: https://github.com/sifr-lang/sifr/pull/2345 (`8fce5ab17ab993903937d1be8588285606d61c84`).

M4 stdin append semantics evidence wave implementation:

- Closed the repeated-`Command.stdin_bytes(...)` decision by treating each call as an append in call order.
- Extended `process_sync_bytes_env_cwd_stdin` to write stdin through two calls (`b"pipe-"`, then `b"bytes"`) and assert the child receives the concatenated `b"pipe-bytes"` payload.
- Updated M4 process traceability to document append semantics and removed the follow-up boundary for deciding append vs replace behavior.

M4 stdin append semantics evidence targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_bytes_env_cwd_stdin.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_output_text.sifr` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2176 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`290.39s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`96 passed`, `0 failed`, `cache_hits=24/25`, `report_signature=f84374f7aa32a96e`).
- Post-`origin/main` merge rerun over the async process output wave: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`361.07s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`97 passed`, `0 failed`, `cache_hits=24/26`, `report_signature=36054c952f8fafec`).

M4 stdin append semantics evidence review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-stdin-append-semantics-review-pass-1.md`: `PASS`; reviewer verified the two-call fixture uniquely distinguishes append-in-call-order from replace-with-first or replace-with-last semantics, traceability honestly closes only the stdin append decision, no out-of-scope process lifecycle APIs are introduced, and the validation set is sufficient for this evidence slice.
- Merged as PR #2348: https://github.com/sifr-lang/sifr/pull/2348 (`afffaa3f8e40b9af0bbdffe13bafb61e053afb03`).

M4 method-form blocking workload diagnostics implementation:

- Extended workload annotation collection to record class method annotations as qualified names such as `Child.wait` and `Child.kill`.
- Preserved qualified class-method workload metadata through stdlib bootstrap and external class imports, mirroring existing class-method default/vararg propagation.
- Checked method calls in async contexts against the qualified workload metadata, so imported stdlib process methods now trigger the same direct-async diagnostics as top-level workload functions.
- Added fail fixtures for `child.wait()` and `child.kill()` inside async functions.
- Updated M4 process traceability to close the method-form `@blocking_io` enforcement follow-up.

M4 method-form blocking workload diagnostics targeted local validation:

- `cargo check -p sifr_lowering -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_child_wait_method_direct_async_rejected.sifr` -> expected FAIL with `SIFR-ASYNC-0003` at `child.wait()`.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_child_kill_method_direct_async_rejected.sifr` -> expected FAIL with `SIFR-ASYNC-0003` at `child.kill()`.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr` -> PASS; sync `Child.wait()` remains accepted outside async contexts.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_child_kill_wait.sifr` -> PASS; sync `Child.kill()` remains accepted outside async contexts.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2180 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 422 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`417.65s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`97 passed`, `0 failed`, `cache_hits=24/26`, `report_signature=36054c952f8fafec`).

M4 method-form blocking workload diagnostics review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-process-method-workloads-review-pass-1.md`: `PASS`; reviewer verified qualified class-method workload collection, stdlib/project import propagation, method-call async diagnostics, bounded false-positive risk, new fail fixtures, and traceability honesty. Non-blocking note: keep unrelated network-phase files out of this PR.
- Merged as PR #2350: https://github.com/sifr-lang/sifr/pull/2350 (`cdfca07b19a6675463113c881525df620fa6eb44`).
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`343.95s`, warm target `<=2m`). Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`97 passed`, `0 failed`, `cache_hits=26/26`, `report_signature=36054c952f8fafec`).

M4 sync stdout/stderr pipe readers targeted local validation:

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_pipe_readers.sifr` -> PASS; sync `spawn` with stdout/stderr `Stdio("pipe")` exposes one-shot `PipeReader.read_all()` handles and typed double-read / double-extraction errors.
- Existing process regressions `process_sync_output_text`, `process_spawn_wait_status`, `process_timeout_status`, `process_signal_status`, `process_child_kill_wait`, and `process_async_run_output` -> PASS.
- Expected async diagnostics `process_blocking_direct_async_rejected`, `process_shell_timeout_direct_async_rejected`, and `process_kill_direct_async_rejected` -> expected FAIL with `SIFR-ASYNC-0003` / `SIFR-ASYNC-0007` / `SIFR-ASYNC-0003`.
- Emission check for `process_spawn_pipe_readers` -> PASS; emitted Rust includes `__SIFR_PROCESS_CHILDREN`, `__SIFR_PROCESS_PIPE_READERS`, `__sifr_process_child_stdout`, `__sifr_process_pipe_read_all`, and `std::io::Read::read_to_end`.
- Emission check for `process_sync_output_text` -> PASS; ordinary output emits `std::process::Command` without `__SIFR_PROCESS_CHILDREN`, `__SIFR_PROCESS_PIPE_READERS`, or pipe helper functions.
- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2180 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 420 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`321.21s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`98 passed`, `0 failed`, `cache_hits=2/26`, `report_signature=559a90cf856fe902`).
- Post-`origin/main` rebase rerun after the stdin append semantics ledger merge: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`232.37s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`98 passed`, `0 failed`, `cache_hits=25/26`, `report_signature=559a90cf856fe902`).
- Post-`origin/main` rebase rerun after the method-form blocking diagnostics merge: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`206.64s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`98 passed`, `0 failed`, `cache_hits=24/26`, `report_signature=559a90cf856fe902`).
- Post-`origin/main` rebase rerun after the method-form blocking diagnostics ledger merge: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`177.84s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`98 passed`, `0 failed`, `cache_hits=26/26`, `report_signature=559a90cf856fe902`).

M4 sync stdout/stderr pipe readers review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-pipe-readers-review-pass-1.md`: `PASS`; reviewer verified the wave is limited to sync stdout/stderr pipe readers, `Command.stdout` / `Command.stderr` default to inherit and accept typed `Stdio` modes, `Child.stdout` / `Child.stderr` transfer one-shot `PipeReader` handles, `PipeReader.read_all()` returns bytes or typed `ProcessError` without data-dependent panics, generated spawn/pipe helpers are gated to child/pipe users and ordinary output does not emit pipe tables, file-size guardrails remain under 900 lines after the split, manifests include the new fixture, and docs do not overclaim stdin `PipeWriter`, streaming reads, async pipes/communicate, timeout/cancellation/scoped supervision, or Windows support.
- `reviews/ad-hoc-production-concurrency-runtime-m4-pipe-readers-review-pass-2.md`: `PASS`; post-rebase reviewer verified the branch preserved PR #2348 stdin append semantics, kept the pipe-reader wave after that evidence without conflict markers or stale claims, revalidated the original sync-only pipe-reader invariants, reproduced targeted checks, confirmed the post-rebase create-pr report (`98 passed`, `0 failed`, platform golden `pass=5`, `skip=2`, `report_signature=559a90cf856fe902`), and found no blocking correctness, generated-code-safety, lifecycle, panic-freedom, gating, validation, or documentation issues.
- `reviews/ad-hoc-production-concurrency-runtime-m4-pipe-readers-review-pass-3.md`: `CHANGES_REQUESTED`; post-PR #2350 reviewer revalidated the original pipe-reader invariants but found the branch was still behind PR #2351 and would drop the method-form diagnostics merge-link and merge-ledger validation lines. The next revision rebases onto current `origin/main` and preserves those method diagnostics evidence lines before the pipe-reader block.
- `reviews/ad-hoc-production-concurrency-runtime-m4-pipe-readers-review-pass-4.md`: `PASS`; final reviewer verified the pass-3 blocker was fixed by preserving the PR #2350 merge-link and merge-ledger validation from PR #2351 before the pipe-reader block, rechecked traceability for both method-form diagnostics fixtures plus `process_spawn_pipe_readers`, confirmed the sync-only pipe-reader implementation and typed-error/panic-freedom invariants, confirmed helper gating and documentation boundaries, and verified the latest create-pr evidence (`98 passed`, `0 failed`, `cache_hits=26/26`, `report_signature=559a90cf856fe902`).
- Merged as PR #2352: https://github.com/sifr-lang/sifr/pull/2352 (`cdd33ba0fd0469f65a0f4f26bf5fdcf8555e2bfd`).
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`160.57s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`98 passed`, `0 failed`, `cache_hits=26/26`, `report_signature=559a90cf856fe902`).

M4 async process run timeout targeted local validation:

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr --quiet` -> PASS.
- `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2181 files checked before the `origin/main` merge, then 2184 files checked after splitting async process preamble helpers from `process_runtime.rs`.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_timeout.sifr` -> PASS; `async_run_timeout` returns typed success and timeout `Status` evidence through `Awaitable[Result[Status, ProcessError]]`.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_output.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_timeout_status.sifr` -> PASS.
- Emission check for `process_async_run_timeout` -> PASS; timeout-only usage emits `__sifr_process_async_run_timeout` and the shared status helper without emitting the async output helper.
- Post-`origin/main` merge checks for `process_async_run_timeout.sifr` and `process_spawn_pipe_readers.sifr` -> PASS, proving the async timeout wave coexists with the sync pipe-reader merge.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS before merging `origin/main`; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`391.68s`, warm target `<=2m`). Included create-pr e2e pass suite (`98 passed`, `0 failed`, `report_signature=559a90cf856fe902`).
- Post-`origin/main` merge and async-preamble split rerun: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`571.32s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`99 passed`, `0 failed`, `cache_hits=7/26`, `report_signature=42aaf1077a936d74`).
- Broad non-lane probe `cargo test -p sifr_codegen --quiet` exposed unrelated stale/generated-code test failures and produced no accepted gate signal; the authoritative create-pr lane above passed after the focused checks.

M4 async process run timeout review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-run-timeout-review-pass-1.md`: `PASS`; reviewer verified public `sifr.process.async_run_timeout`, stdlib metadata, lowering, generated Tokio process timeout behavior, typed invalid-timeout errors, kill-and-reap timeout status evidence, helper emission gating, manifests, and traceability. Non-blocking notes covered redundant timeout `success = false`, NaN fixture coverage, and the existing raw multi-line Rust expression pattern.
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-run-timeout-review-pass-2.md`: `PASS`; post-`origin/main` merge reviewer verified the async preamble split was behavior-preserving, pipe-reader behavior from PR #2352 was preserved, timeout-only helper emission stayed minimal, no user-triggerable panic path was introduced, and the post-merge create-pr validation (`99 passed`, `0 failed`, `report_signature=42aaf1077a936d74`) was sufficient.
- Merged as PR #2354: https://github.com/sifr-lang/sifr/pull/2354 (`dd24a7c3234df280a437acf0f5f5c394bdbc5f56`).

M4 sync stdin pipe writer targeted local validation:

- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2187 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_pipe_writer.sifr` -> PASS; sync child stdin `PIPE` extraction supports repeated byte writes and explicit close/EOF.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_pipe_readers.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_child_kill_wait.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_sync_output_text.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/process_pipe_writer_method_direct_async_rejected.sifr` -> expected FAIL with `SIFR-ASYNC-0003`.
- Emission check for `process_spawn_pipe_writer` -> PASS; emitted Rust includes the child table, writer table, child stdin extraction, pipe write/close helpers, and `std::io::Write::write_all`.
- Emission check for `process_sync_output_text` -> PASS; ordinary sync output emits no process child or pipe-writer helper tables.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 423 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`422.39s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`100 passed`, `0 failed`, `cache_hits=22/26`, `report_signature=458ad42c8c1b262c`).

M4 sync stdin pipe writer review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-pipe-writer-review-pass-1.md`: `PASS`; reviewer verified typed `PipeWriter.write_all`/`close` behavior, one-shot child stdin extraction, repeated writes before close, spawn stdio mode arity/order across all layers, no generated runtime panic path, symmetric preamble gating, file-size guardrail compliance, honest docs/manifests, and accepted the table-wide writer mutex during sync blocking writes as non-blocking for this slice.
- Merged as PR #2357: https://github.com/sifr-lang/sifr/pull/2357 (`81eb29e671c8ac0b79928f4825c1daaf6bcfbf7a`).

M4 stdin guardrails follow-up:

- Closed stale duplicate PR #2356 after PR #2357 / PR #2358 landed the sync pipe-writer implementation and merge ledger from a different branch.
- Added a typed `ProcessError` in sync `spawn(command)` when `Command.stdin_bytes(...)` was configured, so one-shot output stdin payloads cannot be silently ignored by child-handle spawn.
- Threaded `Command.stdin_mode` through `async_run(...)`, `async_run_timeout(...)`, and `async_output(...)` into stdlib intrinsic metadata, async lowerers, and generated async process helpers.
- Added generated typed owned-pipe deferral errors for non-inherit `Command.stdin(...)` modes across async run/output/timeout, matching the existing async `stdin_bytes(...)` deferral until async owned pipes/communicate land.
- Extended `process_spawn_pipe_writer`, `process_async_run_output`, and `process_async_run_timeout` fixture coverage and tightened M4 traceability wording.

M4 stdin guardrails targeted local validation:

- `cargo fmt` and `python3 -m json.tool` for create-pr / merge e2e manifests -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_spawn_pipe_writer.sifr` -> PASS; now covers sync `spawn` rejecting `Command.stdin_bytes(...)` with a typed error.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_output.sifr` -> PASS; async run/output still work and reject both `stdin_bytes(...)` and non-inherit stdin modes with typed owned-pipe deferral errors.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_timeout.sifr` -> PASS; async timeout still returns success/timeout status evidence and rejects non-inherit stdin modes with typed owned-pipe deferral errors.
- Emission checks for `process_async_run_output`, `process_async_run_timeout`, and `process_spawn_pipe_writer` -> PASS; generated Rust includes the new async stdin-mode guards, 6/7-arg async helper signatures, timeout validation, sync spawn `stdin_bytes` typed error, and existing pipe-writer helpers.
- `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2187 files checked, 900-line limit.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 423 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`232.35s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`100 passed`, `0 failed`, `cache_hits=24/26`, `report_signature=458ad42c8c1b262c`).

M4 stdin guardrails review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-stdin-guardrails-review-pass-1.md`: `PASS`; reviewer verified sync `spawn(...)` now rejects `stdin_bytes(...)` before spawning, async run/output/timeout thread `stdin_mode` through public wrappers, stdlib metadata, lowerers, and generated helper signatures in the correct order, all async helpers return typed owned-pipe deferral errors for non-inherit stdin modes, fixtures cover the new guardrails, docs do not overclaim future async pipe/communicate work, file-size guardrails remain under 900 lines, and the create-pr lane evidence is sufficient.
- Merged as PR #2359: https://github.com/sifr-lang/sifr/pull/2359 (`408368be330fd0399ba6eeaf6cb060661a1104c8`).
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`214.95s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`100 passed`, `0 failed`, `cache_hits=24/26`, `report_signature=458ad42c8c1b262c`).

M4 async process output timeout implementation:

- Added public `sifr.process.async_output_timeout(command, seconds)` returning `Awaitable[Result[Output, ProcessError]]`.
- Added stdlib intrinsic metadata and lowering for `process_async_output_timeout` with the same owned command argument ordering as async output plus explicit `has_stdin` and timeout arguments.
- Added a generated Tokio helper that validates finite non-negative timeout values, rejects unsupported async stdin modes with typed `ProcessError`s, drains stdout/stderr asynchronously on normal completion, and kills then waits for timed-out children before returning timeout `Output` evidence.
- Added the Tokio `io-util` dependency feature required by async stdout/stderr drains across both generated projects and grouped e2e harness crates, and kept helper gating independent from plain async output.
- Added `process_async_output_timeout` fixture coverage to create-pr and merge manifests, process traceability, and the supported-host matrix without claiming async spawn/wait/communicate, public async pipes, cancellation, scoped supervision, or Windows support.

M4 async process output timeout targeted local validation:

- `cargo fmt` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo test -p sifr_codegen test_generate_project_emits_tokio_dependency_when_required --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_output_timeout.sifr` -> PASS; async output timeout captures stdout/stderr on normal completion, returns typed timeout `Output` evidence after kill/wait, and rejects invalid timeout and unsupported stdin shapes.
- Adjacent process regressions `process_async_run_output`, `process_async_run_timeout`, and `process_spawn_pipe_writer` -> PASS.
- Emission checks for `process_async_output_timeout` -> PASS; generated Rust includes `__sifr_process_async_output_timeout`, `tokio::select`, `AsyncReadExt`, stdout/stderr `read_to_end`, child kill/wait on timeout, and timeout `Status` construction.
- Emission check for `process_async_run_timeout` -> PASS; timeout-only status usage does not emit `__sifr_process_async_output_timeout` or the plain async output helper.
- Initial create-pr lane exposed the grouped e2e harness still rendering Tokio without `io-util`; fixed the harness dependency renderer and contract test so grouped fixture crates match generated project dependencies.
- `cargo test -p sifr test_generate_cargo_toml_required_tokio_uses_runtime_features --quiet` -> PASS after the harness dependency fix.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS after the harness dependency fix; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`184.51s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`101 passed`, `0 failed`, `cache_hits=25/26`, `report_signature=9212e77abfa82acc`).

M4 async process output timeout review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-output-timeout-review-pass-1.md`: `PASS`; reviewer verified the public wrapper, stdlib metadata, intrinsic lowering, and generated helper agree on 8-argument ordering; async output timeout validates timeout values, rejects unsupported stdin modes with typed `ProcessError`s, drains stdout/stderr asynchronously on normal completion, kills and waits on timeout, returns typed timeout `Output` evidence, gates independently from plain async output, propagates Tokio `io-util` through generated projects and grouped e2e harness crates, and keeps docs honest about remaining async spawn/wait/communicate, public async pipes, cancellation, scoped supervision, text-mode, and Windows follow-ups.
- Merged as PR #2362: https://github.com/sifr-lang/sifr/pull/2362 (`2a3cefce45ea1a4ed7ab3eb414affc42471f3844`).
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`183.33s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`101 passed`, `0 failed`, `cache_hits=25/26`, `report_signature=9212e77abfa82acc`).

M4 async stdin-byte communicate implementation:

- Threaded `Command.stdin_data` through `async_output(...)` and `async_output_timeout(...)` into stdlib intrinsic metadata, async intrinsic lowerers, and generated helper signatures while preserving typed rejection for non-inherit `Command.stdin(...)` modes.
- Replaced the plain async output helper's `Command.output()` path with explicit Tokio child spawning, conditional stdin piping, concurrent stdin write, stdout/stderr drains, and child wait so `Command.stdin_bytes(...)` is consumed as one-shot communicate input.
- Updated the async output-timeout helper to use the same concurrent stdin write and stdout/stderr drains inside the timeout race; the timeout arm still kills and waits/reaps the child before returning typed timeout `Output` evidence.
- Extended `process_async_run_output` and `process_async_output_timeout` fixtures to prove appended stdin bytes are delivered in order and that `Stdio("pipe")` remains a typed owned-pipe deferral until public async pipes land.

M4 async stdin-byte communicate targeted local validation:

- `cargo fmt` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_output.sifr` -> PASS; async output now echoes appended stdin bytes and still rejects `Stdio("pipe")`.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_output_timeout.sifr` -> PASS; async output timeout now echoes appended stdin bytes and still returns timeout evidence.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_timeout.sifr` -> PASS.
- Emission checks for `process_async_run_output` and `process_async_output_timeout` -> PASS; generated helpers include `stdin: Vec<u8>`, `AsyncWriteExt`, stdin `write_all`, and `__stdin.take()` so EOF is observed after one-shot input.
- Emission check for `process_async_run_timeout` -> PASS; status-only timeout usage still emits only `__sifr_process_async_run_timeout` and no async output/output-timeout helpers.
- `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2188 files checked.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; 423 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`192.32s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`101 passed`, `0 failed`, `cache_hits=25/26`, `report_signature=9212e77abfa82acc`).
- Post-review docs-nit rerun: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`154.89s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`101 passed`, `0 failed`, `cache_hits=26/26`, `report_signature=9212e77abfa82acc`).

M4 async stdin-byte communicate review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-communicate-stdin-review-pass-1.md`: `PASS`; reviewer verified wrapper/stdlib/lowerer/helper argument order, deadlock-free concurrent Tokio stdin/stdout/stderr/wait orchestration with `__stdin.take()` EOF, typed non-inherit stdin guardrails, timeout kill/wait reaping, helper gating, tests, scope-honest docs, and panic/lifecycle boundaries. Non-blocking docs nit: supported-host output-timeout row under-reported stdin-byte communicate coverage.
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-communicate-stdin-review-pass-2.md`: `PASS`; reviewer verified the supported-host matrix docs nit was closed, no public async pipe/spawn/wait/cancellation/scoped/text/Windows overclaim was introduced, implementation files remained unchanged from the pass-1 review, and refreshed create-pr validation evidence was sufficient.
- Merged as PR #2365 (`0c4c4a68411628d0f4ad137f9bdf4bdec004522b`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`245.69s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`101 passed`, `0 failed`, `cache_hits=22/26`, `report_signature=9212e77abfa82acc`).

M4 async process spawn/wait implementation:

- Added `AsyncChild`, `async_spawn(...)`, and `async_wait(...)` to `sifr.process` as the first native async child lifecycle surface. Top-level `async_wait(own child)` consumes the handle; method-form `AsyncChild.wait()` delegates to the same generated async wait helper.
- Added `_sifr.process.process_async_spawn` and `process_async_wait` intrinsic metadata and lowerers, plus generated Tokio helper emission with a private async-child handle table. `async_wait` removes the child from the table before awaiting so each async child is observed at most once.
- Kept public async owned pipes out of scope: `async_spawn` rejects `Command.stdin_bytes(...)` and explicit `stdin/stdout/stderr` stdio modes with typed `ProcessError` evidence until async pipe handles land.
- Added `process_async_spawn_wait` fixture coverage to create-pr and merge manifests for nonzero async wait status, method-form wait, one-shot wait errors, stdin-byte rejection, and explicit stdio-mode deferral.

M4 async process spawn/wait targeted local validation:

- `cargo fmt` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_output.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_output_timeout.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_run_timeout.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr` -> PASS; validates async spawn/wait status, one-shot wait, stdin-byte rejection, and explicit stdio-mode deferral.
- Emission checks for `process_async_spawn_wait` -> PASS; generated code includes `__SIFR_PROCESS_ASYNC_CHILDREN`, `tokio::process::Child`, `__sifr_process_async_spawn`, `__sifr_process_async_wait`, `AsyncChild::new`, and table-level closed/unknown wait errors.
- Emission checks for `process_async_run_timeout` and `process_async_run_output` -> PASS; those fixtures do not emit async spawn/wait helper state.
- `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2189 files checked.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; 423 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS after post-review cleanup; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`282.19s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`102 passed`, `0 failed`, `cache_hits=22/26`, `report_signature=5e93ca9f74a9781c`).
- Post-`origin/main` merge rerun after preserving PR #2367 sync terminate evidence: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`225.06s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`103 passed`, `0 failed`, `cache_hits=25/27`, `report_signature=2593463768412da4`).

M4 async process spawn/wait review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-spawn-wait-review-pass-1.md`: `PASS`; reviewer verified the narrow async child lifecycle wave, inherited-stdio-only spawn scope, typed `stdin_bytes` and stdio-mode deferrals, one-shot table-backed async wait without a mutex guard across await, generated-helper gating, honest traceability/host-matrix/manifests, and no user-triggerable panic path. Non-blocking notes covered the then-unused `AsyncChild._waited` field, dead shared-prelude collector branches mirroring sync spawn/wait, signal-status coverage remaining sync-only, the async process runtime file nearing the guardrail, and possible stdout/stderr deferral fixture coverage.
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-spawn-wait-review-pass-2.md`: `PASS`; reviewer confirmed the unused `AsyncChild._waited` field was removed from the public class and stdlib type metadata, the added `stdout(Stdio("pipe"))` typed deferral assertion is correct, refreshed create-pr validation evidence matches the tree, and no new blockers or scope/documentation mismatches were introduced.
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-spawn-wait-review-pass-3.md`: `CHANGES_REQUESTED`; post-`origin/main` merge reviewer verified the branch preserved PR #2367 sync terminate evidence and the pass-2 async spawn/wait implementation, but found a duplicate `M4 async process spawn/wait: in progress` line in the implementation PR list.
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-spawn-wait-review-pass-4.md`: `PASS`; reviewer confirmed the duplicate PR-list entry was collapsed to a single in-progress spawn/wait row after PR #2367, conflict markers remained absent, `git diff --check` stayed clean, and no new blocker was introduced.
- Merged as PR #2369 (`2acbcec324571381b7d5099041402bb7461c77b5`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`255.22s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`103 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=2593463768412da4`).

M4 sync PipeReader streaming reads implementation:

- Added `PipeReader.read(max_bytes)` and `PipeReader.close()` to `sifr.process` as `@blocking_io` sync pipe-reader APIs returning typed `Result[..., ProcessError]`.
- Added private `_sifr.process.process_pipe_read` and `process_pipe_reader_close` metadata, intrinsic lowering, generated helpers, and prelude filtering.
- `process_pipe_read` validates positive bounded read sizes, caps one read at 1 MiB, maps host I/O failures to typed `ProcessError`, preserves the reader handle after partial chunks, and removes the private reader handle at EOF. `process_pipe_reader_close` explicitly removes a partially-read handle.
- Kept async pipes, sendability/shareability checks, process supervision, and text-mode pipe decoding out of this slice.
- Added `process_pipe_reader_streaming` fixture coverage to create-pr and merge manifests, plus M4 traceability and supported-host matrix updates.

M4 sync PipeReader streaming reads targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo fmt` and `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2195 files checked, 900-line limit. Touched hand-maintained files remain below the cap, including `crates/sifr_codegen/src/stdlib_filter/implementation.rs` at 789 lines and `crates/sifr_codegen/src/preamble/process_child_pipes.rs` at 563 lines.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_pipe_reader_streaming.sifr` -> PASS; bounded sync reads return ordered chunks, invalid sizes produce typed `ProcessError`, EOF closes the reader, and explicit close rejects later reads.
- Adjacent pipe regressions `process_spawn_pipe_readers` and `process_spawn_pipe_writer` -> PASS.
- Emission check for `process_pipe_reader_streaming` -> PASS; emitted Rust includes `__SIFR_PROCESS_PIPE_READERS`, `__sifr_process_pipe_read`, `__sifr_process_pipe_reader_close`, `__sifr_process_pipe_read_all`, and the 1 MiB per-read guard without emitting async process child state.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 425 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`461.65s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`105 passed`, `0 failed`, `cache_hits=22/27`, `report_signature=d08ce200366c588c`).
- Post-`origin/main` merge rerun after the async runtime split merge: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`310.92s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`105 passed`, `0 failed`, `cache_hits=25/27`, `report_signature=d08ce200366c588c`).

M4 sync PipeReader streaming reads review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-pipe-reader-streaming-review-pass-1.md`: `PASS`; reviewer verified the public `PipeReader.read(max_bytes)` / `PipeReader.close()` surface preserves typed `ProcessError`, stdlib metadata/lowerers/registry/helper signatures agree, bounded reads validate positive sizes and cap one read at 1 MiB, partial reads preserve the private reader handle, EOF and explicit close remove it, prelude filtering emits sync pipe state without async child state, fixture/manifests/traceability/host matrix/ledger are honest about async pipes/sendability/supervision/text-mode deferrals, and file-size guardrails remain under 900 lines. Non-blocking follow-ups were recorded for PR hygiene around unrelated network/http files, future structured helper construction cleanup, later lock-scope hardening before concurrent sync readers matter, and optional direct-async rejection fixture coverage.
- Merged as PR #2377 (`8a5aa80e6d738bb2e2e21639b250321c9bb1a621`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`866.58s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`105 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=d08ce200366c588c`).

M4 process handle boundary diagnostics implementation:

- Marked `Child`, `AsyncChild`, `PipeReader`, `PipeWriter`, `AsyncPipeReader`, and `AsyncPipeWriter` as non-send and non-share-safe in the shared lowering boundary classifier.
- This centralizes process-owned handle restrictions for task spawn arguments, offload worker captures, CPU/offload return types, channel elements, nested fields, and share-safety checks without adding API-specific fallback paths.
- Added fail fixtures for `PipeReader` task-boundary movement, `AsyncChild` task-boundary movement, `PipeWriter` `spawn_blocking` capture, `Child` `spawn_cpu` return, `PipeReader` channel transfer, `PipeReader` shared-state publication, `AsyncPipeReader` task-boundary movement, `AsyncPipeWriter` channel transfer, and `AsyncPipeWriter` shared-state publication.

M4 process handle boundary diagnostics targeted local validation:

- `cargo fmt` -> PASS.
- Direct checks for `process_pipe_reader_task_boundary_rejected`, `process_async_child_task_boundary_rejected`, `process_pipe_writer_spawn_blocking_capture_rejected`, `process_child_spawn_cpu_return_rejected`, `process_pipe_reader_channel_element_rejected`, `process_pipe_reader_shared_rejected`, `process_async_pipe_reader_task_boundary_rejected`, `process_async_pipe_writer_channel_element_rejected`, and `process_async_pipe_writer_shared_rejected` -> PASS; each emits the intended `SIFR-OWN-0010`, `SIFR-OWN-0011`, `SIFR-OWN-0012`, or `SIFR-TYPE-0002` diagnostic.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 434 fail tests completed.
- Guardrails and focused checks -> PASS: `cargo fmt`, `cargo fmt --check`, `git diff --check`, `python3 scripts/check_file_size_guardrails.py` (2206 files checked, 900-line limit), `python3 scripts/check_hir_maintainability_guardrails.py`, and `cargo check -p sifr_lowering -p sifr_driver -p sifr --quiet`.
- Broad non-lane probe `cargo test -p sifr -- test_e2e_pass --nocapture` -> FAIL in unrelated existing text/I/O and bytes conversion fixtures (`cpython_io_subset`, `stdlib_io_consolidated`, `open_context_manager`, `open_read`, `open_readline`, `open_write`, `bytes_conversion_errors`). This was not used as an accepted gate signal for this wave; the authoritative create-pr lane below passed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`622.31s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`105 passed`, `0 failed`, `cache_hits=23/27`, `report_signature=d08ce200366c588c`).
- Post-review docs/review-artifact rerun: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`593.82s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`105 passed`, `0 failed`, `cache_hits=11/27`, `report_signature=d08ce200366c588c`).
- Post-`origin/main` rebase rerun after top-level async child lifecycle and async owned pipe ledger merges: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`137.68s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`107 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=640c40bcdf03a864`).

M4 process handle boundary diagnostics review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-process-handle-boundaries-review-pass-1.md`: `PASS`; reviewer verified the central non-send/non-share-safe classifier hook for process-owned handles, the task/offload/channel fail fixtures, the absence of generated-runtime panic paths, and scope honesty. Follow-up notes requested staging hygiene around unrelated network/HTTP files, a populated review artifact, share-safety fixture coverage, and refreshed validation evidence.
- `reviews/ad-hoc-production-concurrency-runtime-m4-process-handle-boundaries-review-pass-2.md`: `PASS`; reviewer verified the pass-1 artifact is populated, `process_pipe_reader_shared_rejected.sifr` closes the share-safety coverage gap, all six fail fixtures emit the intended diagnostics, create-pr validation evidence is refreshed, traceability remains honest about remaining M4 work, and no new panic or file-size issue exists. The only remaining PR requirement is to keep unrelated network/HTTP files out of the staged commit.
- `reviews/ad-hoc-production-concurrency-runtime-m4-process-handle-boundaries-review-pass-3.md`: `PASS`; post-rebase reviewer verified current `origin/main` process work is preserved, `AsyncPipeReader` and `AsyncPipeWriter` are included in the process-handle classifier, all nine fail fixtures pin the intended diagnostics, traceability removes only the now-closed pipe sendability/shareability follow-up while preserving later M4 cancellation/supervision/termination/text follow-ups, and no conflict markers, panic paths, file-size issue, or out-of-scope PR files remain. Reviewer requested only a fresh post-rebase create-pr lane before merge.
- Merged as PR #2382 (`c9576ee61b38947bbfdda53c797f0659c2889dca`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; no advisories. Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`107 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=640c40bcdf03a864`).

M4 method-form async child kill/terminate implementation:

- Added method-form `AsyncChild.kill()` and `AsyncChild.terminate()` to `sifr.process`, backed by private `_sifr.process.process_async_kill` and `process_async_terminate` intrinsics returning typed awaitable `Result[None, ProcessError]`.
- Added gated generated Tokio helpers for async child lifecycle mutation. `AsyncChild.kill()` uses `tokio::process::Child::start_kill()` so the child handle remains waitable; `AsyncChild.terminate()` requests Unix SIGTERM through a Tokio child-process command, preserves the handle for later wait observation, and returns a typed unsupported `ProcessError` on non-Unix hosts.
- Kept top-level async kill/terminate helper shape, public async owned pipes, cancellation-safe observation, scoped process supervision, and async shell APIs out of scope for this wave.
- Added `process_async_child_kill_terminate` fixture coverage to create-pr and merge manifests for method-form async kill, method-form async terminate, subsequent wait status observation, and closed-handle typed errors.

M4 method-form async child kill/terminate targeted local validation:

- `cargo fmt` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_child_kill_terminate.sifr` -> PASS; validates method-form async kill/terminate, subsequent wait observation, and closed-handle typed errors.
- Emission checks for `process_async_child_kill_terminate` -> PASS; generated code includes `__sifr_process_async_kill`, `__sifr_process_async_terminate`, `start_kill`, Tokio-backed Unix `kill -TERM`, and typed non-Unix unsupported errors.
- Emission checks for `process_async_run_output` -> PASS; async run/output users that do not import `AsyncChild` do not emit async child table, spawn, wait, kill, or terminate helper state.
- `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2194 files checked.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; 425 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`262.18s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`104 passed`, `0 failed`, `cache_hits=25/27`, `report_signature=c0cb8434172d790c`).

M4 method-form async child kill/terminate review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-child-kill-terminate-review-pass-1.md`: `PASS`; reviewer verified method-form `AsyncChild.kill()` / `AsyncChild.terminate()` surface, typed awaitable private intrinsics, non-consuming `start_kill()` behavior, Unix-only async SIGTERM with typed non-Unix fallback, wait removal before await, helper gating for non-`AsyncChild` async run/output users, fixture/manifests/docs/host matrix honesty, and file-size guardrail compliance.
- Merged as PR #2372 (`b634ee26e6115158cea08740b347237ae094a86b`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`247.52s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`104 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=c0cb8434172d790c`).

M4 async process runtime split implementation:

- Split async child-process helper builders out of `process_async_runtime.rs` into `process_async_child_runtime.rs`, keeping child table, child id allocation, async spawn body assembly, async wait, async kill, and async terminate builders together.
- Preserved the existing `build_process_async_items(...)` public prelude builder contract and generated helper names, leaving process behavior unchanged.
- Reduced `process_async_runtime.rs` from 875 lines to 693 lines and kept the new child module at 236 lines, creating room for later public async pipe and cancellation work under the 900-line guardrail.

M4 async process runtime split targeted local validation:

- `cargo fmt` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_child_kill_terminate.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_output_timeout.sifr` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`289.08s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`104 passed`, `0 failed`, `cache_hits=23/27`, `report_signature=c0cb8434172d790c`).

M4 async process runtime split review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-runtime-split-review-pass-1.md`: `PASS`; reviewer verified behavior-preserving split, identical async spawn parameter order, verbatim child table and spawn/wait/kill/terminate body movement, private sibling-module visibility, no stale call sites, file-size guardrail headroom, and honest docs. Non-blocking notes covered duplicated private `string_ty()` helpers and mixed single-statement/vector builder return shapes.
- Merged as PR #2375 (`53001f2055574e8d0136acbe7d0ae308097bb1bf`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`191.15s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`104 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=c0cb8434172d790c`).

M4 top-level async child kill/terminate implementation:

- Added public `sifr.process.async_kill(child)` and `sifr.process.async_terminate(child)` wrappers that borrow `AsyncChild` and preserve the handle for subsequent `AsyncChild.wait()` observation.
- Tightened awaited plain-call lowering so non-copy awaited arguments are routed through signature-aware call adaptation instead of the leaf fast path, preserving borrowed top-level helper calls such as `async_kill(&child).await?`.
- Added `process_async_top_level_kill_terminate` to cover top-level async kill, top-level async terminate, later wait observation, and typed closed-handle errors.

M4 top-level async child kill/terminate targeted local validation:

- `cargo fmt` -> PASS.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_async_top_level_kill_terminate.sifr | rg -n "fn async_kill\\(|let _kill|let _terminate|let _again|async_kill\\(|async_terminate\\(" -C 2` -> PASS; emitted calls borrow `&killed_child` / `&terminated_child`.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_top_level_kill_terminate.sifr` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_child_kill_terminate.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr` -> PASS.
- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo fmt --check`, `git diff --check`, `python3 scripts/check_file_size_guardrails.py`, and `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- Post-rebase `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`184.34s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`106 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=dc7d767be4dbcf7c`).

M4 top-level async child kill/terminate review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-top-level-kill-terminate-review-pass-1.md`: `PASS`; reviewer verified borrowed top-level async child lifecycle helpers, generic non-copy awaited-call convention routing, method/top-level semantic parity, fixture/manifests/docs honesty, host-limited Windows wording, file-size guardrails, and no user-triggerable panic path. Non-blocking notes covered optional comments/wording only.
- Merged as PR #2378 (`a064cf3e5074ab81a61da455233369bafe340dc1`) on 2026-06-08.
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-top-level-kill-terminate-ledger-review-pass-1.md`: `PASS`; reviewer verified the merged PR #2378 link/status, merge SHA/date, ledger validation evidence, remaining-work wording, and docs-only PR readiness.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`203.29s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`106 passed`, `0 failed`, `cache_hits=27/27`, `report_signature=dc7d767be4dbcf7c`).

M4 async owned process pipes implementation:

- Added public `AsyncPipeReader` and `AsyncPipeWriter` handles, plus `AsyncChild.stdin()`, `AsyncChild.stdout()`, and `AsyncChild.stderr()` transfer methods for `async_spawn` children spawned with `Stdio("pipe")`.
- Added async pipe read/write helpers backed by private Tokio pipe handle tables. `AsyncPipeReader.read_all()` consumes the reader, `read(max_bytes)` preserves the handle across partial reads and closes on EOF, and `close()` explicitly releases a reader. `AsyncPipeWriter.write_all(...)` supports repeated async writes and `close()` removes the writer so the child observes EOF.
- Updated `async_spawn` to configure `stdin/stdout/stderr` modes through Tokio `Stdio`, while continuing to reject `Command.stdin_bytes(...)` for spawn so one-shot communicate input remains on `async_output(...)`.
- Added `process_async_spawn_pipes` coverage and retired the old `process_async_spawn_wait` assertions that expected explicit pipe modes to be deferred.

M4 async owned process pipes targeted local validation:

- `cargo fmt` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_pipes.sifr` -> PASS; validates async stdin/stdout/stderr pipe transfer, repeated async writes, explicit writer close/EOF, `read_all`, bounded `read`, EOF close, explicit reader close, one-shot extraction errors, and closed-handle typed errors.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr` -> PASS; validates adjacent async spawn/wait behavior after pipe modes became supported.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_top_level_kill_terminate.sifr` -> PASS.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/process_async_spawn_pipes.sifr | rg -n "AsyncPipeReader|AsyncPipeWriter|__sifr_process_async_child_stdout|__sifr_process_async_child_stdin|__sifr_process_async_pipe_read|__sifr_process_async_pipe_write_all|AsyncReadExt|AsyncWriteExt|Stdio::piped|read_to_end|write_all" -C 2` -> PASS; emitted Rust includes the async pipe handle types, Tokio pipe helpers, `Stdio::piped()`, and async read/write extension use.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`255.36s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`107 passed`, `0 failed`, `cache_hits=25/27`, `report_signature=640c40bcdf03a864`).

M4 async owned process pipes review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-async-owned-pipes-review-pass-1.md`: `PASS`; reviewer verified no mutex guard is held across awaits, async pipe handle ownership and typed closed-handle behavior, `async_spawn` stdio mode semantics versus one-shot async output guards, generated prelude gating/dedup wiring, fixture/manifests/docs honesty, and no new user-triggerable panic path. Non-blocking notes covered read-error versus write-error handle survival asymmetry, EOF-then-close coverage, shared async handle id diagnostics, and future large-pipe deadlock stress coverage; EOF-then-close coverage was added to `process_async_spawn_pipes`.
- Merged as PR #2381 (`a3ecf108720c73f31b7ae6c7067fd9bbdbbb82b4`) on 2026-06-08.
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-owned-pipes-ledger-review-pass-1.md`: `PASS`; reviewer verified the PR #2381 merge record, merge SHA/date, create-pr validation evidence, implementation PR list, and traceability wording around `async_spawn(...)` public async pipe I/O versus one-shot async output pipe-mode deferrals.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`212.71s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`107 passed`, `0 failed`, `cache_hits=23/27`, `report_signature=640c40bcdf03a864`).

M4 sync process terminate implementation:

- CPython scan evidence: inspected `/Users/yaseralnajjar/work/sifr/cpython/Lib/subprocess.py`, `Doc/library/subprocess.rst`, `Lib/test/test_subprocess.py`, `Lib/asyncio/subprocess.py`, and `Lib/test/test_asyncio/test_subprocess.py` for `Popen.terminate`, `Process.terminate`, `SIGTERM`, and terminate tests. CPython maps POSIX terminate to `SIGTERM`, while Windows has a distinct terminate/status behavior; this Sifr wave implements Unix SIGTERM evidence and keeps Windows host-limited.
- Added public `sifr.process.terminate(child)` and `Child.terminate()` as `@blocking_io` sync lifecycle APIs returning typed `Result[None, ProcessError]`.
- Added `_sifr.process.process_terminate` metadata and a focused child-lifecycle lowerer module so existing process lowering stays below the 900-line file-size guardrail while preserving `spawn`, `wait`, and `kill` behavior.
- Added a generated `__sifr_process_terminate` child-table helper. On Unix it requests SIGTERM for the immediate child handle and preserves the handle for later `wait`; on non-Unix it returns a typed unsupported `ProcessError` until host-specific termination/status mapping is fixture-backed.
- Added `process_child_terminate_wait`, top-level and method-form async rejection fixtures, create-pr/merge manifest entries, M4 traceability updates, and a supported-host matrix row for sync graceful terminate.

M4 sync process terminate targeted local validation:

- `python3 -m json.tool verification/validation_lanes/create_pr_e2e_manifest.json` and `python3 -m json.tool verification/validation_lanes/merge_e2e_manifest.json` -> PASS.
- `cargo fmt` and `cargo fmt --check` -> PASS.
- `git diff --check` -> PASS.
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2192 files checked, 900-line limit. The child-lifecycle lowerer split keeps `crates/sifr_codegen/src/intrinsics/registry/process.rs` at 692 lines.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_child_terminate_wait.sifr` -> PASS; top-level and method-form terminate request Unix SIGTERM and preserve wait observation.
- Expected async diagnostics `process_terminate_direct_async_rejected` and `process_child_terminate_method_direct_async_rejected` -> expected FAIL with `SIFR-ASYNC-0003`.
- Adjacent process regressions `process_child_kill_wait`, `process_spawn_wait_status`, and `process_signal_status` -> PASS.
- Emission check for `process_child_terminate_wait` -> PASS; emitted Rust includes `__sifr_process_terminate`, cfg-gated Unix/non-Unix helpers, host `kill` invocation with `-TERM`, and typed non-Unix unsupported `ProcessError`.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS; fail suite reported 425 fail tests completed.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded (`404.86s`, warm target `<=2m`) and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`102 passed`, `0 failed`, `cache_hits=23/27`, `report_signature=5e93ca9f74a9781c`).
- Post-`origin/main` rebase rerun after the async stdin-byte communicate ledger merge: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`606.36s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`102 passed`, `0 failed`, `cache_hits=26/27`, `report_signature=5e93ca9f74a9781c`).

M4 sync process terminate review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-process-terminate-review-pass-1.md`: `PASS`; reviewer verified the public top-level and method-form APIs, `_sifr.process.process_terminate` metadata, child-lifecycle lowerer split, generated cfg-gated `__sifr_process_terminate` helper, child-table handle preservation for later `wait`, typed non-Unix unsupported `ProcessError`, prelude filtering, fixtures, manifests, host matrix, traceability, and file-size guardrails. Non-blocking follow-ups remain for narrowing the child-table mutex hold around the host signal request and replacing the host `kill` command with a reviewed Rust host-signal dependency or shim in a later lifecycle hardening wave.
- Merged as PR #2367 (`3db5c05e923c2a414a18992cb919923088600bbb`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`625.99s`, warm target `<=2m`). Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`102 passed`, `0 failed`, `cache_hits=25/27`, `report_signature=5e93ca9f74a9781c`).

M4 timeout process-group cleanup targeted local validation:

- `cargo fmt --check` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2212 files checked pre-rebase and 2213 files checked post-rebase, 900-line limit.
- `git diff --check` -> PASS.
- `cargo test -p sifr_codegen lowers_process_timeout_intrinsics_via_registry` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_timeout_group_cleanup.sifr` -> PASS.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/process_async_timeout_group_cleanup.sifr` -> PASS.
- Existing timeout regressions `process_timeout_status`, `process_async_run_timeout`, `process_async_output_timeout`, and `process_async_shell_exec` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS before rebase; report `target/validation_lane_reports/create-pr.latest.json`; advisories: warm wall-time budget exceeded and warm-cache hit rate below advisory target. Included guardrails, diagnostic contracts, developer tooling, performance budgets, generated-code quality, crate tests, platform golden (`pass=6`, `skip=1`), and create-pr e2e pass suite (`112 passed`, `0 failed`, `cache_hits=22/30`, `report_signature=5a56bd55dcf7d12c`).
- Post-`origin/main` rebase rerun after the scoped process supervision merge: `cargo test -p sifr_codegen lowers_process_timeout_intrinsics_via_registry`, `process_timeout_group_cleanup`, `process_async_timeout_group_cleanup`, `process_scoped_spawn_handle`, and `process_async_output_timeout` -> PASS; `scripts/run_all_tests.sh --profile create-pr` -> PASS. Rebased report included platform golden (`pass=6`, `skip=1`) and create-pr e2e pass suite (`113 passed`, `0 failed`, `cache_hits=20/30`, `report_signature=5cbbb189c83d1068`); advisories: warm wall-time budget exceeded and warm-cache hit rate below advisory target.

M4 timeout process-group cleanup review loop:

- `reviews/ad-hoc-production-concurrency-runtime-m4-timeout-group-cleanup-review-pass-1.md`: `PASS`; reviewer verified per-spawn Unix process groups, TERM-to-KILL process-group escalation, immediate-child reaping, suppressed helper command output, descendant-leak fixtures, manifest coverage, and honest non-Unix traceability. Non-blocking notes remain for stripped Unix hosts without `kill(1)`, optional argv descendant fixture expansion, escaped sessions/nohup as future hardening boundaries, and optional comments around async pipe drops on timeout.

M0 targeted local validation:

- `python3 scripts/generate_concurrency_runtime_inventory.py` -> PASS; generated 135 CPython evidence entries from the phase source-of-truth list.
- `python3 -m json.tool verification/platform/platform_contract.json`, `verification/platform/golden/manifest.json`, and `verification/stdlib/concurrency_runtime_substrate_inventory.json` -> PASS.
- `cargo test -p sifr_stdlib bare_stdlib_tail_matches_reserved_concurrency_runtime_roots` -> PASS.
- `cargo run -q -p sifr -- check --isolated verification/platform/golden/unsupported_cpython_concurrency_imports.sifr` -> expected FAIL with `SIFR-IMPORT-0008` and `sifr.runtime`.
- `bash scripts/run_platform_golden.sh` -> PASS; 4 passed, 3 skipped, including the blocked M0a legacy-surface gate.
- `cargo test -p sifr test_e2e_fail` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; warm wall-time advisory only. During this run, expired performance waivers from 2026-06-06 were removed after `verification/performance/check_budgets.py` passed with an empty waiver set.

M0a targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo clippy --workspace -- -D warnings` -> PASS.
- `cargo test -p sifr_stdlib legacy_concurrency_runtime_modules_are_not_embedded_public_sources` -> PASS.
- `cargo test -p sifr_lowering unsupported_legacy_stdlib_module_has_import_code_and_replacement_args` -> PASS.
- `cargo test -p sifr_driver --lib` -> PASS; 140 tests.
- `cargo test -p sifr test_e2e_fail` -> PASS.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`70 passed`, `0 failed`).

M1 targeted local validation:

- `cargo fmt --check` -> PASS.
- `cargo clippy --workspace -- -D warnings` -> PASS.
- `cargo test -p sifr_lowering task_runtime_m1 -- --nocapture` -> PASS.
- `cargo test -p sifr_codegen task_select -- --nocapture` -> PASS.
- `cargo test -p sifr test_e2e_fail -- --nocapture` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/task_spawn_scoped_named_owner.sifr` -> PASS.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/task_spawn_scoped_without_owner_rejected.sifr` -> expected FAIL with `SIFR-TYPE-0002` for missing active structured task owner.
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/task_select_first_completion.sifr` -> PASS.
- `scripts/run_e2e_pass.sh --profile create-pr` -> PASS; create-pr pass suite covered 71 fixtures with 71 passed and 0 failed.
- `python3 scripts/check_hir_maintainability_guardrails.py` -> PASS.
- `python3 scripts/check_file_size_guardrails.py` -> PASS; 2114 files checked, 900-line limit.
- `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded. Included guardrails, diagnostic contracts, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`71 passed`, `0 failed`).

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
- Public APIs classified with shared terminal states and stability levels.
- `unsupported-with-diagnostic`, `waived-with-rationale`, `host-limited`, `deferred-to-phase-X`, and `rejected` surfaces.
- Sifr e2e pass/fail fixtures added.

M0 CPython scan evidence:

- CPython checkout: `/Users/yaseralnajjar/work/sifr/cpython` at `14cbd0e6afa98355bdc6749b8230fed4c9b21bd6`.
- Scanner: `scripts/generate_concurrency_runtime_inventory.py`.
- Output artifacts: `verification/stdlib/concurrency_runtime_substrate_inventory.md`, `verification/stdlib/concurrency_runtime_substrate_inventory.json`, `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md`, `verification/stdlib/concurrency_runtime_workload_database.md`, and `verification/stdlib/concurrency_runtime_m0_traceability.md`.
- Source/test/doc files scanned: 135 total entries from the exact phase source-of-truth patterns.
- Extracted signal summary: context/warnings/signal 17 files, 139 public functions, 84 classes, 328 methods, 92 constants, 294 test methods; queue/concurrency 111 files, 431 public functions, 317 classes, 1365 methods, 110 constants, 695 test methods; subprocess/process 7 files, 119 public functions, 20 classes, 315 methods, 33 constants, 291 test methods.
- Negative bare CPython import fixtures added for `asyncio`, `queue`, `subprocess`, `concurrent.futures`, `multiprocessing`, `signal`, `contextlib`, `warnings`, and `threading`.

## Required Tracking Artifacts

Create and keep current during implementation:

- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.md`
- `verification/stdlib/concurrency_runtime_substrate_inventory.json`
- `verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md`
- `verification/stdlib/concurrency_runtime_workload_database.md`
- `verification/stdlib/concurrency_runtime_m0_traceability.md`
- `verification/stdlib/concurrency_runtime_m0a_legacy_surface_traceability.md`
- `verification/stdlib/concurrency_runtime_m1_traceability.md`
- `verification/stdlib/concurrency_runtime_m2_sync_traceability.md`
- `verification/stdlib/concurrency_runtime_m3_offload_traceability.md`
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md`
- `verification/platform/supported_host_matrix.md`
- one traceability document per milestone domain under `verification/stdlib/`

## Review Ownership

- Designated compiler/runtime reviewer role: agent reviewer invoked through `agent review` for M0 implementation review; phase owner remains runtime/stdlib implementation owner. M1 cannot start until post-M0 review returns `PASS` and M0a is complete.
- Typed IPC design approval process: M6 requires a named design artifact reviewed by the phase owner and designated compiler/runtime reviewer, then recorded here before any serialization crate is selected.

## API Tier Decision Index

M0 phase-level decisions:

| Surface | Support tier | Terminal state | Rationale | CPython evidence | Sifr fixture or design artifact |
| --- | --- | --- | --- | --- | --- |
| structured runtime work model | `production-substrate` | `production-substrate` | Scopes own async, blocking, CPU, process, and future worker work units; handles are affine and outcomes are typed. | `Lib/test/test_asyncio/test_taskgroups.py`, `Lib/test/test_concurrent_futures/`, multiprocessing evidence | M0 structured-work design artifact |
| `sifr.task` | `production-public` | `production-public` | Structured task API is the recommended async model. | `Lib/test/test_asyncio/test_tasks.py`, `Lib/test/test_asyncio/test_taskgroups.py` | M1 task traceability document |
| `sifr.sync` | `production-public` | `production-public` | Channels and synchronization are the recommended queue/backpressure model. | `Lib/test/test_queue.py`, `Lib/test/test_asyncio/test_queues.py`, `Lib/test/test_asyncio/test_locks.py` | M2 sync traceability document |
| `sifr.runtime` / `sifr.parallel` | `production-public` | `production-public` | Explicit blocking and CPU offload replace executor parity as the production model. | `Lib/test/test_concurrent_futures/` | M3 offload traceability document |
| `sifr.process` | `production-public` | `production-public` | Native process supervision and owned pipes replace `subprocess` parity as the production model. | `Lib/test/test_subprocess.py`, `Lib/test/test_asyncio/test_subprocess.py` | M4 process traceability document |
| `sifr.signal` / `sifr.resource` / diagnostics / context | `production-public` | `production-public` | Structured shutdown, cleanup, diagnostics, and explicit context are production ergonomics. `ContextError` and `DiagnosticError` are owned by this milestone. | `Lib/test/test_signal.py`, `Lib/test/test_contextlib.py`, `Lib/test/test_warnings/` | M5 ergonomics traceability document |
| `sifr.ipc` | `production-substrate` | `production-substrate` | Typed IPC is the foundation for future supervised process workers. | `Lib/test/_test_multiprocessing.py`, `Lib/test/test_multiprocessing_spawn/` | M6 IPC design artifact |
| `sifr.asyncio`, `sifr.queue`, `sifr.subprocess`, `sifr.concurrent.futures`, `sifr.multiprocessing` | `rejected` / `unsupported-with-diagnostic` / `internal-only` | `rejected` / `unsupported-with-diagnostic` | CPython-shaped modules remain evidence sources only; existing implementations are removed, kept internal-test-only, or routed to diagnostics. | CPython module tests listed in phase source of truth | Negative import/removal fixtures |
| legacy CPython-shaped surface removal gate | `production-substrate` | `production-substrate` | Public legacy surfaces must be removed, hidden, or diagnosed before production runtime/process APIs are implemented. | CPython module tests listed in phase source of truth | M0A removal/diagnostic traceability document |
| Python global `warnings` filter model | `rejected` | `rejected` | Runtime diagnostics use tracing/metrics and typed Sifr diagnostics, not global Python warning filters. | `Lib/test/test_warnings/` | M5 warning-global rejection fixture |
| Rust ecosystem choices | `internal-only` | `internal-only` | Use `internal_docs/dependency_policy.md` plus the locked Rust Ecosystem Decisions table from the phase doc. Ring 2 generated-runtime core covers Tokio with `current_thread`, Tokio Util, conditional Futures Util, Tokio `sync`, Tokio process/std process, Tokio signal, and tracing. Ring 3 feature-gated substrate covers Crossbeam Channel if sync cross-thread channels remain public, std sync/OnceLock, Rayon, Rustix only after a documented std/Tokio gap with host-matrix fixtures, metrics after M5 metric schema approval, and conditional thiserror. Ring 4 typed-IPC-only covers Serde/Postcard. Reject Flume, async-channel, futures-channel, direct Parking Lot, new Once Cell, Scopeguard, production tracing-subscriber, IPC Serde JSON, Bincode, Signal Hook, Nix, direct Mio/Bytes/DashMap, runtime Anyhow/Eyre, and bespoke replacements. | N/A | Dependency policy plus phase-doc decision table plus M0 ledger verification |
| `JoinSet` drop | `production-public` | `production-public` | Live/non-empty `JoinSet` values must be consumed by `join_all()` or `cancel_all().await`; unobserved drop is a compile-time diagnostic. | `Lib/test/test_concurrent_futures/` | M3 JoinSet drop diagnostic fixture |
| Rayon pool architecture | `internal-only` | `internal-only` | Top-level `sifr.parallel` uses a lazy private default pool sized from `available_parallelism()` without configuring Rayon's global pool; configured parallelism uses explicit `Pool(config)` private Rayon pools. A first default-pool construction failure is cached as a typed runtime error for the process lifetime. There is no mutable public shutdown or reconfiguration API; process teardown releases the private default pool. | N/A | M3 pool architecture decision record |
| Existing `sifr.asyncio` veneer | `internal-only` / `unsupported-with-diagnostic` | `unsupported-with-diagnostic` | Existing veneer entry points are implementation debt; M1 does not build on or extend them, and M0 records removal, internal-test-only, or diagnostic disposition. New runtime APIs use `sifr.task`, `sifr.sync`, and `sifr.process`. | `Lib/test/test_asyncio/` | M1 veneer-free implementation fixture |
| `JoinSet` result ordering | `production-public` | `production-public` | `join_all().await` returns results in submission order, `cancel_all().await` returns cancellation evidence in submission order, and `JoinItemId` is an opaque user-side correlation token with no query API. | `Lib/test/test_concurrent_futures/` | M3 JoinSet ordering fixture |
| Shell subprocess effect | `production-substrate` | `production-substrate` | Shell subprocess usage is marked with `@shell_exec` in addition to `@blocking_io`; shell APIs require explicit shell selection and async/offload diagnostics. | `Lib/test/test_subprocess.py` | M4 shell effect fixture |

Every decision must include:

- surface
- support tier: shared terminal state plus stability level from the platform contract
- terminal state
- rationale
- CPython evidence, when applicable
- Sifr fixture or design artifact

## Waiver Index

The M0 generated inventory is the authoritative complete classification set for CPython-derived evidence and public/native boundary decisions. This hand-maintained index records milestone-level waivers and representative non-goal decisions that need explicit future revisit rules.

| Surface | Terminal state | Rationale | Revisit rule | CPython evidence | Sifr regression fixture |
| --- | --- | --- | --- | --- | --- |
| `signal.pause` | `unsupported-with-diagnostic` | Safe arbitrary signal-handler wakeup is not accepted in this phase; production shutdown uses structured signal streams instead. | Revisit only in a future safe signal-handler or structured signal-stream expansion that proves deterministic cancellation/wakeup behavior across the supported host matrix. | `Lib/test/test_signal.py`, `Doc/library/signal.rst` | M5 must add a negative diagnostic fixture for static `sifr.signal.pause` use before closure. |
| `sifr.asyncio` new APIs | `rejected` | No new `sifr.asyncio` APIs ship in this phase. | Revisit only through a new Sifr-native API design; migration compatibility is not sufficient. | `Lib/test/test_asyncio/` | M1/M2/M4 fixtures prove native APIs do not depend on the veneer. |

Every waiver must include:

- surface
- terminal state: shared platform terminal state, usually `unsupported-with-diagnostic`, `waived-with-rationale`, `host-limited`, `deferred-to-phase-X`, or `rejected`
- rationale
- revisit rule
- CPython evidence
- Sifr regression fixture
