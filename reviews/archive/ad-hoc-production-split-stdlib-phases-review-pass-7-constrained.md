**PASS 7 — REVIEW**

---

**GAP 1: `executor.map(timeout)` not addressed alongside `Future.result(timeout)`**

Pass 6 fixed `Future.result(timeout)` to yield a typed `ExecutorError::Timeout`. But `executor.map` also accepts a `timeout` parameter (`executor.map(fn, *iterables, timeout=None)`). The timeout on `map` raises `TimeoutError` in CPython when iterating past the deadline. If the phases fixed the return type of `map` but did not address its timeout parameter's typed terminal state, there is a hole: the typed item-result iterator needs to express timeout per-item or as an iterator-level exhaustion state consistent with `ExecutorError::Timeout`.

---

**GAP 2: `Future.cancel()` on already-running futures — return value semantics untyped**

Python's `Future.cancel()` returns `False` if the future is already executing (cannot be cancelled) and `True` if it was pending and successfully cancelled. Pass 6 addressed `shutdown(cancel_futures=True)` giving pending futures a cancelled terminal state, but the interactive `Future.cancel()` call involves a two-outcome typed return. If the design does not distinguish `CancelResult::AlreadyRunning(False)` from `CancelResult::Cancelled(True)`, callers have no typed signal to branch on, and the state machine is incomplete.

---

**GAP 3: `as_completed(timeout)` typed exhaustion**

Pass 6 fixed `wait`/`as_completed` to require homogeneous collections, but `as_completed(fs, timeout=None)` raises `TimeoutError` in CPython when the timeout expires before all futures complete. In Sifr's non-raising model this needs a typed exhaustion signal — either the iterator yields `Result<T, ExecutorError>` where `ExecutorError::Timeout` signals end-of-iteration, or the iterator terminates and the caller checks remaining futures. If this is unspecified, iteration over `as_completed` with a timeout has no typed contract.

---

**GAP 4: `wait(return_when=FIRST_EXCEPTION)` semantics in a non-raising world**

`concurrent.futures.wait` accepts `return_when` values: `ALL_COMPLETED`, `FIRST_COMPLETED`, `FIRST_EXCEPTION`. The `FIRST_EXCEPTION` variant in CPython terminates when any future raises. In Sifr, worker failures are typed (previously fixed), so a future carrying a typed error value is not an "exception" but a `Result::Err`. The design needs to specify whether `FIRST_EXCEPTION` maps to "first future whose result is `Err(_)`" or is unsupported. If unaddressed, `return_when` is either silently ignored or semantically mismatched.

---

**GAP 5: `io.StringIO` not separated from `TextIOWrapper` encoding requirement**

Pass 6 applied an explicit encoding requirement to `io.TextIOWrapper`. But `io.StringIO` is an in-memory text stream that takes no `encoding` argument — it operates on native unicode strings already. If the encoding requirement was applied broadly to the text-IO surface without carving out `StringIO`, this is a contradiction: requiring encoding on `StringIO` is wrong, but omitting the carve-out leaves the spec ambiguous. The phases need an explicit statement that `StringIO` is encoding-free (or unsupported with separate rationale).

---

**GAP 6: `threading.local()` — distinct from `contextvars`, no explicit ruling**

`contextvars` was marked entirely unsupported. However, `threading.local()` is a separate mechanism (class-based, not context-variable-based) for thread-local storage. These are often used interchangeably in Python code. If `threading.local()` has no explicit position in the concurrency phase — supported with typed contract, or unsupported with diagnostic — there is a missing ruling. Leaving it implicit while explicitly ruling out `contextvars` creates an asymmetry that will surface in user-facing error messages.

---

**GAP 7: `bytes.decode(errors=)` / `str.encode(errors=)` error-handler typing**

The text/i18n phase addressed explicit encoding for `open()` and `TextIOWrapper`. But `str.encode(encoding, errors)` and `bytes.decode(encoding, errors)` take an `errors` parameter with runtime-dispatch values (`'strict'`, `'ignore'`, `'replace'`, `'surrogateescape'`, etc.). Static compilation cannot express these as a runtime string. The phases need either: a typed `EncodeErrors` enum restricting to statically-known variants, or a diagnostic blocking non-`'strict'` error handlers with CPython evidence. No ruling here leaves the i18n phase incomplete for the encode/decode surface.

---

**FAIL** — 7 actionable gaps remain across the executor, future state machine, io, and text surfaces.
