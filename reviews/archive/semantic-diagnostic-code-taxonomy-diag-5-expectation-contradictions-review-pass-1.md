# milestone_diag_5 slice 3 review (pass 1)

Scope reviewed: uncommitted working-tree changes on branch `codex/diag-5-expectation-contradictions` against `main`. Slice intent (per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) and the milestone DoD bullet at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1013](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1013)): "add e2e fixture expectation contradiction detection so overlapping `expect-error` assertion locations cannot claim incompatible diagnostic codes, and load all fail-fixture expectation contracts before compiling the fail corpus."

Files in scope:

- [crates/sifr/tests/e2e.rs](crates/sifr/tests/e2e.rs) — added `LocatedCompileFailureExpectation`, `expectation_locations_overlap`, `expectation_location_label`, `validate_expectation_contradictions`; rewired `extract_compile_failure_expectations` to validate before returning; rewired `test_e2e_fail` to preload all `(path, source, expected)` triples before compiling; added `test_expected_error_contract_rejects_contradictory_overlapping_locations`.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) — added the in-progress slice 3 status line at [:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76).

Out-of-scope DoD bullets explicitly carried forward to later slices: centralized baseline normalization ([:1011](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1011)) and the JSON/compact/human renderer fixture-level test ([:1023,:1033](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1023)). Slice 1 / slice 2 reviews already documented these as carry-forward items, and slice 3's status line correctly limits its claim to contradiction detection and front-loaded contract loading.

## Verdict

**Block on must-fix A.** The contradiction-detection contract is encoded in a way that is structurally unreachable in any real fixture: every `# expect-error` marker is its own comment line, so two `LocatedCompileFailureExpectation` values produced by `extract_compile_failure_expectations` always have distinct `line_number`. Because `expectation_locations_overlap` short-circuits to `false` whenever `left.line_number != right.line_number`, the validator is a no-op for every input the harness can ever construct. The only thing the slice ships in working order is fail-corpus front-loading; the unit test passes only because it bypasses the extractor and constructs `LocatedCompileFailureExpectation` values with hand-picked colliding `line_number`s.

The remaining findings (B–E) are should-fix / nit and would harden the slice once must-fix A is resolved.

## Contract verification

Intended contract: "within one fixture, two `expect-error` annotations on overlapping spans must not assert incompatible codes for the same diagnostic location" + "load all fail-fixture expectation contracts before compiling the fail corpus."

1. **Front-loading of fail-fixture contracts.** [crates/sifr/tests/e2e.rs:2699-2706](crates/sifr/tests/e2e.rs:2699) collects `(path, source, expected)` for every fail-corpus path into a `Vec` before the per-case loop at [:2708-2733](crates/sifr/tests/e2e.rs:2708). `extract_compile_failure_expectations` is called eagerly inside `.map(...)` and `.collect::<Vec<_>>()` forces it. No `compile_source` runs before all paths have been parsed once. ✓ — within the limits of the panic-on-first-failure behaviour, see Finding C.
2. **Detection runs at harness load time.** [crates/sifr/tests/e2e.rs:707-713](crates/sifr/tests/e2e.rs:707) panics from inside `extract_compile_failure_expectations`, which is invoked during the fail-corpus preload and from the unit/smoke fuzz tests. So the check runs before any compilation. ✓ on placement; effectively a no-op for fixtures, see Finding A.
3. **"Same diagnostic location" semantics.** This is where the implementation diverges from the contract. The matcher in `failure_matches_expectation` ([crates/sifr/tests/e2e.rs:822-831](crates/sifr/tests/e2e.rs:822)) compares only `(code, optional column)` — the diagnostic's source line is captured in the rendered span ([crates/sifr/tests/e2e.rs:3152](crates/sifr/tests/e2e.rs:3152)) but never read by the matcher. The expectation grammar at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1014-1021](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1014) likewise has no line qualifier, only an optional `col=`. So the only "location" the grammar can talk about is "(any line, optional column)." But the validator gates on `marker_source_line`, which is something else entirely — see Finding A. ✗
4. **Failure mode is a panic with a fixture-scoped message.** [crates/sifr/tests/e2e.rs:707-713](crates/sifr/tests/e2e.rs:707) panics with `FAIL <path> invalid expect-error marker: <error>`. The error body produced at [:670-676](crates/sifr/tests/e2e.rs:670) names both colliding markers and their labels (line/col). Reasonable diagnostic shape if the check ever fired. ✓
5. **Issue status truthfulness.** [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) declares the slice in progress with the right two-clause framing. ✓ (subject to Finding A being resolved before the line is flipped to `[x]`).

## Findings

### Finding A (must-fix) — same-source-line gate makes the contradiction detector inert in production

`expectation_locations_overlap` at [crates/sifr/tests/e2e.rs:642-653](crates/sifr/tests/e2e.rs:642) requires `left.line_number == right.line_number` before any column comparison. `line_number` is set at [crates/sifr/tests/e2e.rs:700-703](crates/sifr/tests/e2e.rs:700) as `line_index + 1` of the *comment line containing the marker*, not the line of the diagnostic the marker is supposed to assert against.

