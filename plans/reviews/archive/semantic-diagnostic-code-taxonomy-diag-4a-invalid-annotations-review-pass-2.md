## `milestone_diag_4a` slice 2b.8 — HIR invalid type-annotation shape migration to active `SIFR-TYPE-0007` — review pass 2

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-invalid-annotations`.
- Delta vs. pass 1: a single new e2e fail fixture, [crates/sifr/tests/e2e/fail/integer_literal_too_large_type_annotation.sifr](../crates/sifr/tests/e2e/fail/integer_literal_too_large_type_annotation.sifr:1), addressing pass-1 residual R1 (the only migrated `resolve_annotation_expr` site without a 1:1 fixture). No further changes to [crates/sifr_hir/src/lower/typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs:380), no further checklist edits to [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:43), no other pass-1 findings re-opened.
- Validation re-executed by the implementer: `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`, `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=75.13s`).

## Findings

### F1 — Pass-1 residual R1 is closed: the new fixture exercises site #1 cleanly

The pass-1 finding R1 flagged that emission site #1 — [`integer literal too large for type annotation`](../crates/sifr_hir/src/lower/typing_and_functions.rs:419) — was the only one of the 11 migrated sites without a dedicated `.sifr` fixture, and recommended adding one with a literal larger than `i64::MAX`. The new fixture matches that recommendation exactly:

```sifr
# expect-error: SIFR-TYPE-0007: integer literal too large for type annotation

def consume(value: 999999999999999999999999999999) -> int:
    return 0

def main():
    print(consume(1))
```

Trigger validity:

- The literal is 30 nines (≈ 10²⁹), which is roughly ten orders of magnitude above `i64::MAX = 9_223_372_036_854_775_807` (19 digits). The Ruff parser tokenizes this as `Number::Int` (no `.`/`e`), so the [`Expr::NumberLiteral` arm at line 414](../crates/sifr_hir/src/lower/typing_and_functions.rs:414) enters the `Number::Int` branch, [`i.as_i64()` at line 416](../crates/sifr_hir/src/lower/typing_and_functions.rs:416) returns `None`, and the [overflow branch at line 419](../crates/sifr_hir/src/lower/typing_and_functions.rs:419) fires `invalid_type_annotation(ctx, "integer literal too large for type annotation")` — i.e., the migrated `SIFR-TYPE-0007` site, never the sibling "only integer literals are supported in type annotations" branch (which requires a non-`Int` `Number`, e.g. a float).
- Direct probe with `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/integer_literal_too_large_type_annotation.sifr` prints `type error: [main] integer literal too large for type annotation`, confirming that exactly one diagnostic with the expected substring is produced.
- `cargo test -p sifr --test e2e -- test_e2e_fail` is now `1 passed; 0 failed; ... 25 filtered out`, i.e. the e2e fail harness ([e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561), `failure.code == expected.code && failure.message.contains(message)`) accepts the fixture's `# expect-error: SIFR-TYPE-0007: integer literal too large for type annotation` line — so the structured code stamp (not just the rendered text) is verified.

The fixture body shape is identical to the other ten siblings (single `def consume(value: <bad>) -> int: return 0` plus a trivial `main` shim that calls it once), so it slots into the established style without introducing any new harness assumption.

### F2 — Coverage table is now 11/11 with no other surface changes

Updated mapping of in-scope `invalid_type_annotation` call sites to fixtures:

