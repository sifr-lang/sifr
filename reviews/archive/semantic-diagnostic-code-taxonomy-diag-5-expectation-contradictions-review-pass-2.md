# milestone_diag_5 slice 3 review (pass 2)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-expectation-contradictions` against `origin/main`, layered on top of [reviews/semantic-diagnostic-code-taxonomy-diag-5-expectation-contradictions-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-expectation-contradictions-review-pass-1.md). Slice intent (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76)): "add e2e fixture expectation contradiction detection so overlapping `expect-error` assertion locations cannot claim incompatible diagnostic codes, and load all fail-fixture expectation contracts before compiling the fail corpus."

Files in scope:

- [crates/sifr/tests/e2e.rs](crates/sifr/tests/e2e.rs) — added `LocatedCompileFailureExpectation`, `expectation_locations_overlap`, `expectation_location_label`, `validate_expectation_contradictions`, `parse_compile_failure_expectations`, `format_expectation_contract_errors`; rewired `extract_compile_failure_expectations` to delegate to the non-panicking parser; rewired `test_e2e_fail` to preload every `(path, source, expected)` triple before compiling and to accumulate every fixture's contract errors before panicking; added `test_expected_error_contract_rejects_contradictory_overlapping_locations`.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) — slice 3 status line at [:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) is still in progress (`[ ]`), correctly framed.

Out-of-scope DoD bullets explicitly carried forward to later slices: centralized baseline normalization ([:1011](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1011)) and the JSON/compact/human renderer fixture-level test ([:1023, :1033](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1023)). No regression in scope from prior slices.

## Verdict

**Approve.** All five pass-1 findings (A must-fix, B/C should-fix, D/E nits) are resolved. Local validation on the affected tests passes (`cargo test -p sifr --test e2e -- --skip test_e2e_pass --skip test_e2e_fail` → 30 passed; `cargo test -p sifr --test e2e test_e2e_fail` → 1 passed). The slice now actually enforces the contradiction contract on real input and front-loads contract validation across the entire fail corpus.

One semantic note worth recording (not a blocker): the current `expectation_locations_overlap` interprets a `None` column as "this marker does not claim a specific assertion point and therefore cannot contradict anything," which is the opposite of pass 1's Resolution-1 sketch ("None covers any column"). The implementer's reading is the more defensible one — see Semantic note below — but it differs from the brief's accompanying description and from how pass 1 phrased the fix.

## Pass 1 findings — disposition

### Finding A (must-fix) — same-source-line gate makes the contradiction detector inert. RESOLVED.

`expectation_locations_overlap` at [crates/sifr/tests/e2e.rs:642-651](crates/sifr/tests/e2e.rs:642) no longer mentions `line_number`. Overlap is decided in column space:

```rust
match (left.expectation.column, right.expectation.column) {
    (Some(left_column), Some(right_column)) => left_column == right_column,
    _ => false,
}
```

The accompanying comment at [:644-645](crates/sifr/tests/e2e.rs:644) documents the choice: "Unqualified markers assert code existence only; they do not claim every column. Contradictions require both markers to name the same explicit assertion point." `line_number` is preserved on `LocatedCompileFailureExpectation` only as a label for the diagnostic message produced at [:670-682](crates/sifr/tests/e2e.rs:670), not as a gating field.

The detector now fires on real input. The new extractor-level regression at [crates/sifr/tests/e2e.rs:3067-3075](crates/sifr/tests/e2e.rs:3067) drives `parse_compile_failure_expectations` against the literal source `"# expect-error[col=4]: SIFR-TYPE-0002\n# expect-error[col=4]: SIFR-NAME-0001\n"` — two markers on consecutive comment lines (line 1 and line 2 of the synthetic source) — and asserts the returned `Err` names both `marker line 1`, `marker line 2`, and `for column 4`. That is precisely the failure mode pass 1 said was unreachable, now reachable.

### Finding B (should-fix) — unit test does not cover the path through `extract_compile_failure_expectations`. RESOLVED.

`test_expected_error_contract_rejects_contradictory_overlapping_locations` at [crates/sifr/tests/e2e.rs:3041-3142](crates/sifr/tests/e2e.rs:3041) now exercises three distinct surfaces:

1. The pure pair-checker (`validate_expectation_contradictions`) on hand-built `LocatedCompileFailureExpectation` values for the same-column case, the disjoint-columns case, the unqualified-vs-explicit case, and the same-code-different-columns case.
2. The production extractor (`parse_compile_failure_expectations`) against a literal two-line source string, asserting the returned error names both source-line numbers and the explicit column.
3. The accumulation behaviour (`parse_compile_failure_expectations` against a four-marker source with two distinct conflicting pairs at columns 4 and 9), asserting `errors.len() == 2`.

