**PASS 10 — FAIL**

Four remaining gaps found.

---

**Gap 1: Error handler rejection mechanism is unspecified (text/i18n)**

The table says encode-only handlers are "rejected on decode," but doesn't specify *how*. For a statically-typed language this is load-bearing: is this a compile-time type error (the encode-handler type is rejected by the decode API signature), or a runtime `Result::Err`? If runtime, which typed error variant? Without specifying the enforcement layer, implementers may produce compile-time rejection in one codec path and runtime rejection in another, creating inconsistency. Specify: encode-only handlers should be a distinct type that the decode API does not accept by signature, eliminating the need for a runtime check.

---

**Gap 2: as_completed timeout — partial iteration state undefined**

The spec defines `ExecutorError::Timeout` when the deadline expires before all futures complete, but does not describe what the caller has already received. In Python, `as_completed` with timeout yields completed futures up to the moment `TimeoutError` is raised — the caller observes a partial sequence, then gets the error. Sifr's iterator-based equivalent must specify: (a) are already-yielded items valid and owned by the caller, (b) does the iterator produce `Result::Err(ExecutorError::Timeout)` as a terminal item or terminate early, and (c) are pending futures left in a defined state. Without this, callers cannot write correct partial-result recovery logic.

---

**Gap 3: FIRST_EXCEPTION trigger condition ambiguous in typed model**

The remediation says FIRST_EXCEPTION falls back to ALL_COMPLETED if "no typed worker failure occurs." But "typed worker failure" is not defined. In Python, FIRST_EXCEPTION triggers on any worker exception. In Sifr, workers return `Result` types — does returning `Err(...)` count as a worker failure triggering FIRST_EXCEPTION, or only executor-level faults (panics, cancellations, timeouts)? If `Err` return values do *not* trigger early return, FIRST_EXCEPTION is nearly inert in normal Sifr typed-error workflows — the semantics collapse to ALL_COMPLETED in most real programs. This must be specified explicitly: either `Err` results are worker failures for FIRST_EXCEPTION purposes, or they are not (with rationale).

---

**Gap 4: shutdown(cancel_futures=True) — running vs. pending distinction absent**

Python specifies that `shutdown(cancel_futures=True)` cancels only *pending* (not-yet-started) futures; futures already executing continue to completion. The remediation routes pending futures through the cancellation path but says nothing about in-flight tasks. Omitting this distinction leaves implementers free to either interrupt running tasks or not. For a language with "if it compiles, it works" semantics, the contract for running-task behavior under shutdown must be explicit: running tasks complete normally and their results are available; only unstarted futures transition to `Cancelled`.

---

**FAIL** — four actionable gaps remain: (1) encode-only rejection enforcement layer, (2) partial iteration state at as_completed timeout, (3) FIRST_EXCEPTION trigger definition in typed model, (4) running-vs-pending distinction in shutdown cancellation.
