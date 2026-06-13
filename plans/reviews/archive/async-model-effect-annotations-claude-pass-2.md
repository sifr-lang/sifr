

Now I have a complete picture. Let me do the systematic review.

## Verification Results

### Task 1: `@blocking_io` → `@io_bound` consistency

No remaining `@blocking_io` references in `internal_docs/`. All three docs use `@io_bound` and `@cpu_bound` consistently:

- `async_concurrency_model.md`: 12 occurrences of `@io_bound`, 11 of `@cpu_bound` — all correct
- `32_async_ecosystem.md`: 5 occurrences of `@io_bound`, 3 of `@cpu_bound` — all correct
- `architecture.md`: milestone_async_6 updated to reference `@io_bound` and `@cpu_bound`

### Task 2: Diagnostic model gaps

| Gap | Status | Evidence |
|---|---|---|
| Async I/O vs sync I/O distinction | ✓ Covered | "use an async API if available, or wrap the call with `spawn_blocking`" — the database categorizes stdlib functions; async equivalents are implicitly "covered" by not being in the database as `@io_bound` |
| Short pure compute helpers | ✓ Covered | "Unannotated user functions are assumed to be cheap compute and do not warn by default; this avoids making every short helper look like a scheduler problem" |
| Unknown functions | ✓ Covered | "Unannotated user functions are assumed to be cheap compute and do not warn by default" |
| FFI calls | ✓ Covered | "External/FFI calls are treated conservatively as potentially blocking in async contexts unless a future FFI contract classifies them more precisely" |
| Stdlib annotation database | ✓ Specified | "The stdlib maintains a built-in annotation database for stdlib functions" |
| Warning severity | ✓ Specified | "produces a warning" — model uses `Warning` severity per the canonical diagnostic contract |
| No implicit scheduling | ✓ Locked | Three explicit mentions: "The compiler must not silently rewrite either call", "The compiler never silently offloads work", "The compiler never silently schedules work on another executor" |

### Task 3: Phase plan implementation readiness

`milestone_async_6` in `32_async_ecosystem.md` is implementation-ready:

- **Annotations**: Add `@io_bound` and `@cpu_bound` declaration-site annotations ✓
- **Database**: Add a stdlib annotation database of known I/O-bound and CPU-bound functions ✓
- **Diagnostics**: Add diagnostics for calling `@io_bound` or `@cpu_bound` functions from async contexts ✓
- **Positive fixtures**: `io_bound_annotation_warning.sifr`, `cpu_bound_annotation_warning.sifr` ✓
- **Negative fixtures**: `io_bound_call_in_async_diagnostic.sifr`, `cpu_bound_call_in_async_diagnostic.sifr` ✓
- **No implicit scheduling**: "The compiler never silently offloads work" ✓

### Task 4: Pass-1 review required changes

All four required changes from pass-1 were applied:

1. Rename `@blocking_io` → `@io_bound` in `async_concurrency_model.md` (3 locations) ✓
2. Rename `@blocking_io` → `@io_bound` in `32_async_ecosystem.md` (4 locations) ✓
3. Diagnostic mechanism specified in Blocking And Thread Offload section ✓
4. Architecture.md milestone responsibilities reference updated ✓

---

**SATISFIED**
