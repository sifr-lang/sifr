# INT-2B Bigint Transition Diagnostic — Review Pass 1

Branch: `int-2b-bigint-transition-diagnostic`
Scope reviewed: uncommitted working-tree changes registering `SIFR-INT-0011` as a transition warning emitted from annotation resolution when source spells `bigint`, plus driver test, registry/docs sync, and the auto-generated docs page.

## Files reviewed

- `crates/sifr_diagnostics/src/codes.rs` — new constant + active entry + ACTIVE list inclusion.
- `crates/sifr_hir/src/lower/mod.rs` — new `LoweringWarningDiagnostic::BigIntTransitionAlias` variant + `warn_bigint_transition_alias` helper.
- `crates/sifr_hir/src/lower/typing_and_functions.rs` — emit-point in `resolve_annotation_expr`.
- `crates/sifr_driver/src/frontend/module_lowering.rs` — render the new variant as a structured diagnostic.
- `crates/sifr_driver/src/tests/single_file_frontend.rs` — `test_type_check_source_surfaces_bigint_transition_warning`.
- `docs/errors/SIFR-INT-0011.md` — auto-generated docs page (gen-error-docs).
- `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` — registry tables.

## What is correct

1. **Code allocation.** `SIFR-INT-0011` matches the slot reserved for "temporary `bigint` transition alias or stale public `bigint` usage" in [internal_docs/integer_model.md:460](internal_docs/integer_model.md:460). Severity is `Warning`, consistent with the milestone INT-2B acceptance criterion ("emits intentional `SIFR-INT-0011` transition diagnostics only", [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:168](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:168)).
2. **Registry entry.** `active_entry!` in [codes.rs:783-793](crates/sifr_diagnostics/src/codes.rs:783) declares the entry with empty `declared_args` and `dedupe_args`, which matches the static template (no formatting placeholders). The `representative_fixture_path` correctly points at the new driver test.
3. **ACTIVE list registration.** [codes.rs:1605](crates/sifr_diagnostics/src/codes.rs:1605) adds the constant to `ACTIVE_DIAGNOSTIC_CODES`, satisfying `check_diagnostic_code_coverage.py`'s expectation that every active code has a non-test compiler-source use (the use lives in [module_lowering.rs:184](crates/sifr_driver/src/frontend/module_lowering.rs:184)).
4. **Type::BigInt preservation.** In [typing_and_functions.rs:439-442](crates/sifr_hir/src/lower/typing_and_functions.rs:439), the new branch fires the warning and returns `Type::BigInt` directly — short-circuiting the `resolve_type_annotation` fallthrough but yielding the same type. Downstream `Type::BigInt` codegen/runtime paths are untouched, matching the explicit non-goal of "Keep bigint as a temporary transition type path."
5. **Lookup ordering preserves user shadowing.** The new check sits *after* `lookup_type_alias` and `class_types`, so a user-defined `type bigint = …` or `class bigint:` continues to win without triggering the transition warning. That is the correct behavior for a name-based transition diagnostic.
6. **Type-alias coverage.** Because user `type` aliases run their RHS through `resolve_annotation_expr` ([type_aliases.rs:144](crates/sifr_hir/src/lower/type_aliases.rs:144)), `MyBig = bigint` will warn at the alias declaration site without any further wiring.
7. **Driver-level test.** `test_type_check_source_surfaces_bigint_transition_warning` ([single_file_frontend.rs:254-282](crates/sifr_driver/src/tests/single_file_frontend.rs:254)) exercises the full structured-rendering path: it confirms the code, severity, message_template equality, args emptiness, primary span presence, file label, and line. The asserted line of `Some(2)` correctly points at the annotation, not the constructor expression. The `assert_eq!(diagnostics.len(), 1)` also locks in the explicit non-goal that the constructor call `bigint(1)` does *not* warn in this slice.
8. **Docs sync.** `docs/errors/SIFR-INT-0011.md`, `docs/errors/diagnostic-codes.md`, and `internal_docs/diagnostic_codes.md` are all populated and ordered consistently with siblings; `check_diagnostic_docs_sync.py` (which runs `gen-error-docs --check`) was run locally and passed.
9. **Severity numbering.** Reserving `SIFR-INT-0011` for a `Warning` deviates from the existing `SIFR-*-0901` convention used for `TYPE_ARITHMETIC_OVERFLOW_RISK` and `FLOW_UNREACHABLE_STATEMENT`, but is consistent with the slot pre-assigned in `internal_docs/integer_model.md`. Not a regression.