Two structural facts make this gate impossible to satisfy from real input:

1. `parse_expect_error_line` ([crates/sifr/tests/e2e.rs:611-640](crates/sifr/tests/e2e.rs:611)) requires the source line to *start* with `# expect-error:` or `# expect-error[`. Inline trailing markers on a code line (e.g. `a: int = "a"  # expect-error: SIFR-TYPE-0002`) do not match either prefix and are silently ignored. So a marker can only live on its own dedicated comment line.
2. `extract_compile_failure_expectations` walks `source.lines().enumerate()` ([:687-690](crates/sifr/tests/e2e.rs:687)) and produces at most one `LocatedCompileFailureExpectation` per source line. The `line_number` written into the value is exactly the source line index of that comment.

Conclusion: the multiset of `line_number` values produced by `extract_compile_failure_expectations` is *guaranteed* to have all-distinct entries. `expectation_locations_overlap` therefore returns `false` for every pair of real markers. `validate_expectation_contradictions` walks every pair, never finds an overlap, and always returns `Ok(())`. The "FAIL <path> invalid expect-error marker" panic at [:707-713](crates/sifr/tests/e2e.rs:707) is unreachable from the fail corpus.

You can also see the symptom at the unit-test level: `test_expected_error_contract_rejects_contradictory_overlapping_locations` at [crates/sifr/tests/e2e.rs:3003-3079](crates/sifr/tests/e2e.rs:3003) constructs `LocatedCompileFailureExpectation` values directly with hand-picked colliding `line_number`s (12/12, 5/5, 5/5, 8/8). It never goes through `extract_compile_failure_expectations`, so it cannot expose the gating problem. The test asserts only that the validator's pure pair-checker behaves correctly *given inputs the production extractor cannot produce*.

The slice's "overlapping `expect-error` assertion locations" contract has no other plausible reading under the current grammar:

- The grammar at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1014-1021](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1014) defines a marker as `(canonical code, optional 1-based column)`. There is no line qualifier.
- `failure_matches_expectation` ([crates/sifr/tests/e2e.rs:822-831](crates/sifr/tests/e2e.rs:822)) matches solely on `(code, optional column)`. The diagnostic's source line is observable on the rendered span (`line: Some(2)` in the unit test fixture at [:3152](crates/sifr/tests/e2e.rs:3152)) but it is dropped on the floor.

So the only span dimensions a marker can disagree on are `(code, column-or-None)`. "Overlap" should be defined in those dimensions, not in the marker's *own comment-line position*. Two reasonable resolutions:

1. **Drop the line gate.** Compare every pair of markers in the fixture by `(column == column) ∨ (left.column.is_none()) ∨ (right.column.is_none())`. Two markers that conflict on column-space are flagged regardless of where the comments sit in the source. This matches the matcher's actual semantics ("line-agnostic, column-aware") and is the smallest change that makes the slice's contract enforceable.
2. **Add a `line=N` qualifier to the grammar.** Extend `parse_expect_error_line` to accept `expect-error[line=N,col=M]:` (or `[line=N]:`), wire `line` into both `failure_matches_expectation` and `validate_expectation_contradictions`, and only then keep the same-line gate. This is a larger change, partially overlaps with the duplicate-discrimination follow-up flagged by [reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-5-harness-contract-review-pass-1.md) ("Finding 1"), and arguably belongs in its own slice — but it is the only way to make `line_number` mean something at the matcher level.

Either resolution requires a corresponding e2e regression: a fixture preloaded with two contradictory markers that *would* be authored in real life (i.e. on consecutive comment lines), so the panic path is exercised end-to-end. As shipped, neither the unit test nor any fail-corpus fixture demonstrates the validator firing.

Until one of those resolutions lands, the slice's behavioural claim ("overlapping assertion locations cannot claim incompatible diagnostic codes") is not actually enforced.

### Finding B (should-fix) — unit test does not cover the path through `extract_compile_failure_expectations`

`test_expected_error_contract_rejects_contradictory_overlapping_locations` ([crates/sifr/tests/e2e.rs:3003-3079](crates/sifr/tests/e2e.rs:3003)) calls `validate_expectation_contradictions` directly with synthetic `LocatedCompileFailureExpectation` values. It does not exercise the production wiring at [:707-713](crates/sifr/tests/e2e.rs:707), so even if the gate at [:642-653](crates/sifr/tests/e2e.rs:642) were correct (Finding A), a future refactor that disconnects the validator from the extractor would not be caught.

Add a test that drives `extract_compile_failure_expectations` against an in-memory `&str` source containing genuinely contradictory markers, asserting (a) it panics, and (b) the panic message names the fixture path and both markers. This is essentially the same shape as `test_expected_error_contract_rejects_messages_legacy_and_unknown_codes` but for the contradiction case.

This finding becomes the *primary* reproducer for Finding A: such a test, written correctly today, would fail (the extractor panics on no real input).

