**REVIEW PASS 11 — FAIL**

Three significant gaps remain.

---

**Gap 1: as_completed handle ownership is under-specified**

The remediation states pending futures remain live/cancellable through "original handles." This assumes callers retain separate handle references after passing futures into as_completed. The spec does not address what happens when futures are moved (consumed) into the iterator — a common pattern where the caller holds no independent handle. If ownership transfers to the iterator on construction, the "original handle" no longer exists and the cancellation guarantee is unreachable by the caller.

*Required resolution:* Specify the ownership model explicitly. Either (a) as_completed borrows futures, not moves them, so original handles remain valid; or (b) as_completed returns cancellation tokens alongside each yielded result, allowing post-timeout cancellation of pending items. The current text implies (a) but does not enforce it.

---

**Gap 2: FIRST_EXCEPTION not_done futures have no defined lifecycle**

The remediation specifies the return shape (done, not_done partition) and trigger conditions, but is silent on what happens to futures in not_done after the call returns. Two conflicting behaviors are possible:

- Not_done futures continue executing in the background (implicit background executor state, potential resource leak if caller ignores them).
- Not_done futures are suspended/cancelled when FIRST_EXCEPTION returns (but then they are effectively already done, making the partition label misleading).

This matters especially for the interaction with shutdown(cancel_futures=True): if not_done futures are still executing after FIRST_EXCEPTION, calling shutdown(cancel_futures=True) should cancel only the not-yet-started subset of not_done — but the remediation for shutdown does not account for futures already partitioned into not_done by FIRST_EXCEPTION.

*Required resolution:* State whether not_done futures remain scheduled on the same executor and, if so, whether subsequent shutdown calls act on them. Define who is responsible for driving or cancelling them.

---

**Gap 3: shutdown(cancel_futures=True) result observability is contradicted by executor teardown**

The remediation states that already-running futures "continue to completion and results remain observable." Observable by whom, and through what mechanism? If shutdown is blocking (waits for running futures to finish before returning), observability is implicit — results are stored in Future state before shutdown returns. But if shutdown is non-blocking or has a timeout, the executor may tear down its internal dispatch machinery before running futures complete, making results unobservable even though the futures nominally finish.

The spec does not define whether shutdown blocks until running futures complete. Without this, "results remain observable" is either a tautology (if blocking) or unimplementable (if non-blocking with teardown).

*Required resolution:* Specify shutdown's blocking semantics. If blocking: results are observable through normal Future polling/await after shutdown returns. If non-blocking: define the window during which results remain accessible and what happens if the caller polls after that window.

---

**Minor note (not a FAIL criterion, but worth tracking):**

The error handler dual-path — compile-time error for invalid static literals, runtime CodecError::InvalidErrorHandlerForOperation for invalid dynamic names — is architecturally coherent but creates a user-facing asymmetry. Code that is structurally identical except for how the handler name is spelled (static vs. dynamic) gets different error timing. This should be explicitly documented as intentional, not left as an emergent property of the implementation strategy, otherwise future reviewers may attempt to "fix" it.

---

**Summary**

| # | Gap | Severity |
|---|-----|----------|
| 1 | as_completed handle ownership undefined — cancellation guarantee may be unreachable | Significant |
| 2 | FIRST_EXCEPTION not_done futures have no lifecycle after return; interacts poorly with shutdown | Significant |
| 3 | shutdown(cancel_futures=True) observability contradicted without blocking semantics defined | Significant |

**FAIL** — resolve gaps 1–3 before Pass 12.