## Concerns and gaps

### 1. Coverage is annotation-only; two other source positions silently bypass the warning

The slice description scopes the warning to "annotation resolution when source spells `bigint`," but the underlying milestone goal is broader — the acceptance criterion is "`bigint` is gone from public docs/tests **or** emits intentional `SIFR-INT-0011` transition diagnostics only." Two non-annotation positions still resolve the literal string `"bigint"` to `Type::BigInt` without any warning:

- **`isinstance` narrowing**: [statements.rs:1830](crates/sifr_hir/src/lower/statements.rs:1830) routes `isinstance(x, bigint)` through `sifr_type_system::infer::resolve_type_annotation`, which has its own `"bigint" => Some(Type::BigInt)` arm at [infer.rs:51](crates/sifr_type_system/src/infer.rs:51). No warning fires.
- **TypeVar bounds**: [type_bounds.rs:5-19](crates/sifr_hir/src/lower/type_bounds.rs:5) has a hand-rolled `lookup_named_type` with its own `"bigint" => Some(Type::BigInt)` branch used by bound resolution. No warning fires.

Whether these are acceptable gaps depends on whether the team treats "annotation" strictly (assignment, return, parameter, attribute, alias RHS — covered) or broadly ("any user-visible mention of the word `bigint` in source — also covered"). For a one-warning transition diagnostic the strict reading is defensible, but the gap is undocumented in the slice description and arguably surprising. **Recommendation**: either extend the warning emission to those two paths in this slice, or call them out explicitly as a follow-up in the milestone tracker so the next slice picks them up before bigint removal.

### 2. Three independent name-resolution paths still hardcode `"bigint"`

Stemming from concern (1): there are now three places that map `"bigint"` to `Type::BigInt` independently — [typing_and_functions.rs:439](crates/sifr_hir/src/lower/typing_and_functions.rs:439) (warns), [infer.rs:51](crates/sifr_type_system/src/infer.rs:51) (silent), and [type_bounds.rs:12](crates/sifr_hir/src/lower/type_bounds.rs:12) (silent). When the eventual removal of `bigint` lands, all three will need to be deleted in lockstep. Not a bug today, but worth flagging in the milestone follow-ups.

### 3. Test coverage is the bare minimum

Only one driver-level test exists for the warning. None of the following branches are covered by tests:

- Function return annotation (`def f() -> bigint:`).
- Parameter annotation (`def f(x: bigint):`).
- Field/attribute annotation in a class body.
- Generic argument (`list[bigint]`, `dict[str, bigint]`).
- Type-alias RHS (`type MyBig = bigint`).
- Two annotations in the same module producing two warnings (verifies it fires per occurrence and the dedupe path, which has empty `dedupe_args`, behaves correctly).
- Negative case: a user-defined `type bigint = int` shadowing does not warn.

The change is small enough that a single happy-path test is defensible, but a HIR-level unit test asserting `LoweringWarningDiagnostic::BigIntTransitionAlias` is pushed onto `ctx.warnings` for at least the alias-RHS and return-annotation cases would lock in the documented behavior. **Recommendation**: add at least the multi-occurrence and alias-RHS cases.

### 4. Message string is duplicated five times

The user-facing message string lives in:
- [codes.rs:789](crates/sifr_diagnostics/src/codes.rs:789) (registry `message_template`).
- [module_lowering.rs:185](crates/sifr_driver/src/frontend/module_lowering.rs:185) (rendered `message`).
- [module_lowering.rs:186](crates/sifr_driver/src/frontend/module_lowering.rs:186) (rendered `message_template`).
- [single_file_frontend.rs:267](crates/sifr_driver/src/tests/single_file_frontend.rs:267) (test asserted `message_template`).
- [single_file_frontend.rs:271](crates/sifr_driver/src/tests/single_file_frontend.rs:271) (test asserted `message`).

