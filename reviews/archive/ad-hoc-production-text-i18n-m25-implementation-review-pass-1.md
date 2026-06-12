I've reviewed the M2.5 implementation across the runtime, intrinsic registry, stdlib types, codegen lowerers, e2e fixture, and docs. Below is the severity-ranked review.

---

# M2.5 Unicode Segmentation — Code Review

## Findings

### Medium — New clippy `cast_possible_wrap` warnings in `sifr_runtime::unicode`

`crates/sifr_runtime/src/unicode.rs:140`, `154`, `155` introduce three `as i64` conversions from `usize`:

```rust
.map(|(start, grapheme)| (start as i64, grapheme.to_string()))
…
start as i64,
start.saturating_add(segment.len()) as i64,
```

Running `cargo clippy -p sifr_runtime --features unicode -- -D warnings` produces three new `clippy::cast_possible_wrap` errors at those lines (in addition to four pre-existing `unreadable_literal` errors in the M2 `unicode_data/generated.rs`, which are out of scope). The same file already uses `i64::from(...)` for the property-record numeric returns (lines 66/96/107), and `crates/sifr_runtime/src/int.rs:119,124` use `i64::try_from(...)` — those are the codebase conventions.

Severity is Medium rather than Blocker because:
- `.github/workflows/local-first-validation.yml:21-23` marks `cargo clippy --workspace -- -D warnings` as `continue-on-error: true` (advisory).
- `scripts/run_all_tests.sh` does not invoke workspace clippy (only generated-code-quality clippy).

But `AGENTS.md` lists workspace clippy as a standard build/test command, and these warnings are new debt on top of an already-failing baseline. Recommendation: replace with `i64::try_from(start).unwrap_or(i64::MAX)` or equivalent to align with `int.rs` precedent.

### Low — Existing unicode intrinsics now drag `UnicodeSegmentation` as an additional required feature

`crates/sifr_codegen/src/intrinsics/registry.rs:66-88` merges all four new segmentation intrinsics into the same match arm that already covered `unicode_data_version`, `unicode_normalize`, `unicode_name`, `unicode_case_fold`, etc. As a result, any program that uses only `unicode_name` (or any other non-segmentation unicode intrinsic) now lists `unicode-segmentation = "1.13.3"` as a direct dependency in its generated Cargo.toml.

Practical impact is small because `sifr_runtime`'s `unicode` Cargo feature was already monolithic — enabling it for `unicode_name` already pulls in `unicode-segmentation` transitively (Cargo.toml feature line: `unicode = ["dep:unicode-normalization", "dep:unicode-segmentation", "dep:unicode_names2"]`). The pre-existing pattern also already drags `UnicodeNormalization` onto `unicode_name`-only callers, so M2.5 simply maintains the existing approach. Non-blocking; worth tracking if per-feature leanness becomes a goal later.

### Low — `scripts/run_all_tests.sh --profile create-pr` not in the listed focused validation

`AGENTS.md` calls out `scripts/run_all_tests.sh --profile create-pr` as the authoritative pre-PR gate. The listed M2.5 focused validation covers only the targeted unit / e2e / fmt / guardrail steps. The new e2e fixture would be picked up by the full e2e pass suite; running the merge profile (or at minimum create-pr) before opening the PR matches the workflow stipulated by `AGENTS.md` and the M2 evidence pattern.

### Informational — empty review file checked in

`reviews/ad-hoc-production-text-i18n-m25-implementation-review-pass-1.md` is a zero-byte file. Either populate it with this review (or the implementer's notes) or remove it before opening the PR.

---

## Confirmed-correct items

- API surface: `graphemes`, `grapheme_indices`, `words`, `word_boundaries` return owned `Vec<String>` / `Vec<(i64, String)>` / `Vec<(i64, i64, String)>` matching the M2.5 spec; `word_boundaries` correctly emits `(start, end, segment)` triples via `split_word_bound_indices` + `start.saturating_add(segment.len())`. `words` correctly uses `unicode_words()` so punctuation/whitespace are excluded, while `word_boundaries` uses `split_word_bound_indices()` to expose every UAX #29 segment — the e2e fixture asserts this delta (`["Hi", "κόσμε", "123"]` vs `["Hi", ",", " ", "κόσμε", "!"]`).
- Unicode 17.0.0 alignment is enforced by `crates/sifr_runtime/src/unicode.rs:234`: `assert_eq!(unicode_segmentation::UNICODE_VERSION, (17, 0, 0))`.
- Intrinsic signatures in `crates/sifr_stdlib/src/unicode_core.rs:90-117` align with the `.sifr` surface in `lib/sifr/unicode.sifr:133-146`.
- Feature gating is preserved: `unicode-segmentation` is an optional dep under the existing `sifr_runtime/unicode` Cargo feature; `features_for_stdlib_module("sifr.unicode")` includes `UnicodeSegmentation`; `needs_sifr_runtime_unicode` recognises it (`crates/sifr_stdlib/src/features.rs:331-383`); both stdlib-module and intrinsic-required-feature paths are exercised by the updated tests (`unicode_module_emits_runtime_and_unicode_dependencies`, `unicode_intrinsic_features_enable_runtime_unicode_feature`).
- Codegen lowerers route to `sifr_runtime::unicode::*` and the new `lowers_unicode_intrinsics_with_dependency_metadata` test exercises both segmentation feature metadata and the runtime path render.
- e2e fixture `crates/sifr/tests/e2e/pass/text_i18n_unicode_segmentation.sifr` covers combining marks (offset 0/3 invariant), ZWJ emoji, regional-indicator flags, mixed-script + numeric `words`, and word-boundary segment text + offsets (including the 10-byte Greek slice `(4, 14, "κόσμε")`).
- Docs updated: traceability (`verification/stdlib/text_i18n_m25_traceability.md`), dependency decisions table (`verification/stdlib/text_i18n_dependency_decisions.md`) now records Unicode 17.0.0 alignment and the opt-in runtime feature, execution ledger marks M2 done / M2.5 in-progress and records M2.5 evidence.
- Sentence boundaries and streaming cursors remain deferred, as specified.

---

VERDICT: PASS
