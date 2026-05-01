# milestone_diag_5 slice 1 review (pass 2)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-harness-contract` against `main`, building on [reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-1.md). Slice intent unchanged from pass 1: tighten e2e `expect-error` parsing to canonical registry-backed `SIFR-<FAMILY>-dddd` codes only, accept `# expect-error[col=<1-based-column>]: CODE`, reject `[Edddd]` and message-substring expectations, stop extracting secondary codes, and rewrite fail fixtures to code-only assertions.

## Status of pass 1 findings

| # | Pass 1 finding | Severity | Pass 2 status |
| --- | --- | --- | --- |
| 1 | Duplicate-code expectations no longer disambiguate | must-fix | Resolved (consume-on-match + expanded fixtures) |
| 2 | No e2e fixture exercises `[col=N]:` end-to-end | must-fix | Partially addressed (unit-test only; integration path still untested) |
| 3 | Reserved-code rejection reachable but untested | should-fix | Resolved |
| 4 | Several validator branches lack tests | should-fix | Resolved |
| 5 | Misleading "legacy pseudo-code" message | should-fix | Resolved |
| 6 | Loose discovery extractor / dead `_expected_errors` plumbing | should-fix | Resolved |
| 7 | Stale unannotated-fixture count in inventory | nit | Resolved in prose, **listing still stale** (see Finding A) |
| 8 | `closest_active_diagnostic_code` per-call cost | nit | Not addressed (deferred — acceptable) |
| 9 | Demo expectation violated the new grammar | nit | Resolved |
| 10 | Inventory wording missing `[col=N]:` qualifier | nit | Resolved |

### Verification of resolved findings

- **Finding 1 — duplicate-code disambiguation.** `match_compile_failure_expectations` ([crates/sifr/tests/e2e.rs:765-779](crates/sifr/tests/e2e.rs:765)) now consumes one failure per expectation via a `consumed: Vec<bool>` shadow; `failure_matches_expectation` ([crates/sifr/tests/e2e.rs:754-763](crates/sifr/tests/e2e.rs:754)) honors optional column. `test_e2e_fail` ([crates/sifr/tests/e2e.rs:2625-2663](crates/sifr/tests/e2e.rs:2625)) now uses the matcher and reports the missing expectation. The two affected fixtures expanded to one expectation per intended diagnostic site:
  - [crates/sifr/tests/e2e/fail/bounded_multi_error_recovery.sifr](crates/sifr/tests/e2e/fail/bounded_multi_error_recovery.sifr) → 3 `SIFR-TYPE-0002` expectations for 3 assignment sites.
  - [crates/sifr/tests/e2e/fail/bounded_multi_error_recovery_repeated_type_errors.sifr](crates/sifr/tests/e2e/fail/bounded_multi_error_recovery_repeated_type_errors.sifr) → 8 `SIFR-TYPE-0002` expectations for 8 sites (matches the recovery cap).
  This restores the discrimination power lost in pass 1: a regression that drops the bounded-recovery cap will fail the matcher because the third (or eighth) expectation will find no remaining unconsumed failure. Unit-tested by `test_failure_matching_consumes_failures_and_honors_columns` ([crates/sifr/tests/e2e.rs:2929-2984](crates/sifr/tests/e2e.rs:2929)) including the duplicate-column negative and the too-many-code-only-expectations negative.

- **Finding 3 — reserved-code rejection.** `test_expected_error_contract_rejects_messages_legacy_and_unknown_codes` ([crates/sifr/tests/e2e.rs:2877-2897](crates/sifr/tests/e2e.rs:2877)) now asserts that `SIFR-INTERNAL-0002` is rejected and the message contains `"Reserved"`. Pinned against the registry surface ([crates/sifr_diagnostics/src/codes.rs:1295-1308](crates/sifr_diagnostics/src/codes.rs:1295)) which has `state: DiagnosticState::Reserved`; `DiagnosticState::as_str(Reserved) == "Reserved"` ([crates/sifr_diagnostics/src/codes.rs:166-171](crates/sifr_diagnostics/src/codes.rs:166)). ✓

