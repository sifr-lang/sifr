I've cross-checked the revised design against the round-1 findings and the current M7 substrate. All 15 concerns from round 1 have concrete answers in the revised text and hook into APIs that actually exist in the tree.

## Round-1 resolution audit

**#1 propagate() vs child-fallback contract (design §Cancellation-Unwind step 2, lines 82-96).**
Design now specifies (a) a *biased* select with the cancellation arm always polled first, and (b) that `propagate()`'s `yield_once` returns `Pending`, control returns to the select, the sticky cancellation arm wins on re-poll, and dropping the body future prevents `propagate()` from ever returning its malformed-fallback error. The "sticky" property is what makes this work — `resume_fallback_after_claim` fires the fallback hook synchronously (cancellation.rs:162) *before* `yield_once` returns Pending, so on re-poll the cancel arm is already Ready. A regression test that arms both branches together is called out. Concrete.

**#2 own-self transfer.** §Owned-Loop Enter And Exit (lines 137-158) routes exit through the existing `PythonAsyncRequest::semantic_close_method` path (async_value.rs:137-160) and lists every failure/panic/drop leg that must call `finish_semantic_close(false)`. Matches the poison edges already present at async_declaration.rs:79, 99, 112, 163, 273 plus Drop on line 163-167.

**#3 enter-conversion recovery.** Line 137-139 explicitly keeps the sole `ObjectHandle` in generated code; enter uses a *borrowed* receiver so conversion failure leaves the manager available for exit; line 188 confirms aexit still runs on this path.

**#4 cleanup carrier as task-local + shutdown drain.** Lines 97-103 make the task-local swap explicit *and* separate the M7 registry's direct-cancel-and-drain from the carrier path.

**#5 ambient carrier for async main.** Lines 117-123: root carrier wrapped around generated async `main`; explicit internal runtime error before acquisition on missing invariant; direct Tokio handles disclaimed as a Sifr surface.

**#6 diagnostic ordering under reservation.** Lines 52-56: shape/obligation error emitted *first* when invalid; reservation only after a retained valid declaration. Tests both orders from real source.

**#7 Arc-shared PythonError replay lifetime.** Lines 158-164: primary and request are Arc co-owners of one pinned triple, final owner releases. Consistent with `PythonError`/`PythonExceptionReplay`/`ForeignObject` all being Clone with `Arc<ForeignObjectInner>` (python_error.rs:7-8, 50, foreign_object.rs:11-15).

**#8 SifrBoundaryError registration.** Lines 164-165: eager registration before the owned loop starts, resolved under loop-thread GIL.

**#9 async closure outcome shape.** Lines 170-176 name the exact type `Result<Result<Option<ReturnValue>, LoopControl>, ActiveError>` with the outer `Cancelled` branch added by the select; explains outcome replay in the enclosing function after exit.

**#10 module split.** Lines 205-213: named directories and files for lowering, codegen, and runtime; sync M5 relocates to `python_context/sync.rs`.

**#11 aiosqlite live driver.** Line 240: `aiosqlite>=0.20,<1` named; local SQLite DB, offline, no external service.

**#12 native AsyncExitCause boundary.** Lines 26-28 and 198-202: native `HirAsyncWithKind::UserDefined`/`AsyncExitCause` explicitly out of M8 scope. Verified `AsyncExitCause` in types_and_errors.rs:599-644 still lacks Break/Continue — the design accepts this and keeps the Python variant off that path.

**#13 obligation discharge across every outcome.** Lines 189-195: enumerates fallthrough, return, break, continue, typed error, timeout, cancellation, runtime fault, and post-enter conversion failure; runtime `finish_semantic_close` proves the single dynamic discharge; focused per-branch tests mandated.

**#14 nested LIFO ordering.** Lines 110-116 spell out the innermost-first release order and mixed sync/async nesting.

**#15 distinct ExitDecision terminal.** Lines 145-150: `PythonTerminalValue::ExitDecision` as a distinct variant, dedicated `submit_async_context_exit` that rejects every other terminal variant.

## Remaining ambiguities (non-blocking)

- The design names a "sticky private cancellation notification" but leaves the concrete primitive open (tokio `Notify::notify_one`, `AtomicBool` + registered `Waker`, or channel). The correctness argument in step 2 requires *stickiness* (notification set before the select first re-polls the arm must make that arm Ready). This is an implementation choice; the required property is stated. Reviewers of Wave 1 codegen should verify the chosen primitive satisfies it.
- Behavior when `parent.claim(...)` returns `CancelledBeforeClaim` (outer cancelled *before* body entry) is not called out separately. The claim API (cancellation.rs:100-105) returns this error; generated code has to route it as a pre-acquisition bounded runtime failure. Not a design gap severe enough to block, but worth naming in the cancellation-scope test list.
- The runtime evidence-attachment API for "exit failure attached as secondary evidence" is implied to reuse M5's secondary-evidence recorder; the design does not explicitly name the function. M5 authorities are already listed as reusable, so this is inferable.
- Wave 1's synthetic construct-HIR tests carry all correctness weight until Wave 2 lifts the reservation. The design lists the branches to cover but does not enumerate specific test names. This is normal for a design doc but leaves reviewer coverage-checking to the Wave 1 PR.

None of these are correctness gaps or contradictions; they're implementation-time judgments that Wave 1 code review can settle.

## Bottom line

Every round-1 blocker is resolved with concrete references to APIs that already exist in `sifr_runtime`, and no new contradictions were introduced. The two-wave split, the child-carrier + biased-select handoff, the semantic-close ownership channel, the Arc-shared replay lifetime, the ambient main carrier, and the aiosqlite Wave 2 evidence are each sufficient to begin Wave 1 implementation.

VERDICT: SATISFIED
