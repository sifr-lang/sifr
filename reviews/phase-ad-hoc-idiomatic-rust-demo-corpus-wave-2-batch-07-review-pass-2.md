Let me review each file systematically against the production-grade bar.

---

## demos/fnmatch/idiomatic.rs

**Issue 1: `fnmatch` and `fnmatchcase` are identical — case sensitivity is missing**

In Python's `fnmatch`, matching is case-sensitive by default (unlike `fnmatchcase` which forces case-sensitive matching). The comment at line 2 acknowledges this distinction should exist, but `fnmatchcase` at line 45 is a direct alias of `fnmatch`, both performing case-sensitive matching:

```rust
fn wildcard_match(name: &str, pattern: &str) -> bool {
    // ... case-sensitive matching only ...
}

fn fnmatch(name: &str, pattern: &str) -> bool {
    wildcard_match(name, pattern)
}

fn fnmatchcase(name: &str, pattern: &str) -> bool {
    wildcard_match(name, pattern)  // identical to fnmatch
}
```

In Python: `fnmatch("HELLO", "hello")` → `True` (case-insensitive), but `fnmatchcase("HELLO", "hello")` → `False`.
In Rust: both return `false`. The behavioral difference between `fnmatch` and `fnmatchcase` is not reflected.

This is a **behavioral equivalence bug** and **actionable**.

**Issue 2: Non-idiomatic iteration style**

Lines 6–9 collect into `Vec<char>` with manual index management:

```rust
let name: Vec<char> = name.chars().collect();
let pattern: Vec<char> = pattern.chars().collect();
let (mut ni, mut pi) = (0usize, 0usize);
```

An idiomatic Rust implementation would use direct `Chars` iterators with `.next()`, which is more educational and cleaner:

```rust
let mut name_chars = name.chars();
let mut pattern_chars = pattern.chars();
// then use pattern_chars.next() instead of pattern[pi]
```

This is **ownership/API/design quality** and **actionable**.

---

## demos/textwrap/idiomatic.rs

**Issue 3: `dedent` mishandles lines shorter than `min_indent`**

The `chars().skip(min_indent)` at line 54 silently produces an empty string for lines with fewer than `min_indent` characters:

```rust
line.chars().skip(min_indent).collect()
```

If a line has exactly `min_indent` whitespace characters and no other content (e.g., `"  "` with `min_indent=2`), `skip(2)` yields an empty string, not preserving the semantic difference between "empty line" and "line that was only whitespace."

Python's `textwrap.dedent` would produce different results for these edge cases. This doesn't affect the current test suite, but it is **non-idiomatic behavior** that could cause subtle bugs.

**Verdict**: Not clearly actionable without a failing test, but worth noting.

**Issue 4: `wrap` splits on ASCII space only**

Line 29 uses `normalized.split(' ')` rather than `normalized.split_whitespace()`:

```rust
for word in normalized.split(' ') {
```

If input contains tabs or other whitespace characters that `normalize_whitespace` collapsed to spaces, this is fine. But if the input already contains literal space characters, this works correctly. The code is **not incorrect** given the preceding normalization step.

**Verdict**: No actionable issue.

---

## demos/string/idiomatic.rs

**Issue 5: `capwords` uses `to_uppercase()`/`to_lowercase()` on single chars**

Lines 16–20 operate on individual `char` values:

```rust
let mut result = first.to_uppercase().collect::<String>();
result.push_str(&chars.as_str().to_lowercase());
```

These methods are not locale-aware and operate on single scalar values, not grapheme clusters. For ASCII this is correct, but `"ß".to_uppercase()` produces `"SS"`, and `"İ".to_lowercase()` produces `"i\u{307}"` — neither fits the model of "one uppercase char + rest lowercase."

**Verdict**: Not actionable for an ASCII-focused demo, but worth documenting if this were to evolve.

---

## Summary

**Actionable issues found:**

1. **demos/fnmatch/idiomatic.rs:38-45** — `fnmatch` and `fnmatchcase` are identical. The case-sensitivity distinction from Python's `fnmatch` module is missing.
2. **demos/fnmatch/idiomatic.rs:6-9** — Manual `Vec<char>` indexing is non-idiomatic; direct `Chars` iterators would be cleaner and more educational.

**Not actionable** (minor or edge-case-only):
- `textwrap/dedent` line-length handling — works for test suite
- `string/capwords` Unicode handling — correct for ASCII demo scope
- `ValueError` manual implementation — acceptable for demo without `thiserror`
