# INT-2B Bigint Warning Coverage — Review Pass 1

Branch: `int-2b-bigint-warning-coverage`
Scope reviewed: uncommitted working-tree changes that extend `SIFR-INT-0011` (the `bigint` transition-alias warning, severity `Warning`) from annotation resolution alone to two additional source positions explicitly called out as gaps by the prior slice's review:

1. `isinstance(_, bigint)` second argument.
2. `TypeVar` bounds and constraints — both PEP 695 inline syntax (`def f[T: bigint]`, `class C[T: bigint]`) and the legacy declaration form (`T = TypeVar("T", bigint)`, `bound=`, `constraints=`).

The follow-up note that scopes this slice lives at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426): *"add `SIFR-INT-0011` coverage for silent `bigint` mentions in `isinstance` and TypeVar bounds if they remain before removal"*. The earlier review that called these out is [reviews/integer-model-int-2b-bigint-transition-diagnostic-review-pass-1.md:30-37](reviews/integer-model-int-2b-bigint-transition-diagnostic-review-pass-1.md:30) ("Concern 1: Coverage is annotation-only; two other source positions silently bypass the warning").

## Files reviewed

- `crates/sifr_hir/src/lower/builtin_calls.rs` — emits the warning when `isinstance(_, bigint)` is lowered.
- `crates/sifr_hir/src/lower/typevar_annotations.rs` — emits the warning from every TypeVar bound/constraint parse path.
- `crates/sifr_driver/src/tests/single_file_frontend.rs` — three new driver-level tests covering one entry path each.

No diagnostics-registry, generated-docs, or phase-tracker files are touched (and none need to be — the existing `SIFR-INT-0011` registration already covers the new emit sites).

## Local validation observed during review

Run from the working tree:

- `cargo test -p sifr_driver -- bigint` → 4 passed (`test_type_check_source_surfaces_bigint_transition_warning` plus the three new tests).
- `cargo clippy -p sifr_hir -p sifr_driver` → clean.
- `cargo fmt --check` → clean.
- `python3 scripts/check_hir_maintainability_guardrails.py` → PASS.
- Hand-crafted CLI probe `class C[T: bigint]:` via `cargo run -q -p sifr -- --diagnostic-format=json check` → 1 warning at the bound span (CLI-level dedupe applies; see Concern 1).

`scripts/run_all_tests.sh --profile quick` was not run during this review — should be run before opening the PR per [AGENTS.md](AGENTS.md).

## What is correct

