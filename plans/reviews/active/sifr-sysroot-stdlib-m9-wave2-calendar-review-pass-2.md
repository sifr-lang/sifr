## Findings

**`crates/sifr_stdlib/src/calendar.rs:40-47` — `calendar_weekday_i64` widened to i128**

The fix correctly addresses pass-1's overflow concern:
- `i128::from(i64)` is infallible (free widening). 
- Worst-case magnitude is roughly `y * (1 + 1/4 + 1/400) + month_offset + day + 6`. With `y = i64::MIN as i128 - 1 ≈ -9.22e18` and `day` up to `i64::MAX ≈ 9.22e18`, the sum stays well below `i128::MAX ≈ 1.7e38`. No debug-build overflow possible.
- `rem_euclid(7)` on `i128` always returns a value in `[0, 7)`, so `i64::try_from(...)` cannot fail. The `unwrap_or(0)` is a defensible defensive fallback (line 46) — never triggers, but doesn't introduce a panic.
- `rem_euclid(7)` cannot panic (divisor is a non-zero constant); no `.unwrap()`/`.expect()` introduced.

**`crates/sifr_stdlib/src/calendar.rs:49-65` — `month_offset` parameter widened to `i128`**

Match arms still cover the documented 1..=12 range with a `_ => 0` default. Type widening is purely interface alignment with the new caller — no semantic change.

**`crates/sifr_stdlib/tests/api_behavior.rs:94-99` — extreme boundary coverage**

- `int(i64::MIN)` and `int(i64::MAX)` flow through `SifrIntBridge::from(i64)` → `SifrInt::from(i64)`, which round-trips losslessly; `to_i64_saturating` (interop.rs:56-64) returns the same bound on the result. No saturation is exercised here (the inputs already fit `i64`), but that's fine — the test is checking the wide arithmetic path, not saturation itself.
- `month < 3` path (`i64::MIN, 1, 1`): exercises the `y = year - 1` underflow that was the pass-1 concern. Now safe under i128. ✓
- `month >= 3` path (`i64::MAX, 12, 31`): exercises the upper bound. ✓
- Range assertion `(0..=6).contains(&weekday)` is correct (Python `calendar.weekday` returns 0..=6).

**Other observations**

- No new panics introduced. `days_in_month` and `is_leap_year_i64` continue to operate on `i64`, which is correct (no overflow risk there: only modulo by small constants).
- The `int(2024), int(2), int(29) → int(3)` assertion still matches Python semantics (Thursday=3). ✓
- The i128 widening is localized to the one helper that needs it; nothing leaks into the public API surface.

The follow-up fully addresses the pass-1 observation: pathological-year overflow in `calendar_weekday_i64` is eliminated, and the new extreme-bound assertions lock in the fix.

VERDICT: PASS
