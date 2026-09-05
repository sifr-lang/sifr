# M10 Wave 2 Regex Interop — Code Review

## Findings (file:line)

### Behavior preservation — match/find/replace/findall/split & flag variants

- `crates/sifr_stdlib/src/regex.rs:8-103` — all twelve helpers mirror the deleted intrinsic semantics:
  - `re_replace` calls `re.replace_all(text, replacement)` with the same arg ordering as the old `lower_re_replace` (`text=args[2]`, `replacement=args[1]`). The `Replacer` impl for `&str` preserves `$N` capture substitution.
  - `re_find_start`/`re_find_end` use `usize_to_i64_saturating` (`crates/sifr_stdlib/src/regex.rs:105-117`) instead of the old raw `as i64` cast. This is strictly safer for huge inputs and observably identical for any string fitting in memory.
  - `regex_with_flags` (`crates/sifr_stdlib/src/regex.rs:86-103`) prefixes the same `(?i)/(?m)/(?s)/(?x)` markers with the same bit values used by the old `build_flags_prefix_stmts`. Flag bits 2/8/16/64 match the CPython constants exposed in `stdlib/sifr/re.sifr:16-19`.
- `stdlib/_sifr/regex.sifr:1-50` — twelve `@rust(sifr_stdlib.regex.*)` declarations with `panic=trusted_no_panic`. Inspecting the Rust bodies confirms none of them panic (`regex::Regex::new` returns Result; `is_match`/`find`/`find_iter`/`split`/`replace_all` are panic-free on valid regex; `i64::try_from(...).unwrap_or(...)` is infallible).
- `stdlib/sifr/re.sifr:1-13,21-55` — imports underscored as `_re_*_impl` and public wrappers preserve the same public arities/return shapes that `cpython_re.sifr`/`stdlib_re_consolidated.sifr` e2e fixtures exercised. `Pattern`, `Match`, `compile`, `compile_flags`, `search`, `sub`, `findall`, `split`, `fullmatch[_flags]`, `finditer` all unchanged.

### Error bridge

- `crates/sifr_codegen/src/rust_interop_direct.rs:91-131` — `message_error_fields` now accepts an `Error`-parented class iff (a) every field is `Type::Str` and (b) a field named `message` exists. For `RegexError { message: Str, detail: Str }` (`crates/sifr_lowering/src/lower/typing_and_functions/signatures_and_effects.rs:247-267`) this emits `RegexError { message: e.to_string(), detail: e.to_string() }`, byte-for-byte the shape the old `regex_error_expr` produced.
- For mixed-typed Error subclasses (`JSONDecodeError`, `TOMLDecodeError`, `JsonLimitError`) the predicate returns `None` and the bridge falls back to the raw error value. That fall-through would not compile if such a class were ever produced via direct interop, but those error types are still owned by intrinsic dispatch — no regression today. Mildly broader scope than wave 1 but acceptable for this migration; only `RegexError` is exercised. Worth noting that an all-string error class like `JsonIntegerRangeError { message, path, profile }` would silently splat the same display text into all three fields if it were ever routed here — out of scope for this PR but flag for future waves.
- `crates/sifr_codegen/src/rust_interop_direct.rs:369-411` — added unit test asserts the exact emitted shape including both `message` and `detail` from `__sifr_bridge_error.to_string()`. Sound.

### Feature/dependency planning

- `crates/sifr_stdlib_model/src/features.rs:607-608` — `sifr.re` and `_sifr.regex` now return `&[]` (no retained direct deps), while `sifr.pathlib` still returns `&[StdlibFeature::Regex]`. Correct: `crates/sifr_codegen/src/intrinsics/registry.rs:72-79` keeps `glob_pattern`/`rglob_pattern` as intrinsics that emit `regex::Regex::new(...)` directly, so the retained `regex = "1.12.3"` direct dep is still required for that path.
- `crates/sifr_stdlib_model/src/features/generated_stdlib_features.rs:27` (unchanged) — `sifr.re | _sifr.regex | sifr.pathlib => ["regex"]` still enables the `sifr_stdlib` `regex` feature. Combined with `crates/sifr_stdlib/src/lib.rs:37-38` (`#[cfg(feature = "regex")] pub mod regex;`) and `crates/sifr_stdlib/Cargo.toml:30,55` (`regex = ["dep:regex", "dep:sifr_runtime"]`), the sysroot wiring is consistent.
- `crates/sifr_stdlib_model/src/features_tests.rs:239` — adds `sifr.re` to the no-direct-third-party-deps allowlist.

### Intrinsic ownership removal

- `crates/sifr_codegen/src/intrinsics/registry.rs:22,475-486` — twelve `re_*` entries removed cleanly; `mod re;` removed; no orphan references (`grep registry::re` returns nothing).
- `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:714-740` — test renamed `re_intrinsics_are_owned_by_compiled_stdlib_declarations`, asserts `lower_intrinsic` returns `None` for all twelve symbols. Solid regression guard.
- `crates/sifr_codegen/src/intrinsics/registry/re.rs` — deleted (482 lines), no stragglers.
- `crates/sifr_stdlib_model/src/lib.rs:146` — `intrinsic_regex()` still registered for the bootstrap-fallback shape, consistent with the pattern used for previously-migrated leaves (`_sifr.uuid`, `_sifr.platform`, etc.).

### Tests & docs

- `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:257-312` — covers (i) `sifr_stdlib::regex::*` call emission for all twelve helpers, (ii) the exact `map_err` shape, (iii) `SifrIntBridge::from(flags)` for flag variants, (iv) empty intrinsic set for `_sifr.regex` and `sifr.re`, (v) `_sifr.regex` listed in `sifr.re`'s transitive deps, (vi) public/private export hygiene.
- `crates/sifr_stdlib/tests/api_behavior.rs:248-289` — exercises all helpers, the IGNORECASE flag, and a malformed-pattern error path.
- `internal_docs/sifr_sysroot_and_stdlib_architecture.md` and `internal_docs/stdlib_native_surface_ownership.toml` updated coherently. `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md` records wave-2 status and validation evidence.

### Panic safety

- No new `unwrap`/`expect`/`panic` on user-triggerable paths. `i64::try_from(value).unwrap_or(i64::MAX)` is total. `regex::Regex::new` returns `Result`. `trusted_no_panic` annotations are defensible.

### Minor observations (non-blocking)

- `crates/sifr_codegen/src/rust_interop_direct.rs:107-110` — silent fallback to `value` when the Error subclass has non-string fields. Will fail to compile loudly if ever exercised. A `debug_assert!`/log would be friendlier, but matches the convention of the surrounding bridge code. Not a blocker.
- `stdlib/sifr/re.sifr:91-100,167-176,209-214` — `search_match`/`_finditer_materialize`/`Pattern.finditer` catch `RegexError` and re-raise `RegexError(e.message)`, dropping `e.detail`. Preexisting behavior, unchanged in this wave.

---

VERDICT: PASS
