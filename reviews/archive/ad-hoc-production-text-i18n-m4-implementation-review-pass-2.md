# Review Result: **PASS** (follow-up to pass 1)

## Scope

Re-check of the M4 text/i18n translation bundle substrate after pass 1 returned PASS with six non-blocking observations. This pass verifies that pass-1 observation #1 (typed-error contract divergence) has been reconciled and that no new material regression was introduced. Unrelated dirty files (concurrency review notes, structured-work model, concurrency execution ledger) are out of scope.

## Pass-1 observation #1 (CatalogParseError vs CatalogError) — resolved

- `issues/ad-hoc-production-text-i18n-platform-substrate.md:315` now reads `CatalogError` (was `CatalogParseError`). `git diff` confirms a single-line change scoped to the typed-error enumeration; no other phase-contract semantics moved.
- Downstream docs were already on `CatalogError`: `verification/stdlib/text_i18n_m4_traceability.md:11`, `verification/stdlib/text_i18n_dependency_decisions.md:15`, `verification/stdlib/text_i18n_substrate_inventory.{md,json}`. The phase contract and traceability are now self-consistent.
- Runtime emits `CatalogError` from `read_mo_catalog_file` (`crates/sifr_runtime/src/i18n.rs:153-154`) and from validation/parse paths, so contract matches implementation.

No new docs were touched in a way that re-introduces the divergence.

## Material M4 re-check (high level)

| Requirement | Verdict | Evidence |
| --- | --- | --- |
| `Bundle`/`Message`/`Translator` + explicit fallback chains | ✓ | `lib/sifr/i18n.sifr:138-281`; `Translator.with_fallback` returns a fresh immutable `Translator` (`:192-197`); `translate_message` dispatches plain/context/plural/context-plural (`:269-281`). |
| `.mo` charset decoding via M1 encoding substrate | ✓ | Single integration point: `crates/sifr_runtime/src/i18n/translation.rs:233` → `encoding::decode_text(data, charset, "strict")`. No alternative decode path. |
| Constrained plural parser (no general engine) | ✓ | `translation.rs:418-661`: token set limited to `Number`, `N`, parens, `?`, `:`, comparison/arithmetic/logical ops, `!`; any other byte returns `Err("invalid token in plural expression at byte offset N")` (`:653-657`); depth and source length capped (`:440-447`, `:461-463`, `:496-499`). No interpreter or host eval. |
| Context/plural/missing-key/missing-path fixtures | ✓ | `crates/sifr/tests/e2e/pass/text_i18n_translation_bundles.sifr`: primary+fallback (`:34-36`), context (`:39,64`), plural (`:40-41,65`), context-plural (`:58,66`), missing-key fallback to msgid/plural (`:44-45,52-53`), malformed plural rejection (`:77`), missing-path rejection (`:82-87`). |
| No `gettext.install` / global `_` mutation | ✓ | No `install`/`textdomain`/`gettext` symbols in `lib/sifr/i18n.sifr` or runtime i18n modules; `Translator.with_fallback` is immutable; inventory keeps `gettext.install/textdomain/global underscore` as `unsupported-with-diagnostic`. |
| Panic-free runtime contract | ✓ | All `unwrap()`/`expect()` in `translation.rs` remain inside `#[cfg(test)]`; parser propagates with `?`; ICU paths use `map_err`. |
| Docs alignment | ✓ | Phase contract now uses `CatalogError`; traceability, dependency-decisions, substrate inventory all agree. |

## Diff hygiene

M4-scoped changes only:

- `crates/sifr_codegen/src/intrinsics/registry/i18n.rs` (+87)
- `crates/sifr_runtime/src/i18n.rs` (+196) and new `crates/sifr_runtime/src/i18n/translation.rs`
- `lib/sifr/i18n.sifr` (+195)
- `issues/ad-hoc-production-text-i18n-platform-substrate.md` (1 line: `CatalogParseError`→`CatalogError`)
- `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md` (+22 ledger lines)
- `verification/stdlib/text_i18n_*` (small alignment edits)
- `crates/sifr/tests/e2e/pass/text_i18n_translation_bundles.sifr` (new fixture)

Nothing strays outside M4 scope; no destructive edits to neighbouring substrates.

## Pass-1 deferred items (still non-blocking)

Observations 2–6 from pass 1 are unchanged and remain M5 polish (cached parsed catalog inside `Bundle`, bare `Bundle(...)` constructor vs validated factory, M4 checklist flip at PR-land, Tier 1 charset fixture, precomputed default English plural). None are material to M4 acceptance.

## Verdict

**PASS.** The typed-error contract discrepancy flagged in pass 1 is resolved without collateral damage. All material M4 requirements (fallback chain, M1-substrate-backed decoding, constrained plural parser, context/plural/missing-key/missing-path fixtures, no global mutation, panic-free contract, doc alignment) remain met. No blockers.
