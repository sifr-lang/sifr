# Review Result: **PASS** (follow-up to pass 2)

## Scope

Pass 3 verifies the small remediation applied after pass 2:

- `non_empty_form` helper added so that empty translation forms in a `.mo` catalog surface as `None` from `Catalog::lookup` / `Catalog::lookup_plural` instead of `Some("")`. This lets `Translator` walk to fallback bundles when a catalog entry exists but its `msgstr` is intentionally empty (draft stubs).
- New runtime unit assertion exercises the empty-translation entry through the parsed-catalog API.

The check is whether that change introduced new blockers or regressed any of the six material M4 requirements. Unrelated dirty files (concurrency review notes, structured-work model, concurrency-execution ledger) are out of scope.

## Remediation re-check

| Item | Verdict | Evidence |
| --- | --- | --- |
| Empty-form helper returns `None` | ✓ | `crates/sifr_runtime/src/i18n/translation.rs:75-82` — `non_empty_form` returns `None` for empty strings, `Some(form.clone())` otherwise. Both call sites use it: plain `lookup` at `:50` and `lookup_plural` at `:71`. |
| Empty-form unit assertion | ✓ | `crates/sifr_runtime/src/i18n/translation.rs:756-793` — `mo_catalog_supports_context_plural_and_declared_charset` injects `(b"empty", b"")` at `:763` and asserts `catalog.lookup(None, "empty") == None` at `:773`. |
| Translator behavior on empty primary | ✓ | `lib/sifr/i18n.sifr:199-213` — when `lookup` returns `None`, the translator continues to fallback bundles, then to `message_id` as the terminal fallback. The new bundle-level `None` flows through that chain. Plural fallback (`:231-247`) similarly returns `singular`/`plural` for `count==1`/else. |
| Consistency with explicit-fallback contract | ✓ | The change preserves the `Bundle`/`Translator`/explicit-fallback shape mandated by the phase contract instead of silently returning `""`. No global state, no implicit msgid fallback at the bundle layer. |

## Material M4 re-verification

| Requirement | Verdict | Evidence |
| --- | --- | --- |
| Native `Bundle`/`Message`/`Translator` + explicit fallback chains | ✓ | `lib/sifr/i18n.sifr:138-281`; `Translator.with_fallback` constructs a fresh instance (`:192-197`); `translate_message` dispatches across plain/context/plural/context-plural (`:269-281`). |
| `.mo` compatibility backend only, no gettext globals | ✓ | No `install` / `textdomain` / `bindtextdomain` symbols in `lib/sifr/i18n.sifr` or runtime i18n modules; inventory keeps `gettext.install` / global `_` / `textdomain` as `unsupported-with-diagnostic` / `deferred-to-phase-adapter` (`verification/stdlib/text_i18n_substrate_inventory.md:46-47`). |
| M1 encoding substrate reuse | ✓ | Single decode integration point: `crates/sifr_runtime/src/i18n/translation.rs:241-244` calls `encoding::decode_text(data, charset, "strict")`. Latin-1 fixture (`caf\xe9` → `café`) verified by `mo_catalog_supports_context_plural_and_declared_charset` and the e2e fixture. |
| Constrained safe plural-expression parser | ✓ | `translation.rs:561-671` tokenizer is closed over `n`, decimal integers, parens, `? :`, `! && \|\| == != < <= > >= + - * / %`, and unary `!`/`-`/`+`; any other byte returns `Err("invalid token in plural expression at byte offset N")` (`:662-666`). Source length cap `MAX_PLURAL_EXPR_LEN = 1000` (`:11`, `:449-451`), depth cap `MAX_PLURAL_DEPTH = 20` (`:12`, `:470-472`, `:506-508`). No interpreter/host eval. |
| Panic-free runtime contract | ✓ | All `unwrap()` / `expect()` in `i18n/translation.rs` live inside `#[cfg(test)]`. Parser propagates with `?`; arithmetic uses `checked_*` (`:344-348`, `:400-422`); divide/modulo by zero returns `Err`. The new `non_empty_form` is total over its inputs — `forms.get(index)?` handles out-of-bounds, no indexing. |
| Fixture/documentation coverage | ✓ | E2E fixture `crates/sifr/tests/e2e/pass/text_i18n_translation_bundles.sifr` covers primary+fallback (`:34-36`), context (`:39,64`), plural (`:40-41,65`), context-plural (`:58,66`), missing-key fallback to msgid/plural (`:44-45,52-53`), malformed plural rejection (`:77`), missing-path rejection (`:82-87`), and file-loader round-trip (`:89-104`). Traceability, dependency-decisions, substrate inventory and execution ledger remain aligned (`verification/stdlib/text_i18n_m4_traceability.md`, `text_i18n_dependency_decisions.md`, `text_i18n_substrate_inventory.{md,json}`, `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:208-210,348-359,407-415`). |

## Focused validation reproduced locally on this pass

- `cargo test -p sifr_runtime --features i18n i18n -- --nocapture` — 12 i18n runtime tests pass, including the new empty-entry assertion inside `mo_catalog_supports_context_plural_and_declared_charset`.
- `cargo clippy -p sifr_runtime --features i18n --tests -- -D warnings` — clean.
- `cargo test -p sifr_codegen i18n -- --nocapture` — `lowers_i18n_intrinsics_with_dependency_metadata` passes.
- `cargo test -p sifr_stdlib i18n -- --nocapture` — 3/3 pass.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/text_i18n_translation_bundles.sifr` — cache hit, fixture green.

## Diff hygiene

M4-scoped changes only — same surface as pass 2, plus the small remediation inside `translation.rs`. No leakage into neighbouring substrates; no destructive edits.

## Pass-1/2 deferred items (still non-blocking)

Pass-1 observations 2–6 (cached parsed catalog inside `Bundle`, bare `Bundle(...)` constructor vs validated factory, M4 checklist flip at PR-land, Tier 1 charset fixture, precomputed default English plural) remain M5 polish. None are material to M4 acceptance.

## Non-blocking observations new to this pass

1. **Behavioral note for the M5 docs sweep.** The bundle-layer change diverges from CPython `gettext.GNUTranslations`, where an empty `msgstr` resolves to `""` (and the higher-level `gettext`/`ngettext` methods then implement msgid fallback). Sifr instead surfaces `None` from `Bundle.lookup*` so the explicit `Translator` chain is what walks to fallbacks and msgid. This is a deliberate, explicit-fallback design choice and matches the substrate inventory's `compatibility-adapter` classification, but it is worth one sentence in the M5 user-facing docs so adapter consumers don't expect literal gettext lookup semantics from the raw `Bundle`. Not a blocker — the contract was always native-API-first.

## Verdict

**PASS.** The pass-2 remediation (empty-form `None` and its assertion) is correct, narrow, and consistent with the explicit-fallback `Translator` design. All material M4 requirements remain satisfied: native `Bundle`/`Message`/`Translator` with explicit fallback chains, `.mo`-only compatibility backend with no gettext globals, single M1-encoding-substrate decode path, constrained safe plural parser with depth/length caps, panic-free user-data paths (no data-dependent `unwrap`/`expect`/`panic`), and fixture/documentation coverage. No blockers; no re-review required.
