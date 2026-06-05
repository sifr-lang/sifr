**PASS 4 — FAIL**

Four significant gaps remain.

---

**1. contextvars propagation not in M1 audit scope**

CPython asyncio tasks copy the current `ContextVar` context at spawn time; workers read/write isolated copies. The M1 audit is scoped to "control/sync" — which will catch `asyncio.Event`, `asyncio.Lock`, and scheduling order, but will not surface ContextVar propagation divergence unless a fixture explicitly exercises it. If Sifr's structured-concurrency executor doesn't replicate per-task context copies, programs using `contextvars.ContextVar` will silently observe stale values rather than fail to compile or produce a typed error. This violates the "if it compiles, it works" guarantee. **Action:** extend the M1 audit scope to include a ContextVar propagation fixture, or explicitly declare ContextVar out-of-scope with a follow-on tracking item before M1 is marked closed.

---

**2. Cross-phase unblock condition is under-specified**

Item 2 says text/i18n M1 is unblocked by "sifr.io/runtime file-object surface prerequisite PR/items." Concurrency/runtime also modifies async file handles (asyncio-based open/read paths land in M2). These are two separate phase timelines touching the same sifr.io/runtime surface. The current framing leaves text/i18n M1's unblock condition pointing at a moving target — if the runtime file-object PR ships as part of concurrency/runtime M2, text/i18n M1 cannot close until after concurrency/runtime M2, but no phase document declares this sequencing. **Action:** the prerequisite must name a concrete deliverable anchor (a specific PR, a tagged interface version, or an explicit "ships no later than concurrency/runtime M1") so the cross-phase gate is deterministic, not implicit.

---

**3. ThreadPoolExecutor exception propagation contract missing**

Item 3 declares `initializer`/`initargs` unsupported. That is a construction-time gap. The runtime gap is separate: `Future.result()` in CPython re-raises whatever exception the worker thread raised. In Sifr, worker code runs under Result/Option semantics, but an arbitrary Python-surface exception crossing a thread boundary back to the caller requires a defined wrapping: which `E` type does it become, and does it arrive as `Err(…)` or trigger a panic? Without this contract, the only safe implementation is to panic on unhandled worker errors — which violates the no-user-triggerable-panic guarantee — or to swallow them silently. Neither is acceptable, and neither is currently addressed by the sendability gap remediation. **Action:** define the thread-worker-exception-to-Result wrapping contract as a prerequisite item in this phase, even if the answer is "worker functions must return `Result`; non-Result panics are a programmer invariant violation caught at compile time."

---

**4. `open()` with `encoding=None` falls outside both fixed items' scope**

Item 2 covers binary open/file-handle conformance. Item 4 covers non-UTF-8 codec args in urllib.parse. Neither covers `open(filepath)` or `open(filepath, mode='r')` — no explicit encoding argument — which in CPython resolves to `locale.getpreferredencoding(False)`, not UTF-8. If Sifr silently substitutes UTF-8 for the absent encoding, programs that rely on locale encoding (common in system-level or legacy interop code) will compile and run but produce wrong output on non-UTF-8 systems. This is a behavioral divergence with no compile-time or runtime signal. The M0 binary I/O audit as currently scoped will not catch it because `mode='r'` is text mode, not binary. **Action:** M0's audit scope must explicitly include a `open(file)` no-encoding fixture. If Sifr mandates explicit encoding arguments, that should produce a compile-time diagnostic ("encoding argument required"), consistent with the static diagnostic pattern established for urllib.parse in item 4.

---

**Summary**

All four are actionable before the affected milestones can close: (1) extend M1 audit scope or create a ContextVar tracking item; (2) name a concrete cross-phase anchor for the sifr.io/runtime prerequisite; (3) define the thread-worker exception-to-Result contract; (4) add a default-encoding-None fixture to M0 and specify the diagnostic behavior.

**FAIL**
