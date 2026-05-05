# milestone_diag_5 slice 1 review (pass 1)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-harness-contract` against `main`, against the milestone_diag_5 slice 1 contract in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:74](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:74) and the broader milestone_diag_5 definition at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:996-1030](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:996).

Slice intent (from the issue's "in progress" entry): tighten e2e `expect-error` parsing to canonical registry-backed `SIFR-<FAMILY>-dddd` codes only, accept `# expect-error[col=<1-based-column>]: CODE` qualifier, reject `[Edddd]` and message-substring expectations, stop extracting secondary codes from diagnostic messages, and rewrite fail fixtures to code-only assertions. Harness-only — no compiler emission changes.

## What the slice changes

- E2E harness ([crates/sifr/tests/e2e.rs](crates/sifr/tests/e2e.rs)):
  - Imports `active_registry_entries`, `registry_entry`, `DiagnosticState` from `sifr_diagnostics::codes` ([crates/sifr/tests/e2e.rs:19](crates/sifr/tests/e2e.rs:19)).
  - `CompileFailureExpectation` swaps `message_contains: Option<String>` for `column: Option<u32>` ([crates/sifr/tests/e2e.rs:80-84](crates/sifr/tests/e2e.rs:80)).
  - `CompiledFailure` gains `column: Option<u32>` populated from the primary span ([crates/sifr/tests/e2e.rs:86-91](crates/sifr/tests/e2e.rs:86), [crates/sifr/tests/e2e.rs:578-597](crates/sifr/tests/e2e.rs:578)).
  - Secondary `[Edddd]` extraction (`diagnostic_error_code`) is deleted; the loop in `compile_source` now produces exactly one `CompiledFailure` per emitted diagnostic ([crates/sifr/tests/e2e.rs:578-597](crates/sifr/tests/e2e.rs:578)).
  - New parser surface: `parse_expected_error`, `parse_expected_error_parts`, `parse_expect_error_line`, `validate_expected_error_code`, `extract_compile_failure_expectations`, `closest_active_diagnostic_code`, `edit_distance` ([crates/sifr/tests/e2e.rs:599-751](crates/sifr/tests/e2e.rs:599)). Validation walks: empty → reject; `[` or `E` prefix → "legacy pseudo-code" reject; whitespace/`:` inside code → "message substrings are not accepted" reject; non-canonical regex → reject; not in registry → reject with edit-distance hint; reserved → reject as not-active.
  - `is_diagnostic_code` regex tightened from `is_ascii_alphanumeric` to `is_ascii_uppercase || is_ascii_digit` for the family prefix ([crates/sifr/tests/e2e.rs:710-725](crates/sifr/tests/e2e.rs:710)).
  - `is_message_error_code` and `normalize_error_code` are deleted.
  - `compile_failures_to_messages` renders `CODE@colN: msg` when a column is present ([crates/sifr/tests/e2e.rs:753-761](crates/sifr/tests/e2e.rs:753)).
  - `extract_expect_errors` now returns the payload after `]:` for the bracketed form (lenient discovery extractor; not used for validation) ([crates/sifr/tests/e2e.rs:421-434](crates/sifr/tests/e2e.rs:421)).
  - `test_e2e_fail` swaps to `extract_compile_failure_expectations` and matches by `(code, optional column)` against `CompiledFailure` ([crates/sifr/tests/e2e.rs:2607-2655](crates/sifr/tests/e2e.rs:2607)).
  - `test_expectation_parsing_contract` updated to canonical samples + one bracketed sample ([crates/sifr/tests/e2e.rs:2817-2843](crates/sifr/tests/e2e.rs:2817)).
  - `test_expected_error_contract_with_messages` is replaced by two new tests:
    - `test_expected_error_contract_accepts_canonical_codes_and_columns` ([crates/sifr/tests/e2e.rs:2845-2856](crates/sifr/tests/e2e.rs:2845))
    - `test_expected_error_contract_rejects_messages_legacy_and_unknown_codes` ([crates/sifr/tests/e2e.rs:2858-2875](crates/sifr/tests/e2e.rs:2858))

- Fail fixtures (139 files under [crates/sifr/tests/e2e/fail/](crates/sifr/tests/e2e/fail/)): every `# expect-error: SIFR-FAMILY-NNNN: <message>` is rewritten to `# expect-error: SIFR-FAMILY-NNNN`. No fixture adopts the new `[col=N]:` qualifier.

- Inventory ([internal_docs/diagnostic_emission_inventory.md:10,136](internal_docs/diagnostic_emission_inventory.md:10)): updates the marker count to 147 fail-fixture expectations and rewrites the E2E expectation paragraph to describe the slice 1 contract.

- Issue status ([issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11,74](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11)): wave bumped to milestone_diag_5; "in progress" entry added for slice 1.

## Verification of the slice contract

- "Accepts only active registry-backed canonical SIFR-<FAMILY>-dddd codes": `validate_expected_error_code` ([crates/sifr/tests/e2e.rs:668-708](crates/sifr/tests/e2e.rs:668)) walks the registry via `registry_entry` and rejects anything that isn't `DiagnosticState::Active`. Confirmed by inspection — the `Some(entry) if entry.state == DiagnosticState::Active => Ok(())` arm is the only success path. The `Some(entry)` Reserved arm and the `None` arm both error.
- "Rejects [Edddd]": covered by the `code.starts_with('[') || code.starts_with('E')` guard ([crates/sifr/tests/e2e.rs:673-677](crates/sifr/tests/e2e.rs:673)) and asserted by `test_expected_error_contract_rejects_messages_legacy_and_unknown_codes` against `[E2507]`.
- "Rejects message-substring expectations": covered by the `':' || whitespace` guard ([crates/sifr/tests/e2e.rs:679-686](crates/sifr/tests/e2e.rs:679)) and asserted by the same negative test against `SIFR-TYPE-0002: assignment to immutability`.
- "Rejects unknown/reserved codes": unknown asserted by the negative test (`SIFR-TYPE-9999`); reserved is reachable but not asserted (see Finding 3).
- "Optional `[col=<1-based-column>]:` qualifier": `parse_expect_error_line` ([crates/sifr/tests/e2e.rs:615-644](crates/sifr/tests/e2e.rs:615)) parses the qualifier, rejects unknown qualifier names, requires `]:`, and rejects non-positive integers. Asserted positively (`col=12`) by `test_expected_error_contract_accepts_canonical_codes_and_columns`.
- "Compile failure matching uses emitted top-level diagnostic code and optional primary column": `compile_source` reads `diagnostic.code` directly and `diagnostic.spans.iter().find(|span| span.is_primary).and_then(|span| span.column)` ([crates/sifr/tests/e2e.rs:578-597](crates/sifr/tests/e2e.rs:578)). The render side at [crates/sifr_diagnostics/src/render/mod.rs:121-158](crates/sifr_diagnostics/src/render/mod.rs:121) produces at most one primary span and column is char-based 1-based ([crates/sifr_diagnostics/src/render/mod.rs:270-281](crates/sifr_diagnostics/src/render/mod.rs:270)). The matcher in `test_e2e_fail` uses `failure.code == expected.code && (expected.column.is_none() || failure.column == expected.column)` ([crates/sifr/tests/e2e.rs:2628-2635](crates/sifr/tests/e2e.rs:2628)). ✓
- "Harness no longer extracts secondary [Edddd] codes from diagnostic message text": `diagnostic_error_code` is deleted; `compile_source` no longer pushes a synthetic second failure entry. The pre-existing guardrail `test_decimal_fail_fixtures_do_not_emit_legacy_pseudo_codes` ([crates/sifr/tests/e2e.rs:2658-2684](crates/sifr/tests/e2e.rs:2658)) keeps working because it inspects `failure.code.starts_with("E25")` and `failure.message.contains("[E25")` — both still surface from the single per-diagnostic entry. ✓
- "Fail fixtures rewritten to code-only assertions": `grep -E '^# expect-error' crates/sifr/tests/e2e/fail/*.sifr` produces 147 markers; none contain `[E`, none contain `:` after the code, none use the new `[col=N]:` qualifier. All 139 modified files are pure `: <message>` removals. ✓
- Local validation: not run (per the review brief — "do not modify files"). The reviewer should run `scripts/run_all_tests.sh --profile quick` before committing.

## Findings

### 1. (must-fix) Duplicate-code expectations no longer disambiguate; two existing fixtures lose discrimination power

The matcher at [crates/sifr/tests/e2e.rs:2628-2635](crates/sifr/tests/e2e.rs:2628) is non-consuming: `errors.iter().any(...)` is evaluated independently for each expectation, so N expectations of the same `(code, column)` pair all match the same single failure. Previously the message-substring matcher disambiguated within a code, so two `# expect-error: SIFR-TYPE-0002: …expected 'int', got 'str'…` and `…expected 'bool', got 'int'…` expectations were equivalent to "two distinct diagnostics, one of each shape." After the slice they collapse to "at least one SIFR-TYPE-0002 anywhere in the failure list."

Concretely affected:

- [crates/sifr/tests/e2e/fail/bounded_multi_error_recovery.sifr](crates/sifr/tests/e2e/fail/bounded_multi_error_recovery.sifr): now has two identical `# expect-error: SIFR-TYPE-0002` lines on lines 3 and 4 covering three distinct error sites (`a: int = "a"`, `b: int = "b"`, `c: bool = 1`). The fixture cannot fail if the harness regresses to emitting only one diagnostic — both expectations match the same failure. Pre-slice the two messages disambiguated `int vs str` from `bool vs int`.
- [crates/sifr/tests/e2e/fail/bounded_multi_error_recovery_repeated_type_errors.sifr](crates/sifr/tests/e2e/fail/bounded_multi_error_recovery_repeated_type_errors.sifr): a single `# expect-error: SIFR-TYPE-0002` is meant to certify recovery up to the bounded cap across eight assignments. The fixture cannot detect a regression that drops the cap to one diagnostic.

The milestone explicitly anticipates this case at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1017](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1017): *"The `col` qualifier is required only when one source line intentionally expects multiple diagnostics and the code alone is not enough to disambiguate the expected location."* This slice ships the qualifier but uses it on zero fixtures, so the regression coverage these two fixtures used to provide is silently lost.

