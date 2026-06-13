# milestone_diag_5 slice 1 review (pass 3)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-harness-contract` against `main`, building on [reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-2.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-2.md). Slice intent unchanged: tighten e2e `expect-error` parsing to canonical registry-backed `SIFR-<FAMILY>-dddd` codes only, accept `# expect-error[col=<1-based-column>]: CODE`, reject `[Edddd]` and message-substring expectations, stop extracting secondary codes, and rewrite fail fixtures to code-only assertions.

## Status of pass 2 findings

| # | Pass 2 finding | Severity | Pass 3 status |
| --- | --- | --- | --- |
| 2 (carryover from pass 1) | `[col=N]:` not exercised end-to-end | should-fix | Substantially resolved (helper factored + integration-style unit test against a real `RenderedDiagnostic`); a narrow primary-span discrimination gap remains — see Finding A |
| 8 (carryover from pass 1) | `closest_active_diagnostic_code` per-call cost | nit | Deferred (acceptable) |
| A | Stale unannotated-fixture listing | nit | Resolved |
| B | Smoke fuzz coverage premise near-vacuous | nit | Resolved (test renamed; semantics now match) |
| C | Inventory phrasing for `[col=N]:` understated | nit | Resolved |
| D | `_expected_stderr` dead state | nit (pre-existing) | Not addressed; acceptable to defer |

### Verification of resolved findings

- **Finding 2 — column path coverage.** The `RenderedDiagnostic → CompiledFailure` conversion is now factored into `compiled_failure_from_rendered` ([crates/sifr/tests/e2e.rs:575-587](crates/sifr/tests/e2e.rs:575)) and consumed by `compile_source` ([crates/sifr/tests/e2e.rs:562-573](crates/sifr/tests/e2e.rs:562)). The new `test_rendered_diagnostic_column_is_used_for_expect_error_matching` ([crates/sifr/tests/e2e.rs:2992-3031](crates/sifr/tests/e2e.rs:2992)) constructs a real `sifr_diagnostics::RenderedDiagnostic` with a primary span carrying `column: Some(9)`, runs it through the factored helper, and asserts:
  1. `failure.column == Some(9)` (helper extracts the primary-span column).
  2. `match_compile_failure_expectations(&[expected_col9], &[failure]).is_ok()` against `# expect-error[col=9]: SIFR-TYPE-0002`.
  3. `compile_failures_to_messages` produces `"SIFR-TYPE-0002@col9: type mismatch"` (the `CODE@colN: msg` rendering branch at [crates/sifr/tests/e2e.rs:744-752](crates/sifr/tests/e2e.rs:744)).

  This closes most of pass 2 Finding 2: helper, matcher, and rendering all run against a public-type `RenderedDiagnostic`. The residual gap is narrow (see Finding A below) — small enough that it is no longer a should-fix blocker for slice 1.

- **Finding A from pass 2 (stale listing).** [internal_docs/diagnostic_emission_inventory.md:218](internal_docs/diagnostic_emission_inventory.md:218) no longer contains `own_parameter_method_mutation_requires_mut.sifr` or `own_parameter_mutation_requires_mut.sifr`. The text-block listing now contains exactly 86 entries (`awk '/^\`\`\`text$/{flag=1; next} /^\`\`\`$/{flag=0} flag' internal_docs/diagnostic_emission_inventory.md | wc -l` → 86), and `find crates/sifr/tests/e2e/fail -name '*.sifr' | xargs grep -L '# expect-error' | wc -l` → 86. Diffing the listed names against the actual unannotated set produces no differences. ✓

- **Finding B from pass 2 (smoke fuzz premise).** Test renamed to `test_smoke_fuzz_valid_expectation_extractors_no_panic` ([crates/sifr/tests/e2e.rs:3203-3221](crates/sifr/tests/e2e.rs:3203)). The body still appends one valid marker (`# expect-error: SIFR-TYPE-0002`) and exercises `extract_compile_failure_expectations` over random ASCII; the new name now correctly advertises the property under test (no-panic on validation-passing payloads), matching the renaming option offered in pass 2. The Unicode sibling test ([crates/sifr/tests/e2e.rs:3223-3237](crates/sifr/tests/e2e.rs:3223)) was also updated so its expectations are canonical (`SIFR-TYPE-0002`), keeping it consistent with the strict parser. ✓

