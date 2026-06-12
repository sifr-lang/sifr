## Review summary

**Result: PASS — no material blockers.**

Pass 1's two non-blocking observations called out for remediation are both addressed, and no new blockers were introduced.

### Pass 1 follow-ups verified

1. **Optional-feature clippy** (pass 1 obs #3): `.unwrap()` calls in `crates/sifr_runtime/src/i18n.rs:208,229,234` are now `.expect("…")`, covered by `#![cfg_attr(test, allow(clippy::expect_used))]` at `crates/sifr_runtime/src/lib.rs:2`. The ledger records `cargo clippy -p sifr_runtime --features i18n --tests -- -D warnings` (`issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:335`). I re-ran it locally on a clean `sifr_runtime` — it passes with zero warnings. `cargo test -p sifr_runtime --features i18n i18n` still passes 5/5.

2. **Inventory status drift** (pass 1 obs #4): `verification/stdlib/text_i18n_substrate_inventory.md:3` is now `Status: M3 in progress.` and `verification/stdlib/text_i18n_substrate_inventory.json:3` is `"status": "m3-in-progress"`. The dependency-decisions doc header is also re-stamped to M3 with ICU4X 2.2 language.

### Confirmed no regressions in scope

- **Runtime i18n** (`crates/sifr_runtime/src/i18n.rs`): every fallible ICU surface returns `Result<_, String>`; integer casts use `try_from`; date/time/decimal/plural inputs are all checked; `host_locale` is read-only, skips `C`/`POSIX`, strips codeset/modifier, and only returns canonicalizable values.
- **Stdlib feature gating** (`crates/sifr_stdlib/src/features.rs`): five `IcuX` features added; `sifr.i18n` → 5 ICU + `SifrRuntime`; `RuntimeFeatures { i18n, unicode }` correctly merges into a single deduplicated `sifr_runtime = { …, features = ["i18n", "unicode"] }` line, with the combined-features test asserting this.
- **Codegen intrinsics** (`crates/sifr_codegen/src/intrinsics/registry/i18n.rs` + `registry.rs`): all 8 intrinsics wrap runtime calls with `map_err` into the contract-stated `LocaleIdError` / `FormatError` / `PluralRulesError` struct literals; arities match the typed signatures in `i18n_core.rs`.
- **Public API** (`lib/sifr/i18n.sifr`): `LocaleId`, `parse_locale`, `canonicalize_locale`, `maximize_locale`, `minimize_locale`, `host_locale`, `NumberFormatter`, `DateTimeFormatter`, `PluralRules`, `Collator`, plus style/rule/strength string constants. Object-scoped, no globals.
- **E2E fixture** (`crates/sifr/tests/e2e/pass/text_i18n_locale_formatting.sifr`): 14 happy/error/tolerant-host assertions match a 14-element expected vector.
- **Docs**: `text_i18n_m3_traceability.md` now points each backlog row at the specific fixture/runtime evidence; `text_i18n_dependency_decisions.md` reflects `LocaleCanonicalizer/LocaleExpander::new_extended` and the i18n Cargo feature.

Unrelated dirty concurrency-runtime files were ignored as instructed.
