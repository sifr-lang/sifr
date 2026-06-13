**PASS 5 — Remaining Gaps Review**

---

**Gap 1: ContextVar single-task usage scope is unresolved**

Remediation 1 marks cross-task ContextVar *propagation* out of scope. It does not answer whether `contextvars.ContextVar` as a type is supported within a single task (reads and writes that never cross an executor or asyncio boundary). If ContextVar is entirely unsupported, any usage needs a diagnostic. If only propagation is out of scope, single-task reads/writes need a spec — behavior, type, and lifetime. The current remediation addresses only the propagation diagnostic fixture and leaves this prior question open. Implementers will guess.

---

**Gap 2: ThreadPoolExecutor future cancellation is untyped**

Remediation 3 defines two terminal states: `Ok(T)` for success and `Err(ExecutorError::Worker(E))` for worker failure. Python futures have a third distinct terminal state: cancelled. `Future.cancel()` is part of the standard API. The remediation makes no mention of cancellation — no `ExecutorError::Cancelled` variant, no "out of scope" marking, no diagnostic. If cancellation is silently omitted, callers matching on ExecutorError are working with an incomplete set of variants, which undermines the "never panics or swallows" guarantee. This needs either a named variant or an explicit out-of-scope marker with a compile-time diagnostic for `.cancel()` calls.

---

**Gap 3: M3 encoding policy outcome is ambiguous and creates a contradiction risk**

Remediation 4 defers the question of `open(path)` without encoding to M3 locale/default-encoding policy. But the remediation does not commit to what M3 actually delivers. Two outcomes are possible after M3 lands:

- Option A: `open()` without encoding becomes legal and uses the locale default.
- Option B: explicit encoding remains always required, M3 only defines what error message is emitted.

Option A conflicts with Sifr's static typing model — locale-dependent encoding means the type of a file handle's character codec is not knowable at compile time. Option B makes M3's relevance to this remediation unclear. The phase plan needs to state which option M3 commits to, or the current text will be interpreted differently by the text/i18n and runtime teams.

---

**Gap 4: Dynamically-specified binary mode collides with encoding enforcement**

Remediations 2 and 4 interact poorly. Remediation 2 anchors binary file I/O (`mode="rb"`, `mode="wb"`) to sifr.io/runtime — binary handles don't need encoding. Remediation 4 says `open(path, mode=dynamic_var)` without encoding returns a typed unsupported-default-encoding error when the mode is not statically knowable.

At runtime, `dynamic_var` might be `"rb"`. In that case the encoding error is a false positive — binary mode does not require an encoding argument. The current text has no exclusion for dynamically-specified binary modes from the encoding-required diagnostic path. The boundary condition must be stated: either dynamic binary mode is inferred and excluded from the encoding check, or `open()` with a non-literal mode string requires encoding unconditionally (which would make binary-mode calls with a dynamic mode string permanently illegal, a significant usability restriction that should be explicit).

---

**Gap 5: "Where possible" in worker compile-time error coverage is unactionable**

Remediation 3 says non-Result user failure paths in worker functions are "compile-time errors where possible." The qualifier "where possible" is undefined. Implementers need to know: possible under what conditions? What determines whether a worker's failure path is statically analyzable? Without a bounded definition — at minimum a list of cases where static detection is guaranteed and cases where it is not — this becomes an open-ended implementation decision rather than a spec commitment. The typed ExecutorError fallback has no clear triggering conditions, so the "where possible / typed ExecutorError otherwise" split will be resolved inconsistently across compiler passes.

---

**FAIL**

Five actionable gaps remain. The highest-risk items are Gap 3 (M3 policy ambiguity creates a downstream contradiction) and Gap 4 (false-positive encoding diagnostics on dynamic binary-mode opens). Gaps 1 and 2 are scope definition omissions that will produce inconsistent behavior. Gap 5 is an implementation guidance deficit that will surface as spec disputes during code review.