Recommended fix in this slice (or the immediate follow-up) — add column qualifiers so each line's expected diagnostic is distinguishable, e.g.:

```
# expect-error[col=14]: SIFR-TYPE-0002    # a: int = "a" → "a" at col 14
# expect-error[col=14]: SIFR-TYPE-0002    # b: int = "b"
# expect-error[col=15]: SIFR-TYPE-0002    # c: bool = 1
```

Even with `[col=N]:` adopted, the non-consuming matcher still allows two identical expectations to match a single failure (e.g. two `[col=14]` lines covering distinct sources lines but the same column). For full discrimination the matcher would need consume-on-match semantics. Consider either:
  a. extending the qualifier to `[line=N,col=M]:` (line is already in the rendered span), or
  b. making the matcher consume failures (each expectation reserves one error from a remaining set) — straightforward with a `Vec<bool>` shadow.

If consume-on-match is out of scope for slice 1, please at minimum add columns to the two `bounded_multi_error_recovery*` fixtures and document this lossy-matching limitation in the inventory or issue, so it doesn't silently survive into the milestone DoD ("Tests cannot accidentally bless …").

### 2. (must-fix) No e2e fixture exercises the `[col=N]:` qualifier end-to-end

Every assertion of the new `[col=N]:` grammar is in unit tests of the parser (`test_expectation_parsing_contract`, `test_expected_error_contract_accepts_canonical_codes_and_columns`). No fixture uses `[col=N]:` and `test_e2e_fail` therefore never exercises:

