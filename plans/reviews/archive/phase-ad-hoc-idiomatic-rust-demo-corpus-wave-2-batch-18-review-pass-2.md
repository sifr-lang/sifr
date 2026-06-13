## Review Summary

All three files are in solid shape. I found only two minor semantic deviations from the Python APIs they reference, both in `iterators_and_randomness/idiomatic.rs`.

---

### `demos/iterators_and_randomness/idiomatic.rs`

**No high-severity issues.**

**Medium (1): `choices` creates a new RNG each call, inconsistent with Python's global RNG**
- Line 84: `let mut rng = rand::thread_rng();` is created fresh inside `choices`, and again inside `randrange` at line 99. Python's `random.choices` uses the shared global RNG state. This is a semantic deviation — not incorrect, but means the function's random output is not reproducible in the same way Python's is, and callers who expect to seed a single RNG get different behavior.

**Low (1): `compare_digest` under the `secrets` module name may set wrong expectations**
- Line 102–112: `compare_digest` is printed as `secrets.compare_digest` (line 179), which in Python's `secrets` module carries an explicit constant-time guarantee (timing-attack resistant). The current implementation uses bitwise operations that *are* likely constant-time, but the code does not assert or document this, and the `secrets` print label may mislead readers into thinking it matches the `secrets` module's contract.

---

### `demos/error_handling/idiomatic.rs`

**No actionable issues.** Custom error types, `From` conversions, `Result` fallible functions, and infallible conversions are all correct and idiomatic.

---

### `demos/decorators/idiomatic.rs`

**No actionable issues.** Functions with Sifr decorator comments (`// @log`, `// @validate`) are plain, clean Rust functions. The decorators are annotations on the Sifr source side; the Rust companion correctly shows the desugared result.
