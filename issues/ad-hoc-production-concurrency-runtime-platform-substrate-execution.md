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
- M4: pending.
- M5: pending.
- M6: pending.
- M7: pending.

## Validation Evidence

Record local validation for each milestone before opening its PR.

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