This duplication is consistent with how the sibling warnings in `module_lowering.rs::warning_diagnostic` are written, so it is not a regression — but it is a maintainability tax. There is no automated check that the registry's `message_template` matches the string passed by `module_lowering.rs`. If wording is later refined, all five sites must be updated together. Not blocking; worth a follow-up to either centralize the message constant or add a sync check.

### 5. Test source mixes annotation and constructor positions

The fixture string `"def main():\n    value: bigint = bigint(1)\n"` contains both an annotation and a constructor call. The test passes today because `assert_eq!(diagnostics.len(), 1)` matches the explicit non-goal "no constructor-call warning for `bigint(...)` in this slice." This is fine, but it tightly couples the test to that non-goal. When the constructor warning lands in a future slice the test will break in a way that is not obvious from its name. **Suggestion**: keep the constructor in the source so the non-goal stays under regression coverage, but rename the assertion comment or add a second test using only `value: bigint = 1` so the expected-count is robust to future slices.

### 6. Existing fixtures now emit warnings without baseline updates

Per the user note, ~33 demo and e2e files annotate `bigint` and now print `SIFR-INT-0011` to stderr during quick validation. I verified no e2e snapshot/baseline currently captures stderr warnings for these fixtures (no matches under `crates/sifr/tests/verification/**` or `verification/**` reference `SIFR-INT-0011` or the new wording, and `test_e2e_pass`/`test_e2e_fail` only assert on compile success/exit-code/stdout — they ignore stderr warnings). So the noise is benign for the test suite. The downstream concern is reviewer/operator UX: every quick-validation run from now until bigint cleanup will print a wall of warnings, which can mask other diagnostics. Acceptable for the transition window but worth scheduling the cleanup work soon.

### 7. Wording — minor

The user-facing message is action-oriented and clear. The registry `summary` ("Temporary bigint transition alias used.") is terse and does not mirror the recommendation. Consistent with sibling entries; no change needed.

## Behavioral regressions

None observed. `Type::BigInt` is still produced for `bigint`; the warning is non-fatal, does not poison the binding, does not interact with the error-taint machinery, and the constructor/runtime/codegen paths are unmodified.

## Diagnostic registry / docs consistency

- Code constant, active entry, `ACTIVE_DIAGNOSTIC_CODES`, `docs/errors/SIFR-INT-0011.md`, `docs/errors/diagnostic-codes.md`, and `internal_docs/diagnostic_codes.md` are mutually consistent.
- `representative_fixture_path` resolves to a real file (`fixture_file_exists` check passes).
- `gen-error-docs --check` was run locally per the user's validation list.
- `SIFR-INT-0011` is correctly placed lexicographically in both registry tables (between `SIFR-INT-0004` and `SIFR-DECIMAL-0001`).

## Are warning-only annotation diagnostics the right scope?

Yes for this slice. The milestone explicitly says transition diagnostics rather than removal, and this is a multi-PR plan: removing `Type::BigInt` and the codegen/runtime backing would inflate this PR significantly. Warning severity (rather than error) is appropriate so existing user code keeps compiling during the transition. The narrow scope (annotation positions only, no constructor warning) is internally consistent.

The two soft issues are (a) the un-warned `isinstance` and TypeVar-bound paths described above, and (b) the lack of a "deprecated since vX.Y" or "will become an error in" data point — not present on sibling warnings either, so not a blocker.

## Suggested follow-ups (non-blocking for this slice)

1. Add a HIR-level unit test exercising `warn_bigint_transition_alias` directly, covering at least: alias RHS, return annotation, and multiple-occurrence emission.
2. Decide whether `isinstance(x, bigint)` and `TypeVar('T', bound=bigint)` should also warn before bigint removal lands; track in the INT-2B follow-up bullet at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:419](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:419).
3. Consider centralizing the warning message string (single `const`) or adding a sync check between registry `message_template` and `module_lowering.rs`.
4. Schedule the demo/fixture migration so the wall-of-warnings noise is bounded in time.

## Validation re-run expectations

The user-listed local validations (fmt, docs sync, code coverage, hir guardrails, targeted cargo tests, clippy, full quick lane) are appropriate and already passed at signature `e1bf653aaa770517`. No additional validations needed for the slice as scoped.

VERDICT: SATISFIED
