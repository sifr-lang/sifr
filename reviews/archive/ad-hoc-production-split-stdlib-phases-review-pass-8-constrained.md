## Pass 8 Review

---

**Finding 1 — Future.cancel: typed outcome covers only two of CPython's three false-return cases (FAIL)**

CPython returns `False` for three distinct states: future is already running, already finished (success/error), and already cancelled. The Pass 7 remediation maps to `Cancelled` vs `AlreadyRunning`, which silently conflates "already done" and "already cancelled" under `AlreadyRunning`. This is semantically wrong. A caller distinguishing retry-after-cancel from "result already available" gets incorrect information. The typed enum needs a third variant — `AlreadyDone` or equivalent covering both completed and pre-cancelled futures — to faithfully represent the `False` surface without misleading names.

---

**Finding 2 — wait(FIRST_EXCEPTION): return shape, not just first-failure mapping (FAIL)**

CPython's `wait()` always returns `(done, not_done)` — a full partition of futures at the moment the condition triggered. For `FIRST_EXCEPTION`, "done" includes every future that completed *before and including* the first exception-raising one, not only the failing future. The remediation describes mapping "exception to first future with typed worker failure Err(_)" — this addresses what to do with the failing future but says nothing about the full `done` set or the `not_done` remainder. If the implementation returns only the single failing future's result, callers that process the full done-set (standard CPython idiom) will silently lose results. The return type must be `(Vec<completed>, Vec<pending>)` or equivalent, with the failing future present inside `completed` alongside any prior successes.

---

**Finding 3 — executor.map timeout: absolute deadline vs rolling per-item deadline is unspecified (FAIL)**

CPython starts a monotonic clock at the `map()` call site; each iteration of the returned iterator checks time remaining against that single absolute deadline. The remediation says "timeout reports ExecutorError::Timeout on next item result that cannot be delivered before deadline" — "deadline" implies absolute, but the phrase "next item result" leaves open whether the implementation resets the clock between items. A per-item reset would allow arbitrarily long total execution (N items × timeout seconds), violating CPython semantics. The implementation spec must state explicitly: one absolute deadline computed once at `map()` call time, decremented across all item waits, with `ExecutorError::Timeout` on first item that exceeds the remaining budget.

---

**Finding 4 — str.encode/bytes.decode: encode-only error handlers lack decode-context diagnostic (FAIL)**

`xmlcharrefreplace` is encode-only. Passing it as the `errors` argument to `bytes.decode()` raises `LookupError` in CPython. The remediation lists it among supported handlers without distinguishing decode vs encode applicability. "Where compatible" is not strong enough — if applied statically (typed enum or statically known literal), the compiler has enough information at the call site to reject `xmlcharrefreplace` on a decode call. Without this diagnostic, the handler silently succeeds or silently falls back, both wrong. The encode-only restriction for `xmlcharrefreplace` (and the decode-only restriction for `surrogateescape` in its primary form) must be encoded in the type system or checked statically at the call site with a clear error.

---

**Finding 5 — io.StringIO: newline parameter behavior unaddressed (minor, but flag)**

The Pass 7 remediation covers only the encoding-free guarantee. CPython's `StringIO` accepts a `newline` parameter controlling universal newlines translation (default `None` normalises `\r\n` and `\r` to `\n` on read; `newline=''` disables). If `newline` is accepted but silently ignored, callers processing mixed-newline input get wrong data. This is narrower than the encoding issue but should be explicitly listed as either supported with correct semantics or diagnosed as unsupported-parameter.

---

**Verdict: FAIL**

Four actionable findings remain:
1. `Future.cancel` typed outcome missing `AlreadyDone` variant.
2. `wait(FIRST_EXCEPTION)` return shape must include full `(done, not_done)` partition.
3. `executor.map` timeout must be specified as a single absolute deadline, not per-item.
4. `xmlcharrefreplace` (encode-only) must be statically rejected on decode call sites.
