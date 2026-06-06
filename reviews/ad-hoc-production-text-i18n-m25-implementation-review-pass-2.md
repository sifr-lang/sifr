Now I have enough evidence to write the review.

---

# M2.5 Unicode Segmentation — Code Review (Pass 2, post-remediation)

## Findings

No blocking issues.

### Informational — Saturating helper is correct and idiomatic

`crates/sifr_runtime/src/unicode.rs:162-164` introduces:

```rust
fn usize_to_i64_saturating(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
```

This is the right primitive: `i64::try_from(usize)` fails only when the input exceeds `i64::MAX` (≈ 9.2 EiB on 64-bit targets), so the `unwrap_or(i64::MAX)` branch is theoretically reachable but practically unreachable for any real Sifr string. Naming, return type, and call-site usage in `grapheme_indices` (line 140) and `word_boundaries` (lines 154-155) all line up with the pass-1 recommendation and the `int.rs` convention. No `cast_possible_wrap` warnings remain on the new segmentation paths — confirmed by re-running `cargo clippy -p sifr_runtime --features unicode -- -D warnings`, which now reports only the four pre-existing `unreadable_literal` errors in `crates/sifr_runtime/src/unicode_data/generated.rs:10507,13189` from the M2 generated table (out of scope, called out in pass 1).

### Informational — Helper is private and only paired with `saturating_add`

`start.saturating_add(segment.len())` (line 155) is also saturating, so the end offset cannot wrap before reaching the `usize → i64` conversion. Together the two saturating steps give a consistent monotonic-offset guarantee. No action needed; just worth recording.

### Informational — Pass-1 non-blocking observations still apply

- `cargo clippy --workspace -- -D warnings` is still advisory in `.github/workflows/local-first-validation.yml` (continue-on-error); not regressed by this remediation.
- The intrinsic-registry coalescing observation (`crates/sifr_codegen/src/intrinsics/registry.rs:66-88`) and the unrelated `--profile create-pr` run reminder from pass 1 are unchanged. Neither is a blocker.
- The pass-1 review file is now populated (`reviews/ad-hoc-production-text-i18n-m25-implementation-review-pass-1.md`), resolving the earlier empty-file note.

---

## Confirmed-correct items (delta only)

- `crates/sifr_runtime/src/unicode.rs:140,154-155` now route through `usize_to_i64_saturating` and match the codebase convention used by `crates/sifr_runtime/src/int.rs:119,124`.
- Tests at `crates/sifr_runtime/src/unicode.rs:255-260` still cover `grapheme_indices` (offsets 0 and 3) and `word_boundaries[0] == (0, 2, "Hi")`, exercising the new conversion path.
- `cargo fmt --check`, the targeted `unicode` test, and the e2e fixture continue to pass after the remediation.

---

VERDICT: PASS