- the `failure.column == Some(column)` branch of the matcher ([crates/sifr/tests/e2e.rs:2630-2634](crates/sifr/tests/e2e.rs:2630)),
- the primary-span column extraction in `compile_source` ([crates/sifr/tests/e2e.rs:587-592](crates/sifr/tests/e2e.rs:587)),
- the `CODE@colN: msg` rendering in the failure summary ([crates/sifr/tests/e2e.rs:756-759](crates/sifr/tests/e2e.rs:756)).

If `RenderedDiagnostic.spans[…].column` ever changed semantics (byte vs char offset, 0-based vs 1-based, end-column vs start-column), nothing in this slice would catch it. Recommend adding at least one fixture that uses `[col=N]:` against a known-stable diagnostic with a primary span (e.g. one of the parser/decimal cases). Pairing it with Finding 1 fixes both gaps.

### 3. (should-fix) Reserved-code rejection is reachable but untested

`validate_expected_error_code`'s `Some(entry)` arm with non-Active state ([crates/sifr/tests/e2e.rs:696-700](crates/sifr/tests/e2e.rs:696)) is the only branch that distinguishes reserved from unknown. The registry currently has at least one reserved entry — `SIFR-INTERNAL-0002` ([crates/sifr_diagnostics/src/codes.rs:1295-1308](crates/sifr_diagnostics/src/codes.rs:1295)). No test exercises this branch, so a regression that turns the `if entry.state == DiagnosticState::Active` guard into `_ => Ok(())` would slip through both unit tests and `test_e2e_fail`. Add an assertion in `test_expected_error_contract_rejects_messages_legacy_and_unknown_codes` (or a sibling test) that `parse_expected_error("SIFR-INTERNAL-0002")` returns an error containing `Reserved` — this also pins the registry's reserved entry surface.

