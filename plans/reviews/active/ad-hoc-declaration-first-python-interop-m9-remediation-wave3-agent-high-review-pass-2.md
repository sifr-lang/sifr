## Verdict

**BLOCKED**

Reviewer: agent, high reasoning, fast service tier, read-only full `main...HEAD` review.

## Findings

1. **High — retained-result cancellation still has a lost-cancellation race.**
   `finish_retained_callback_finalization` checks `is_notified()` and returns without calling `release_and_resume_parent()` when false. Cancellation can arrive after that check but before the scope is dropped. The parent then reports the request as claimed and notifies the child, while dropping the claim merely clears the exact handler; it does not resume the parent fallback. The release/resume operation must run unconditionally after finalization so either ordering is handled.

2. **High — failed context entry bypasses retained-owner reconciliation and cleanup.**
   Synchronous context entry returns immediately through `mapped_try`, and async entry only poisons the object and returns. Neither path drains/releases an existing retained callback owner or attaches its typed failure evidence. Enter-failure handling must close the callback owner without incorrectly invoking a successful-context exit.

3. **High — failed receiver-lifetime asyncio registration can permanently leak its target and captures.**
   A provisional `AsyncioCallback` with an active invocation deliberately `Box::leak`s its target during drop. Receiver retention occurs only after the Python registration operation succeeds. If Python starts the callback and then registration fails or is cancelled, nothing reclaims the leaked target. Receiver-lifetime callbacks need terminal provisional rollback/join semantics comparable to retained-result groups.

4. **High — the required Rust auto-trait backstop for opaque Python identity remains absent.**
   Generated Python opaque wrappers contain only sendable fields and can auto-implement `Send`/`Sync`. Attachment-site capture analysis exists, but the independent Rust-level defense required by the original M9 review is missing.

## Four-blocker closure matrix

| Prior blocker | Status | Assessment |
|---|---|---|
| Cancellation waits for retained rollback | Open | Ordering improved, but a check/drop race can swallow parent cancellation. |
| Early later-owner failures preserve evidence | Partial | Conversion/setup/lookup/request construction are wrapped, but executable proof and context-entry failures remain open. |
| Pub/Sub proves active drain | Closed | The bridge deterministically waits for active work, releases it during close, gathers the result, and derives the Sifr marker from that result. |
| Public docs mark PYCB active | Closed | Public diagnostics docs mark `PYCB` active and only `PYZC` reserved. |

## Original M9 finding status

| Finding | Status |
|---|---|
| Capture classification and foreign identity safety | Partial — opaque-wrapper auto-trait backstop open. |
| Exact bidirectional asyncio cancellation and close | Closed for accepted entries. |
| Async unregister authority during shutdown | Closed. |
| Exception-safe, nonblocking retained rollback | Open — cancellation race and provisional receiver leak. |
| Typed retained failures on later operations/close/context | Partial — context entry and executable early-failure proof open. |
| Sync Python-primary reconciliation | Closed. |
| Escaped foreign callable capture release | Closed. |
| Authoritative examples/profiles/evidence | Partial — generated cancellation/shutdown evidence incomplete. |
| Durable activation documentation | Closed. |

## Required closure evidence

- Compiled generated-wrapper cancellation/finalization race coverage, including shutdown interaction.
- Executable sync/async early later-owner failure coverage.
- Sync/async context-entry failure coverage with retained failure evidence and capture release.
- Receiver-lifetime asyncio registration failure/cancellation coverage after a callback starts.
- Compile-fail/auto-trait proof that Python opaque wrappers cannot satisfy `Send + Sync`.
- Durable documentation of the context-entry and provisional receiver rollback semantics.

PR #2984 is not review-satisfied and must not advance to the full merge gate until these findings are closed.
