I've reviewed the remediation. Reading the runtime feature gate, the stdlib feature emission, the registry intrinsics, and the traceability updates against the pass-1 baseline.

# M2 Unicode Core Re-Review (post-remediation)

## Remediation verification

- **`crates/sifr_runtime/Cargo.toml:13-18`** — `unicode-normalization` and `unicode_names2` are correctly marked `optional = true` and gated behind a `unicode` feature using the modern `dep:` syntax (no implicit features leaked).
- **`crates/sifr_runtime/src/lib.rs:7-10`** — `unicode` and `unicode_data` modules are `#[cfg(feature = "unicode")]`, so the 14,779-line generated table compiles out of default workspace builds and out of all non-Unicode generated projects.
- **`crates/sifr_stdlib/src/features.rs:343,362-371,401-412`** — `needs_sifr_runtime_unicode` is computed once before iteration, then threaded into the `sifr_runtime` dep renderer. It correctly triggers on either `sifr.unicode`/`_sifr.unicode` modules or on the `UnicodeNames`/`UnicodeNormalization` intrinsic-level features added by `additional_required_features` (`registry.rs:80-83`). Because the BTreeSet+packages set de-duplicates a second `sifr_runtime` push from `sifr.encoding`+`sifr.unicode`, the first (and only) emission inherits the unicode flag regardless of iteration order.
- **Tests** — new `unicode_module_emits_runtime_and_unicode_dependencies` and `unicode_intrinsic_features_enable_runtime_unicode_feature` (`features.rs:501-531`) lock in both pathways; the pre-existing `runtime_and_tokio_features_render_owned_dependency_specs` was tightened to assert `!dep.contains("features")` so a regression that always emits the unicode flag would fail.
- **Performance evidence** — `build-project-001-additional-modules` RSS dropped from 390,545,408 B → 313,999,360 B against a 342,556,672 B budget; the original failure mode is no longer reachable for non-Unicode projects.
- **Traceability** — `verification/stdlib/text_i18n_m2_traceability.md` gains a `Runtime feature gating` row quoting the budget delta; `text_i18n_dependency_decisions.md` records the gate.

## Findings

All non-blocking. Ordered by severity.

### Low
1. **Redundant user-level deps for `sifr.unicode`.** `features_for_stdlib_module("sifr.unicode")` (`features.rs:320-324`) still pushes `UnicodeNames` and `UnicodeNormalization`, which emits direct top-level `unicode_names2 = "3.1.0"` and `unicode-normalization = "0.1.25"` Cargo entries in the generated project. All generated code routes through `sifr_runtime::unicode::*` (verified via grep: `unicode_normalization::`/`unicode_names2::` appear only inside `sifr_runtime`), so these crates would be pulled in transitively by `sifr_runtime = { features = ["unicode"] }` alone. Cargo de-dups by version, so this is correctness- and binary-size-safe but adds two unused direct deps to every Unicode-using `Cargo.toml`. Cleanup candidate, not a blocker.

2. **`additional_required_features` for `unicode_case_fold` is overbroad.** `registry.rs:66-83` adds both `UnicodeNames` and `UnicodeNormalization` for every `unicode_*` intrinsic including `unicode_case_fold`, which only reads the embedded `CASE_FOLDING` table. Same root cause as (1) and would resolve together if the per-module wiring is trimmed.

3. **Runtime tests are not in the merge gate.** `scripts/run_all_tests.sh` does not invoke `cargo test -p sifr_runtime`, so the new `unicode::tests::*` cases (and the rest of the runtime crate's tests) only run via explicit `cargo test -p sifr_runtime --features unicode`. Pre-existing scope, not introduced by this PR. Worth flagging because the feature gate now means even a workspace-wide `cargo build --workspace` / `cargo clippy --workspace` won't compile `unicode.rs` or `unicode_data/generated.rs` unless `--features unicode` is forwarded. Lint/regressions in those two files can only surface today through explicit-feature invocations or via the generated-code corpus.

### Carryover (already noted in pass 1, still applies)
4. No compile-time assertion that the hardcoded `UNICODE_DATA_VERSION = "17.0.0"` matches `unicode_normalization::UNICODE_VERSION` / `unicode_names2` data version. Dependency doc says skew is a release blocker but nothing in code enforces it.

## Verdict

Remediation cleanly resolves the RSS budget regression at the right layer: a Cargo feature on the runtime, with deterministic feature-flag emission keyed off both stdlib-module and intrinsic-level requirements. Existing M2 behavior (panic-freedom, typed errors, e2e fixture coverage, generated-marker excluding the table from the line-count guardrail) is preserved. No blockers; the listed observations are cleanup opportunities.

VERDICT: PASS
