# INT-2B fixed-width fail fixture markers — review pass 1

Branch: `int-2b-fixed-width-fail-fixture-markers`
Scope: convert the two `expect-error` markers in
[`crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr`](crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr)
from indented, decorative form to canonical leading-line form so the e2e
harness actually enforces the diagnostic codes and columns.

## Change under review

```diff
+# expect-error[col=23]: SIFR-INT-0001
+# expect-error[col=24]: SIFR-INT-0004
+
 def main():
-    # expect-error[col=23]: SIFR-INT-0001
     too_wide: uint8 = 2 ** 8
-    # expect-error[col=24]: SIFR-INT-0004
     too_large: uint8 = 10 ** 5000
```

`git status --short` reports exactly one path modified, matching the slice
description. No source/harness/snapshot churn.

## Correctness of harness binding

`parse_expect_error_line` ([crates/sifr/tests/e2e.rs:614](crates/sifr/tests/e2e.rs:614))
calls `line.strip_prefix("# expect-error[")` and `line.strip_prefix("# expect-error:")`
without trimming leading whitespace. The previous indented form
`    # expect-error[...]` matched neither prefix, so the parser returned `None`
and the markers contributed zero `LocatedCompileFailureExpectation`s to the
file. `parse_compile_failure_expectations` therefore handed `test_e2e_fail`
([crates/sifr/tests/e2e.rs:2802](crates/sifr/tests/e2e.rs:2802)) an empty
expectation list, which means `match_compile_failure_expectations` (the for-loop
at [crates/sifr/tests/e2e.rs:878](crates/sifr/tests/e2e.rs:878)) executed zero
iterations and trivially returned `Ok(())`. Net effect: prior to this slice the
fixture only asserted "compilation fails for some reason" — the explicit codes
and columns documented inline were not enforced.