| # | Code site | Emitted message | Fixture |
|---|---|---|---|
| 1 | [419](../crates/sifr_hir/src/lower/typing_and_functions.rs:419) | `integer literal too large for type annotation` | [integer_literal_too_large_type_annotation.sifr](../crates/sifr/tests/e2e/fail/integer_literal_too_large_type_annotation.sifr:1) **(new)** |
| 2 | [423](../crates/sifr_hir/src/lower/typing_and_functions.rs:423) | `only integer literals are supported in type annotations` | [invalid_float_literal_type_annotation.sifr](../crates/sifr/tests/e2e/fail/invalid_float_literal_type_annotation.sifr:1) |
| 3 | [437](../crates/sifr_hir/src/lower/typing_and_functions.rs:437) | `unsupported type annotation base` | [invalid_type_annotation_base.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation_base.sifr:1) |
| 4 | [453](../crates/sifr_hir/src/lower/typing_and_functions.rs:453) | `dict type annotation requires exactly 2 type parameters` | [dict_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/dict_type_annotation_wrong_arity.sifr:1) |
| 5 | [463](../crates/sifr_hir/src/lower/typing_and_functions.rs:463) | `dict type annotation requires [K, V] syntax` | [invalid_type_annotation.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr:1) (registry representative) |
| 6 | [498](../crates/sifr_hir/src/lower/typing_and_functions.rs:498) | `Result type annotation requires exactly 2 type parameters` | [result_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/result_type_annotation_wrong_arity.sifr:1) |
| 7 | [518](../crates/sifr_hir/src/lower/typing_and_functions.rs:518) | `Result type annotation requires [T, E] syntax` | [result_type_annotation_wrong_syntax.sifr](../crates/sifr/tests/e2e/fail/result_type_annotation_wrong_syntax.sifr:1) |
| 8 | [542](../crates/sifr_hir/src/lower/typing_and_functions.rs:542) | `Callable type requires exactly 2 type parameters: [[param_types], return_type]` | [callable_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_arity.sifr:1) |
| 9 | [555](../crates/sifr_hir/src/lower/typing_and_functions.rs:555) | `Callable parameter types must be a list: Callable[[int, str], bool]` | [callable_type_annotation_param_list_required.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_param_list_required.sifr:1) |
| 10 | [574](../crates/sifr_hir/src/lower/typing_and_functions.rs:574) | `Callable type requires [[param_types], return_type] syntax` | [callable_type_annotation_wrong_syntax.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_syntax.sifr:1) |
| 11 | [699](../crates/sifr_hir/src/lower/typing_and_functions.rs:699) | `unsupported type annotation expression` | [invalid_type_annotation_expression.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation_expression.sifr:1) |

All 11 in-scope migrations now have 1:1 fixtures pinning both the active `SIFR-TYPE-0007` code and a verbatim slice of the unchanged emitted text.

### F3 — Diff is still tightly scoped