### 4. (should-fix) Several validator branches lack tests

The negative test ([crates/sifr/tests/e2e.rs:2858-2875](crates/sifr/tests/e2e.rs:2858)) covers three of seven error paths (`message substrings`, `legacy pseudo-code`, `unknown diagnostic code`). The remaining four paths in `validate_expected_error_code` and `parse_expect_error_line` have no test coverage:

- empty payload — `# expect-error: ` → "expected a diagnostic code after expect-error" ([crates/sifr/tests/e2e.rs:669-671](crates/sifr/tests/e2e.rs:669))
- non-canonical regex — e.g. `SIFR-` or `SIFR-AB` → "expected canonical SIFR-<FAMILY>-dddd code" ([crates/sifr/tests/e2e.rs:688-692](crates/sifr/tests/e2e.rs:688))
- malformed bracket without `]:` — e.g. `# expect-error[col=12 SIFR-…` → "expected expect-error qualifier syntax …" ([crates/sifr/tests/e2e.rs:621-628](crates/sifr/tests/e2e.rs:621))
- unknown qualifier — e.g. `# expect-error[line=3]: SIFR-…` → "unknown expect-error qualifier" ([crates/sifr/tests/e2e.rs:630-634](crates/sifr/tests/e2e.rs:630))
- invalid column value — e.g. `[col=0]:`, `[col=abc]:`, `[col=]:` → "invalid expect-error column" ([crates/sifr/tests/e2e.rs:635-642](crates/sifr/tests/e2e.rs:635))

These are short to add and pin the grammar's negative space, which is the milestone DoD's "expectation grammar accepts canonical top-level codes only and rejects message substrings, unknown forms, and unknown registry codes" ([issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1028](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1028)).

### 5. (should-fix) Misleading "legacy pseudo-code" message for any `E…` or `[…]` input

