**FAIL** — four significant gaps remain.

---

**1. `backslashreplace` misclassified as encode-only**

Pass 8 statically rejects encode-only handlers on decode call sites, using `xmlcharrefreplace` as the canonical example. However, `backslashreplace` is bidirectional since Python 3.1: it is valid on both encode and decode. If the implementation classifies it in the encode-only rejection set, any `bytes.decode(errors='backslashreplace')` call site would be incorrectly diagnosed as a compile error. Similarly, `surrogatepass` is valid for encode and decode of UTF-8/UTF-16/UTF-32 and must not appear in the encode-only bucket. The classification table needs an explicit tripartite split: encode-only, decode-only, and bidirectional, rather than a binary encode-only vs everything-else scheme.

---

**2. `as_completed()` timeout model inconsistent with fixed `executor.map`**

Pass 8 established that `executor.map` computes one absolute monotonic deadline at call time. `concurrent.futures.as_completed()` is a separate code path with its own `timeout` parameter. If `as_completed()` was not updated in the same pass, it likely still measures `timeout` as a per-call-to-`next()` budget rather than an absolute deadline. A caller doing `for f in as_completed(fs, timeout=5)` expects 5 seconds total, not 5 seconds per yield. The fix must apply the same single-deadline model: deadline computed at `as_completed()` call time, remaining budget decremented on each yield.

---

**3. `wait(FIRST_EXCEPTION)` no-exception fallback unspecified**

The pass 8 description covers the exception path (first typed worker failure, full (done, not_done) partition) but omits the no-exception case. In CPython, `wait(FIRST_EXCEPTION)` behaves identically to `ALL_COMPLETED` when no future raises. If this fallback is absent, a call where all futures succeed will either return prematurely (before all complete) or block indefinitely. The specification must explicitly state: if no future raises, block until all futures complete and return `(all, empty_set)`.

---

**4. `shutdown(cancel_futures=True)` typed outcome mismatch**

Pass 8 gave `Future.cancel()` typed outcomes: `Cancelled`, `AlreadyRunning`, `AlreadyDone`. `executor.shutdown(cancel_futures=True)`, introduced in Python 3.9, triggers cancellation of pending futures internally without going through the public `cancel()` API. If shutdown-triggered cancellation does not produce the `Cancelled` variant in each affected future's outcome, match arms written against the typed outcome enum will silently fail to handle shutdown-cancelled futures. The implementation must route shutdown cancellation through the same typed outcome path, or explicitly document that shutdown produces a separate `ShutdownCancelled` variant that callers must also handle.
