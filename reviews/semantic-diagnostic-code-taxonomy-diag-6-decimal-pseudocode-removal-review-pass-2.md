# milestone_diag_6 slice 1 review (pass 2)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-next-from-rendered` against `main`, plus the deltas added since [reviews/semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-1.md). Slice intent unchanged: drop `[E25xx]` text from decimal diagnostics, fixtures, and verification baselines, while preserving the eight top-level `SIFR-DECIMAL-000x` codes.

## Status of pass-1 findings

| # | Finding | Pass-2 status | Evidence |
| --- | --- | --- | --- |
| 1 | Stale demo `expect-error: [E2505]` in `forbidden_float_constructor/main.sifr` | **Resolved** | [demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3](demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3) now reads `# expect-error: SIFR-DECIMAL-0005: Decimal(float_value) is not allowed; use Decimal("...") for exact construction` |
| 2 | Stale `Verification Baseline Surface` row at inventory:262 | **Resolved** | [internal_docs/diagnostic_emission_inventory.md:262](internal_docs/diagnostic_emission_inventory.md:262) now reads `SIFR-DECIMAL-0001 in compact/json/human output with no message-embedded pseudo-code` / `Done in milestone_diag_6 slice 1; keep as decimal renderer regression coverage` |
| 3 | Stale `diagnostic range E2501-E2508` literal in `decimal_diagnostics` demo | **Resolved** | All three siblings updated: [demos/decimal_diagnostics/main.sifr:14](demos/decimal_diagnostics/main.sifr:14), [demos/decimal_diagnostics/idiomatic.rs:34-36](demos/decimal_diagnostics/idiomatic.rs:34), [demos/decimal_diagnostics/emitted.rs:19-21](demos/decimal_diagnostics/emitted.rs:19) now print `diagnostic range SIFR-DECIMAL-0001 through SIFR-DECIMAL-0008 is reserved and enforced` |
| 4 | Stale `[E25xx]` mentions in three demo doc comments | **Resolved** | [demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs:7](demos/decimal_diagnostics/negative_cases/decimal_round_scale_out_of_range/idiomatic.rs:7) now says `deterministic SIFR-DECIMAL-0007 error`; [demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs:7](demos/decimal_verification/negative_cases/forbidden_float_conversion/idiomatic.rs:7) → `SIFR-DECIMAL-0005`; [demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs:7](demos/decimal_verification/negative_cases/forbidden_mixed_arithmetic/idiomatic.rs:7) → `SIFR-DECIMAL-0004` |
| 5 | Phase 28 doc still reserves the `E2501-E2508` range | **Resolved** | [internal_docs/phases/28_decimal_type_and_exact_numeric_semantics.md:184-196](internal_docs/phases/28_decimal_type_and_exact_numeric_semantics.md:184) now reserves `SIFR-DECIMAL-0001 through SIFR-DECIMAL-0099` and lists `SIFR-DECIMAL-0001..0008` for each required code |
| 6 | No guardrail asserting `[E25` is gone from decimal messages | **Resolved** | New `test_decimal_fail_fixtures_do_not_emit_legacy_pseudo_codes` at [crates/sifr/tests/e2e.rs:2590-2616](crates/sifr/tests/e2e.rs:2590) iterates every `tests/e2e/fail/*.sifr` containing `SIFR-DECIMAL-` and asserts `!failure.code.starts_with("E25") && !failure.message.contains("[E25")` |
| 7 | Status-entry phrasing (process pointer) | N/A | Not a defect; merge will flip `[ ]`→`[x]` and append PR URL |

All six addressable pass-1 findings have been folded in; no findings were deferred.

## Verification of the pass-2 deltas

### Demo updates