The guard at [crates/sifr/tests/e2e.rs:673-677](crates/sifr/tests/e2e.rs:673) is:

```rust
if code.starts_with('[') || code.starts_with('E') {
    return Err(format!(
        "legacy pseudo-code '{code}' is not accepted; use canonical SIFR-<FAMILY>-dddd"
    ));
}
```

This conflates two distinct authoring mistakes:

- A genuine legacy pseudo-code: `E2507`, `[E2507]` — message is accurate.
- A bracketed canonical code: `[SIFR-TYPE-0002]` — caller probably mistyped quoting; the message blames "legacy pseudo-code" which doesn't direct them to the actual fix (drop the brackets). The grammar at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1013](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1013) does not include brackets, so the rejection itself is correct — only the wording is off.

A small refinement: gate the "legacy pseudo-code" message to `[E…]` / `E\d+` shape, and reject other bracketed forms with "expected canonical SIFR-<FAMILY>-dddd code, got '[…]'". Same severity, clearer error.

Same code path also rejects any code whose first character is `E`. This works today (no SIFR family begins with `E`), but if a future family does (e.g. `SIFR-ENUM-0001` would still start with `S`, but a hypothetical `EXXX-…` family root would not match), the check needs to be tightened to `E[0-9]` only or equivalent. Worth a one-line regex tighten now to remove the latent footgun.

### 6. (should-fix) The reviewed-but-loose discovery extractor and the strict validator are now two parsers

`extract_expect_errors` ([crates/sifr/tests/e2e.rs:421-434](crates/sifr/tests/e2e.rs:421)) is the lenient extractor used by `discover_fixtures` and the smoke fuzz tests. After this slice it accepts the bracketed form via a naïve `]:` split with no qualifier validation — `# expect-error[anything-here]: garbage` returns the payload `garbage`. `extract_compile_failure_expectations` is the strict path used by `test_e2e_fail`. These now diverge:

- A fixture with `# expect-error[bogus]: SIFR-TYPE-0002` would be silently parsed as `SIFR-TYPE-0002` by discovery, but would panic in `test_e2e_fail` with "unknown expect-error qualifier 'bogus'".
- Discovery's `_expected_errors` field ([crates/sifr/tests/e2e.rs:77](crates/sifr/tests/e2e.rs:77)) is dead — it's set by `extract_expect_errors` but never read (it is assigned to `Vec::new()` in five other code paths and never consumed via `expected_errors`). The lenient extractor exists only to feed dead state and the fuzz smoke tests.

Two cleanups, both low risk:

- Delete `_expected_errors` and the call to `extract_expect_errors` in `discover_fixtures` ([crates/sifr/tests/e2e.rs:896](crates/sifr/tests/e2e.rs:896)). Update the fuzz/Unicode smoke tests ([crates/sifr/tests/e2e.rs:3046-3081](crates/sifr/tests/e2e.rs:3046)) to call `extract_compile_failure_expectations` against an in-memory path, or to call `parse_expect_error_line` directly — both will exercise the strict parser the way real fixtures do.
- If the lenient form is intentional (e.g. for forward compat with new qualifiers), document that intent at the function — right now the doc-comment says "Extract expected error payloads" with no mention that it skips qualifier validation.

### 7. (nit) Stale unannotated-fixture count in the inventory paragraph

[internal_docs/diagnostic_emission_inventory.md:153](internal_docs/diagnostic_emission_inventory.md:153) says "There are 88 fail fixtures with no `# expect-error` today". Actual count from `find crates/sifr/tests/e2e/fail -name '*.sifr' | xargs grep -L 'expect-error' | wc -l` is 86 (out of 232 total). The slice already touched this file — easy to update while it's under edit. Pre-existing (was `88` before this slice as well), but called out because the slice updates the same paragraph block.

### 8. (nit) `closest_active_diagnostic_code` recomputes per parse failure