After the change both markers sit at column 1 with no leading whitespace, so
they take the `"# expect-error["` branch, parse cleanly, and feed the matcher
(`failure.code == expected.code && failure.column == Some(expected_column)` at
[crates/sifr/tests/e2e.rs:866](crates/sifr/tests/e2e.rs:866)). This closes the
N1 finding from
[reviews/integer-model-int-2b-const-expression-fitting-review-pass-2.md:125](reviews/integer-model-int-2b-const-expression-fitting-review-pass-2.md:125)
exactly as recommended ("convert the inline annotations to canonical
`# expect-error[col=N]: SIFR-INT-NNNN` lines preceding each test line").

The leading blank line between the markers and `def main():` matches the sister
fixture
[`fixed_width_literal_out_of_range.sifr`](crates/sifr/tests/e2e/fail/fixed_width_literal_out_of_range.sifr),
which is the closest precedent in the same diagnostic family. Convention
parity is maintained.

## Column values

Counting 1-based columns over the actual on-disk lines (verified via `awk`,
`length($0)` reports 28 and 33 chars respectively):

```
Line 5: "    too_wide: uint8 = 2 ** 8"
        1234^   ^   ^^    ^^ ^^^^ ^   <- structure
        col 5  13  15-19 21 23   28
```

- 4 leading spaces (cols 1–4)
- `too_wide` (cols 5–12)
- `:` (col 13), ` ` (col 14)
- `uint8` (cols 15–19)
- ` ` (col 20), `=` (col 21), ` ` (col 22)
- `2` (col 23) ← start of `2 ** 8` expression

```
Line 6: "    too_large: uint8 = 10 ** 5000"
```

- 4 leading spaces (cols 1–4)
- `too_large` (cols 5–13)
- `:` (col 14), ` ` (col 15)
- `uint8` (cols 16–20)
- ` ` (col 21), `=` (col 22), ` ` (col 23)
- `1` (col 24) ← start of `10 ** 5000` expression

These columns correspond to the start of the RHS expression range, which is
precisely what the diagnostics emit. Both
[`crates/sifr_hir/src/lower/fixed_width_fitting.rs:35`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:35)
(SIFR-INT-0001) and
[`crates/sifr_hir/src/lower/fixed_width_fitting.rs:266`](crates/sifr_hir/src/lower/fixed_width_fitting.rs:266)
(SIFR-INT-0004) call `ctx.error_with_code_at(..., range)` where `range` is the
RHS expression range threaded through `validate_fixed_width_initializer`. The
primary span column is then surfaced through
`compiled_failure_from_rendered` at
[crates/sifr/tests/e2e.rs:584](crates/sifr/tests/e2e.rs:584) and matched 1:1.
The fact that the author's local `cargo test -p sifr --test e2e test_e2e_fail`
run reports 265 passing fail tests after the change — including this fixture
now exercising both markers — independently confirms both columns line up with
the emitted diagnostics.

Diagnostic-code mapping is also correct:

- `2 ** 8 == 256` overflows `uint8` (range `0..=255`) → `SIFR-INT-0001`
  (`INT_FIXED_WIDTH_OUT_OF_RANGE`,
  [crates/sifr_diagnostics/src/codes.rs:62](crates/sifr_diagnostics/src/codes.rs:62)).
- `10 ** 5000` is a 5001-decimal-digit result, well past the 4096-digit
  `INTEGER_EVAL_DECIMAL_DIGIT_BUDGET` → `SIFR-INT-0004`
  (`INT_EVAL_BUDGET_EXCEEDED`,
  [crates/sifr_diagnostics/src/codes.rs:64](crates/sifr_diagnostics/src/codes.rs:64)).

## Harness compatibility

- The two marker lines are at the top of the file, before any code, matching
  the pattern used by the other 178 fail fixtures (`grep -rln "expect-error"`
  returns 180 fixtures total; spot-checking confirms all use leading-column
  markers).
- A repo-wide check for indented markers
  (`grep -rn "^[[:space:]]\+# expect-error"
  crates/sifr/tests/e2e/fail/`) returns no hits after the fix, so this fixture
  was the only stragglers and there is no parallel cleanup outstanding.
- `parse_expect_error_line` does not constrain the markers to be contiguous
  with the failing statement, only that they appear somewhere in the file
  ([crates/sifr/tests/e2e.rs:704](crates/sifr/tests/e2e.rs:704)). Hoisting them
  to the file head is fully supported.
- `validate_expectation_contradictions`
  ([crates/sifr/tests/e2e.rs:664](crates/sifr/tests/e2e.rs:664)) only flags
  same-column markers with different codes; cols 23 and 24 are distinct, so no
  contradiction.

## Scope discipline

`git status --short` shows only the fixture file changed. No supporting
helpers, no documentation churn, no snapshot updates. This is a tightly
scoped polish slice consistent with the project workflow item-by-item rule.

## Closure of the broader follow-up

The implementation checklist bullet at
[issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:441](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:441)
reads:

> Carry remaining follow-ups from INT-2A/INT-2B reviews: clean up fixed-width
> diagnostic formatting/fallback paths as those code paths become reachable.

This slice resolves N1 from pass-2 in full (the only fixture-marker hygiene
finding in flight). It does **not** resolve the carryover items still
documented as non-blocking in
[reviews/integer-model-int-2b-const-expression-fitting-review-pass-2.md:117](reviews/integer-model-int-2b-const-expression-fitting-review-pass-2.md:117):

- F4 — module-level `LIMIT: int = 10 ** 5000` emits `SIFR-INT-0004` from the
  `remember_module_const_integer` path while a function-body `int`-annotated
  copy does not.
- F5 — `lower_integer_const_expr_simple` has no `Expr::Name` arm and
  `negate_simple_expr` only flips literal numerics at module scope.
- F7 — folded `value: uint8 = SOME_NAME` short-circuits the `HirExpr::Name`
  move-tracking branch in
  [crates/sifr_hir/src/lower/statements.rs:1084](crates/sifr_hir/src/lower/statements.rs:1084).

So flipping that bullet from `[ ]` to `[x]` would be premature on this PR.
The PR description (and any follow-up tracker) should make clear this slice
addresses the fixture-marker portion only, and the broader fallback-path
cleanup remains open. That framing matches how prior INT-2B sub-items were
handled (each landed as its own discrete PR under the same parent bullet).

## Local validation

The author reports `git diff --check` and `cargo test -p sifr --test e2e
test_e2e_fail -- --nocapture` (265 fail tests passing). I did not re-run the
full suite. The `test_e2e_fail` run is the targeted gate for this change —
prior to the fix it would have passed with the markers being silent
no-ops, but the diff now causes the harness to actively assert
`SIFR-INT-0001 @col23` and `SIFR-INT-0004 @col24` against the real
diagnostics. A passing run after the change is the enforcement signal we
want.

For PR submission I would still suggest running
`scripts/run_all_tests.sh --profile quick` per AGENTS.md and recording the
report signature, since this is the authoritative gate. Targeted-test-only
validation has historically been called out as insufficient for PR sign-off
on this phase.

## Panics / no-user-path violations

None. The change is fixture text only; no new code paths.

## Verdict

VERDICT: SATISFIED

No blocking findings. The two markers now bind to the harness, the columns
match the emitted primary spans, the codes match the active diagnostic
registry entries, and the scope is exactly the documented slice. One minor
note for the PR description (not blocking): clarify that this slice closes
the N1 fixture-marker hygiene finding only, and that the broader fixed-width
diagnostic formatting/fallback cleanup bullet stays open until F4/F5/F7
land. Recommend running `scripts/run_all_tests.sh --profile quick` and
attaching the report signature to the PR before merging, per AGENTS.md.
