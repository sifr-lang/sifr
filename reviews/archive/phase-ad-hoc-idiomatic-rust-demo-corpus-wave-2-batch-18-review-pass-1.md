Now I have a complete picture of all three files. Here is my review.

---

## Review Summary

All three files have been substantially improved over the auto-generated predecessors. However, two files contain issues that are actionable.

---

## `demos/error_handling/idiomatic.rs`

**Finding 1 — `validate_range`: exclusive bounds replaced with inclusive bounds (medium)**

Sifr source:
```python
if x < lo: raise ...
if x > hi: raise ...
```
i.e., valid range is `lo <= x <= hi` (both exclusive).

Rust companion (current):
```rust
if (lo..=hi).contains(&x) { Ok(x) } else { Err(...) }
```
This uses `..=` (inclusive on both ends). The semantics differ from Sifr at the boundary: for `x = lo` or `x = hi`, Sifr's checks pass while the Rust check would also pass (both are correct), but for values just outside the bounds the behavior aligns. The test cases happen to pass, but the logic is not the same as Sifr's.

**Finding 2 — `safe_divide`: regular division instead of floor division (medium)**

Sifr uses `a // b` (floor division). Rust uses `a / b` (truncating division). For positive operands they are identical, so the visible output is correct. However, for negative operands (e.g., `(-7) // 3 == -3` in Python vs. `-7 / 3 == -2` in Rust) they diverge. The companion does not accurately model Sifr's `//` semantics.

---

## `demos/decorators/idiomatic.rs`

**Finding 1 — `Doubler` is not a callable object (medium)**

Sifr defines `Doubler` with `__call__(self, x: int) -> int`, invoked as `doubler(4)`. The Rust companion defines `Doubler` as a plain tuple struct with an `apply(self, x: i64) -> i64` method, invoked as `doubler.apply(4)`. The comment `// callable object direct` is therefore misleading — the Rust version is not a callable object at all.

**Finding 2 — Decorator comments are misleading (high)**

The file header comments say:
```
// @log
// @validate
```
These are written exactly as they appear in the Sifr source, implying Rust supports decorator syntax. The Rust companion has no decorator machinery whatsoever — these lines are inert comments above plain function definitions. The demo cannot show decorator behavior (decorator stacking, pass-through metadata) in idiomatic Rust the way Sifr does.

---

## `demos/iterators_and_randomness/idiomatic.rs`

No actionable findings. The output of all functions aligns with the Sifr demo for the exercised inputs.
