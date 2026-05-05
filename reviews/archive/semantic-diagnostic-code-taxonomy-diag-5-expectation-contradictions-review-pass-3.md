# milestone_diag_5 slice 3 review (pass 3)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-expectation-contradictions` against `origin/main`, layered on top of [reviews/semantic-diagnostic-code-taxonomy-diag-5-expectation-contradictions-review-pass-2.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-expectation-contradictions-review-pass-2.md). Slice intent (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76), updated this pass): "add e2e fixture expectation contradiction detection so overlapping explicit `expect-error[col=N]` assertion locations cannot claim incompatible diagnostic codes, while unqualified markers continue to assert code existence only; load all fail-fixture expectation contracts before compiling the fail corpus."

Files in scope (uncommitted diff only):

- [crates/sifr/tests/e2e.rs](crates/sifr/tests/e2e.rs) — `LocatedCompileFailureExpectation`, `expectation_locations_overlap`, `expectation_location_label`, `validate_expectation_contradictions`, non-panicking `parse_compile_failure_expectations`, thinned `extract_compile_failure_expectations`, accumulator wiring in `test_e2e_fail`, and `test_expected_error_contract_rejects_contradictory_overlapping_locations`.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) — slice 3 in-progress status line at [:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) reworded to specify the new (narrower) contract.

Out-of-scope DoD bullets explicitly carried forward to later slices remain the same as pass 1/2: centralized baseline normalization and the JSON/compact/human renderer fixture-level test.

## Verdict

**Satisfied — no must-fix blockers.** All three pass-2 findings (F must-fix, G should-fix/discuss, H nit) are resolved cleanly. The author chose pass-2 finding F's resolution path 2 — relax the contract so unqualified `# expect-error: …` markers no longer "claim" a column for contradiction purposes — and propagated that decision through:

1. The validator at [crates/sifr/tests/e2e.rs:642-652](crates/sifr/tests/e2e.rs:642), now `(Some(l), Some(r)) => l == r, _ => false` with an inline two-line comment explaining the chosen reading.
2. The unit test at [crates/sifr/tests/e2e.rs:3047-3148](crates/sifr/tests/e2e.rs:3047), updated with `unqualified_marker_does_not_claim_column` (lines 3095-3111) and `disjoint_columns` (3113-3129) as positive negative-cases that pin reading B against accidental drift back to reading A.
3. The issue status line at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76), rewritten to "explicit `expect-error[col=N]` assertion locations cannot claim incompatible diagnostic codes, while unqualified markers continue to assert code existence only," capturing the contract decision durably.

The `test_expectation_parsing_contract` regression that blocked pass 2 is gone — the test now passes against the working tree.

Local re-run on the working tree:

- `cargo fmt --check` → clean (no output).
- `git diff --check` → clean (no whitespace errors).
- `cargo test -p sifr --test e2e test_expectation_parsing_contract` → `1 passed; 0 failed`.
- `cargo test -p sifr expected_error_contract` → `4 passed; 0 failed` (includes the new contradiction test).
- `cargo test -p sifr --test e2e test_e2e_fail` → `1 passed; 0 failed` in 1.91s.
- `cargo test -p sifr failure_matching_consumes` → `1 passed; 0 failed`.
- `cargo test -p sifr smoke_fuzz_valid_expectation_extractors` → `1 passed; 0 failed`.
- `cargo test -p sifr --test e2e -- --skip test_e2e_pass` → `31 passed; 0 failed` in 16.62s.

The unrelated `test_e2e_pass` codegen failure noted in pass 2 (closure mutability for `nested_function_nonlocal_accumulator.sifr`) was not transited by this run because it is filtered by `--skip test_e2e_pass`; that issue is independent of slice 3 and remains for a separate task.

## Pass 2 follow-up status

### Finding F (must-fix) — RESOLVED

`expectation_locations_overlap` at [crates/sifr/tests/e2e.rs:642-652](crates/sifr/tests/e2e.rs:642) now reads:

```
match (left.expectation.column, right.expectation.column) {
    (Some(left_column), Some(right_column)) => left_column == right_column,
    _ => false,
}
```