- `forbidden_float_constructor/main.sifr` is the only `expect-error` annotation under `demos/` (`grep -rln "expect-error" demos/` returns just this file). The new value `SIFR-DECIMAL-0005:` matches `parse_expected_error` ([crates/sifr/tests/e2e.rs:595](crates/sifr/tests/e2e.rs:595)) — `is_diagnostic_code("SIFR-DECIMAL-0005")` is true via [crates/sifr/tests/e2e.rs:636-651](crates/sifr/tests/e2e.rs:636), and the trailing message clause feeds substring matching at [crates/sifr/tests/e2e.rs:2560-2566](crates/sifr/tests/e2e.rs:2560). The demo remains outside the harness's auto-validation set, but it is now textually consistent with the slice contract.
- `demos/decimal_diagnostics/main.sifr:14` and the two Rust idiomatic siblings now agree on the new range string. The `idiomatic.rs` and `emitted.rs` siblings differ only because `prettyplease`/`rustfmt` wraps the longer string into a `println!(\n    "...",\n)` form; the printed text is the same.
- The three doc-comment updates in `demos/decimal_*/negative_cases/.../idiomatic.rs` are pure documentation, no code surface affected. Each correctly maps the retired `[E2507]/[E2505]/[E2504]` to the matching `SIFR-DECIMAL-000{7,5,4}` constant per the registry at [crates/sifr_diagnostics/src/codes.rs:44-54](crates/sifr_diagnostics/src/codes.rs:44).

### Inventory and phase doc

- The `Verification Baseline Surface` row at inventory:262 now matches the actual baselines: I cross-checked against [crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt:2](crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt:2) (`error [SIFR-DECIMAL-0001] [main] Decimal() received invalid exact literal '12.34.56' (x1)`), [check-human.stderr.txt:1](crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-human.stderr.txt:1) (`type error: [main] Decimal() received invalid exact literal '12.34.56'`), and [check-json.stderr.txt](crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt) (`"code": "SIFR-DECIMAL-0001"` with no `[E2501]` in either `message` or `args.message.value`). The row's right-hand "Done in milestone_diag_6 slice 1" wording follows the existing convention used by other completed migration rows.
- Inventory:138 was renamed from "Current fail-fixture and harness-sample code markers" to "Original fail-fixture and harness-sample pseudo-code markers". The header now correctly frames the table at lines 140-151 as a historical migration ledger rather than a live state map.
- Phase 28 doc lines 184-196 rewrite the reserved range and the eight required codes to canonical form. The reserved range was widened from `E2501-E2508` (eight slots) to `SIFR-DECIMAL-0001..0099` (one hundred slots), which is consistent with the family-namespace policy in the issue at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:283-289](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:283).

### New e2e guardrail

[crates/sifr/tests/e2e.rs:2590-2616](crates/sifr/tests/e2e.rs:2590) — traced by hand:

1. Walks `tests/e2e/fail` via `read_dir_file_paths_sorted` ([crates/sifr/tests/e2e.rs:2694-2703](crates/sifr/tests/e2e.rs:2694)), which filters to `.sifr` files and sorts lexicographically.
2. Filters to fixtures whose source contains the literal `SIFR-DECIMAL-`. All 15 modified fail fixtures match (their `# expect-error:` lines start with `SIFR-DECIMAL-000X:`); a `grep -l "SIFR-DECIMAL-" crates/sifr/tests/e2e/fail/*.sifr` returns exactly the same 15 paths.
3. Calls `compile_source(&source).expect_err(...)`. `compile_source` ([crates/sifr/tests/e2e.rs:572-593](crates/sifr/tests/e2e.rs:572)) pushes one `CompiledFailure { code: diagnostic.code, message: ... }` per driver diagnostic, then auto-pushes a *second* `CompiledFailure { code: extracted, ... }` whenever the message contains a `[Edddd]` substring (via `diagnostic_error_code` at [crates/sifr/tests/e2e.rs:678-684](crates/sifr/tests/e2e.rs:678)). This means the assertion catches *both* a regression that puts `[E25xx]` in a top-level `code` field and a regression that puts `[E25xx]` only inside the message text — the auto-extracted secondary failure is sufficient for the message-text case, and `failure.message.contains("[E25")` is a belt-and-suspenders second check.
4. The `assert!(checked > 0)` final guard prevents a silent no-op if the fixture directory layout changes.

The eight `SIFR-DECIMAL-000x` codes are each exercised by at least one of the 15 fixtures (verified by `grep -h "expect-error" crates/sifr/tests/e2e/fail/{bigdecimal,decimal,float_from}*.sifr`):

