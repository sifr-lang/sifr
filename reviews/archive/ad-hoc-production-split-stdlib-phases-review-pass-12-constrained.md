## Pass 12 Review

**Scope:** Verify Pass 11 remediations are complete and search for remaining gaps across concurrency, text/i18n, platform, and network phases.

---

### FINDING 1 — `wait()` not_done ownership: parallel gap to Pass 11 as_completed fix (CONCURRENCY)

Pass 11 corrected `as_completed` to borrow rather than consume caller Future handles. `wait(fs, timeout, return_when)` has the same structural requirement but was not explicitly addressed in the remediation.

`wait()` returns `(done, not_done)`. The caller must be able to use `not_done` as live, cancellable handles — passing them to a subsequent `wait()`, calling `.cancel()`, or submitting to another executor. If the spec allows `wait()` to consume futures from `fs` to build `done`, then `not_done` cannot be returned as live handles over the same underlying futures; they would be dangling.

The fix applied to `as_completed` must be stated equivalently for `wait()`: `wait()` borrows `fs` (or yields shared references into it), so both return sets are valid, independent views over the same future objects. Without this, the Pass 11 fix is incomplete for the full `concurrent.futures` surface.

**Action:** Add a parallel ownership clause to `wait()` in the concurrency spec, mirroring the `as_completed` borrow/clone wording.

---

### FINDING 2 — `CancelledError` has no dedicated typed variant (CONCURRENCY)

Pass 11 clarified `shutdown(cancel_futures=True)` only cancels not-yet-started futures and that already-running futures complete normally. It did not specify the typed result when the caller later calls `.result()` or `.exception()` on a future that *was* cancelled (not-yet-started, caught by shutdown).

Python distinguishes three terminal states for a future: raised an exception, was cancelled, timed out on `.result(timeout=...)`. In Sifr's Result model these need distinct variants. If "cancelled" and "raised" collapse into a generic `Err`, code that must distinguish them — e.g., retry logic that should retry on cancellation but not on a domain exception — cannot do so at the type level without downcasting.

This is a direct hole left by the shutdown observability remediation: it describes observable completion for running futures but leaves the typed terminal state for cancelled futures unspecified.

**Action:** Define a `FutureError` sum type with at least `Cancelled`, `TimedOut`, and `Panicked(E)` variants. Specify that `.result()` on a cancelled future returns `Err(FutureError::Cancelled)`.

---

### FINDING 3 — Stateful codec finalization is consuming but spec treats it as borrowed (TEXT/I18N)

Pass 11 addressed static vs dynamic codec handler diagnostics. It did not address `IncrementalEncoder`/`IncrementalDecoder` finalization semantics.

A stateful incremental decoder accumulates partial multi-byte sequences between calls. Calling `decode(chunk, final=True)` flushes accumulated state and logically exhausts the decoder. This is a consuming operation in Rust ownership terms. If the spec models `final=True` as a `&mut self` call (borrowed), a caller can invoke `decode` again on an exhausted decoder and receive either incorrect output (double-flush of empty state) or a silent no-op, both of which violate the "if it compiles, it works" guarantee.

Two conformant options exist: (a) make `decode(..., final=True)` take `self` by value, preventing post-finalization calls at compile time; or (b) track exhaustion in the type state and return `Err(CodecError::DecoderExhausted)` on subsequent calls. Neither is specified.

**Action:** Choose and document one of the two options above for `IncrementalDecoder::decode` with `final=True`. Apply the same decision to `IncrementalEncoder::encode` with `final=True`.

---

### Summary

Three actionable gaps remain:

| # | Area | Severity | Root cause |
|---|------|----------|------------|
| 1 | `wait()` ownership | High — contradicts Pass 11 fix spirit | Pass 11 remediation scoped to `as_completed` only |
| 2 | `CancelledError` variant | High — type system hole | Shutdown observability fix left terminal state untyped |
| 3 | Incremental codec finalization | Medium — correctness risk | Codec remediation addressed diagnostics, not state lifecycle |

**FAIL** — three significant gaps remain.