i.e. only `(Some(c), Some(c))` overlaps; `None` markers are silent about location and never participate in contradiction detection. This is the narrower reading B from pass-2 finding G, adopted as the slice's official contract.

`test_expectation_parsing_contract` at [crates/sifr/tests/e2e.rs:2944-2979](crates/sifr/tests/e2e.rs:2944) is unchanged in shape and now passes — its three-marker mix `(SIFR-PARSE-0002, None) + (SIFR-TYPE-0002, None) + (SIFR-DECIMAL-0007, Some(7))` produces zero contradictions because no marker pair is `(Some, Some)` with equal columns.

The new contradiction unit test at [crates/sifr/tests/e2e.rs:3047-3148](crates/sifr/tests/e2e.rs:3047) was inverted to match: `unqualified_marker_does_not_claim_column` (None × Some(9), distinct codes) is now asserted `is_ok()` at [:3111](crates/sifr/tests/e2e.rs:3111), the inverse of pass 2's expectation. The same-column distinct-code case at [:3048-3071](crates/sifr/tests/e2e.rs:3048) remains the canonical positive flagging case.

The narrowing also obviates the "all-None markers in a fail fixture trigger N·(N-1)/2 contradictions" quadratic-blowup caveat from pass 2 finding E — under reading B, all-None marker fixtures now never trigger contradictions, so the quadratic risk only matters for `[col=N]`-heavy fixtures (currently zero such fixtures in the fail corpus). ✓

### Finding G (should-fix / discuss) — RESOLVED

Reading B selected and documented:

- Source-code documentation: [crates/sifr/tests/e2e.rs:646-647](crates/sifr/tests/e2e.rs:646) carries a two-line comment immediately above the `match` branches stating "Unqualified markers assert code existence only; they do not claim every column. Contradictions require both markers to name the same explicit assertion point." A future fixture author opening the validator will see the rule before they read the code.
- Issue documentation: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) now reads "overlapping explicit `expect-error[col=N]` assertion locations cannot claim incompatible diagnostic codes, while unqualified markers continue to assert code existence only," anchoring the contract in the slice's planning record.
- Test documentation: the negative-case ordering and naming (`unqualified_marker_does_not_claim_column` first, then `disjoint_columns`, then `repeated_same_code`) makes the contract self-evident from the test body.

The chosen reading aligns the contradiction validator with matcher semantics (the matcher at [crates/sifr/tests/e2e.rs:864-878](crates/sifr/tests/e2e.rs:864) is greedy first-match, so two `None` markers with distinct codes are never matcher-contradictory either). The slice no longer over-reaches relative to what the matcher itself rejects. ✓

### Finding H (nit) — RESOLVED

`parse_compile_failure_expectations` at [crates/sifr/tests/e2e.rs:722-728](crates/sifr/tests/e2e.rs:722) now path-prefixes contradiction strings before appending them:

```
errors.extend(
    contradiction_errors
        .into_iter()
        .map(|error| format!("{}: {error}", fixture_path.display())),
);
```

So a multi-fixture contradiction failure surfaces as e.g.

```
FAIL fixture/a.sifr: contradictory expect-error markers: SIFR-… at marker line 5 conflicts with …
FAIL fixture/b.sifr: contradictory expect-error markers: SIFR-… at marker line 7 conflicts with …
```

Each contradiction line now self-attributes to a fixture, matching the spirit (if not the exact byte-shape) of the parse-error format `<path>:<line> invalid expect-error marker: <reason>`. The two shapes differ in the `:<line>` suffix because contradiction strings already embed `marker line N` in the body, which is fine. Not a blocker, intentional, and the operator UX target from finding H is met. ✓

## New observations (non-blocking)

### Observation I — fail corpus has no `[col=N]` markers, so the validator is currently exercised only in unit tests

`grep -rcE "expect-error\[col=" crates/sifr/tests/e2e/fail/` returns zero hits across all 232 fail fixtures. Under reading B, contradictions require `(Some, Some)` overlap, so no production fail fixture can currently trigger the validator. This is the deliberate consequence of pass 2's recommended narrowing — the validator exists to prevent *future* `[col=N]`-heavy fixtures from making contradictory claims, not to retro-flag the current corpus.