[crates/sifr/tests/e2e.rs:727-732](crates/sifr/tests/e2e.rs:727) iterates the entire active registry computing edit distances for each parse failure. Today the registry is small and parse failures only happen at fixture-load time, so this is cheap. If the registry grows or the harness runs validation per discovery for many fixtures, a cached `&[(id, ...)]` would amortize. Not worth fixing in this slice — flagging only because the function is in the hot path of fixture-load panics.

### 9. (nit) Demo expectation now violates the new grammar

[demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3](demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3) still carries a message substring:

```
# expect-error: SIFR-DECIMAL-0005: Decimal(float_value) is not allowed; use Decimal("...") for exact construction
```

Today this demo is not run through the e2e harness (`grep -rn demos/decimal_types/negative_cases crates/ scripts/` is empty), so the slice's tests pass. But the file is now grammar-illegal: if the harness is ever pointed at it, `validate_expected_error_code` will reject `SIFR-DECIMAL-0005: Decimal(float_value)…` with "message substrings are not accepted". This is the same demo flagged in [reviews/semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-6-decimal-pseudocode-removal-review-pass-1.md) Finding 1 (where it was canonicalized but the message kept). Either drop the message substring in this slice (one-line edit) or carve out a documented exclusion for `demos/`. The slice already touches the inventory paragraph that governs this surface.

### 10. (nit) Wording / grammar mismatch in the inventory paragraph

[internal_docs/diagnostic_emission_inventory.md:136](internal_docs/diagnostic_emission_inventory.md:136) says the slice "removes message-substring matching, `[Edddd]` pseudo-code acceptance, and secondary-code extraction from diagnostic messages". It does not mention the new `[col=N]:` qualifier the slice introduces, and a reader of the inventory wouldn't know it exists. Mirror the issue's grammar declaration ([issues/…:1010-1015](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1010)) into the inventory note for completeness.

## Regressions in e2e failure matching

Net regression vs. pre-slice behavior, after this slice:

- (Finding 1) Multiple expectations of the same code on a single fixture now collapse: pre-slice they were disambiguated by message; post-slice they all match any single emitted failure of that code. Two existing fixtures (`bounded_multi_error_recovery*`) silently lose coverage.
- (Finding 2) The `[col=N]:` qualifier is the documented disambiguation tool but is not used by any fixture; the column-matching path is therefore untested at the integration level.
- No other matching path regresses. Single-code single-failure fixtures (the bulk) match exactly as before. Decimal pseudo-code emission was already removed in milestone_diag_6 slice 1, so the deletion of `diagnostic_error_code` does not silently mask a still-emitted secondary code (verified by [crates/sifr/tests/e2e.rs:2658-2684](crates/sifr/tests/e2e.rs:2658) guardrail and by `grep -rn '\[E25' crates/sifr_hir/ crates/sifr_type_system/` returning empty).

## Suggested action plan for pass 2

Must-fix before merge:

1. Pick a disambiguation strategy for the duplicate-code case (Finding 1) — at minimum add `[col=N]:` qualifiers to the two `bounded_multi_error_recovery*` fixtures; ideally also add consume-on-match semantics in the `test_e2e_fail` matcher.
2. Add at least one fail fixture exercising `[col=N]:` end-to-end (Finding 2).

Should-fix in this slice:

3. Add a reserved-code rejection unit test (Finding 3).
4. Add unit tests for the four currently-uncovered validator error paths (Finding 4).
5. Tighten the "legacy pseudo-code" message to its actual case and the `E…` guard to `E\d` (Finding 5).
6. Delete dead `_expected_errors` plumbing and align discovery on the strict parser (Finding 6).

Nits, can defer to a follow-up:

7. Update the unannotated-fixture count and the grammar mention in the inventory (Findings 7, 10).
8. Migrate or exclude the demo expectation (Finding 9).
9. Cache the active-registry list for `closest_active_diagnostic_code` if the registry grows (Finding 8).
