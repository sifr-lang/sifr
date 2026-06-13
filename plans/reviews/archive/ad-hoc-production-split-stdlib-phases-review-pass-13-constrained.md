**FAIL**

Four significant gaps remain after pass 12 remediation.

---

**Gap 1: `gather()` ownership and `return_exceptions` typing**

The wait ownership fix (pass 12) established that observation handles must be borrowed, not consumed. `gather()` has the same unresolved ownership question — does it borrow or consume its future arguments? The answer must be consistent with wait: borrow, so callers retain cancel handles.

More critically, `gather(return_exceptions=True)` cannot be typed as `List[T]` in a sound system. Each element may be a success or a per-task error. The return type must be `List[Result[T, FutureError[E]]]` to express per-element failure. Without this, callers distinguish results from errors through runtime checks, defeating Sifr's type guarantees. The current spec is silent on gather's element-level result type.

---

**Gap 2: `as_completed()` ownership and timeout signal**

`as_completed(fs, timeout=None)` yields futures in completion order. Ownership is unspecified: if it consumes the input futures, callers lose cancel handles — violating the pattern established for wait. It must borrow.

The timeout interaction is also unresolved. When the overall timeout fires mid-iteration, Python raises `TimeoutError`. Sifr must integrate this with `FutureError::TimedOut` from pass 12, but the spec does not define what the iterator yields or signals at timeout boundary — whether it terminates the iterator, returns a typed sentinel, or surfaces `TimedOut` inline.

---

**Gap 3: Codec mid-stream recoverable error with partial output**

Pass 12 addressed finalization/exhaustion. The unaddressed case: `IncrementalDecoder.decode(data, final=False)` with `errors='replace'` or `errors='ignore'` encountering a malformed sequence mid-buffer. Python continues decoding after the replacement/skip and returns the partial valid output alongside continuation state.

`Result<String, DecodeError>` cannot express this: an `Err` path loses the partial output; an `Ok` path loses the error signal. The spec needs a wrapper type — something like `(String, Option<DecodeWarning>)` — to express partial success with a recoverable diagnostic. This is distinct from the hard-stop `Err` path for `errors='strict'`. Without it, implementations will silently either discard partial output or swallow the error signal.

---

**Gap 4: `TaskGroup` aggregated error type**

Python 3.11 `asyncio.TaskGroup` collects errors from all child tasks and raises `ExceptionGroup` on context manager exit. If Sifr targets modern Python parity and includes `TaskGroup`, the exit error type cannot reuse single-future `FutureError<E>` — multiple concurrent child failures must be aggregated, e.g. `TaskGroupError<E>(Vec<FutureError<E>>)`.

The current spec is silent on this. If `TaskGroup` is deferred to a later phase, the spec must say so explicitly and note that `FutureError<E>` is insufficient for the aggregated case, to prevent implementors from silently misapplying the single-task error type.

---

**Summary**

| # | Area | Nature |
|---|------|--------|
| 1 | `gather()` | Ownership unspecified; `return_exceptions` element type missing |
| 2 | `as_completed()` | Ownership unspecified; timeout/`FutureError` integration missing |
| 3 | Codec mid-stream | Partial output + recoverable error not expressible in `Result` |
| 4 | `TaskGroup` | Aggregated error type absent or undeferral noted |