The slice still meets its stated intent (the validator is wired into the fail-corpus contract loader, accumulates across fixtures, and is exercised through `parse_compile_failure_expectations` in the unit test at [crates/sifr/tests/e2e.rs:3073-3093](crates/sifr/tests/e2e.rs:3073), including the four-marker accumulation case). No action needed — flagging only so the next slice author knows that adding a deliberately-contradictory fail fixture is the easiest way to add live coverage if that's ever wanted.

### Observation J — explicit `None × None` distinct-code negative case missing from the new unit test

The new unit test at [crates/sifr/tests/e2e.rs:3047-3148](crates/sifr/tests/e2e.rs:3047) covers `Some(4) × Some(4)` (positive flag), `None × Some(9)` distinct-code (negative), `Some(4) × Some(9)` distinct-code (negative), and `None × Some(9)` same-code (negative). It does not include an explicit `None × None` distinct-code negative case, which is the marker shape most likely to drift back under reading A.

This case is indirectly guarded by `test_expectation_parsing_contract` (which has two `None` markers with distinct codes and now passes), so a regression toward reading A would be caught — but by a different test, in a different file region, named after parser grammar coverage rather than contradiction contract. Adding a one-line `None × None` distinct-code case to `test_expected_error_contract_rejects_contradictory_overlapping_locations` would tighten the test's self-explanatory scope. Not a blocker.

### Observation K — `extract_compile_failure_expectations` panic path no longer explicitly tested

After the refactor, `extract_compile_failure_expectations` at [crates/sifr/tests/e2e.rs:748-755](crates/sifr/tests/e2e.rs:748) is a thin wrapper that calls `parse_compile_failure_expectations` and panics with `format_expectation_contract_errors(&errors)`. The new tests exercise `parse_compile_failure_expectations` directly (returning `Result<…, Vec<String>>`) but no test exercises the `extract_compile_failure_expectations` panic path itself. This is fine — the panic call is trivial and is exercised transitively by every other expectation-consuming test that hits the happy path — but worth noting that the explicit panic-message contract is now covered only by integration through `test_e2e_fail`'s panic-on-contract-violation path, not by a dedicated unit test. Not a blocker.

## Validation reproduction

| Command | Result |
| --- | --- |
| `cargo fmt --check` | clean |
| `git diff --check` | clean |
| `cargo test -p sifr --test e2e test_expectation_parsing_contract` | `1 passed; 0 failed` |
| `cargo test -p sifr expected_error_contract` | `4 passed; 0 failed` |
| `cargo test -p sifr --test e2e test_e2e_fail` | `1 passed; 0 failed` (1.91s) |
| `cargo test -p sifr failure_matching_consumes` | `1 passed; 0 failed` |
| `cargo test -p sifr smoke_fuzz_valid_expectation_extractors` | `1 passed; 0 failed` |
| `cargo test -p sifr --test e2e -- --skip test_e2e_pass` | `31 passed; 0 failed` (16.62s) |

Did not run `scripts/run_all_tests.sh --profile quick` (review-only). Recommend running it before merging per [AGENTS.md](AGENTS.md), as the user noted in the brief.

## Summary

- **Pass 2 must-fix F**: resolved by adopting reading B (narrow contract). `test_expectation_parsing_contract` passes.
- **Pass 2 should-fix G**: resolved by documenting reading B in three places (validator comment, issue status line, unit-test naming).
- **Pass 2 nit H**: resolved by path-prefixing each contradiction error string at the parse-aggregator boundary.
- **Observation I**: the validator currently has no live fail-corpus coverage because zero fixtures use `[col=N]` markers — deliberate consequence of reading B; non-blocker.
- **Observation J**: `None × None` distinct-code is covered indirectly by `test_expectation_parsing_contract`; adding an explicit case to the new contradiction test would tighten self-containment but is non-blocking.
- **Observation K**: `extract_compile_failure_expectations`'s panic-message contract is exercised transitively, not by a dedicated unit test; non-blocking.

Issue status line at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) is honest about scope and the chosen contract. After running `scripts/run_all_tests.sh --profile quick` and confirming green, this slice can flip to `[x]` and ship.