1. **Single helper for the typevar-annotation emit sites.** [typevar_annotations.rs:25-29](crates/sifr_hir/src/lower/typevar_annotations.rs:25) introduces `warn_bigint_transition_name` and threads it through every `Expr::Name` arm in both `parse_typevar_bound_expr` and `parse_typevar_declaration_specs`. The helper is a 4-line, pure name-equality + `ctx.warn_bigint_transition_alias(range)` call — no allocation, no formatting. This is the right granularity: emission lives next to the AST shape recognition, not duplicated at every call site.
2. **All seven AST shapes that resolve the literal name `bigint` to a TypeVar bound/constraint are wired.** Specifically, in `parse_typevar_bound_expr`: PEP 695 simple name ([line 36](crates/sifr_hir/src/lower/typevar_annotations.rs:36)) and PEP 695 tuple constraint ([line 43](crates/sifr_hir/src/lower/typevar_annotations.rs:43)). In `parse_typevar_declaration_specs`: positional constraint ([line 82](crates/sifr_hir/src/lower/typevar_annotations.rs:82)), `bound=` keyword ([line 110](crates/sifr_hir/src/lower/typevar_annotations.rs:110)), `constraints=` tuple ([line 134](crates/sifr_hir/src/lower/typevar_annotations.rs:134)), and `constraints=` single name ([line 146](crates/sifr_hir/src/lower/typevar_annotations.rs:146)). I cross-checked each branch — every `Expr::Name` arm that ultimately pushes a `bigint`-derived spec also runs the helper.
3. **`isinstance` emit site is at the right layer.** [builtin_calls.rs:929-934](crates/sifr_hir/src/lower/builtin_calls.rs:929) fires inside `lower_isinstance_call` against `call.arguments.args[1]`, exactly where the literal name is read for the lowered call. The warning fires once per `isinstance(_, bigint)` expression.
4. **No false positives.** The branch-bodies of the helper are guarded by `name == "bigint"`. For non-`bigint` bounds/constraints the new code is a no-op and the existing logic is untouched. None of `int`, `float`, `Comparable`, `Addable`, `Hashable`, user `class`, or user-defined type aliases trip the warning.
5. **Lookup ordering preserves the prior shadowing semantics.** None of the new emit sites consult the scope/alias tables before the name check, but that is consistent with how the previously-shipped annotation emit at [typing_and_functions.rs:439](crates/sifr_hir/src/lower/typing_and_functions.rs:439) treats `bigint`: the warning has always been a string-literal diagnostic gated only on the source spelling, intentionally not on type-resolution outcomes. Users who shadow `bigint` (`type bigint = int`) hit the alias path inside `resolve_annotation_expr` *before* reaching the `bigint`-string check — but for TypeVar bounds and `isinstance` the design treats the literal token as the trigger because the lowering path needs the literal string spec for downstream bound/constraint encoding. The new behavior is consistent with the prior slice's design choice; just note that, unlike `resolve_annotation_expr`, the new emit sites cannot be silenced by a user-defined `bigint` alias. That is acceptable because constraint/bound parsing operates on the literal name shape.
6. **No new lints, no new warnings, no new format diffs, no panic risks.** Each new line is straight diagnostic plumbing.
7. **Tests assert the right invariants for the three covered paths.** Each new test asserts `diagnostics.len() == 1`, the `INT_BIGINT_TRANSITION_ALIAS` code, severity `Warning`, primary span presence, span file, span line, and `byte_end > byte_start` (matches the existing parent-commit test's structure at [single_file_frontend.rs:255-282](crates/sifr_driver/src/tests/single_file_frontend.rs:255)). The asserted line numbers (`Some(3)` for the `TypeVar` and `isinstance` cases, `Some(1)` for the PEP 695 case) point at the line containing the `bigint` token, not at the surrounding statement, which is the user-helpful span.
8. **Existing tests remain green.** The parent-commit test `test_type_check_source_surfaces_bigint_transition_warning` (annotation `value: bigint = bigint(1)`) still asserts exactly one warning. The constructor call `bigint(1)` continues to be silent (consistent with the slice's stated scope, which lists only `isinstance` and TypeVar bounds).

## Concerns and gaps

### 1. `class C[T: bigint]` double-emits at the HIR warnings vector (CLI dedup hides it)

`collect_class_type` is invoked twice from [mod.rs:587, 597](crates/sifr_hir/src/lower/mod.rs:587) — once before alias resolution and once after — and both invocations call `parse_typevar_bound_expr(bound, ctx)` for class type parameters at [classes.rs:267](crates/sifr_hir/src/lower/classes.rs:267). With this slice's change, that means every `class C[T: bigint]:` pushes **two** `LoweringWarningDiagnostic::BigIntTransitionAlias` entries onto `ctx.warnings` with identical `(code, message_template, primary_span)` keys.

This is masked at the CLI by [diagnostics.rs:69 `deduplicate_recovery_diagnostics`](crates/sifr_driver/src/diagnostics.rs:69) (called from `apply_diagnostic_recovery_limits`), which collapses identical `(code, message_template, dedupe_args, primary_span)` pairs. I confirmed empirically: a `--diagnostic-format=json check` of `class C[T: bigint]:` returns one warning. But:

- `crates/sifr_driver/src/frontend/api.rs::type_check_source` does **not** dedup. Anything that consumes `LoweringResult.warnings` directly (e.g. future HIR-level unit tests, snapshot harnesses, alternate frontends) sees a duplicate. A test like `assert_eq!(type_check_source("class C[T: bigint]:\n    pass\n").len(), 1)` would fail today.
- The prior slice's `invalid_typevar_shape` errors in `parse_typevar_bound_expr` already had this property (so this is an inherited architectural shape, not a new bug introduced by this slice). It just becomes visible here because this is the first time a *warning* is emitted from that double-call site.
- The duplicate is a function of where the helper is invoked, not of the helper itself; gating in `collect_class_type` (e.g. only run `parse_typevar_bound_expr` when `validate_iteration_protocols == false`, since the bound shape doesn't change between passes) would fix it for every diagnostic flowing through that function, not only `SIFR-INT-0011`.

Given the parent-commit test depends on `type_check_source.len() == 1` semantics, I think this slice should not ship a code path where that invariant *would* be violated for one of its supported shapes. **Recommendation**: either add a single-pass guard inside `collect_class_type` so the bound parse runs once, or add an explicit `assert_eq!(...len(), 1)` test for `class C[T: bigint]:` that goes through `type_check_source` and force-fixes the issue. A test alone (without the guard) would fail today and is the cleanest way to surface this.

### 2. Three reachable emit sites have no test coverage

The slice's intent is "add `SIFR-INT-0011` coverage for silent `bigint` mentions in `isinstance` and TypeVar bounds." Test coverage of the new emit sites:

| Emit site | File:line | Test |
| --- | --- | --- |
| PEP 695 simple-name bound | typevar_annotations.rs:36 | `test_type_check_source_warns_for_bigint_pep695_bound` ✓ |
| PEP 695 tuple constraint  | typevar_annotations.rs:43 | — |
| `TypeVar("T", bigint)` positional | typevar_annotations.rs:82 | `test_type_check_source_warns_for_bigint_typevar_constraint` ✓ |
| `TypeVar("T", bound=bigint)` keyword | typevar_annotations.rs:110 | — |
| `TypeVar("T", constraints=(bigint, ...))` tuple | typevar_annotations.rs:134 | — |
| `TypeVar("T", constraints=bigint)` single name | typevar_annotations.rs:146 | — |
| `lower_isinstance_call` | builtin_calls.rs:931 | `test_type_check_source_warns_for_bigint_isinstance_target` ✓ |

Four of seven branches are unexercised. They are structurally distinct AST arms (each has its own `match` arm + its own `warn_bigint_transition_name` call), and a regression in any one would be silent until the eventual `bigint` removal slice fails to delete a path. Each test would be ~10 lines copied from the existing template.

**Recommendation**: add at least one regression test per untested branch. Concretely:

- `def f[T: (bigint, str)](value: T) -> T: ...` (PEP 695 tuple constraint).
- `T = TypeVar("T", bound=bigint)` (keyword bound).
- `T = TypeVar("T", constraints=(bigint, str))` (keyword constraints tuple).
- `T = TypeVar("T", constraints=bigint)` (keyword constraints single name) — note this asserts that the `Expr::Name` branch at typevar_annotations.rs:146 fires; without it the line would only be exercised by manual probing.

If concern (1) is also resolved, add a class PEP 695 test (`class C[T: bigint]:`) too, asserting `diagnostics.len() == 1` to lock in the single-emission contract.

### 3. `detect_narrowing_condition` for `isinstance(_, bigint)` is silent (but the warning still fires once per source line)

[statements.rs:1832](crates/sifr_hir/src/lower/statements.rs:1832) re-resolves the second arg of an `if isinstance(...)` for narrowing purposes via `resolve_type_annotation`. It does not warn. This is fine for *user-visible* counting (the same source line goes through `lower_isinstance_call` for the expression itself, which now warns once). But it adds a fourth place in the codebase where the literal string `"bigint"` is mapped to `Type::BigInt` without using the centralized resolver — joining [typing_and_functions.rs:439](crates/sifr_hir/src/lower/typing_and_functions.rs:439), [type_bounds.rs:12](crates/sifr_hir/src/lower/type_bounds.rs:12), and [infer.rs:51](crates/sifr_type_system/src/infer.rs:51). The prior review already flagged this duplication ([reviews/integer-model-int-2b-bigint-transition-diagnostic-review-pass-1.md:39-41](reviews/integer-model-int-2b-bigint-transition-diagnostic-review-pass-1.md:39)). Not new debt from this slice; flagged here only because the slice's existence highlights that the duplication still has not been consolidated and the `bigint` removal slice will need to delete all four sites in lockstep.

### 4. The `bigint(...)` constructor call (`expressions.rs:856`) remains silent

`bigint(1)` produces a `Type::BigInt` value with no warning. The slice's stated scope is `isinstance` + `TypeVar` bounds, so this is intentionally out-of-scope. It is worth recording explicitly in the milestone tracker so that the constructor (and the `from sifr import bigint`-style import alias, if it exists) is not forgotten before `bigint` removal. Currently, `value = bigint(1)` (no annotation) is a fully silent way to introduce a `Type::BigInt` value into the program, which arguably is the strongest "silent bigint mention" remaining in the language. Not blocking this slice; flag the gap.

### 5. Phase tracker not updated

The slice closes part of the bundled follow-up bullet at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:426). The bullet groups several items (`SIFR-INT-0003` registry placement, e2e fail fixture, reserved-name shadowing, `SIFR-INT-0011` `isinstance`/TypeVar coverage, fixed-width formatting, stdlib bootstrap exports, transitive re-exports), so it's defensibly left ticked off only when the whole bundle closes. But the prior slice's review is referenced in a per-slice line ([line 423](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:423)) — this slice should add an analogous line referencing this review's outcome and the eventual PR. Currently there is no acknowledgement of the slice in the tracker. Not a code blocker; a documentation/hygiene gap.

### 6. Diagnostic registry's representative-fixture pointer is not refreshed

[codes.rs:786](crates/sifr_diagnostics/src/codes.rs:786) and [internal_docs/diagnostic_codes.md:95](internal_docs/diagnostic_codes.md:95) still point only at `test_type_check_source_surfaces_bigint_transition_warning`. The field is "representative", so a single canonical pointer is acceptable per the registry contract. Not a blocker. If the team wants the registry to reflect the broadest fixture, the `isinstance` test is arguably more broadly representative; either way, no action required for this slice.

### 7. No `.sifr` demo or e2e fixture

This is a diagnostics-only slice; the existing unit tests are sufficient. No `verification/` or `demos/` change is warranted.

## Verdict-blocking summary

- **Concern 1** (class PEP 695 double-emit at the lowering layer) is a real defect that ships hidden by CLI dedup. Without a guard or a test covering it, the next consumer of `LoweringResult.warnings` (or the next class-aware test) will trip on it. The fix is a one-line gate inside `collect_class_type`, and the test is a copy of the PEP 695 function test with `class C[T: bigint]: pass`.
- **Concern 2** (four untested emit sites) is the slice's own definition of "coverage" being only partly delivered. Each missing branch is structurally distinct and trivially testable.

Concerns 3–7 are non-blocking observations.

## Requested changes

1. Add a regression test (and, if needed, a single-pass guard in `collect_class_type`) so that `class C[T: bigint]:` produces exactly one `SIFR-INT-0011` warning at the lowering layer (`type_check_source` consumer view), not just at the CLI.
2. Add tests for the four currently-untested emit branches (PEP 695 tuple constraint, `bound=` keyword, `constraints=` tuple, `constraints=` single name). Each is a ~10-line copy of the existing template.
3. Add a per-slice line under the INT-2B section of [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) noting this review and the eventual PR, consistent with [line 423](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:423).
4. Run and report `scripts/run_all_tests.sh --profile quick` before the PR.

VERDICT: CHANGES REQUESTED