- **Finding 4 — validator branches.** New `test_expected_error_contract_rejects_malformed_grammar` ([crates/sifr/tests/e2e.rs:2899-2926](crates/sifr/tests/e2e.rs:2899)) covers all five remaining branches: empty payload, non-canonical regex (`SIFR-`), bracketed canonical (`[SIFR-TYPE-0002]`), missing `]:` (`# expect-error[col=12 SIFR-TYPE-0002`), unknown qualifier (`[line=3]:`), and invalid column (`[col=0]:`). `[col=abc]` and `[col=]` hit the same `parse::<u32>()` arm as `[col=0]` and so are correctly subsumed by the one assertion. ✓

- **Finding 5 — guard wording / `E…` tightening.** Guard split into `bare_legacy_code` (`E\d+`) and `bracketed_legacy_code` (`[E\d+]`) ([crates/sifr/tests/e2e.rs:657-668](crates/sifr/tests/e2e.rs:657)); other bracketed forms now fall through to the canonical-shape rejector. Confirmed by `test_expected_error_contract_rejects_malformed_grammar` asserting that `[SIFR-TYPE-0002]` produces the canonical-shape error rather than a "legacy pseudo-code" message. The `is_some_and(|digits| !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()))` predicate also rules out `E`, `[E]`, `EXXX`, `[EXXX]`. ✓

- **Finding 6 — divergent parsers / dead state.** `extract_expect_errors`, `_expected_errors`, `is_message_error_code`, `normalize_error_code`, and `diagnostic_error_code` are all gone (`grep -n` returns empty in [crates/sifr/tests/e2e.rs](crates/sifr/tests/e2e.rs)). Discovery feeds nothing, and the smoke fuzz / Unicode tests at [crates/sifr/tests/e2e.rs:3155-3190](crates/sifr/tests/e2e.rs:3155) now call `extract_compile_failure_expectations`. Caveat in Finding C below. ✓

- **Finding 9 — demo file.** [demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3](demos/decimal_types/negative_cases/forbidden_float_constructor/main.sifr:3) now reads `# expect-error: SIFR-DECIMAL-0005` (no message substring). ✓

- **Finding 10 — inventory grammar.** [internal_docs/diagnostic_emission_inventory.md:136](internal_docs/diagnostic_emission_inventory.md:136) now declares both `expect-error: SIFR-<FAMILY>-dddd` and `expect-error[col=<1-based-column>]: SIFR-<FAMILY>-dddd`. ✓

## Outstanding from pass 1

### Finding 2 (still open, severity downgraded should-fix) — `[col=N]:` not exercised end-to-end

The new unit test `test_failure_matching_consumes_failures_and_honors_columns` ([crates/sifr/tests/e2e.rs:2929-2984](crates/sifr/tests/e2e.rs:2929)) constructs synthetic `CompiledFailure` and `CompileFailureExpectation` values directly and exercises the matcher with `Some(column)` on both sides. That validates the matcher's column-honoring branch and pass 1's primary structural concern, so the severity drops from must-fix to should-fix.

What still has no coverage:

- The primary-span column extraction in `compile_source` ([crates/sifr/tests/e2e.rs:571-577](crates/sifr/tests/e2e.rs:571)): `diagnostic.spans.iter().find(|span| span.is_primary).and_then(|span| span.column)`. If a future change picked the wrong span (e.g. first span instead of primary) or the diagnostic produced multiple primaries, no test would notice.
- The `RenderedDiagnostic.spans[…].column` semantic at [crates/sifr_diagnostics/src/render/mod.rs:270-282](crates/sifr_diagnostics/src/render/mod.rs:270) (1-based, char-counted from line start). A regression to byte-based or 0-based column would silently shift every rendered column without any test failure since no fixture asserts a real compiler-emitted column.
- The `CODE@colN: msg` rendering branch in `compile_failures_to_messages` ([crates/sifr/tests/e2e.rs:744-752](crates/sifr/tests/e2e.rs:744)). Reachable only when a failure has `Some(column)`; `test_failure_matching_consumes_failures_and_honors_columns` builds such failures but doesn't exercise the rendering function. The branch is only hit when a `test_e2e_fail` panic is formatted, and no current fixture forces the panic path with a column-bearing failure.