- **Finding C from pass 2 (inventory phrasing).** [internal_docs/diagnostic_emission_inventory.md:136](internal_docs/diagnostic_emission_inventory.md:136) now reads "for span-backed disambiguation when one source line intentionally expects multiple diagnostics" — exactly the wording recommended in pass 2 and aligned with the issue at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1017](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1017). The same paragraph also tightens the harness-behavior summary: "validates `# expect-error:` markers as active registry-backed `SIFR-<FAMILY>-dddd` codes" replaces the older substring-matching description, and the count line at [internal_docs/diagnostic_emission_inventory.md:10](internal_docs/diagnostic_emission_inventory.md:10) now shows "155 fail-fixture expectations plus harness parser samples", matching `grep -c '# expect-error' crates/sifr/tests/e2e/fail/*.sifr | awk -F: '{sum+=$2} END {print sum}'` → 155. ✓

- **Finding D from pass 2 (`_expected_stderr` dead state).** Unchanged. The field is still set by `discover_fixtures` ([crates/sifr/tests/e2e.rs:919](crates/sifr/tests/e2e.rs:919)) and three test-local constructors and never read (`grep -n '\._expected_stderr' crates/sifr/tests/e2e.rs` returns no consumers). Pass 2 framed this as pre-existing and either-or; carrying it forward is acceptable. Calling it out only for tracking; not a blocker.

## New findings in pass 3

### Finding A (nit) — column test does not discriminate primary-vs-first-span selection

`test_rendered_diagnostic_column_is_used_for_expect_error_matching` ([crates/sifr/tests/e2e.rs:2992-3031](crates/sifr/tests/e2e.rs:2992)) constructs `spans: vec![…]` with a single `DiagnosticSpan` whose `is_primary: true`. The helper under test is:

```rust
diagnostic
    .spans
    .iter()
    .find(|span| span.is_primary)
    .and_then(|span| span.column)
```

A regression to `diagnostic.spans.first().and_then(|span| span.column)` (or any selector that picks index 0 unconditionally) would still pass this test, because the first and only span happens to be primary. The pass 2 worry — "If a future change picked the wrong span (e.g. first span instead of primary) … no test would notice" — is therefore only partially addressed.

Cheap strengthening (no new fixture required): in the same test, prepend a non-primary `DiagnosticSpan` with a different `column` (e.g. `column: Some(1)`, `is_primary: false`) before the existing primary span, and keep the assertion `failure.column == Some(9)`. That makes the test fail under a `.first()` regression while still validating the helper, matcher, and rendering path.

This is a should-fix nit, not a blocker. The primary-span path is structurally simple and the helper has a single call site, so the regression risk is small; meanwhile every other column-related concern from pass 1 / pass 2 (qualifier parsing, validator branches, matcher consumption with column, `CODE@colN:` rendering) is now independently covered. No fail-fixture-level integration test exists either — `grep -rn 'expect-error\[col=' crates/sifr/tests/e2e/fail/ demos/` is still empty — but with the helper factored and unit-tested against a public-type `RenderedDiagnostic`, an end-to-end fixture is no longer the cheapest cure.

### Finding B (nit) — renderer column semantic remains untested at the source-of-truth boundary

The new test fabricates the `column: Some(9)` value rather than letting `sifr_diagnostics::render` compute it from a `SourceMap`. So a regression in the renderer's column convention (a hypothetical future change from 1-based char-counted to 0-based or byte-based at [crates/sifr_diagnostics/src/render/mod.rs:60-79](crates/sifr_diagnostics/src/render/mod.rs:60)) would silently shift every harness-rendered column without breaking a harness test. This is the same boundary pass 2 flagged; the new test pushes coverage one layer closer to the source of truth (now testing the harness-side helper against the real `RenderedDiagnostic` shape) but does not yet pin the renderer's column semantic.