| Code | Fixture(s) |
| --- | --- |
| `SIFR-DECIMAL-0001` | `decimal_invalid_literal_string.sifr` |
| `SIFR-DECIMAL-0002` | `bigdecimal_invalid_literal_string.sifr`, `bigdecimal_constructor_non_literal_string.sifr` |
| `SIFR-DECIMAL-0003` | `decimal_float_mixed_arithmetic.sifr` |
| `SIFR-DECIMAL-0004` | `decimal_bigdecimal_mixed_arithmetic.sifr`, `decimal_forbidden_mixed_arithmetic_seeded.sifr` |
| `SIFR-DECIMAL-0005` | `decimal_constructor_float.sifr`, `decimal_forbidden_float_conversion_seeded.sifr`, `float_from_decimal_forbidden.sifr` |
| `SIFR-DECIMAL-0006` | `bigdecimal_constructor_float.sifr`, `float_from_bigdecimal_forbidden.sifr` |
| `SIFR-DECIMAL-0007` | `decimal_round_scale_out_of_range.sifr`, `decimal_quantize_requires_int_scale.sifr` |
| `SIFR-DECIMAL-0008` | `bigdecimal_quantize_negative_scale_context.sifr`, `bigdecimal_round_requires_int_scale.sifr` |

Combined with the exact-text verification baseline that locks `SIFR-DECIMAL-0001`'s rendered output (`crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/`), the guardrail closes pass-1 finding 6.

### Slice contract still holds

- `grep -rn "\[E25" crates/sifr_hir/ crates/sifr_type_system/` returns nothing — no regression in the message-text removal.
- The 15 fail fixtures all use the uniform `SIFR-DECIMAL-000X: <msg>` shape — no fixture was rewritten with a different separator or rolled back.
- The three `decimal_invalid_literal` baselines still consistently carry `SIFR-DECIMAL-0001` and no `[E2501]` text, including the JSON `args.message.value` field.
- No compatibility shim was added; `decimal_diag_code` stays deleted, `decimal_scale_diagnostic_code` ([crates/sifr_hir/src/lower/decimal_methods.rs:12-18](crates/sifr_hir/src/lower/decimal_methods.rs:12)) still returns a `DiagnosticCode` rather than a bracketed string.
- Top-level `SIFR-DECIMAL-*` identity preservation is intact: every emission site at [decimal_methods.rs](crates/sifr_hir/src/lower/decimal_methods.rs), [expressions.rs:998-1013](crates/sifr_hir/src/lower/expressions.rs:998), and [check.rs:31-46,370-391](crates/sifr_type_system/src/check.rs:31) carries `DiagnosticCode::DECIMAL_*` via `error_with_code`/`TypeError { code: Some(...), ... }`.

## Findings (pass 2)

### 1. Inventory `Current public-code mechanisms to remove` row at line 120 still describes pre-slice state

[internal_docs/diagnostic_emission_inventory.md:120](internal_docs/diagnostic_emission_inventory.md:120):

```
| Message-embedded pseudo-code | decimal/type-system messages and fixture expectations | keeps `[E25xx]` as text inside a broader `SIFR-TYPE-0001` diagnostic | top-level `SIFR-DECIMAL-*` diagnostic code and no secondary message code |
```

Both clauses in the "Current effect" column are now wrong:

- "keeps `[E25xx]` as text" — false after this slice.
- "inside a broader `SIFR-TYPE-0001` diagnostic" — false since `milestone_diag_4a` slice 2b.1.

The two siblings in the same table use a consistent past-tense "removed; …" / "removed before `milestone_diag_4b`; …" pattern (lines 117, 118) for retired mechanisms. This row should follow that convention — e.g., `removed in milestone_diag_6 slice 1; decimal diagnostics now carry top-level SIFR-DECIMAL-000x codes with no message-embedded pseudo-code`. This is the same shape of stale-doc finding as pass-1 finding 2 at line 262, just on a different row of a related table. Small one-line cleanup.

Pass-1's earlier review at [reviews/semantic-diagnostic-code-taxonomy-diag-4a-remove-retired-fallbacks-review-pass-2.md:141](reviews/semantic-diagnostic-code-taxonomy-diag-4a-remove-retired-fallbacks-review-pass-2.md:141) flagged line 120 as "correctly framed as past state", but at that point only the `SIFR-TYPE-0001` clause was stale; this slice's removal of `[E25xx]` makes both clauses stale, which crosses the threshold from "historical context" to "actively misleading description of current code".

### 2. Slice scope alignment — no other gaps

I re-ran the same searches pass 1 used:

