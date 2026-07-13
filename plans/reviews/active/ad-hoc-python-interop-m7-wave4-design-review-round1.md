I have enough context to review. The Wave 3 code, review artifacts, and protocol architecture are consistent with the design proposal. Writing the review now.

# Design Review — M7 Wave 4 (cancellation-aware supervisors + ordered shutdown)

## Race completeness / claim-lease lifetime — accepted

- **A** correctly preserves the atomicity established in Wave 3 (`crates/sifr_runtime/src/cancellation.rs:76-111`): `requested` is still permanently latched, and `claim` under the same mutex still yields exactly one of `Claimed / CancelledBeforeClaim / AlreadyClaimed`. Trading the permanent `exact` slot for a generation-tagged RAII lease is the minimum change that unblocks sequential Python awaits per Sifr task, without opening a new race — a lease drop is a *removal* under the carrier mutex, not a hook fire, so the "fire outside locks" invariant is unaffected.
- The stated ordering — lease dropped **before** waking async/blocking waiters in `PythonTerminal::complete` — is required for the sequential-await case: a woken waiter that reads the outcome may immediately begin the next await and `claim` again; the incumbent lease must have released the exact slot first. Design states this explicitly.
- Stale/idempotent release is well-defined: generation mismatch = no-op. `request_cancel` never removes the exact hook, so lease Drop is the sole owner of hook removal. No cycles: hooks (`SubmissionCancellationBridge::request`) don't hold Arcs back to the carrier.
- Sequential-claim + race + 128-round tests are appropriate.

## Pending / active handoff — accepted

- **B** converts pending from a `usize` count to a `BTreeMap<id, PythonTerminal>` (paralleling the existing `submissions` map in `crates/sifr_runtime/src/python/async_runtime.rs:37-45`). The atomic pending→active move is safe because both are already under `ASYNC_STATE`; ordering (insert-pending before `call_soon_threadsafe`, remove-pending on any of the current unwind sites at `async_runtime.rs:202-233`, atomic move inside `register_submission`) is symmetrical with the Wave 3 code.
- Terminal drain on runtime-loop failure is *the* real hole this wave closes: today, `cancel_registered_submissions()?` at `async_runtime.rs:362` bubbles through `shutdown()` while leaving pending terminals unfulfilled, so any thread blocked on `terminal.wait()` hangs indefinitely. Draining both maps and completing every terminal (releasing leases, dropping Py refs after leaving `ASYNC_STATE`) fixes it without new invariants.

## Supervisor result arbitration — accepted

- **D**/**E**/**F** are internally consistent: fallback children (process observer with `stop_on_fail_fast`, top-level `__SifrBlockingTask`) stay carrier-free; ordinary async and scope-offload (both `tokio::spawn` and scope `spawn_blocking` observers) store the wrapper carrier and go through `request_cancel`. `__sifr_join_all` (`preamble/task_runtime.rs:632-728`) fail-fast collects carriers instead of abort handles, but the drain loop keeps awaiting every observer — the *shape* of the observer join set is unchanged, only the trigger.
- For unclaimed children, the wrapper's fallback binding (`preamble/task_cancellation_runtime.rs:22-27`) makes `request_cancel` semantically identical to today's `.abort()`; for claimed children, it waits Python cleanup. That's the intended difference and doesn't perturb gather/race/select's terminal arbitration (which is receiver-driven, not abort-driven).
- gather requesting all carriers once after the *first* Err/Cancelled (not per-loser) matches current semantics. race requesting after the first winner and select requesting only the loser are the minimum-work forms.

## JoinSet blocking distinctions — accepted

- **F** cleanly splits `__SifrJoinEntry` into `{cancellation, blocking_abort}` (matching the round-2 design comment about "concrete fields"). `add(__SifrTask)` storing the whole carrier — never the extracted abort handle — removes the escape hatch Wave 3 kept behind `abort_handle()` (`task_cancellation_runtime.rs:46-48`).
- Suppression mapping: a claimed carrier whose Python suppressed cancellation yields an Ok/Err terminal → cancel_all sees Ok(TaskResult::Ok/Err) → AlreadyCompleted/AlreadyFailed. That's consistent with `__sifr_timeout`'s Wave-3 suppression branch (`task_runtime.rs:249`).

## Shutdown error semantics — accepted with one gap

- **G** phase ordering (admissions off → callback hook → async cleanup hook → cancel+drain → loop stop → thread join → epilogue) is the shape stipulated in the phase description at `plans/issues/active/ad-hoc-declaration-first-python-interop.md:552-556`, with M9/async-close as documented no-op slots. First-error-priority accumulation that still runs every phase is the correct fix for the current `.and(stop_result, join_result)` at `async_runtime.rs:394`.
- **Nice-to-have gap (not a blocker):** the design says "if exact cancel cannot be queued **or loop failure is observed**, drain pending/active into failure terminals," but doesn't spell out the detection mechanism inside the current `while state.pending_submissions > 0 { state = wait_for_change(state)?; }` / `while !state.submissions.is_empty() { ... }` loops. Implementation must inspect `AsyncLifecycle::Failed` on every wake and break/drain — otherwise a loop-thread panic mid-shutdown produces exactly the hang B is designed to prevent. Worth one sentence in the design: "each wait re-inspects lifecycle; observing Failed short-circuits to drain."

## Compositional testing before Wave 5 — accepted

- With the seam function present but no user-code claimant reachable (typed wrappers gated), end-to-end claimed supervisor behavior is provable via (1) Wave 3's already-landed exact-task terminal ordering + suppression tests, (2) Wave 4's new source-shape pins on `request_cancel`/carrier storage/`__sifr_current_task_cancellation`, and (3) claim-lease unit tests. The wrapper's `request_cancel` is a pure delegation to the inner carrier — nothing in a supervisor context changes its semantics — so composition holds barring gluing bugs, which the source-shape tests catch.
- Wave 4 does not need a synthetic claimant test; Wave 5 supplies the real one when the typed wrapper becomes reachable through the public gate.

## Scope discipline — accepted

- Changes touch cancellation.rs (lease), async_runtime.rs (map + drain), async_terminal.rs (lease-owning `PythonTerminal`), python.rs (guard orchestrator), and the three preamble modules that already own carriers/supervisors/join-set. No frontend, no typed wrapper, no `async_close`, no gate lift — matches the phase's explicit deferrals.
- Task-runtime.rs is 795 lines today (`wc -l`); the incremental fields don't push it past 900. The optional preamble split is a safety valve, not a promise.

## Minor observations (not blockers)

1. Behavior narrowing worth flagging in the PR: with no raw `AbortHandle::abort()` on async JoinSet entries, the "cancelled during a live await" case no longer produces `JoinError::Cancelled`, so the `AlreadyStarted` heuristic in `join_set_runtime.rs:319` narrows to blocking entries only. Either narrow the enum's docstring accordingly or accept the shift explicitly.
2. Lease Drop should tolerate a poisoned carrier mutex (best-effort clear), same discipline `PythonTerminal::lock_state` already uses (`async_terminal.rs:72-77`).
3. Wave 3 round-1 note that `run_coroutine_blocking` should carry `debug_assert!(Handle::try_current().is_err())` is still open; Wave 4 doesn't have to close it but this is a natural time.

## Verdict

Race completeness, lease lifetime, wake ordering, pending/active handoff, failure drain, supervisor arbitration, and JoinSet distinctions are all coherent and safe. Compositional testing is defensible given the gated Wave 5. The single design tightening worth calling out (loop-failure detection inside shutdown waits) is documentation-level, not a shape or state-machine defect.

VERDICT: SATISFIED