Together these cover the full extractor path end-to-end through real source bytes. A future refactor that disconnected the validator from the extractor would now fail item 2.

### Finding C (should-fix) — front-loading short-circuits on the first contradictory fixture. RESOLVED.

`parse_compile_failure_expectations` at [crates/sifr/tests/e2e.rs:693-732](crates/sifr/tests/e2e.rs:693) is non-panicking and returns `Result<Vec<CompileFailureExpectation>, Vec<String>>`. It accumulates per-line parse errors *and* the full set of contradiction errors before returning `Err`.

`test_e2e_fail` at [crates/sifr/tests/e2e.rs:2733-2748](crates/sifr/tests/e2e.rs:2733) calls the non-panicking parser inside the preload loop, accumulates every fixture's `Err` into `contract_errors`, and panics exactly once at [:2741-2746](crates/sifr/tests/e2e.rs:2741) with the joined list. Five fixtures with contradictions would now surface in a single run instead of five fix-and-rerun cycles.

The synchronous panic for legacy callers (`extract_compile_failure_expectations` at [crates/sifr/tests/e2e.rs:742-748](crates/sifr/tests/e2e.rs:742)) is preserved by `unwrap_or_else(panic!)`, so the smoke fuzz at [crates/sifr/tests/e2e.rs:3429](crates/sifr/tests/e2e.rs:3429) and unicode test at [:3445](crates/sifr/tests/e2e.rs:3445) keep their existing semantics unchanged.

### Finding D (nit) — error string redundancy when both markers share the same labelled location. RESOLVED.

The panic body at [crates/sifr/tests/e2e.rs:670-682](crates/sifr/tests/e2e.rs:670) collapses the column suffix when the two location labels are identical:

- Same column: `"… for column 4"` (no trailing duplicate).
- Different columns or one unqualified: `"… for column 4 overlapping any column"` / `"… for column 4 overlapping column 9"`.

Marker-line attribution is kept on each side of `conflicts with` because real fixtures necessarily place each marker on its own source line, so the per-marker line number is informative rather than redundant. The unit test at [:3060-3065](crates/sifr/tests/e2e.rs:3060) asserts `for column 4` (singular) and `marker line 12` (each side), which is exactly the de-duplicated shape pass 1 asked for.

### Finding E (nit) — quadratic pair-walk + early-return reports only one contradiction per fixture. RESOLVED.

`validate_expectation_contradictions` at [crates/sifr/tests/e2e.rs:659-691](crates/sifr/tests/e2e.rs:659) walks every `(left, right)` pair and pushes every conflict into `errors`, returning `Err(errors)` only after the full sweep. The 4-marker fixture in the new unit test ([:3077-3087](crates/sifr/tests/e2e.rs:3077)) verifies both pairs are surfaced (`assert_eq!(multiple_errors.len(), 2)`).

## Semantic note — None vs. column overlap

The brief I was handed says the resolved semantics are *"None covers any column, Some col overlaps same col."* The code on disk says the opposite for `None`:

```rust
match (left.expectation.column, right.expectation.column) {
    (Some(left_column), Some(right_column)) => left_column == right_column,
    _ => false,
}
```

That is, two unqualified markers (or one unqualified and one explicit) **never** trigger a contradiction. The unit test makes this explicit at [crates/sifr/tests/e2e.rs:3089-3105](crates/sifr/tests/e2e.rs:3089) (`unqualified_marker_does_not_claim_column` → `is_ok()`).

I think the implementer's reading is the right one for this slice and the current grammar, for three reasons:

1. **Existing fail-corpus fixtures depend on it.** Every fixture in [crates/sifr/tests/e2e/fail/](crates/sifr/tests/e2e/fail/) with multiple `# expect-error` markers uses unqualified markers (`bounded_multi_error_recovery.sifr`, `bounded_multi_error_recovery_repeated_type_errors.sifr`). Today they all happen to assert the same code, so under either reading the validator stays silent. But the unit-level test `test_expectation_parsing_contract` at [crates/sifr/tests/e2e.rs:2937-2973](crates/sifr/tests/e2e.rs:2937) — which predates this slice on `origin/main` — uses three unqualified-or-explicit markers with three distinct codes (`SIFR-PARSE-0002`, `SIFR-TYPE-0002`, `SIFR-DECIMAL-0007[col=7]`). Under "None covers any column" the validator flags three contradictions in that test and panics; I confirmed this empirically against an earlier draft of the working tree where the overlap function returned `true` on `None`. Under the shipped semantics the test passes.

