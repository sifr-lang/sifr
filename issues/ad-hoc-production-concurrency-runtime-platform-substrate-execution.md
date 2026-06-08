# Ad Hoc Phase Execution: Production Concurrency, Process, And Runtime Substrate

Phase contract: [ad-hoc-production-concurrency-runtime-platform-substrate.md](./ad-hoc-production-concurrency-runtime-platform-substrate.md)

Status: active

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
- [ ] `milestone_concurrency_runtime_4`: Process Runtime
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
  - Result: accepted; this phase now requires wrapping mature Rust runtime/concurrency crates where suitable, locks accepted/rejected crate choices in the phase doc, defers any required surface that the selected ecosystem stack cannot satisfy, and forbids crate-family discovery during implementation.
- Rust ecosystem-first expansion:
  - Result: accepted; locked crate decisions cover Tokio, Tokio Util, conditional Futures Util, Crossbeam Channel, Rayon, conditional targeted Rustix, tracing, metrics, thiserror, Serde, and Postcard, all hidden behind Sifr APIs with exact version/feature plans in the phase doc. Tokio remains `current_thread`; blocking I/O parallelism uses Tokio's blocking pool and CPU parallelism uses Rayon. Flume, async-channel, futures-channel, direct Parking Lot, new Once Cell, Scopeguard, production tracing-subscriber, IPC Serde JSON, Bincode, Signal Hook, Nix, direct Mio/Bytes/DashMap, runtime/language-facing Anyhow/Eyre, and bespoke replacements are not used in this phase.
- Rust ecosystem dependency-lock Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-rust-ecosystem-decisions-review-pass-1.md`
  - Result: `FAIL`; review findings were remediated by explicitly recording the `current_thread` Tokio runtime invariant, documenting that tokio-util 0.7.18 exposes `tokio_util::sync::CancellationToken` through `rt` rather than a nonexistent `sync` feature, aligning no-public-type lists, making tracing attribute macros unavailable, and clarifying Tokio `sync` wrappers in the ledger.
- Rust ecosystem dependency-lock Claude follow-up:
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
- General dependency policy Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-dependency-policy-review-pass-1.md`
  - Result: `PASS`; no blockers found. Non-blocking polish to include `futures-util` in Ring 2 examples was applied.
- General dependency policy Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-dependency-policy-review-pass-2.md`
  - Result: `PASS`; the `futures-util` Ring 2 policy addition stayed consistent with the phase ring table and introduced no drift.
- Conditional dependency tightening review:
  - Result: accepted; `futures-util` is now conditional and added only if M1 proves `join_all`, `race`, `select`, or stream adapters would otherwise require substantial custom `Future`/`poll` code. `rustix` now requires a documented `std`/Tokio capability gap plus supported-host matrix rows and deterministic host-specific fixtures before use.
- Conditional dependency tightening Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-dependency-policy-review-pass-3.md`
  - Result: `PASS`; `futures-util` and `rustix` conditionality is consistent across dependency policy, phase table, resolved decision register, and execution ledger.
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
- Structured runtime work Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-5.md`
  - Result: `FAIL`; remaining `TaskGroup`/scope canonical owner, M1 sibling-cancellation DoD, process handle decision-register row, `sifr.subprocess` freeze wording, `select` call syntax, TaskGroup offload error binding, lock guard wording, no-public-Rust-types model wording, and `Task`/`BlockingTask` audit gaps were remediated.
- Structured runtime work Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-6.md`
  - Result: `FAIL`; remaining `cancel_scope` stable-vs-optional contradiction was remediated, and non-blocking polish for supervised process examples, `spawn_scoped` orientation, and `race`/`select` loser evidence type was applied.
- Structured runtime work Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-7.md`
  - Result: `FAIL`; remaining TaskGroup offload error binding versus `JoinSet.join_all()` wrapper alignment gap was remediated, and non-blocking polish for cancellation scope naming, process example pipe-access intent, and `JoinSet.join_all()` resolved-decision return type was applied.
- Final structured runtime work Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-8.md`
  - Result: `PASS`; TaskGroup offload error binding and `JoinSet.join_all()` wrapper alignment were verified, with only non-blocking wording/ledger polish applied.
- Final blocker-only Claude verification:
  - `reviews/ad-hoc-production-concurrency-runtime-structured-work-review-pass-9.md`
  - Result: `PASS`; no material blockers, contradictions, stale state vocabulary, missing binding decisions, or ambiguous contracts remained.
