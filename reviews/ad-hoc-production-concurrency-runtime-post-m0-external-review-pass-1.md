I've examined the M0/M0a artifacts, golden manifest, evidence matrix, workload database, traceability docs, dependency lock, embedded stdlib surfaces, and ran the platform golden gate locally to verify the state. Producing the gate verdict now.

## Review: post-M0 concurrency/runtime gate — PASS

M1 (`milestone_concurrency_runtime_1`) may start.

### Gate evidence

**1. M0 substrate is complete and internally consistent.**

- CPython scan recorded with checkout pin `14cbd0e6afa98355bdc6749b8230fed4c9b21bd6`, generator `scripts/generate_concurrency_runtime_inventory.py`, 135 evidence entries across all three domains (`concurrency_runtime_cpython_evidence_matrix.md`).
- Product boundary recorded in `concurrency_runtime_substrate_inventory.md:17-31` for every Sifr-native namespace owned by this phase (`sifr.task`, `sifr.sync`, `sifr.runtime`, `sifr.parallel`, `sifr.process`, `sifr.signal`, `sifr.resource`, `sifr.task.Context`, `sifr.ipc`).
- Legacy CPython-shaped disposition recorded at `:33-43` with replacement namespace and revisit rule per surface.
- M0 resolved decisions (`:47-59`) close every M1-relevant binding question: canonical mixed owner `TaskGroup[E]`, public handle `TaskHandle[T, E]`, observed-vs-unhandled child failure semantics, `RaceResult { winner_index, outcome, loser_cancellations }` and `SelectResult { branch_tag, outcome, loser_cancellations }` containers, `task.select(name=awaitable, ...)` named-branch syntax, scoped `ProcessHandle` (M4-binding), offload error binding `TaskGroup[WorkerError[E]]` aligned with `JoinSet.join_all().await -> list[Result[T, WorkerError[E]]]` (M3-binding), and lock/permit await policy (M2-binding).
- Workload database (`concurrency_runtime_workload_database.md`) covers the 15 production APIs with workload/effect classification and named fixture obligations.
- Platform contract artifacts present: `verification/platform/platform_contract.{md,json}`, `supported_host_matrix.md` rows 13–22 for all six concurrency milestones, `verification/platform/golden/manifest.json` with concurrency-owned entries.
- Rust dependency decisions locked in the phase doc Rings 2-6 table; workspace currently carries the M0-compatible baseline (`tokio = 1.52.3` with `macros, rt, time`) — the feature expansion to `sync, process, io-util, signal` and the addition of `tokio-util`, `crossbeam-channel`, `rayon`, `tracing`, etc. is the legitimate scope of M1+ implementation PRs, consistent with the M0 decision-only scope.

**2. M0a legacy CPython-shaped surface removal is merged and effective.**

- Commit `394d18930 Remove legacy concurrency runtime stdlib surfaces` is on `main` (current `HEAD`).
- `lib/sifr/` contains no `asyncio.sifr`, `concurrent.sifr`, `subprocess.sifr`, `threading.sifr`, `queue.sifr`, `multiprocessing.sifr`, `contextlib.sifr`, or `warnings.sifr`.
- `crates/sifr_stdlib/src/sources.rs` no longer embeds those names; the `legacy_concurrency_runtime_modules_are_not_embedded_public_sources` unit test in `crates/sifr_stdlib/src/lib.rs:362-387` locks this in.
- `SIFR-IMPORT-0009` registered (`registry.rs:36`, `parsing_names_and_types.rs:261-276`) and emitted via `sifr_stdlib::unsupported_legacy_stdlib_module` with replacement-namespace args mapping to `sifr.{task,process,runtime,sync,ipc,resource}`.
- Native task lowering is veneer-free: `asyncio_run_entrypoint.rs` deleted; `LowerCtx.asyncio_compat_imports`, `lower_asyncio_compat_call`, the `effective_is_async` entrypoint inference, and the `task_calls.rs` compat dispatch are all gone; the only remaining `asyncio` strings under `crates/sifr_lowering/src` are negative-coverage assertions in `name_import_diagnostics_tests.rs`.
- All nine legacy module names have negative e2e fail fixtures (`crates/sifr/tests/e2e/fail/legacy_sifr_*_removed.sifr`) and are jointly exercised by the platform golden `legacy_sifr_runtime_surfaces_removed.sifr` (now active, `blocked_until: []`).
- Local re-run of `bash scripts/run_platform_golden.sh` with M0/M0a closed: `pass=5 skip=2`; the two skips are correctly gated on `milestone_text_i18n_1` / `milestone_network_http_1` and `milestone_concurrency_runtime_4`, not on M1 entry.
- `internal_docs/async_concurrency_model.md:86` no longer describes `sifr.asyncio` as a supported veneer.

**3. Execution ledger is internally consistent.**

- Milestone checklist: M0 [x], M0a [x], M1..M7 [ ] (`execution.md:30-38`).
- Implementation PRs: M0 = #2310, M0a = #2311 recorded.
- Validation evidence recorded for both M0 (`:319-326`) and M0a (`:328-336`) with named create-pr reports, including platform golden `pass=5 skip=2` and create-pr e2e pass `70 passed, 0 failed`, advisory: warm wall-time budget only.
- Planning Reviews chain is complete and consistent (review passes 1-3 for M0/M0a all `PASS`; M0 implementation review identifies non-blocking polish only).
- Required Tracking Artifacts section lists each M0/M0a artifact and each one exists on disk.
- Pending Reviews lines 226-227 explicitly state M1 is blocked on the post-M0 external review or the recorded five-working-day fallback — this review satisfies that gate.