### Finding C (should-fix) — front-loading short-circuits on the first contradictory fixture

The slice clause "load all fail-fixture expectation contracts before compiling the fail corpus" is satisfied in the narrow sense: no `compile_source` call runs until every path has been parsed once ([crates/sifr/tests/e2e.rs:2699-2733](crates/sifr/tests/e2e.rs:2699)). But because `extract_compile_failure_expectations` panics on the first contradiction it sees ([:707-713](crates/sifr/tests/e2e.rs:707)), if fixture #1 has a contradiction the run aborts before fixture #2..N's contracts are ever parsed.

For a guardrail whose whole point is "front-load the contract failures so they don't pile up at compile time," the more useful behaviour is to parse and validate *every* fixture, accumulate errors, and panic once at the end with the full list. Otherwise an author who writes contradictions in five fixtures has to fix-and-rerun five times.

The minimum lift is to thread `Result<…, Vec<String>>` through `extract_compile_failure_expectations`, accumulate every fixture's outcome in the preload pass, and then panic with the joined message if any failed. Existing call sites that legitimately want "panic on first error" (the smoke fuzz) can opt back in by `.unwrap()`-ing.

This is not a blocker for the slice intent, but the slice is selling "front-loaded contract validation" as a feature, and the current shape only delivers it for the lucky-ordering case.

### Finding D (nit) — error string redundancy when both markers share the same labelled location

The panic body at [crates/sifr/tests/e2e.rs:670-676](crates/sifr/tests/e2e.rs:670) prints `contradictory expect-error markers at <label_left>: <code_left> conflicts with <code_right> at <label_right>`. When both markers have the same `(line_number, column)` (the most common contradiction shape under any of the resolutions in Finding A), the two labels are identical, producing strings like:

```
contradictory expect-error markers at line 12 column 4: SIFR-TYPE-0002 conflicts with SIFR-NAME-0001 at line 12 column 4
```

That extra trailing `at line 12 column 4` is noise for the same-location case and is asserted on by the unit test at [crates/sifr/tests/e2e.rs:3023](crates/sifr/tests/e2e.rs:3023). When the two labels are equal, drop the trailing clause; when they differ, keep it. This keeps the message readable for both the same-location and one-None/one-column shapes.

Trivial; group with whatever change resolves Finding A.

### Finding E (nit) — quadratic pair-walk + early-return reports only one contradiction per fixture

`validate_expectation_contradictions` at [crates/sifr/tests/e2e.rs:662-681](crates/sifr/tests/e2e.rs:662) returns on the first pair it finds. For a fixture with three markers all asserting incompatible codes at the same column, the author sees one error, fixes it, and re-runs to discover the next.

For typical fail fixtures the marker count is small (the largest in the corpus is the eight markers in [crates/sifr/tests/e2e/fail/bounded_multi_error_recovery_repeated_type_errors.sifr](crates/sifr/tests/e2e/fail/bounded_multi_error_recovery_repeated_type_errors.sifr)) so an O(n²) walk is fine, but reporting all contradictions at once is strictly more useful and is a one-line change once the function returns `Result<(), Vec<String>>` per Finding C. Not a blocker.

## Validation status

The brief listed:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr expected_error_contract`
- `cargo test -p sifr --test e2e test_e2e_fail`
- `cargo test -p sifr failure_matching_consumes`
- `cargo test -p sifr smoke_fuzz_valid_expectation_extractors`

I did not re-run these (review-only). All would be expected to pass against the current diff, because:

- The new validator is a no-op on every existing fail fixture (Finding A), so `test_e2e_fail` is unaffected.
- The smoke fuzz at [crates/sifr/tests/e2e.rs:3350-3367](crates/sifr/tests/e2e.rs:3350) only ever appends one `# expect-error:` per sample, so it cannot construct a contradiction.
- The new unit test asserts the validator's pair-checker against synthetic inputs and passes by construction.

Nothing in the validation set distinguishes "the validator works" from "the validator never fires." The full local gate (`scripts/run_all_tests.sh --profile quick`) — required by [AGENTS.md](AGENTS.md) — is not listed in the brief and should be run before merge regardless.

## Summary

- **must-fix A**: same-source-line gate at [crates/sifr/tests/e2e.rs:642-653](crates/sifr/tests/e2e.rs:642) makes the contradiction detector unreachable from real fixtures because every marker occupies its own comment line. Drop the line gate or extend the grammar with `line=N`, and add an extractor-level regression that exercises the panic path.
- **should-fix B**: add a unit test that drives `extract_compile_failure_expectations` against an in-memory contradictory source.
- **should-fix C**: accumulate contract errors across all fail fixtures in the preload pass before panicking, instead of aborting on the first.
- **nit D**: collapse the redundant trailing label in the panic message when both markers share an identical location.
- **nit E**: report all contradictions per fixture, not just the first.

Issue-status line at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) is honest about scope; it should not flip to `[x]` until Finding A is resolved.
