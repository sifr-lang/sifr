Now I have enough context to produce the review.

# M2 Unicode Core Implementation Review

## Scope verification

All M2 substrate elements are present:
- `lib/sifr/unicode.sifr` exposes `data_version`, `normalize`, `is_normalized`, `name`, `lookup`, `category`, `bidirectional`, `combining`, `east_asian_width`, `mirrored`, `decomposition`, `decimal`, `digit`, `numeric_value`, `case_fold`, plus `NFC/NFD/NFKC/NFKD` constants and `UnicodeDataError`.
- `_sifr.unicode` intrinsics typed in `crates/sifr_stdlib/src/unicode_core.rs` and registered in `crates/sifr_stdlib/src/lib.rs:83`.
- Runtime layer `crates/sifr_runtime/src/unicode.rs` wraps `unicode-normalization` + `unicode_names2` + generated UCD tables; generated tables (`crates/sifr_runtime/src/unicode_data/generated.rs`, 14,779 lines) carry the `@generated` marker and `#[rustfmt::skip]`.
- Generator `scripts/generate_unicode_tables.py` (UCD 17.0.0, `UnicodeData.txt`/`EastAsianWidth.txt`/`CaseFolding.txt`) and a guardrail exemption in `scripts/check_file_size_guardrails.py` that skips files containing `@generated` / `DO NOT EDIT` in their first 5 lines.
- Codegen wiring in `crates/sifr_codegen/src/intrinsics/registry/unicode.rs` and registry table in `registry.rs:326-385`; new tests in `registry_core_tests.rs:101`.
- Feature wiring: new `StdlibFeature::UnicodeNames` and `UnicodeNormalization`; `sifr.unicode`/`_sifr.unicode` rooted to both plus `SifrRuntime` in `features.rs:320`.
- E2E fixture `crates/sifr/tests/e2e/pass/text_i18n_unicode_core.sifr` exercises NFC/NFD/NFKC/NFKD, every scalar property, `Cn` default for unassigned, decimal/digit/numeric, `case_fold` of `Straße İ`, and four typed-error paths (bad form, missing name, missing numeric, multi-scalar input).
- Traceability and dependency-decision docs updated; M2 ledger entry recorded.

## Behavior verification

- **Panic-free runtime**: I traced every function in `crates/sifr_runtime/src/unicode.rs`. `partition_point` + filtered `.get()`, `binary_search_by_key().ok()`, and `text.chars()` are all panic-free on arbitrary user input. `parse_numeric` rejects empty strings, divides only after `denominator == 0.0` guard. No `unwrap`/`expect`/`panic!` on user-controlled values.
- **Typed errors**: Invalid form, empty input, multi-scalar input, missing name, missing decimal/digit/numeric, and unknown lookup all return `Err(String)`, mapped to `UnicodeDataError { message }` by `unicode.rs:34`.
- **Multi-scalar case folding**: `parse_case_folding` keeps both `C` and `F` rows and (per CaseFolding.txt structure) overrides `S` with `F` because `mappings[codepoint] = mapping` is keyed by codepoint; the `T` (Turkic) status is correctly skipped. `Straße İ` → `strasse i\u{307}` confirmed.
- **Versioning**: `UNICODE_DATA_VERSION = "17.0.0"` is consistent with the generator constant and the runtime test.
- **Bare `unicodedata`**: Existing fail fixture `bare_cpython_unicodedata_import.sifr` keeps the namespace boundary intact; no new aliases are introduced.

## Non-blocking observations

1. **Version skew is asserted in docs but not in code.** The dependency record claims `unicode-normalization::UNICODE_VERSION` is 17.0.0, but nothing in `unicode.rs` (or a build script) actually compares the crate's `UNICODE_VERSION` constant against the hardcoded `"17.0.0"` in `generated.rs`. A one-line `const _: () = assert!(unicode_normalization::UNICODE_VERSION == (17, 0, 0));` (or a unit test) would turn the version-skew "release blocker" into a compile-time gate. Applies equally to `unicode_names2` when M2.5 lands segmentation.
2. **Redundant `try/except` in `lib/sifr/unicode.sifr`.** Every wrapper does `try: return intrinsic(...); except UnicodeDataError as e: raise UnicodeDataError(e.message)`. The intrinsic already returns `Result[T, UnicodeDataError]`; `lib/sifr/encoding.sifr` uses bare `return encoding_canonical_label(self.label)` for the same pattern. The unicode wrappers can be reduced to direct returns. Functionally identical; just noise.
3. **`bidi` default for gaps is `""` rather than the `DerivedBidiClass.txt` defaults.** The e2e fixture only asserts `bidi("\ufffe") == ""`, which happens to match CPython for U+FFFE, but CPython's `unicodedata.bidirectional` returns derived defaults (e.g., `AL`) for unassigned codepoints inside Arabic ranges. Same caveat applies to gap codepoints that should inherit `category` other than `Cn` via DerivedGeneralCategory. Acceptable for M2 scope, worth recording as a known gap if any consumer (HTTP/IDNA, M3 i18n) relies on derived classes.
4. **`has_generated_file_marker` broadens beyond Rust.** The guardrail exemption checks the first five lines of *every* source category for `@generated` / `DO NOT EDIT`. A future Python/Sifr file could bypass the 900-line cap by adding the marker without actually being generated. Low risk because the rest of the workflow gates this, but a `category == "rust"` constraint (or path-prefix allowlist) would tighten the policy.
5. **`decomposition` returns raw UnicodeData.txt field 5 including `<compat>`/`<font>` tags.** This matches CPython, but the M2 contract doesn't explicitly call out that behavior. The fixture only asserts the canonical case (`\u212B → "00C5"`), so tagged decompositions are uncovered.
6. **`combining: u8` in the generated record is fine** (UCD values are 0–240), but stretches the schema only as far as canonical CCC; if a future generator wants to encode derived properties with wider value ranges, the type will need to grow. Worth a comment in the generator.

## Verdict

Implementation matches the M2 contract: every required surface is wired through `_sifr.unicode` → `sifr_runtime::unicode` → the locked Unicode 17.0.0 data, error paths return typed `UnicodeDataError`, generated tables are marked and excluded from the file-size guardrail, dependency feature metadata and traceability are recorded, and all six local validation commands passed. No correctness, panic-safety, or namespace-boundary blockers found.

VERDICT: PASS