- `grep -rn "\[E25" crates/ demos/ docs/ internal_docs/ verification/ scripts/ issues/ --include="*.rs" --include="*.sifr" --include="*.md" --include="*.txt" 2>/dev/null` after excluding `reviews/`, `target/`, `.git/`, and `issues/archive/`: only the harness self-tests at [crates/sifr/tests/e2e.rs:2607,2759,2773,2788](crates/sifr/tests/e2e.rs:2607), the new guardrail's own assertion text, the inventory table heads (lines 110, 120, 144-151, 136), and historical/design/scope context in the issue file at lines 225, 273, 344, 371, 1003, 1035, 1042 remain. All except inventory:120 are correctly scoped (harness grammar deferred to `milestone_diag_5`, design/scope sections, or already-historical migration ledgers).
- `grep -rln "expect-error" demos/` returns only `demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr`, now using the canonical form.
- The Phase 28 doc no longer claims the `E25xx` range is reserved; it correctly cites `SIFR-DECIMAL-0001..0099`.

### 3. New guardrail surface area is appropriately scoped

Two minor observations on the new test, neither blocking:

- The fixture filter is "source contains `SIFR-DECIMAL-`", which is a heuristic. If a future fixture mentions `SIFR-DECIMAL-` only in a comment but is meant to assert a non-decimal code, the guardrail would still run it. That is a strict superset of the intended scope (it would still fail only on a real `[E25xx]` reintroduction), so it is a defensible heuristic.
- `compile_source` produces one canonical `CompiledFailure` per driver diagnostic *plus* one auto-extracted secondary failure per `[Edddd]` substring it finds. The guardrail therefore double-counts the regression detection: `failure.code.starts_with("E25")` catches the auto-extracted secondary, and `failure.message.contains("[E25")` catches the canonical primary. Both clauses are needed because either could be true in isolation if the harness's `diagnostic_error_code` extraction logic ever changes; keeping both is correct conservative coding, not redundant.

## What looks correct

- Pass-1 findings 1–6 are all folded in cleanly, with no scope creep beyond the slice contract.
- The phase-doc rewrite preserves the original ordering and prose, only swapping `E2501..E2508` → `SIFR-DECIMAL-0001..0008` and the range header. No "compatibility note" or alias table was introduced.
- The new e2e test is independent of `test_e2e_fail` (it's a separate `#[test]` rather than nested inside the existing fail loop), so it produces an actionable, named failure on regression rather than a generic fail-suite count mismatch.
- The inventory's "Original fail-fixture and harness-sample pseudo-code markers" header at line 138 correctly reframes lines 140-151 as historical migration data, not as live state — that table no longer needs row-by-row updates as future slices ship.
- All 15 e2e fail fixtures still match the harness substring matcher exactly: `parse_expected_error("SIFR-DECIMAL-000X: <msg>")` returns `code = "SIFR-DECIMAL-000X"` and the trailing message, and the diagnostic stream from `compile_source` contains both the canonical code and the same trailing message — the matcher at [crates/sifr/tests/e2e.rs:2553-2567](crates/sifr/tests/e2e.rs:2553) accepts both halves.
- The validation matrix the user ran covers every modified surface: `cargo test -p sifr_type_system` (the four `check.rs` sites), `cargo test -p sifr --test e2e test_e2e_fail` (the 15 fail fixtures + the new guardrail), `cargo test -p sifr_hir decimal` (the lowering sites in `decimal_methods.rs` and `expressions.rs`), `python3 scripts/run_verification_hardening.py --suite diagnostics` (the three baseline files), `cargo test -p sifr test_decimal_fail_fixtures_do_not_emit_legacy_pseudo_codes` (the new guardrail explicitly), and `cargo fmt --check` / `git diff --check` (formatting/whitespace).
- No clippy-pedantic-relevant patterns introduced in the new test (no `unwrap_or_default` on `Result`, no `&Vec<T>` parameters, no integer literal type-suffix oddities in user-facing API surface).

## Recommendation

The slice now satisfies its stated contract end-to-end. The only remaining loose end is the one-line inventory cleanup at [internal_docs/diagnostic_emission_inventory.md:120](internal_docs/diagnostic_emission_inventory.md:120) (Finding 1) — same shape as the line 262 fix from pass 1, mechanically trivial to apply. Every other surface I checked is consistent with the slice's intent.

If the line-120 row is updated to past tense (matching the lines 117/118 convention), the slice is ready for PR. If the user prefers to defer that to the next slice's residual cleanup, the deferral should be noted in the slice's status entry at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:73](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:73) so the contradiction is intentional rather than an oversight.

No further validation runs needed beyond the matrix the user already executed.