- No-subprocess-compatibility Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-no-subprocess-compat-review-pass-1.md`
  - Result: `PASS`; docs were clean under the no-backward-compatibility, no-CPython-adapter, `sifr.process`-only decision, with only non-blocking wording/waiver-index polish applied.
- Final no-subprocess-compatibility Claude verification:
  - `reviews/ad-hoc-production-concurrency-runtime-no-subprocess-compat-review-pass-2.md`
  - Result: `PASS`; no backward-compatibility or CPython-shaped adapter commitment remained, and `sifr.subprocess` was verified as legacy implementation debt to remove, keep internal-test-only, or route to unsupported diagnostics.
- M0 implementation Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-m0-implementation-review-pass-1.md`
  - Result: `PASS`; CPython scan, inventory, evidence matrix, workload database, platform contract, host matrix, golden manifest entries, native namespace diagnostics, and M0/M0a gates met M0 requirements. Non-blocking polish for `sifr.contextlib`/`sifr.warnings` disposition and warnings diagnostic steering was applied.
- M0a legacy-surface Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-1.md`
  - Result: `FAIL`; local validation recording, duplicate legacy-import fail fixtures, empty review artifact, and dead `sifr.asyncio` veneer lowering blockers were remediated.
- M0a legacy-surface Claude follow-up:
  - `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-2.md`
  - Result: `PASS`; public legacy modules were verified unreachable, `SIFR-IMPORT-0009` replacement diagnostics were verified, native task lowering was verified free of `sifr.asyncio` compatibility paths, demos/manifests/goldens were clean, validation evidence was recorded, and no blocker remained.
- M0a final legacy-surface Claude confirmation:
  - `reviews/ad-hoc-production-concurrency-runtime-m0a-legacy-surface-review-pass-3.md`
  - Result: `PASS`; pass-1 blockers remained remediated in the current working tree, create-pr validation artifacts were verified with `70 passed`, `0 failed` e2e pass coverage and platform golden `pass=5`, `skip=2`, and the implementation was confirmed ready for the M0a PR.
- Post-M0 external review gate:
  - `reviews/ad-hoc-production-concurrency-runtime-post-m0-external-review-pass-1.md`
  - Result: `PASS`; M0 substrate inventory, CPython scan evidence, workload database, platform contract, dependency decisions, M0a legacy surface removal, validation evidence, and M1 entry gates were verified. M1 may start.
- M1 structured-async implementation Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-1.md`
  - Result: `PASS`; M1 structured task APIs, reserved `ctx` slots, named `select`, and shared spawn enforcement were verified. Non-blocking polish for arbitrary select-branch signature wording, `async_with.rs` decomposition, and sequential same-name task-owner cleanup was applied.
- M1 structured-async implementation Claude review:
  - `reviews/ad-hoc-production-concurrency-runtime-m1-structured-async-review-pass-2.md`
  - Result: `PASS`; `TaskGroup(ctx=None)`, `task.spawn_scoped(..., ctx=None)`, named-branch `task.select(first=..., second=...)`, existing task-boundary enforcement, traceability, manifests, and create-pr validation evidence were verified. Non-blocking demo/select and `spawn_scoped` model-doc polish was applied.
- M1 final post-polish Claude follow-up:
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
- M4: in progress.
- M5: pending.
- M6: pending.
- M7: pending.

## Validation Evidence

Record local validation for each milestone before opening its PR.

M4 async wait cancellation-safe observation merge ledger:

- Merged as PR #2386 (`d54d2c11497e54ca5db3061d8e026ee2afb09154`) on 2026-06-08.
- Merge-ledger validation: `scripts/run_all_tests.sh --profile create-pr` -> PASS; report `target/validation_lane_reports/create-pr.latest.json`; advisory: warm wall-time budget exceeded (`134.60s`, warm target `<=2m`). Included guardrails, diagnostic contracts, frontend/syntax guardrails, developer tooling, performance budgets, verification hardening, generated-code quality, crate tests, platform golden (`pass=5`, `skip=2`), and create-pr e2e pass suite (`108 passed`, `0 failed`, `cache_hits=26/27`, `report_signature=df97adcd1a958b0c`).

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

- Designated compiler/runtime reviewer role: Claude Opus reviewer invoked through `.cursor/skills/talk-to-claude-opus` for M0 implementation review; phase owner remains runtime/stdlib implementation owner. M1 cannot start until post-M0 review returns `PASS` and M0a is complete.
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
