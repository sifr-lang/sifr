I've reviewed the PR contents against each of the concerns you listed. Here is my assessment.

## Review of PR #2955 — Seven-wave M7 plan

**Public gating** — Wave 1 explicitly prepares frontend contracts "behind the existing `SIFR-PYRES-0002` gate" and keeps `cleanup=async_close` gated. Waves 5 and 6 land typed wrappers and consuming lifecycle "behind the public gate." Wave 7 is the sole atomic activation, lifting both gates only after all substrate (owned loop, wrappers, cancellation, terminal shutdown, consuming lifecycle) is present in one production path. Matches the Phase Delivery Rule.

**Cooperative cancellation / abort fallback semantics** — Wave 3 specifies the exact carrier claim mechanic: "at Python-await entry the submission atomically claims that carrier and registers its exact-task cancellation hook"; claimed → signal exact asyncio task + await child + terminal latch; unclaimed → retain Tokio abort; registration race is closed ("aborts before Python submission or is observed by the newly registered submission, never leaving untracked Python work"). Concrete and unambiguous.

**Supervisor coverage** — Split correctly. Wave 3 covers direct task paths (`task.cancel`, cancel-and-join, timeout). Wave 4 covers scope/group fail-fast, race/select losers, and join-set through the same carrier, preserving unclaimed abort behavior. Fail-fast sibling, race/select loser, join-set, suppression, and terminal-latch ordering tests land before typed wrappers depend on them.

**Loop lifecycle** — Wave 2 conditionally wires bootstrap only when async decls / raw intrinsic are present; adds accepting/running/stopping/stopped state machine + submission-id registry; rejects post-shutdown admissions; proves initialization failure cannot leave a thread. Wave 4 defines complete phased shutdown with the M9 callback hook as a no-op ordered slot in the correct position so M9 will not reshape ordering.

**Bodyless suspension** — Wave 1 names the mechanism explicitly (option ii from round-1 §5): mark bodyless async interop declarations as `Suspends` in the existing suspension summary, bypassing `NoSuspend` without adding a variant or losing async identity.

**Async-close sequencing** — Wave 1 keeps `cleanup=async_close` gated; Wave 6 completes the full consuming runtime lifecycle (transfer-before-submission, close-once, poison-on-failure, reject reuse/duplicate/abandonment, cancellation + shutdown interaction) behind the gate; Wave 7 activates atomically with `@python.coroutine`. Runtime completion strictly precedes activation.

**Final evidence** — Wave 7 delivers the full success/failure/conversion/cancellation/suppression/shutdown and async-close matrices, the compiled httpx-style client fixture, concurrent one-loop identity proof, `demos/m7_demo` with real binary output, `CancelledError` mapping, `finally`-waiting proof and suppression semantics, plus capability activation + all doc/roadmap/review/checkbox/PR-link updates.

**Independent reviewability** — Each wave stands alone: Wave 1 is a frontend-only diff that keeps grammar gated; Wave 2 lands runtime substrate with its own raw-path identity proof; Waves 3 and 4 cover direct-path then structured-path cancellation with their own tests; Wave 5 adds typed wrappers with a two-wrapper concurrent one-loop identity assertion (round-1 §6 gap closed); Wave 6 completes async-close lifecycle; Wave 7 activates and closes evidence. Because the syntax remains gated through Waves 1–6, no wave depends on a symbol the next wave has not yet added.

Round-1 blocking findings §1/§2/§3 and refinements §4–§8 are all addressed in the diff text — the round-2 verification artifact in the PR is accurate.

No actionable defects remain.

SATISFIED
