## Verdict: PASS

All five scope requirements hold and every invariant I checked passed cleanly.

**What was verified**

1. **Private `@rust(sifr_stdlib.i18n.*)` declarations** — `stdlib/_sifr/i18n.sifr` declares all 14 `_i18n_*_impl` names with `panic=trusted_no_panic` and `@rust(sifr_stdlib.i18n.…)` bindings; the adapter-policy syntax test (`stateless_private_codegen_tests.rs:59`) covers it.
2. **Public API stability** — `stdlib/sifr/i18n.sifr` retains `LocaleIdError`/`FormatError`/`PluralRulesError`/`CatalogError`/`TranslationError`, all constants (`DATETIME_*`, `PLURAL_*`, `COLLATION_*`), all classes (`LocaleId`, `NumberFormatter`, `DateTimeFormatter`, `PluralRules`, `Collator`, `Message`, `Bundle`, `Translator`), and every previously-public `i18n_*` / helper function. Each public `i18n_*` wrapper catches `ParseError` from the private bridge and re-raises the correct public error class (`sifr/i18n.sifr:52-181`).
3. **Adapter boundary owned by `sifr_stdlib`** — `crates/sifr_stdlib/src/i18n.rs` is the sole caller of `sifr_runtime::i18n::*` for i18n; the file is gated behind `feature = "i18n"` (`crates/sifr_stdlib/src/lib.rs:25`), and `sifr_stdlib`'s `i18n` feature transitively enables `sifr_runtime/i18n` (`crates/sifr_stdlib/Cargo.toml:61`). `SifrIntBridge` conversions are handled at the adapter, not in generated code.
4. **Compiler lowering removed** — `crates/sifr_codegen/src/intrinsics/registry/i18n.rs` is deleted; `mod i18n;` in `registry.rs`, the 14 `lower_intrinsic_rendered` arms, and the i18n branch of `additional_required_features` are all gone. `registry_core_tests.rs:200` now asserts `lower_intrinsic(name, …).is_none()` for all 14 names. `features_for_stdlib_module("sifr.i18n" | "_sifr.i18n")` now returns `&[]`, and `needs_sifr_runtime_i18n` no longer scans module names. Snapshot data (`text_i18n_dependency_snapshots.json`) and harness assertions (`harness_behavior_tests.rs:497`) confirm generated `Cargo.toml` for a plain `sifr.i18n` import contains only `sifr_stdlib = { … features = ["i18n"] }` — no `icu_*` and no explicit `sifr_runtime` line.
5. **Retained direct-ICU path still intact** — `IcuCollator`/`IcuDatetime`/`IcuDecimal`/`IcuLocale`/`IcuPlurals` remain as `StdlibFeature` variants, still map to package specs and to the `"i18n"` runtime feature (`generated_stdlib_features.rs:71-75`), and `needs_sifr_runtime_i18n` still triggers an explicit `sifr_runtime = { features = ["i18n"] }` line for them (covered live by `features_tests.rs:128` `runtime_dependency_can_enable_unicode_and_i18n_together`).

Docs/verification updated: `internal_docs/text_i18n_architecture.md`, `internal_docs/sifr_sysroot_and_stdlib_architecture.md`, `internal_docs/stdlib_native_surface_ownership.toml` (`_sifr.i18n` now `deletion_stage = "migrated-to-sysroot-stdlib"`), and `verification/areas/stdlib_parity/reports/text_i18n_dependency_decisions.md` all reflect the transitive path.

**Non-blocking observations**

- `plans/reviews/active/m10-i18n-interop-opus-review-pass1.md` and `pass2.md` are 0-byte placeholders (only the corresponding `.stderr`/`.log` sidecars carry content). If those files are meant to hold review artifacts, they're empty; if they're just markers, ignore.
- `crates/sifr_stdlib/src/i18n.rs:110-116` retains legacy alias functions `canonicalize_locale` / `format_number` that just forward to their `i18n_*` counterparts. Behavior-verified by `tests/api_behavior.rs:29` and referenced from nowhere else on-branch, so they can eventually be inlined; not required for this PR.
