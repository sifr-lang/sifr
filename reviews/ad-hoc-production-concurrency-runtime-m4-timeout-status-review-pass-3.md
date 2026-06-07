Verified the pass-3 hardening directly against generated code.

**Overflow-safe deadline chain (verified in emit output):**
```rust
let __deadline = std::time::Instant::now()
    .checked_add(std::time::Duration::try_from_secs_f64(__timeout_seconds).map_err(...)?)
    .ok_or_else(|| ProcessError { message: "process timeout is too large for this host clock" })?;
```

All three user-data-dependent panic vectors are now closed:
1. NaN / infinite / negative → `timeout_guard` returns typed `ProcessError` before any conversion (registry/process.rs:429-446).
2. Float too large for `Duration` → `try_from_secs_f64` returns `Err`, wrapped via `process_map_err` → `?` (registry/process.rs:89-94).
3. `Instant + Duration` overflows host clock → `checked_add` returns `None`, `ok_or_else` → typed `ProcessError` → `?` (registry/process.rs:96-114).

Generated code in `process_timeout_status.sifr` confirms the chain emits at all four call sites (output_timeout × 2, shell_output_timeout × 2). Test assertions at registry_extended_tests.rs:133-140 cover `is_finite`, `try_from_secs_f64`, `.checked_add(`, the host-clock error message, and the `__timed_out` tuple.

No `.unwrap()`/`.expect()` on data-dependent values in the timeout path. Pass-2's theoretical `Instant + Duration` overflow band is now closed. Validation evidence (fmt, check, registry test, e2e pass, fail suite 418/418, file-size + HIR guardrails) is consistent with the code state.

RESULT: PASS
