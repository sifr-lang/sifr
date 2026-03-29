I'll review each file for idiomatic Rust quality, focusing on behavioral equivalence, ownership/API design, and readability.

## demos/base64/idiomatic.rs

No actionable issues.

The implementation is clean and idiomatic. Error handling via `ParseError` is well-structured. The `decode_to_utf8` helper avoids repetition. Base16 encode/decode correctly handles hex formatting and parsing.

**Minor note (non-actionable):** The `b16decode` function validates even-length input explicitly, which is more specific than what `from_str_radix` would produce — but this is an improvement, not a deviation.

---

## demos/hashlib/idiomatic.rs

No actionable issues.

The `HashObject` type is sound — it encapsulates name and data, with `update` taking `&mut self`. The `bytes_for_algorithm` helper centralizes the algorithm dispatch. Error handling with `ValueError` and `HashlibError` is consistent.

**Verified behavioral equivalence on the critical path:**
- `new("sha3_256", "")` returns `Err(ValueError)` → `collect_negative_actual_ok()` returns `Vec<bool>` containing `false` → matches Sifr `[False]` expected value.
- `algorithms_guaranteed()` returns `["md5", "sha256"]` → `contains` check returns `true` for `"sha256"` → matches Sifr `true`.

---

## demos/bytes_module/idiomatic.rs

No actionable issues.

Whitespace-tolerant hex parsing (`text.chars().filter(|ch| !ch.is_whitespace()).collect()`) is a reasonable extension that doesn't conflict with the Sifr demo's assertions. The byte search and slice operations (`starts_with`, `ends_with`, `find_byte`) use standard library methods directly. Error handling via `ParseError` is consistent with `base64/idiomatic.rs`.

---

**Summary:** All three files are strong, idiomatic Rust solutions that faithfully replicate the Sifr demo behavior. No actionable issues.