That pin belongs in `crates/sifr_diagnostics` rather than the harness, so I would not block slice 1 on it. Worth a follow-up: add a unit test under `sifr_diagnostics::render` that constructs a `SourceMap` for a multi-character UTF-8 line and asserts `DiagnosticSpan.column` is the 1-based char position from line start. That would close the last column-path coverage gap without dragging the harness into renderer-internal concerns.

## Verification of slice DoD coverage

Same as pass 2; rechecked:

- "Tests cannot accidentally bless message-embedded pseudo-codes" — covered by the message-substring rejection in `validate_expected_error_code` ([crates/sifr/tests/e2e.rs:676-682](crates/sifr/tests/e2e.rs:676)) and `test_expected_error_contract_rejects_messages_legacy_and_unknown_codes` ([crates/sifr/tests/e2e.rs:2884-2902](crates/sifr/tests/e2e.rs:2884)). ✓
- "E2E fail fixtures must assert only top-level code strings" — `grep -rEn '# expect-error[: ]' crates/sifr/tests/e2e/fail/*.sifr | grep -vE 'SIFR-[A-Z0-9]+-[0-9]{4}'` returns empty. All 155 markers are canonical. ✓
- "Harness no longer normalizes or extracts secondary codes" — `compile_source` produces exactly one `CompiledFailure` per `RenderedDiagnostic` via `compiled_failure_from_rendered` ([crates/sifr/tests/e2e.rs:566-571](crates/sifr/tests/e2e.rs:566)); no message-bracket reparsing. ✓
- "Expectation grammar accepts canonical top-level codes only and rejects message substrings, unknown forms, and unknown registry codes" — covered by `validate_expected_error_code` and the three negative tests at [crates/sifr/tests/e2e.rs:2884-2932](crates/sifr/tests/e2e.rs:2884). ✓
- "No transitional `[Edddd]` expectation remains" — `grep -rEn '\[E[0-9]+\]' crates/sifr/tests/e2e/fail demos` returns empty for fixture/demo expectations; the only occurrences in the harness are negative-test inputs and grammar error strings. ✓
- "this milestone must not introduce new `SIFR-TYPE-0001` expectations" — `grep -rn 'SIFR-TYPE-0001' crates/sifr/tests/e2e/fail` is empty. ✓

DoD items not in scope for slice 1 (baseline normalization, duplicate-baseline detection, contradictory-expectation detection, JSON/compact/human renderer fixture-level test) remain for follow-up slices, consistent with the slice's "harness expectation grammar" framing.

## Net regressions vs. main

None observed. Pass 3 closes pass 2's listing/wording nits and substantially closes the column-path coverage gap. The remaining residuals (primary-span discrimination, renderer-side column semantic) are narrower than what pass 2 left open and do not re-introduce any pass-1 must-fix.

## Recommended action plan for pass 4 / follow-up

There are no must-fix or should-fix blockers for slice 1 ship. The following are nits, comfortable to defer or fold into a follow-up:

1. Strengthen `test_rendered_diagnostic_column_is_used_for_expect_error_matching` by prepending a non-primary span with a different column so the test discriminates `.find(|s| s.is_primary)` from `.first()` (Finding A). One-line change, no new fixture.
2. Add a renderer-internal unit test in `crates/sifr_diagnostics/src/render` pinning `DiagnosticSpan.column` as 1-based char position (Finding B). Belongs in a follow-up rather than this slice.
3. Optionally delete `_expected_stderr` if not reserved for a planned use (pass 2 Finding D, still open).
4. Optionally precompute the active-registry list once for `closest_active_diagnostic_code` (pass 1 Finding 8, still deferred).

Local validation status: not run (review brief said "do not modify files"). Recommend `scripts/run_all_tests.sh --profile quick` plus `cargo test -p sifr --test e2e` (especially `test_expected_error_contract_*`, `test_failure_matching_consumes_failures_and_honors_columns`, `test_rendered_diagnostic_column_is_used_for_expect_error_matching`, `test_e2e_fail`, and `test_decimal_fail_fixtures_do_not_emit_legacy_pseudo_codes`) before merging.

Reviewer position: ship-ready. No remaining blocker; the two open nits can be folded into a follow-up touch-up PR or rolled into the next slice.
