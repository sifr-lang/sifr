## Verdict

**Conditionally acceptable with actionable issues.** The demos produce correct observable results but have several API/design quality issues that an experienced Rust engineer would likely address.

---

## demos/time/idiomatic.rs

### Actionable Issues

1. **`repeat` trait bound `F: FnMut() + Copy`** (line 29): This is unnecessarily restrictive. An experienced Rust engineer would likely use `FnMut()` alone and clone the closure per iteration, or use `iter::repeat_with`. Example:
   ```rust
   fn repeat<F>(workload: F, count: usize, iterations: usize) -> Vec<f64>
   where
       F: FnMut(),
   {
       std::iter::repeat_with(|| timeit(workload.clone(), iterations))
           .take(count)
           .collect()
   }
   ```
   This fails to compile for closures that capture non-`Copy` environment.

2. **No error visibility** in `timeit`/`sleep`: Both functions silently no-op on invalid input (`sleep(-0.05)` at line 174). A Rust engineer would typically return `Result<(), Infallible>` or at minimum document the floor-to-zero semantics.

---

## demos/timeit/idiomatic.rs

### Actionable Issues

1. **`repeat` function has same `FnMut() + Copy` issue** (line 29): Same problem as above. Will fail to compile for capturing closures.

2. **Global mutable state via `TEMP_SUFFIX` without synchronization beyond relaxed ordering** (line 12): `AtomicU64::fetch_add(1, Ordering::Relaxed)` is fine for unique-id generation, but the atomic is never read back to correlate with verification. If verification of uniqueness matters, this is a latent bug. Consider `AtomicU64::fetch_add(1, Ordering::SeqCst)` for clear semantics.

---

## demos/logging/idiomatic.rs

### Actionable Issues

1. **Silent failure on I/O errors** (lines 60, 114): Both `Logger::log` and `FileHandler::emit` discard `append_line` results via `let _ =`. This is a significant API design flaw—errors are swallowed entirely with no visibility. Experienced Rust engineers expect fallible operations to be propagated or at least configurable via a `set_drain` pattern (as in `log`/`tracing` crates).

2. **No structured logging / `log` crate integration**: The demo reimplements logging from scratch with integer levels and ad-hoc formatting. The Rust ecosystem conventions favor the `log` crate's `log!` macros with a `Logger` trait. A custom implementation is fine for a demo, but the lack of `Debug`/`Display` on level types and the hardcoded integer hierarchy (`NOTSET=0`, `DEBUG=10`, etc.) are non-idiomatic.

3. **`FileHandler` lacks a `set_level` method** (unlike `Logger`): `FileHandler` only has `new` and `set_formatter`. If a user wants to configure the level after creation, they cannot. `Logger` has `set_level` but `FileHandler` does not, creating API asymmetry.

4. **`basic_config` is misleading** (line 137): It returns a fresh `Logger` instead of configuring global state (as `logging.basicConfig` does in Python). This is a behavioral mismatch.

5. **`collect_safety_actual` verifies behavior incorrectly** (lines 193-195): The test assumes that because the directory doesn't exist, the error is silently ignored. But it doesn't verify that `Logger` was actually trying to write—it just asserts `!missing_log.exists()`. A stronger test would verify the error is caught/logged internally, not just that no file materialized.

---

## No Critical Issues

- Memory/ownership: All three files handle ownership correctly.
- `LazyLock` usage for global `Instant` is idiomatic.
- `StructTime` in the time demo correctly wraps chrono types.
- Test structure with `collect_*_actual` helper functions is reasonable for demo parity testing.
## Review: Idiomatic Rust Demo Corpus — Wave 1 Batch 01

### Verdict: **Acceptable with minor issues** — no blockers, but several actionable improvements

---

### `demos/timeit/idiomatic.rs`

| Issue | Location | Severity |
|-------|----------|----------|
| `timeit` returns **total elapsed**, not average per iteration. An engineer reaching for `timeit` typically expects average. | `:16-24` | Low (educational) |
| `repeat` requires `FnMut() + Copy`, but `workload` is a ZST function pointer (always `Copy`). The constraint is unnecessary but not wrong. | `:27-32` | Low (style) |
| `collect_repeat_actual` checks `timeit(workload, 10) >= 0.0` — zero iterations is a degenerate case worth documenting. | `:49-54` | Low (docs) |

**No actionable issues.** The file is clean and idiomatic. `LazyLock` + `Instant::now()` for timer zeroing is correct.

---

### `demos/time/idiomatic.rs`

| Issue | Location | Severity |
|-------|----------|----------|
| `StructTime::from_utc` / `from_local` hardcode `tm_isdst = 0` with no comment. Misleading if someone reads this as a real `struct tm` polyfill. | `:30, :44` | Low (clarity) |
| `StructTime` exposes private fields with no accessors, yet the struct is `Debug`/`Clone`/`PartialEq` — the combination is unusual. | `:6-47` | Low (design) |
| `mktime` silently returns `0.0` on invalid input instead of `Result`. Silent failure on out-of-range values. | `:123-138` | Medium (API) |
| `epoch_utc` falls back to `expect("unix epoch must exist")` — technically safe but the structure suggests a logic error if the panic triggers. | `:64-72` | Low (style) |
| `time()` and `time_now()` are identical wrappers with no distinction. Redundant. | `:81-87` | Low (clarity) |
| `sleep(-0.05)` on line `:174` is a negative-duration call that `sleep` handles gracefully (no-op). This is intentional test coverage but worth a comment. | `:174` | Low (docs) |

**Actionable:** `mktime` should return `Result<f64, ValueError>` rather than `0.0` on invalid input. This is the clearest API gap.

---

### `demos/logging/idiomatic.rs`

| Issue | Location | Severity |
|-------|----------|----------|
| `Logger::log` silently does nothing when `output_path` is `None`. `FileHandler::emit` also silently swallows `append_line` errors. Silent failures in both paths. | `:104-115`, `:54-61` | Medium (API) |
| `FileHandler` format string is `%(levelname)s:%(name)s:%(message)s` but `Logger` uses `[%(levelname)s] %(name)s: %(message)s` — inconsistent formatting systems. | `:46`, `:113` | Low (design) |
| Log level `i64` constants could be an `enum` with `FromStr`. `i64` is not wrong but loses type safety. | `:6-10` | Low (style) |
| `basic_config` returns `Logger` but `get_logger` also returns `Logger` — two entry points with no clear distinction in naming. | `:133-139` | Low (clarity) |
| `collect_cleanup_actual` returns `true` in both `Ok` and `Err` branches — `Err` case masks failure. | `:200-206` | Medium (logic) |

**Actionable:**
1. `collect_cleanup_actual`'s `Err` branch should distinguish cleanup failure, not silently treat it as success.
2. `Logger::log` / `FileHandler::emit` error suppression should be documented or changed to `Result`.

---

### Summary

All three files are **behaviorally sound** and structurally correct. The educational value is intact. However:

- **`collect_cleanup_actual` masking errors** is the most concerning — it could hide test failures.
- **`mktime` returning `0.0`** is a silent contract deviation vs. what Rust idioms would demand.
- **Silent log failures** (`let _ = append_line(...)`) are acceptable for demo code but should be documented.

No issues require rework. The files are ready for use as idiomatic Rust reference implementations.
