# Review — `milestone_diag_4a` slice 2b.26: call-shape diagnostics migration

Scope reviewed: uncommitted working-tree changes on branch
`codex/semantic-diagnostics-diag-4a-call-shape-diagnostics`. The slice marks
2b.25 as merged (PR #1697) and migrates five call-shape e2e fixtures off the
`SIFR-TYPE-0001` bridge:

- `builtin_sum_wrong_arity.sifr` -> `SIFR-CALL-0001`
- `sorted_unexpected_keyword.sifr` -> `SIFR-CALL-0002`
- `keyword_after_positional_error.sifr` -> `SIFR-CALL-0003`
- `range_duplicate_stop_keyword.sifr` -> `SIFR-CALL-0003`
- `map_callable_arity_mismatch.sifr` -> `SIFR-CALL-0005`

Author-reported local validation:
`cargo run -q -p sifr_diagnostics --bin gen-error-docs`,
`cargo fmt --check`,
`scripts/check_diagnostic_docs_sync.py`,
`scripts/check_diagnostic_schema_sync.py`,
`scripts/check_hir_maintainability_guardrails.py`,
`cargo test -p sifr_hir "call_code"`,
`cargo test -p sifr --test e2e -- test_e2e_fail`,
`cargo test -p sifr_diagnostics`,
`cargo test -p sifr -- --skip test_e2e_pass`,
`cargo clippy --workspace -- -D warnings`.

## Summary

Implementation is correct, the five fixtures are coherent as a single slice,
and validation surface is appropriate. No blockers found. Two minor concerns
about the SIFR-CALL-0005 narrowing and the shape of the `{quantifier}` arg are
worth flagging for follow-up but do not block this PR.

**Verdict: reviewer-satisfied / approved for PR.**

## What I checked

### 1. Fixture re-keying matches emitted messages

Each fixture's `# expect-error:` line was cross-referenced against the call
site that fires:

| Fixture | Code emitted | Message emitted | Fixture line |
| --- | --- | --- | --- |
| [builtin_sum_wrong_arity.sifr:2](crates/sifr/tests/e2e/fail/builtin_sum_wrong_arity.sifr:2) | `CALL_WRONG_POSITIONAL_COUNT` ([expressions.rs:1135](crates/sifr_hir/src/lower/expressions.rs:1135)) | `sum() takes exactly 1 argument(s), got 2` | match |
| [sorted_unexpected_keyword.sifr:2](crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr:2) | `CALL_UNEXPECTED_KEYWORD` ([expressions.rs:1201](crates/sifr_hir/src/lower/expressions.rs:1201)) | `sorted() got an unexpected keyword argument 'bogus'` | match |
| [keyword_after_positional_error.sifr:1](crates/sifr/tests/e2e/fail/keyword_after_positional_error.sifr:1) | `CALL_DUPLICATE_ARGUMENT` ([method_call_args.rs:278](crates/sifr_hir/src/lower/method_call_args.rs:278)) | `greet() got multiple values for argument 'name'` | match |
| [range_duplicate_stop_keyword.sifr:2](crates/sifr/tests/e2e/fail/range_duplicate_stop_keyword.sifr:2) | `CALL_DUPLICATE_ARGUMENT` ([builtin_calls.rs:812](crates/sifr_hir/src/lower/builtin_calls.rs:812)) | `range() got multiple values for argument 'stop'` | match |
| [map_callable_arity_mismatch.sifr:2](crates/sifr/tests/e2e/fail/map_callable_arity_mismatch.sifr:2) | `CALL_NOT_CALLABLE_OR_ARITY` ([expressions.rs:1460](crates/sifr_hir/src/lower/expressions.rs:1460)) | `map() callable expects 1 argument(s), got 2 iterable(s)` | match |

All five rendered code+message pairs are exact matches.

### 2. Registry templates align with emitted messages

[`crates/sifr_diagnostics/src/codes.rs`](crates/sifr_diagnostics/src/codes.rs):

- `SIFR-CALL-0001`: template now `{callable} takes {quantifier} {expected_count} argument(s), got {actual_count}`. Adding `{quantifier}` is forward-compatible with both the new sum() emission ("exactly") and the existing-but-still-bridged `sqrt() takes at most 1 argument(s), got 2` shape in [stdlib_wrong_arg_count.sifr:1](crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr:1) and [method_call_args.rs:72](crates/sifr_hir/src/lower/method_call_args.rs:72). Declared and dedupe arg lists were updated together.
- `SIFR-CALL-0002`: `{callable} got an unexpected keyword argument '{keyword}'` — matches the emission verbatim including the `'` quoting and the article `an`.
- `SIFR-CALL-0003`: `{callable} got multiple values for argument '{argument}'` — matches.
- `SIFR-CALL-0005`: narrowed from `{callee} is not callable with the provided arguments` to `{callable} callable expects {expected_count} argument(s), got {actual_count} iterable(s)`. Dedupe key changed from `["callee"]` to `["callable","expected_count","actual_count"]`. See concern (A) below.

### 3. Generated docs synced

- [docs/errors/SIFR-CALL-0001.md](docs/errors/SIFR-CALL-0001.md), [SIFR-CALL-0002.md](docs/errors/SIFR-CALL-0002.md), [SIFR-CALL-0003.md](docs/errors/SIFR-CALL-0003.md), [SIFR-CALL-0005.md](docs/errors/SIFR-CALL-0005.md) all reflect the new templates and arg lists.
- [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md) rows for `SIFR-CALL-0001/0002/0003/0005` are updated consistently. `SIFR-CALL-0004` is untouched (correct — it was already migrated and is out of scope).

### 4. HIR unit coverage

[`crates/sifr_hir/src/lower/expressions_tests.rs:245-305`](crates/sifr_hir/src/lower/expressions_tests.rs:245)
adds five tests, each asserting both the exact rendered message and the
exact `DiagnosticCode`. Test names share the `_has_call_code` suffix so
`cargo test -p sifr_hir "call_code"` covers all five. `DiagnosticCode`
import is already in the file
([line 2](crates/sifr_hir/src/lower/expressions_tests.rs:2)).

### 5. HIR call-site changes

- [`expressions.rs:1133-1138`](crates/sifr_hir/src/lower/expressions.rs:1133) — sum() now uses `error_with_code(CALL_WRONG_POSITIONAL_COUNT, …)` and embeds `actual_count` so message+code dedupe carries the count.
- [`expressions.rs:1198-1205`](crates/sifr_hir/src/lower/expressions.rs:1198) — sorted() unexpected-keyword path coded.
- [`expressions.rs:1456-1466`](crates/sifr_hir/src/lower/expressions.rs:1456) — map() arity path coded with new `expected_count`/`actual_count` locals (clean refactor).
- [`builtin_calls.rs:798-829`](crates/sifr_hir/src/lower/builtin_calls.rs:798) — range() duplicate-keyword paths (start/stop/step) all migrated and re-worded from `'stop' was provided both positionally and as a keyword` to `range() got multiple values for argument 'stop'`. The new wording matches `SIFR-CALL-0003`'s template; previous unique wording is retired.
- [`method_call_args.rs:1`](crates/sifr_hir/src/lower/method_call_args.rs:1) imports `DiagnosticCode` (was missing); [`method_call_args.rs:277-281`](crates/sifr_hir/src/lower/method_call_args.rs:277) `duplicate_argument_error` now codes `CALL_DUPLICATE_ARGUMENT`. All callers of `duplicate_argument_error` (e.g. positional+keyword overlap on user-defined callables, plus several method-call paths) inherit the structured code in one place — good factoring.

### 6. Issue tracker

[`issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:60-61`](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:60)
flips 2b.25 to `[x] merged` with PR #1697 and adds 2b.26 as `[ ] in progress`
with the right code list. Wording matches the established pattern of prior
slices.

### 7. Slice cohesion

The five fixtures share the "call-shape" theme (positional arity, unexpected
keyword, duplicate argument, higher-order callable arity). They map across
four codes; the SIFR-CALL family is internally scoped to call-site shape
errors, so bundling them in one PR is coherent and keeps the diff
reviewable. Splitting further (e.g. per-code) would create four trivial PRs
without independent value.

## Concerns (non-blocking)

### A. SIFR-CALL-0005 narrowing — template vs. summary drift

The summary still reads "Callable arity failure or expression is not
callable." but the template is now map/iterable-specific
(`{callable} callable expects {expected_count} argument(s), got {actual_count} iterable(s)`).
That shape works for filter() ([expressions.rs:1502](crates/sifr_hir/src/lower/expressions.rs:1502)
emits the exact same form, currently still uncoded), but it does not fit:

- the generic user-callable arity error at [expressions.rs:1659-1666](crates/sifr_hir/src/lower/expressions.rs:1659)
  (`callable '{name}' expects {N} argument(s), got {M}`), which has no
  `iterable(s)` suffix and no leading `callable` keyword;
- a future "expression is not callable" diagnostic, which the summary
  explicitly anticipates.

When those sites are migrated, either (a) the template will need to grow
again, (b) they will need a different code, or (c) the summary should be
narrowed now to "Higher-order callable iterable-arity mismatch" so the
template/summary stay aligned. None of this blocks this PR — just flagging
that 0005 is currently shaped as a single-purpose code despite a broader
description.

### B. `{quantifier}` is in the dedupe-arg list

For SIFR-CALL-0001, `quantifier` is listed under both declared and dedupe
args. That means `sum() takes exactly 1 argument(s), got 2` and a future
`sqrt() takes at most 1 argument(s), got 2` would dedupe distinctly even
when `callable`, `expected_count`, and `actual_count` collide. That is
arguably correct (the quantifier carries semantic meaning — "at most" vs
"exactly" describe different upper-bound/strict-equality constraints), but
it is worth confirming that's the intended dedupe behavior rather than an
artifact of mechanically including every arg in the dedupe list. No action
required if intentional.

### C. Sibling call sites with the same shape remain on the bridge

Out of scope for this slice but worth tracking:

- [`expressions.rs:1210`](crates/sifr_hir/src/lower/expressions.rs:1210) — `sorted() got multiple values for argument 'iterable'` still uses plain `ctx.error(...)` despite matching `SIFR-CALL-0003` shape exactly.
- [`builtin_calls.rs:23`](crates/sifr_hir/src/lower/builtin_calls.rs:23) (zip), [`builtin_calls.rs:831`](crates/sifr_hir/src/lower/builtin_calls.rs:831) (range), [`expressions.rs:1346`](crates/sifr_hir/src/lower/expressions.rs:1346) (enumerate), [`method_call_args.rs:298`](crates/sifr_hir/src/lower/method_call_args.rs:298) (general method calls) — all emit the SIFR-CALL-0002 shape but uncoded.
- [`method_call_args.rs:72`](crates/sifr_hir/src/lower/method_call_args.rs:72) — emits the SIFR-CALL-0001 "at most" shape uncoded.
- [`expressions.rs:1502`](crates/sifr_hir/src/lower/expressions.rs:1502) — filter()'s SIFR-CALL-0005 shape uncoded.

Until those are migrated, two e2e fixtures with visually identical messages
can render with different codes (`SIFR-CALL-000X` vs the legacy
`SIFR-TYPE-0001` bridge) depending on which call site fires. This is the
known cost of the incremental approach documented in the issue tracker, and
the bridge is not yet retired, so it's acceptable. Worth noting in the
follow-up issue tracker so the family's call-shape coverage gets closed
before bridge removal.

## Verification I did not re-run

I trusted the author-reported local validation (listed at the top of this
review) without re-running it, per task instructions to not modify files.
Spot-checks of registry/template/docs/issue-tracker/HIR call-site
consistency all passed.

## Decision

Approved for PR. Concerns A–C are follow-up notes, not change requests.