`git status` shows the same two modified files as pass 1 (`typing_and_functions.rs` plus the issue checklist) and exactly **one new** untracked fixture beyond the pass-1 set, alongside the pass-1 review file itself. No baselines, no `verification/`, no docs, no schema, no registry, no driver edits. Pass-1 finding F8 (registry's `representative_fixture_path` for `SIFR-TYPE-0007` already lines up with the pre-existing `invalid_type_annotation.sifr`) remains satisfied without any registry edit.

### F4 — Validation re-run is consistent with prior slices

`scripts/run_all_tests.sh --profile quick` reports `report_signature=e1bf653aaa770517` (matching the established stable hash carried across slices 2b.3–2b.7 — see pass-1 R6) and `wall_time=75.13s` (a fresh wall-time delta from the pass-1 run's `85.60s`, consistent with the deterministic-set-hash interpretation rather than a stale cache). The full local gate (`cargo fmt --check`, HIR maintainability guardrails, transport-level diagnostic tests, full unit test suite, clippy with `-D warnings`, full e2e fail suite) is green per the implementer's report.

## Residual risks

All carried forward from pass 1; none re-opened or aggravated by the pass-2 delta.

### R1 — Pass-1 R1 (site #1 fixture) — **CLOSED**

Closed by the new [integer_literal_too_large_type_annotation.sifr](../crates/sifr/tests/e2e/fail/integer_literal_too_large_type_annotation.sifr:1) fixture; see F1.

### R2 — Registry `message_template` for `SIFR-TYPE-0007` still does not match any of the 11 emitted strings

[codes.rs:631](../crates/sifr_diagnostics/src/codes.rs:631) declares `"invalid type annotation for {annotation_kind}"` with `declared_args = ["annotation_kind"]`. None of the 11 migrated emissions follow that template; they each render free-form text via the pre-formatted-`String` API at [`LowerCtx::error_with_code`](../crates/sifr_hir/src/lower/mod.rs:228). Same structural drift carried across slices 2b.5 / 2b.6 / 2b.7 — fixtures correctly assert on the rendered text, so e2e contracts stay stable. Non-blocking; concrete design question for the future builder-migration slice that resolves [issue:432](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:432).

### R3 — Class-context double-firing of `resolve_annotation_expr`

`collect_class_type` is invoked twice ([mod.rs:584](../crates/sifr_hir/src/lower/mod.rs:584), [mod.rs:594](../crates/sifr_hir/src/lower/mod.rs:594)), and `resolve_annotation_expr` has no idempotency guard while `LowerCtx::error_with_code` does no exact-duplicate suppression — so a malformed annotation on a class field / init param / method param will push two identical `LoweringError` records. None of the 11 fixtures triggers this (they're all top-level `def` shapes); the `apply_diagnostic_recovery_limits` cap at [diagnostics.rs:179-202](../crates/sifr_driver/src/diagnostics.rs:179) keeps the user-facing fan-out bounded but does not collapse exact duplicates. Pre-existing milestone-level issue — out of scope here, deferred to the dedupe slice.

### R4 — TypeVar bound/constraint shape errors still on the bridge

[mod.rs:271](../crates/sifr_hir/src/lower/mod.rs:271), [277](../crates/sifr_hir/src/lower/mod.rs:277), [299](../crates/sifr_hir/src/lower/mod.rs:299), [313](../crates/sifr_hir/src/lower/mod.rs:313), [325](../crates/sifr_hir/src/lower/mod.rs:325), [335](../crates/sifr_hir/src/lower/mod.rs:335), [347](../crates/sifr_hir/src/lower/mod.rs:347) — "TypeVar constraints must be simple type names" and siblings — live outside `resolve_annotation_expr` and are conceptually annotation-shape errors. Out of scope for this slice; candidate for a near-future `TYPE-0007` family-completion slice.

### R5 — No registry-level test guards `representative_fixture_path` against fixture rename / substring drift

[codes.rs:1465](../crates/sifr_diagnostics/src/codes.rs:1465) only asserts `representative_fixture_path` is `Some(...)` for active codes. With **eleven** sibling fixtures with similar names now present (`invalid_type_annotation.sifr`, `invalid_type_annotation_base.sifr`, `invalid_type_annotation_expression.sifr`, `invalid_float_literal_type_annotation.sifr`, `integer_literal_too_large_type_annotation.sifr`, the three `result_type_annotation_*` and three `callable_type_annotation_*`), a rename or `expect-error` re-key in a later slice could silently desync from the registry's hardcoded path at [codes.rs:630](../crates/sifr_diagnostics/src/codes.rs:630). Same milestone-level pattern flagged 2b.5 onward — absorbed by `scripts/check_diagnostic_code_coverage.py` planned in `milestone_diag_11` per [issue:1236](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1236).

### R6 — `report_signature` matching across slices is the deterministic-set-hash pattern

The hash `e1bf653aaa770517` is now reproduced across slices 2b.3, 2b.4, 2b.5, 2b.7, and both 2b.8 runs (pass 1 wall-time `85.60s`, pass 2 wall-time `75.13s`). Distinct wall-time values across runs with an identical signature reinforce the deterministic-test-set-hash reading rather than a stale cache. No action.

### R7 — No companion `expect-pass` fixture for any of the 11 sites

The new overflow fixture, like its ten siblings, exercises only the failure direction. A regression that made e.g. the `Number::Int` `as_i64() == Some(_)` branch always emit `SIFR-TYPE-0007` regardless of overflow would only be caught by the implicit happy-path coverage in the pass suite, not by anything fixture-local. Consistent with the established style across slices 2b.3–2b.7. Non-blocking.

## Verdict

Satisfied / no blocking findings; reviewer-satisfied. The single pass-2 delta — adding [integer_literal_too_large_type_annotation.sifr](../crates/sifr/tests/e2e/fail/integer_literal_too_large_type_annotation.sifr:1) — closes pass-1 residual R1 with a minimal fixture that demonstrably routes to the [line 419](../crates/sifr_hir/src/lower/typing_and_functions.rs:419) overflow branch (literal ≈ 10²⁹ ≫ `i64::MAX`, parsed as `Number::Int`, `as_i64()` returns `None`, the `invalid_type_annotation` helper emits `SIFR-TYPE-0007` with the expected substring), and the e2e fail harness verifies both the structured code stamp and the message slice. Coverage of the 11 in-scope migrated sites in `resolve_annotation_expr` is now 11/11. All other pass-1 findings (helper DRY, scope split between `TYPE-0007` and the unmigrated unknown-name / Result-error-validity / generic-alias-arity / generic-class-shape / unknown-generic-name / TypeVar paths, registry alignment, tightly-scoped diff, clean checklist transitions) carry forward unchanged. Residual risks R2–R7 are pre-existing and milestone-structural; none is blocking. Local validation gate is green and `report_signature` matches the established stable hash across recent slices.