`grep -rn 'expect-error\[col=' crates/sifr/tests/e2e/fail/ demos/` returns empty — zero fixtures use the qualifier. The cheapest cure is one fixture that pairs `[col=N]:` with a known-stable diagnostic (e.g. a parser case where the primary span's column has been stable across the project lifetime).

### Finding 8 (nit, still open by choice)

`closest_active_diagnostic_code` ([crates/sifr/tests/e2e.rs:718-723](crates/sifr/tests/e2e.rs:718)) still recomputes against the full active registry on every parse failure. Acceptable today; defer.

## New findings in pass 2

### Finding A (nit) — stale "full unannotated set" listing in the inventory

[internal_docs/diagnostic_emission_inventory.md:153](internal_docs/diagnostic_emission_inventory.md:153) prose was updated from "88" to "86", which matches the current `find … | xargs grep -L 'expect-error'` count (86 of 232 fail fixtures). However, the explicit unannotated-fixture listing in the `text` code block further down (between [internal_docs/diagnostic_emission_inventory.md:165](internal_docs/diagnostic_emission_inventory.md:165) and the closing fence) still contains 88 entries. Two of them — `own_parameter_method_mutation_requires_mut.sifr` and `own_parameter_mutation_requires_mut.sifr` — are no longer unannotated; both already carry `# expect-error: SIFR-OWN-0005` in this slice (and were apparently annotated upstream by [PR #1689](https://github.com/sifr-lang/sifr/pull/1689)). Verified by `grep` returning the markers and `git log` showing the addition predates this slice.

The slice already touches the same paragraph block. While the file is under edit, drop those two filenames from the listing so the count and the listing agree.

### Finding B (nit) — smoke fuzz coverage premise is now near-vacuous

`test_smoke_fuzz_expectation_extractors_no_panic` ([crates/sifr/tests/e2e.rs:3155-3174](crates/sifr/tests/e2e.rs:3155)) was originally meaningful with the lenient `extract_expect_errors`, which never panicked. Pass 2 swaps the call to `extract_compile_failure_expectations`, which panics on any line that starts with `# expect-error[…]:` or `# expect-error:` and fails validation. The smoke fuzz only ever appends one known-valid `\n# expect-error: SIFR-TYPE-0002` and otherwise generates random ASCII from the alphabet `\n # : <a-z>`. The probability that random ASCII spells `# expect-error:` followed by an invalid payload across 120 chars and 512 iterations is effectively zero, so the test is now near-tautological for the strict parser leg.

This is not a correctness regression — `test_expected_error_contract_rejects_malformed_grammar` covers the negative space deterministically. It's a premise change worth either:

- replacing the strict-parser fuzz call with `parse_expect_error_line` against deterministic invalid samples and asserting the right error categorization (which is the behavior the smoke test was approximating); or
- documenting the function as "do not panic on payloads where validation succeeds" so the test name matches reality.

The Unicode sibling test ([crates/sifr/tests/e2e.rs:3176-3190](crates/sifr/tests/e2e.rs:3176)) only feeds known-valid markers and is unaffected.

### Finding C (nit) — inventory phrasing for `[col=N]:`

[internal_docs/diagnostic_emission_inventory.md:136](internal_docs/diagnostic_emission_inventory.md:136) describes `expect-error[col=<1-based-column>]: SIFR-<FAMILY>-dddd` as "for future span-backed disambiguation". It's accepted by the parser today (Finding 4 shows the parser tests for it), so "future" understates its present availability. Recommend "for span-backed disambiguation when one source line intentionally expects multiple diagnostics" — mirrors the issue's grammar declaration at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1017-1019](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1017).

### Finding D (nit, pre-existing) — `_expected_stderr` is dead state in the same pattern as `_expected_errors`

The Finding 6 cleanup deleted `_expected_errors`. The sibling `_expected_stderr: Vec<String>` field on `FixtureCase` ([crates/sifr/tests/e2e.rs:76](crates/sifr/tests/e2e.rs:76)) is still set in `discover_fixtures` ([crates/sifr/tests/e2e.rs:913](crates/sifr/tests/e2e.rs:913)) via `extract_expect_stderr(&source)` and never read; the `expected_stderr` local at [crates/sifr/tests/e2e.rs:2707](crates/sifr/tests/e2e.rs:2707) is a separate variable in `test_e2e_runtime_fail` that re-extracts from source. So the field is dead the same way `_expected_errors` was — set but never consumed. Pre-existing, unrelated to slice 1's stated scope; flagging only because the same cleanup pattern just shipped one field over.

If this is intentional (e.g. reserved for a future runtime-fail fast path), the underscore naming convention encodes that. Keep or delete — either is fine; just calling out the parallel.

## Verification of slice DoD coverage

Mapped against the milestone DoD at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1023-1031](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1023):

- "Tests cannot accidentally bless message-embedded pseudo-codes" — covered by the message-substring rejection (`':' || whitespace` guard) and `test_expected_error_contract_rejects_messages_legacy_and_unknown_codes`. ✓
- "E2E fail fixtures must assert only top-level code strings" — `rg "^# expect-error" crates/sifr/tests/e2e/fail -g '*.sifr' | wc -l` = 155 markers, all of the form `# expect-error: SIFR-<FAMILY>-dddd`. None contain `[E`, `:` after the code, or message text. ✓
- "Harness no longer normalizes or extracts secondary codes" — `diagnostic_error_code`/`is_message_error_code`/`normalize_error_code` deleted. `compile_source` produces exactly one `CompiledFailure` per emitted `RenderedDiagnostic`. ✓
- "Expectation grammar accepts canonical top-level codes only and rejects message substrings, unknown forms, and unknown registry codes" — covered by `validate_expected_error_code` and the three negative tests. ✓
- "No transitional `[Edddd]` expectation remains" — `rg '\[E\d' crates/sifr/tests/e2e/fail crates/sifr/tests/e2e.rs -g '*.sifr' -g '*.rs'` reports zero markers. ✓
- "this milestone must not introduce new `SIFR-TYPE-0001` expectations" — none introduced; `rg 'SIFR-TYPE-0001' crates/sifr/tests/e2e/fail` is empty. ✓

DoD items not in scope for slice 1 (baseline normalization, duplicate-baseline detection, contradictory-expectation detection, JSON/compact/human renderer fixture-level test) remain for follow-up slices, consistent with the slice's "harness expectation grammar" framing.

## Net regressions vs. main

- None observed. Pass 1's net regression list is now empty: the duplicate-code coverage hole is closed by consume-on-match + expanded fixtures, and the column path has parser-level + matcher-level (synthetic) coverage. Integration coverage of the column path is the only residual gap.

## Recommended action plan for pass 3

Should-fix in this slice:

1. Add at least one fail fixture exercising `[col=N]:` end-to-end (Finding 2 — the only carryover from pass 1's must-fix list).

Nits, comfortable to defer to a follow-up if scope-bound:

2. Drop `own_parameter_method_mutation_requires_mut.sifr` and `own_parameter_mutation_requires_mut.sifr` from the unannotated fixture listing in [internal_docs/diagnostic_emission_inventory.md](internal_docs/diagnostic_emission_inventory.md) so the listing matches the prose count of 86 (Finding A).
3. Either replace the strict-parser leg of `test_smoke_fuzz_expectation_extractors_no_panic` with deterministic negative samples or rename to reflect that the strict parser is contractually allowed to panic (Finding B).
4. Re-phrase "for future span-backed disambiguation" in the inventory grammar paragraph (Finding C).
5. Optionally delete `_expected_stderr` if not reserved for a planned use (Finding D).

Local validation status: not run (review brief said "do not modify files"). Recommend `scripts/run_all_tests.sh --profile quick` plus `cargo test -p sifr --test e2e` (especially `test_expected_error_contract_*`, `test_failure_matching_consumes_failures_and_honors_columns`, `test_e2e_fail`, and `test_decimal_fail_fixtures_do_not_emit_legacy_pseudo_codes`) before opening the PR.
