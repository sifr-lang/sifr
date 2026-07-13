I have enough context. Writing the round-2 review.

# Review — M7 Wave 3 Cancellation Carrier Design (Round 2)

## Round-1 blockers, closed

| Round-1 issue | Round-2 resolution |
|---|---|
| §1a Py ref drop location | §D "drops Py refs after releasing the mutex"; §A "drops removed Py refs outside the lock" ✓ |
| §1b Setup-callback failure delivery | §A "Setup catches all PyErr/runtime registration failures. It cancels any created task, resolves the terminal latch with a structured setup error, and unwinds pending accounting." ✓ |
| §1c queue failure between reserve and register | §A "A failure to queue setup also releases pending and resolves the latch." ✓ |
| §1d shutdown drain of claimed submissions | §D "queues each exact `task.cancel`, then waits on the Condvar until done callbacks have removed every submission. It stops/joins the loop only afterward." ✓ |
| §2a-b raw blocking cancel path | §D raw uses same driver/registry with internal bridge; shutdown-only user cancel is explicitly acknowledged ✓ |
| §2c GIL release on sync waiter | §B `py.detach(\|\| terminal.wait())`; no `oneshot::blocking_recv`; no Rust-blocking-under-GIL ✓ |
| §3 fallback-only invariant | §E generated `const __SIFR_COOPERATIVE_SUPERVISORS_READY: bool = false` gates claim, plus debug-panic on unexpected claimed carrier in supervisors ✓ |
| §4 timeout suppression semantics | §F decided: `cancel_and_join`→Cancelled; timeout claimed→CancelledError↦Timeout, suppression Ok/Err↦wins ✓ |
| §5 `__SifrBlockingTask` | §E "intentionally remains carrier-free forever" ✓ |
| §6 JoinSet concrete shape | §G "keeps wrapper JoinHandle AbortHandle as today and additionally extracts child carrier fallback hook"; Wave 4 replaces ✓ |
| §7 file split | §H by responsibility (types+impl, scope spawn, scope join, supervisors, cancellation emission) ✓ |
| §9 mutator drop-outside-mutex rule | §A/§D explicit ✓ |

The race-analysis for claim/publish/request (§C) is now atomic in the intended sense: the bridge's `requested` bit is latched under a mutex, `submission_id` publication is under the same mutex, and both readers snapshot-and-branch. Cancel-before-publish → setup cancels the fresh Task on the loop thread. Cancel-after-publish → `cancel_submission(id)` attaches GIL outside the bridge mutex, clones Task+loop under `ASYNC_STATE`, releases state, then `call_soon_threadsafe(task.cancel)`. Double-cancel is idempotent on `asyncio.Task`.

Terminal-latch as a single primitive replacing per-submission oneshots is cleaner than round-1 suggested and satisfies both the async and sync consumer without a `blocking_recv`-in-runtime hazard.

## Remaining gaps — nice-to-have, not blockers

1. **Done-callback error containment is under-specified.** §A says the callback "reads `task.result()` … converts it into a stored terminal outcome," but does not state that the callback wraps its entire body in a catch (Rust panic *and* Python exception). If conversion or a PyO3 downcast raises and is unhandled, the entry is not removed and shutdown hangs. The design should promise symmetry with §A setup: "the done callback catches every PyErr/panic, always produces a terminal outcome (mapping unexpected failures to a runtime-fault outcome), and always removes its registry entry and resolves the latch." Add one sentence.

2. **Off-Tokio invariant for `py.detach(|| terminal.wait())` should be explicit.** Round-1 §2c asked for either `debug_assert!(Handle::try_current().is_err())` on the sync waiter or an explicit statement that `run_coroutine_blocking` is only reachable through the `blocking_io`/`explicit_offload` classification. The design implies the latter but does not state it. A single line ("raw sync waiters run under the compiler's `blocking_io` classification, which forbids invocation from a Tokio worker; a `debug_assert!` on `Handle::try_current().is_err()` enforces this in tests") locks the invariant and forestalls a future regression.

3. **Terminal latch's stored `Waker` needs a re-poll rule.** §B says "stored Waker" but not how re-polls with a different waker are handled. Standard idiom is "overwrite unless the previous waker `will_wake` the new one." A one-liner in the design keeps future readers from re-inventing this.

4. **JoinSet field shape after wave 3 is described in prose, not shape.** §G "keeps wrapper JoinHandle AbortHandle exactly as today and additionally extracts the child carrier fallback hook" is enough to implement, but the emitted struct field ends up carrying two pieces of state whose relationship isn't stated (does the wrapper `AbortHandle` remain the primary handle, is the fallback hook a `Box<dyn Fn()>`, and how does `__sifr_cancel_all` sequence them?). Naming the concrete fields (`entries: Vec<{handle: JoinHandle, wrapper_abort: AbortHandle, child_fallback: Box<dyn Fn() + Send + Sync>}>`) would remove a review comment in the PR.

None of these change the shape or reachable states of the design; they are documentation tightening.

## Scope check

The design keeps Wave 3 tightly bounded: the const gate leaves `claim` dormant so no shipped program can produce a claimed carrier, supervisors keep their pre-carrier behavior with a debug-time trip-wire for wave-ordering slips, `__SifrBlockingTask` is out forever, JoinSet is minimally modified, and the file split is a pre-requisite refactor rather than new behavior. Wave 4/5 concerns (typed wrappers, terminal-await supervisors, activation of the timeout-suppression branch) are explicitly deferred.

The listed minimum tests cover every branch surfaced in round 1: setup failure with counter unwind, queue failure unwind, cancel-before-publication, in-flight cancel with Python `finally` before terminal, Python suppression wins, two independent claimed submissions, raw-concurrent no GIL deadlock, claimed shutdown drain, Py-ref drop outside lock, unclaimed direct-cancel unchanged, and codegen snapshots proving the wave boundary (carrier field, fallback-only supervisor, claim gate false).

## VERDICT: SATISFIED
