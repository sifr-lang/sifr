## Verdict: **PASS**

Migration is coherent and matches the pattern of previously merged waves (JSON, encoding, TOML, URL). Public API preserved, panic-safety honest, compiler-intrinsic ownership fully cleared, dependency plan correctly re-plumbs through `sifr_stdlib/unicode`.

### Substantive checks that passed

- **Public API shape preserved.** `stdlib/sifr/unicode.sifr` keeps `UnicodeDataError`, all scalar property methods returning `Result[..., UnicodeDataError]`, `case_fold`/`graphemes`/`words` returning owned strings, and `grapheme_indices`/`word_boundaries` returning `list[tuple[int, str]]` / `list[tuple[int, int, str]]`. Tuple reconstruction via `_flat_part` + `_trusted_int` correctly walks the 2- and 3-item strides emitted by the Rust `*_flat` helpers.
- **Error bridge consistent.** Private declarations use `Result[..., ParseError]`; public wrappers re-raise as `UnicodeDataError`. `stateless_private_codegen_tests` verifies `map_err(...ParseError { message: __sifr_bridge_error.to_string() })`. Same shape as `_sifr.encoding`.
- **Compiler-intrinsic ownership fully retired.** `registry.rs` match arms for `unicode_*` removed, `registry/requirements.rs` requirements block removed, `registry/unicode.rs` deleted, and `registry_core_tests::unicode_intrinsics_are_owned_by_compiled_stdlib_declarations` asserts `lower_intrinsic("unicode_*")` returns `None`. `sifr_stdlib_model/src/unicode_core.rs` intrinsic registrations renamed to the `_unicode_*_impl` bootstrap forms with `ParseError`.
- **Dependency plan correctly re-plumbed.** `features_for_stdlib_module("sifr.unicode") = &[]` cancels the retained-direct-dep emission; `planned_sifr_stdlib_features` still maps to `["unicode"]`; `needs_sifr_runtime_unicode` no longer fires on module presence. Result: `sifr.unicode`-only projects emit `sifr_stdlib` (unicode) alone, and `sifr_stdlib`'s `unicode = ["dep:sifr_runtime", "sifr_runtime/unicode"]` pulls the runtime transitively. Combined `sifr.unicode + sifr.i18n` correctly drops the redundant `unicode` feature from the direct `sifr_runtime` line (Cargo will still union it via `sifr_stdlib`). Snapshots in `text_i18n_dependency_snapshots.rs` / `text_i18n_dependency_snapshots.json` and fixture assertions in `harness_behavior_tests` all agree.
- **Cargo.lock aligns.** `sifr_stdlib` no longer directly depends on `unicode-normalization` / `unicode-segmentation` / `unicode_names2`; they remain in the `sifr_runtime` graph, which is where the actual usage lives.
- **Panic safety honest.** All `sifr_stdlib::unicode::*` wrappers delegate to `sifr_runtime::unicode::*`, whose implementations avoid `.unwrap()`/`panic!` (fallible paths return `Result<_, String>`; `case_fold`/`graphemes`/etc. are infallible over `str`). `SifrIntBridge` wrapping for `combining/decimal/digit` mirrors existing patterns in `math`/`regex`/`calendar`/`base64`.

### Non-blocking observations (pre-existing doc rot, not caused by this PR)

- `internal_docs/text_i18n_architecture.md:17–20` still describes direct `sifr_runtime` emission for `sifr.encoding`, `sifr.unicode`, and `sifr.i18n`. Already stale after the encoding wave; unchanged here. Worth a follow-up sweep once M10 waves settle rather than a blocker for this PR (consistent with encoding wave's precedent).
- `verification/areas/stdlib_parity/reports/text_i18n_dependency_decisions.md:8` says "`sifr_runtime/unicode` is emitted only for `sifr.unicode` users." Post-migration the emitted crate is `sifr_stdlib` with feature `unicode`, which transitively activates `sifr_runtime/unicode`. Substantive claim (unicode code stays out of non-unicode projects) still holds; phrasing rot.

Neither observation touches correctness, generated Cargo behavior, public API, or panic-safety.
