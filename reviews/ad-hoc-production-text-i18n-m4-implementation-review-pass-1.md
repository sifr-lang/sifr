# Review Result: **PASS** (with non-blocking observations)

## Scope checks

| Requirement | Verdict | Evidence |
| --- | --- | --- |
| `Bundle`/`Message`/`Translator` + explicit fallback chains | ✓ | `lib/sifr/i18n.sifr:138-281` — `Translator.with_fallback` produces an immutable next-translator, `translate_message` dispatches across plain/context/plural/context-plural. |
| Context-aware lookup | ✓ | `translation.rs:241-246` (`menu\u{4}open` key), runtime test `mo_catalog_supports_context_plural_and_declared_charset`. |
| Plural-aware lookup | ✓ | `translation.rs:54-72`; e2e covers `fichier`/`fichiers`. |
| Missing-key fallback | ✓ | `translate*` falls through to `singular`/`plural`/`message_id`; e2e asserts `"missing"`, `"boxes"`, `"missing plural"`. |
| Missing-path errors | ✓ | `read_mo_catalog_file` maps `std::fs::read` failure into `CatalogError`; e2e asserts. |
| M1 encoding substrate for `.mo` charset | ✓ | `translation.rs:232-235` calls `encoding::decode_text(data, charset, "strict")`; `latin-1` fixture `caf\xe9` → `café` verified by runtime test and e2e. |
| Constrained safe plural parser (no general engine) | ✓ | `translation.rs:418-661` tokenizer accepts only `n`, decimals, parens, `? :`, comparison/arithmetic/logical operators, and `!`; non-ASCII or unknown bytes (e.g., `@`) return typed `Err`; depth and length capped at 20 and 1000. Identifiers other than single-byte `n` are rejected (multi-char names like `eval` produce a sequence of bare `N` tokens that fail at parse time). No interpreter/host-engine dispatch. |
| Panic-free runtime contract | ✓ | All `unwrap()`/`expect()` matches in `translation.rs` are inside `#[cfg(test)]`. Parser uses `?` throughout; `read_u32`/`read_slice` use checked arithmetic. ICU calls use `map_err`. |
| No `gettext.install` / global `_` mutation | ✓ | `lib/sifr/i18n.sifr` adds no module-level mutables, no `_` shim; `Translator.with_fallback` returns a new instance instead of mutating. `host_locale` reads env but never writes. Inventory keeps `gettext.install/textdomain/global underscore` as `unsupported-with-diagnostic`. |
| Docs/traceability alignment | ✓ | `text_i18n_m4_traceability.md`, `text_i18n_substrate_inventory.{md,json}`, `text_i18n_dependency_decisions.md` all updated; M4 scan evidence appears in the execution ledger at `issues/.../execution.md:406-413` with PR slot reserved at line 194. |

## Non-blocking observations

1. **Error name divergence from phase contract.** Phase doc enumerates `CatalogParseError` (`issues/ad-hoc-production-text-i18n-platform-substrate.md:316`); implementation chose `CatalogError` (broader — covers both parse and file I/O). The traceability and dependency-decisions docs already reflect `CatalogError`, but the phase contract's typed-error list wasn't reconciled. Consider aligning in M5 (either rename the runtime error or update the phase doc errata).

2. **`Bundle` re-parses on every lookup.** `lib/sifr/i18n.sifr:163-181` stores raw bytes and each `lookup*` calls `i18n_mo_*` which re-runs `Catalog::parse(data)` (`crates/sifr_runtime/src/i18n.rs:159-192`). Safe and correct, but O(catalog) per lookup. M5 polish opportunity to keep a parsed-once representation behind `Bundle`.

3. **`Bundle(...)` constructor skips validation.** Direct `Bundle(bytes)` (used in the e2e) defers all checks to first lookup; only `bundle_from_mo_bytes(...)` calls `i18n_mo_validate`. Each lookup still parses, so this is safe; consider deprecating the bare constructor in favor of the validated factory.

4. **M4 checklist not flipped.** `issues/.../execution.md:28` still shows `[ ]` for `milestone_text_i18n_4` and line 194 lists the PR as "pending"; validation evidence is recorded at lines 346-357. Expected since the PR isn't open yet — flag when PR lands.

5. **Tier 1 charset coverage.** The fixture exercises `latin-1` (Tier 0); declared Windows-125x catalogs aren't covered by an end-to-end fixture. The M1 substrate clearly supports them, so this is just additional coverage worth adding to M5 demos.

6. **`PluralFormula::default_english`** parses `"n != 1"` on every catalog without a `Plural-Forms` header. Cheap, but a `const`-style precomputed `Expr` would avoid the per-catalog parse cost; non-blocking.

No blockers found. The translation bundle substrate is correct, panic-free, routes `.mo` charset decoding through the M1 encoding registry, uses a properly constrained plural parser with no general expression evaluation, exposes context/plural/fallback/missing-key/missing-path behavior, and avoids process-global mutation.