2. **Author intuition.** `# expect-error: SIFR-TYPE-0002` reads as "somewhere in this fixture there must be a TYPE-0002 diagnostic," not "every column of this fixture asserts TYPE-0002." Treating it as the latter manufactures a contradiction whenever an author writes two distinct unqualified expectations on the same fixture — which is a normal pattern for fixtures that emit two unrelated diagnostics.

3. **Matcher fidelity.** `failure_matches_expectation` at [crates/sifr/tests/e2e.rs:853-862](crates/sifr/tests/e2e.rs:853) plus the consume-once loop at [:864-878](crates/sifr/tests/e2e.rs:864) means two `(None, codeA)` and `(None, codeB)` markers can never be "claimed" by a single emitted diagnostic — they target disjoint code-keyed pools. So they don't overlap in any operational sense. The pass-1 sketch would have over-fired on a perfectly well-formed fixture.

The trade-off is that an author who *means* to pin "this fixture must emit exactly one diagnostic at column 5 with code A" cannot express that with an unqualified marker today; they must add `[col=…]`. That's consistent with the slice 1 grammar tightening and feels like the correct push, not a regression.

The brief's "None covers any column" wording should be considered out of date; the code's "Contradictions require both markers to name the same explicit assertion point" comment at [:644-645](crates/sifr/tests/e2e.rs:644) is authoritative.

## Contract verification

1. **Front-loading of fail-fixture contracts.** `test_e2e_fail` reads, parses, and validates every fail-corpus path inside [crates/sifr/tests/e2e.rs:2730-2738](crates/sifr/tests/e2e.rs:2730) and only enters the compile loop at [:2747](crates/sifr/tests/e2e.rs:2747) once *all* contract errors have been accumulated and (if any) reported via the pre-compile panic at [:2740-2744](crates/sifr/tests/e2e.rs:2740). No `compile_source` runs while any fixture's contract is unparsed or invalid. ✓
2. **Accumulation rather than first-error abort.** Every fixture's parse errors are collected into `contract_errors`; every fixture's contradictions are collected inside `parse_compile_failure_expectations`. The full picture surfaces in one panic. ✓
3. **Detection runs on real fixture shapes.** The grammar still constrains `# expect-error[...]:` markers to whole comment lines ([:611-639](crates/sifr/tests/e2e.rs:611)), so two markers always sit on distinct source lines. Because overlap is now line-agnostic, this is fine — the test at [:3067-3075](crates/sifr/tests/e2e.rs:3067) demonstrates the production extractor catching contradictions across lines 1 and 2. ✓
4. **Failure mode is fixture-scoped and informative.** `parse_compile_failure_expectations` prepends `<fixture_path>:` to every contradiction error at [:720-724](crates/sifr/tests/e2e.rs:720), so the eventual panic banner names each offending fixture by path and lists every conflicting pair within it. `format_expectation_contract_errors` at [:734-740](crates/sifr/tests/e2e.rs:734) then prefixes each line with `FAIL` for grep-friendly output. ✓
5. **Issue status truthfulness.** [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) is still `[ ] in progress` with the right two-clause framing. It can flip to `[x]` once this review is signed off and a PR is opened. ✓

## Validation status

I ran (review-only, no edits):

- `cargo test -p sifr --test e2e -- --skip test_e2e_pass --skip test_e2e_fail` → `30 passed; 0 failed`. Covers `test_expected_error_contract_rejects_contradictory_overlapping_locations`, `test_expectation_parsing_contract`, the rest of the harness contract suite, and the smoke fuzz at [:3413-3431](crates/sifr/tests/e2e.rs:3413) which keeps appending one `# expect-error: SIFR-TYPE-0002` per sample (cannot construct contradictions, still passes).
- `cargo test -p sifr --test e2e test_e2e_fail` → `1 passed`. The full fail corpus preloads cleanly and every fixture's compile-time matching still passes.

I did not run `scripts/run_all_tests.sh --profile quick`; that gate is required by [AGENTS.md](AGENTS.md) before merge but is out of scope for a review-only sweep on a working tree this small.

## Summary

- **Finding A**: resolved — overlap is now column-only with `None` interpreted conservatively, validator fires on real input.
- **Finding B**: resolved — extractor-level regression added with real consecutive marker lines.
- **Finding C**: resolved — non-panicking parser + per-fixture accumulation in `test_e2e_fail`.
- **Finding D**: resolved — column suffix collapses on identical labels.
- **Finding E**: resolved — every conflicting pair is reported per fixture.

Slice 3 is reviewer-satisfied. Recommend opening the PR. The brief's wording on `None` overlap should be updated to match the shipped semantics (or the implementer should call out the divergence in the PR description) so the next reviewer doesn't go looking for a contradiction that isn't there.
