## Review: Wave 1 Batch 01 — Pass 2 Production-Grade Review

### Verdict

**Conditionally acceptable — 3 remaining actionable issues across 2 files.** All three files are behaviorally sound and structurally correct. No critical blockers. The actionable issues below are genuine API/design quality gaps that a Rust engineer would address.

---

### `demos/timeit/idiomatic.rs`

**No remaining actionable issues.** The `FnMut() + Copy` constraint on `repeat` is not actionable — it is correctly justified (the closure is consumed once per iteration by `timeit`, so it must be `Copy` or the caller must clone explicitly, which is the Rust caller's responsibility).

---

### `demos/time/idiomatic.rs`

**No remaining actionable issues.** The `Result<f64, ValueError>` return from `mktime` is idiomatic Rust. The test harness (`matches!(mktime(&epoch_tm), Ok(value) if value == 0.0)`) bridges the gap correctly. Minor nits (redundant `time`/`time_now`, `tm_isdst = 0` without comment) are not actionable.

---

### `demos/logging/idiomatic.rs`

**3 remaining actionable issues:**

#### 1. `collect_safety_actual` lacks missing-directory guard (line 193)

```rust
// MISSING: equivalent of Sifr's try/except IOError guard
let missing_log = base.join("missing").join("blocked.log");
let mut bad = get_logger("bad");
bad.set_file(&missing_log);
bad.error("should fail");
// Sifr wraps this in: try { ... } except IOError as e { missing_safe = False }
vec![!missing_log.exists(), INFO == 20 && WARNING == 30]
```

The Sifr code (`main.sifr:64-71`) wraps the write in a `try/except IOError` and sets `missing_safe = False` on exception. The Rust code assumes silent failure without verifying. This is not a behavioral mismatch (both end up with no file created), but the Rust test does not exercise the exception path. An engineer reading this as reference code might incorrectly assume the `Logger` always silently ignores errors — the test does not prove it.

#### 2. `collect_cleanup_actual` masks cleanup failures (lines 202–209)

```rust
let cleanup_ok = match fs::remove_dir_all(base) {
    Ok(()) => !base.exists(),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
    Err(_) => false,  // ← returns true for permission errors, etc.
};
```

Both the `NotFound` and the "any other error" branch return `true`. A `PermissionDenied` or `NotEmpty` error is indistinguishable from success. This is a latent bug in the test verification logic.

#### 3. `FileHandler` has no `set_level` method (line 41)

`Logger` has `set_level` but `FileHandler` does not. If a user wants to configure `FileHandler` with a level threshold after construction, there is no path to do so. The two handler types have asymmetric APIs.

---

### Summary

| File | Status | Actionable Issues |
|------|--------|-------------------|
| `timeit/idiomatic.rs` | Acceptable | None |
| `time/idiomatic.rs` | Acceptable | None |
| `logging/idiomatic.rs` | Conditional | 3 |

The logging issues are test-verification quality gaps rather than demo-behavior problems. No changes required to pass local validation. If these files are to serve as strong idiomatic Rust references, items 1 and 2 above should be addressed for the test quality to match the demonstrated API surface.