**4. M1 entry gates from the phase doc are all met.**

- Post-M0 external review: this review (PASS) — to be recorded in `Planning Reviews`. Five-working-day fallback procedure not needed.
- M0a complete: yes (merged in `394d18930`).
- TaskGroup-only mixed-owner decision: recorded; no distinct `task.Scope`/`runtime.Scope` introduced (`inventory.md:51`).
- Current-thread Tokio invariant: recorded in the phase doc Rust Ecosystem table and the Resolved Decisions row and called out in the execution ledger Rust-ecosystem review chain.
- Offload/parallelism split (Tokio blocking pool for blocking I/O, Rayon for CPU): recorded in the phase doc Resolved Decisions and inventory.
- Sendability/shareability ownership plan: M1 owns task-boundary captures; M2/M3/M4/M6 own subsequent extensions; representative-fixture assignments made.
- Task context API slot reservation: M1 reserves `ctx: Option[sifr.task.Context] = None` on `spawn_scoped` and `TaskGroup` constructors; M5 implements semantics without changing M1 API shape (Resolved Decisions row + phase doc M1 scope).
- Legacy veneer disposition: removed/diagnosed via SIFR-IMPORT-0009; no production API depends on legacy surfaces.

**5. No M1-blocking ambiguity remains.**

The decision register has two rows whose phase-doc cells still read "M0 is the binding gate; ... This row must be updated with the M0 outcome before M{3,4}'s first implementation PR" — the M0 outcome is recorded in `concurrency_runtime_substrate_inventory.md` resolved-decisions table (`ProcessHandle` for scoped process spawn; `WorkerError[E]` for offload error binding). These rows are M3/M4 entry concerns, not M1 entry concerns.

### Non-blocking follow-ups (not gating M1)

Tracked here so they don't get lost; none of these block opening the M1 implementation PR.

1. **Phase-doc Resolved Decisions cells for "Scoped process spawn return type" and "TaskGroup offload error binding"** (`issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:897-898`) still say "M0 is the binding gate; no pre-M0 default" with a pending-update note. The M0 outcomes (`ProcessHandle`; `TaskGroup[WorkerError[E]]`) are recorded in the M0 inventory but not yet copied into these phase-doc rows. Apply before M3's first PR (offload binding) and M4's first PR (process handle) — both are decision-only edits.

2. **Bare-import diagnostic suggestion for `warnings`** maps to `sifr.runtime` (`crates/sifr_stdlib/src/lib.rs:165`), but the phase doc replaces the Python global filter with structured tracing/diagnostics. `sifr.resource` or a dedicated `sifr.diagnostics` steering string would be more aligned. M5 owns the warnings-direction work; adjust the diagnostic suggestion text then.

3. **Dead `deferred_module_reason` arms** for `"sifr.contextlib"` and `"sifr.warnings"` in `crates/sifr_lowering/src/lower/imports.rs:38-50` are unreachable now that `report_unknown_stdlib_module` consults the legacy table first. Safe to remove in M1 prep or a small follow-up.

4. **No-op `let effective_is_async = func.is_async;`** remains in `crates/sifr_lowering/src/lower/typing_and_functions/annotations_and_function_lowering.rs:439` as a leftover from the deleted `asyncio.run` entrypoint inference. Collapse during the M1 lowering work that will touch this file.

5. **Repurposed historical fail fixtures** (`asyncio_run_requires_coroutine.sifr`, `asyncio_create_task_outside_scope_rejected.sifr`, `concurrent_future_result_type_rejected.sifr`, `process_pool_not_available.sifr`, etc.) now assert only `SIFR-IMPORT-0009`. Their original `SIFR-TYPE-0002` / `SIFR-ASYNC-0005` / `SIFR-NAME-0004` coverage was against the deleted modules; M1 needs to add equivalent type/async/name fixtures against the real `sifr.task` / `sifr.runtime` APIs as they land, and the duplicates can then be consolidated.

6. **Workspace `tokio` feature expansion** is M1's job: M1 needs `sync` (and likely `tokio-util` with `rt, io-util, time`); M3 will need `crossbeam-channel`, `rayon`; M4 will add `process`, `io-util`; M5 will add `signal`, `tracing`. Stay strictly within the Ring 2/Ring 3 locked table — no crate-family discovery.

7. **Two empty placeholder review files** sit untracked in the working tree: `reviews/ad-hoc-production-concurrency-runtime-post-m0-external-review-pass-1.md` and `reviews/ad-hoc-production-concurrency-runtime-post-m0-review-pass-1.md`. Pick one as the canonical post-M0 external review artifact, populate it with this review, reference it from `execution.md` `Planning Reviews`, and delete the other to avoid a dangling link.

8. **Inventory polish (non-blocking)**: `Lib/test/test_concurrent_futures/*` rows in the evidence matrix carry the placeholder native mapping `phase evidence`; consider updating to `sifr.runtime / sifr.parallel` for symmetry with the `Lib/concurrent/futures/*` source rows — this is documentation polish, not a contract issue.

### Result

PASS — record this review at `reviews/ad-hoc-production-concurrency-runtime-post-m0-external-review-pass-1.md` and reference it in `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` under `Planning Reviews`, then begin `milestone_concurrency_runtime_1` (Structured Async Runtime).
