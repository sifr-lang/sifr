---
name: semantic-diagnostic-code-taxonomy-diag-4a-typevar-constraint-diagnostics-review-pass-1
description: Review pass 1 — verify slice 2b.23 migrates the generic-call TypeVar constraint application failure from the SIFR-TYPE-0001 bridge to the new active SIFR-TYPE-0010 code end-to-end.
---

# Review — `milestone_diag_4a` slice 2b.23: TypeVar constraint application diagnostics

- Branch: `codex/semantic-diagnostics-diag-4a-typevar-constraint-diagnostics`
- Scope: introduce active `SIFR-TYPE-0010` ("TypeVar constraints are not satisfied by the inferred concrete type"), migrate the single generic-call constraint emission inside [`lower_call`](../crates/sifr_hir/src/lower/expressions.rs:1935) from `ctx.error(...)` (legacy `CompilePhase::TypeCheck` → `SIFR-TYPE-0001` bridge at [sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137)) onto `ctx.error_with_code(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED, ...)`, re-key the existing `typevar_constraints_violation.sifr` fixture, add a structured-identity unit test, and emit the standard registry/docs surface.
- Pass: 1
- Prior related reviews:
  - [reviews/semantic-diagnostic-code-taxonomy-diag-4a-typevar-shape-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-typevar-shape-review-pass-1.md) (slice 2b.10 — TypeVar bound/constraint *declaration shape* migration to `SIFR-TYPE-0007`; explicitly deferred this slice's "constraint conformance failure" path to a later slice — F6).
  - [reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-signature-diagnostics-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-protocol-signature-diagnostics-review-pass-1.md) (slice 2b.22 — most recent merged slice on this cadence).
  - [reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-1.md) (slice 2b.20 — sibling check at the same call site, migrated to `SIFR-PROTO-0001`).

## Summary

The slice does what was advertised and stays inside the declared narrow scope. It introduces one new constant (`DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED`, [crates/sifr_diagnostics/src/codes.rs:37](../crates/sifr_diagnostics/src/codes.rs:37)), one new registry entry pinned to the existing fixture ([codes.rs:663-673](../crates/sifr_diagnostics/src/codes.rs:663)), one new entry in `ACTIVE_DIAGNOSTIC_CODES` ([codes.rs:1360](../crates/sifr_diagnostics/src/codes.rs:1360)), routes the single in-scope call site at [crates/sifr_hir/src/lower/expressions.rs:1935-1949](../crates/sifr_hir/src/lower/expressions.rs:1935) through `LowerCtx::error_with_code` ([mod.rs:237](../crates/sifr_hir/src/lower/mod.rs:237)) with the new code, leaves the sibling free-form `ctx.error(...)` calls in `lower_call` (argument type-mismatch, lines 1886/1898) untouched, generates `docs/errors/SIFR-TYPE-0010.md`, regenerates `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md`, re-keys `crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr` from `SIFR-TYPE-0001` to `SIFR-TYPE-0010`, and adds a structured-identity unit test in [expressions_tests.rs:1855-1867](../crates/sifr_hir/src/lower/expressions_tests.rs:1855) that asserts both the exact rendered message and `e.code == Some(...)`.

The implementer-reported validation set (`gen-error-docs`, `cargo fmt`/`cargo fmt --check`, `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir typevar_constraints_violation`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`) covers exactly the surface this slice touches; nothing else is at meaningful risk of regressing. The phase tracker bookkeeping (2b.22 → merged with PR 1694; 2b.23 → in progress, PR pending) is correct and matches the wording cadence of prior slices.

I did not find any blockers. The findings below are confirmations and minor non-blocking observations, including one stylistic note on inline emission vs. the `*_diagnostics.rs` helper-module pattern recently used by sibling slices, one observation about implicit class-constructor coverage, one observation about positional vs. named placeholder formatting, and one carry-forward observation about pre-existing unit-test gaps for the *unmigrated* sibling check at lines 1886/1898.

## Findings

### 1. New code constant, registry entry, and active-codes array membership are coherent (confirmation)

- [crates/sifr_diagnostics/src/codes.rs:37-38](../crates/sifr_diagnostics/src/codes.rs:37) declares `pub const TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED: Self = Self::new("SIFR-TYPE-0010", Severity::Error);`. The id slot `0010` continues the `SIFR-TYPE-` numerical sequence after `0007`/`0008`/`0009`, matches the family-base constant `SIFR-TYPE-0000` reserved at [codes.rs:383](../crates/sifr_diagnostics/src/codes.rs:383), and does not collide with the warning/note tier `0901`/`0902`. The constant name is parallel to the registered family conventions (`TYPE_INVALID_ANNOTATION`, `TYPE_CONTAINER_ELEMENT_CONFLICT`, `TYPE_UNPACK_SHAPE_MISMATCH`) and reads cleanly at the call site.

- [codes.rs:663-673](../crates/sifr_diagnostics/src/codes.rs:663) registry entry:

  ```
  active_entry!(
      "SIFR-TYPE-0010",
      "TYPE",
      "TypeVar constraints are not satisfied by the inferred concrete type.",
      Severity::Error,
      "crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr",
      "type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'",
      "sifr_hir::lower::expressions",
      [arg!("actual"), arg!("constraints"), arg!("type_param")],
      ["actual", "constraints", "type_param"]
  ),
  ```

  The `Severity::Error` slot matches the constant's severity (`Self::new(..., Severity::Error)`). The `representative_fixture_path` resolves to a real file that emits exactly this code post-migration (verified — see finding 5). The `message_template` is byte-identical to the call-site format string (after substituting positional `{}` for named `{actual}`/`{constraints}`/`{type_param}`). The `owner_module = "sifr_hir::lower::expressions"` correctly identifies the lowering file containing the emission; this is the call-site convention slice 2b.21 and 2b.22 already established (`SIFR-PROTO-0003` → `sifr_hir::lower::statements`, `SIFR-PROTO-0002` → `sifr_hir::lower::classes`). The `declared_args` and `dedupe_args` lists are the same three-name set in the same order, both consistent with the named placeholders in the template; this is how slice 2b.22 also wired its three-arg helper.

- [codes.rs:1360](../crates/sifr_diagnostics/src/codes.rs:1360) inserts `DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED` into `ACTIVE_DIAGNOSTIC_CODES` between `TYPE_UNPACK_SHAPE_MISMATCH` (0009) and `TYPE_ARITHMETIC_OVERFLOW_RISK` (0901). Insertion order is the established id-major-then-numeric convention. Membership of this array is what the `check_diagnostic_schema_sync.py` and `check_diagnostic_docs_sync.py` gates traverse to ensure all active codes have docs/registry entries; no orphan-code risk.

### 2. Single-site call-site migration; message preservation is exact (confirmation)

[crates/sifr_hir/src/lower/expressions.rs:1935-1949](../crates/sifr_hir/src/lower/expressions.rs:1935):

```rust
if !constraints.is_empty()
    && !constraints.iter().any(|constraint| {
        type_satisfies_constraint(concrete_ty, constraint, ctx)
    })
{
    ctx.error_with_code(
        DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED,
        format!(
            "type '{}' does not satisfy constraints ({}) required by type parameter '{}'",
            concrete_ty.display_name(),
            constraints.join(", "),
            tv_name
        ),
    );
}
```

vs. previous emission via raw `ctx.error(...)` — the format string, argument order, and runtime values are identical to the pre-migration code (verified against the diff at [expressions.rs:1937-1948](../crates/sifr_hir/src/lower/expressions.rs:1937)). Only the routing changes: the `LoweringError` now carries `code: Some(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED)` ([mod.rs:237-244](../crates/sifr_hir/src/lower/mod.rs:237)) instead of `code: None` ([mod.rs:220-227](../crates/sifr_hir/src/lower/mod.rs:220)).

A repo-wide grep for `does not satisfy constraints` (excluding `third_party/`, `target/`, archived issues, archived reviews) returns exactly the expected set:

| Source | Line | Form |
|---|---|---|
| [crates/sifr_hir/src/lower/expressions.rs](../crates/sifr_hir/src/lower/expressions.rs:1943) | 1943 | call-site format string (positional `{}`) |
| [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs:669) | 669 | registry template (named placeholders) |
| [crates/sifr_hir/src/lower/expressions_tests.rs](../crates/sifr_hir/src/lower/expressions_tests.rs:1864) | 1864 | unit-test assertion (post-substitution rendered text) |
| [crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr](../crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr:1) | 1 | e2e `expect-error` substring (post-substitution) |
| [docs/errors/SIFR-TYPE-0010.md](../docs/errors/SIFR-TYPE-0010.md:13) | 13 | generated docs row |
| [internal_docs/diagnostic_codes.md](../internal_docs/diagnostic_codes.md:86) | 86 | generated registry-table row |

No other call site emits the constraint-violation message, and no archived/active fixture other than the re-keyed one pins this exact substring. The migration is therefore atomic — one emission point, one fixture, one docs row.

### 3. End-to-end wiring verified (confirmation)

The `LoweringError` → `CompileError` → `CompilerDiagnostic` pipeline preserves the structured code:

- [crates/sifr_hir/src/lower/mod.rs:237-244](../crates/sifr_hir/src/lower/mod.rs:237): `error_with_code` records `code: Some(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED)` on the `LoweringError`.
- [crates/sifr_driver/src/frontend/module_lowering.rs:47-51](../crates/sifr_driver/src/frontend/module_lowering.rs:47): when `error.code` is `Some`, it calls `CompileError::with_code(message, CompilePhase::TypeCheck, code)`, which sets `code: Some(code)` on the `CompileError`.
- [crates/sifr_driver/src/diagnostics.rs:125-141](../crates/sifr_driver/src/diagnostics.rs:125): `diagnostic_code()` returns `code.code()` (i.e., `"SIFR-TYPE-0010"`) when `Some(code)` is present, falling back to the phase-based `"SIFR-TYPE-0001"` only when `code.is_none()`.

Net result: the diagnostic surfaces as `SIFR-TYPE-0010` end-to-end. Both the e2e fixture's substring/code assertion at [crates/sifr/tests/e2e.rs:2552-2567](../crates/sifr/tests/e2e.rs:2552) (`failure.code == expected.code && failure.message.contains(message)`) and the new unit test at [expressions_tests.rs:1862-1866](../crates/sifr_hir/src/lower/expressions_tests.rs:1862) (`e.code == Some(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED)`) gate this — either failing would surface immediately under `cargo test -p sifr --test e2e -- test_e2e_fail` or `cargo test -p sifr_hir typevar_constraints_violation`.

### 4. Slice scope correctly excludes the sibling argument-type-mismatch branches (confirmation)

`lower_call` at [crates/sifr_hir/src/lower/expressions.rs:580](../crates/sifr_hir/src/lower/expressions.rs:580) emits multiple distinct error categories during generic-call lowering. After this slice:

| Branch | Site | Status |
|---|---|---|
| TypeVar constraint conformance failure | [expressions.rs:1935-1949](../crates/sifr_hir/src/lower/expressions.rs:1935) | **Migrated** to `SIFR-TYPE-0010` (this slice) |
| Generic-arg type mismatch (unresolved TypeVars present) | [expressions.rs:1885-1894](../crates/sifr_hir/src/lower/expressions.rs:1885) | Unchanged — raw `ctx.error("argument N ('p') of function 'F': expected '...', got '...'")`, still routes through `SIFR-TYPE-0001` |
| Generic-arg type mismatch (resolved TypeVars) | [expressions.rs:1897-1906](../crates/sifr_hir/src/lower/expressions.rs:1897) | Unchanged — same form, same bridge |
| Protocol bound on TypeVar (`T: SomeProtocol`) | [expressions.rs:1924-1933](../crates/sifr_hir/src/lower/expressions.rs:1924) via [protocol_diagnostics::bound_not_satisfied](../crates/sifr_hir/src/lower/protocol_diagnostics.rs:5) | **Already migrated** to `SIFR-PROTO-0001` (slice 2b.20) |

This matches the slice statement ("the existing typevar_constraints_violation fixture and the generic-call constraint check"). The two adjacent argument type-mismatch branches share a different free-form template (`argument N ('p') of function 'F': expected …, got …`) that is structurally distinct from the constraint-violation message — there is no risk of accidentally co-migrating them. The constraint check and the protocol-bound check sit in the same `for (tv_name, concrete_ty) in &bindings` loop at [expressions.rs:1912-1951](../crates/sifr_hir/src/lower/expressions.rs:1912) but operate on disjoint sub-classifications of `specs` (`required_bounds` vs. `constraints`, partitioned by [expressions.rs:1916-1922](../crates/sifr_hir/src/lower/expressions.rs:1916) using the `__constraint__:` prefix encoded by [mod.rs:259-263](../crates/sifr_hir/src/lower/mod.rs:259)). They cannot fire on the same `tv_name`/`concrete_ty` pair if `concrete_ty` would satisfy *neither*: the protocol bound check runs over `required_bounds`, the constraint check over `constraints`. Since `parse_typevar_bound_expr` and `parse_typevar_declaration_specs` either populate constraints (positional/`constraints=` kw) or a bound (`bound=` kw or PEP 695 single-name) but not both for the same TypeVar, the two checks are exclusive in practice. A TypeVar with constraints will only ever route through `SIFR-TYPE-0010`; a TypeVar with a protocol bound will only ever route through `SIFR-PROTO-0001`. No risk of duplicated diagnostics post-migration.

### 5. Re-keyed fixture covers the new identity (confirmation)

[crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr:1](../crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr:1) flips from `# expect-error: SIFR-TYPE-0001: type 'float' does not satisfy constraints (int, str) required by type parameter 'T'` to `# expect-error: SIFR-TYPE-0010: ...` — the message substring is preserved verbatim, only the code prefix changes. The fixture body (lines 2-12) is unchanged: `T = TypeVar("T", int, str); def echo(x: T) -> T: return x; def main(): bad: float = echo(1.5)`. Walking the fixture through `lower_call`: `echo` is registered in `ctx.generic_functions` (via `register_generic_function` during typing pass), `ctx.type_param_bounds["echo"]["T"]` is populated with `["__constraint__:int", "__constraint__:str"]` by [mod.rs:262-263](../crates/sifr_hir/src/lower/mod.rs:262) and [parse_typevar_declaration_specs](../crates/sifr_hir/src/lower/mod.rs:296), `bindings["T"] = Float` is inferred from `echo(1.5)`, the constraints partition produces `["int", "str"]`, neither satisfies `Float`, so the new emission fires — exactly the path the e2e harness exercises.

The harness contract at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) checks `failure.code == expected.code && failure.message.contains(expected.message_contains)` — both halves are gated by the new fixture line, so a regression that broke either the code-routing or the message-rendering would fail this single fixture.

### 6. Unit-test coverage adds structured identity assertion (confirmation)

The new test at [crates/sifr_hir/src/lower/expressions_tests.rs:1855-1867](../crates/sifr_hir/src/lower/expressions_tests.rs:1855):

```rust
#[test]
fn test_typevar_constraints_violation_has_type_code() {
    let result = lower_source(
        "from typing import TypeVar\n\nT = TypeVar(\"T\", int, str)\n\ndef echo(x: T) -> T:\n    return x\n\ndef main():\n    bad: float = echo(1.5)\n    print(bad)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            == "type 'float' does not satisfy constraints (int, str) required by type parameter 'T'"
            && e.code == Some(DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED)
    }));
}
```

This is the structured-identity gate at the lowering layer. The two assertions — exact message equality (`==`, not `contains`) and `e.code == Some(...)` — together pin the *full* identity. A regression that left the message alone but stripped the code would fail this test even though the message-only e2e contract would still pass; the inverse is also caught. Test source mirrors the e2e fixture's body, so the two layers exercise identical lowering input.

The placement (after `test_generic_class_subscript_arity_mismatch_errors`, before `test_match_tuple_pattern_requires_tuple_subject`) keeps the test in the existing TYPE-family cluster of unit tests in this file. The naming convention `test_typevar_constraints_violation_has_type_code` matches the established `test_*_has_*_code` pattern used throughout `expressions_tests.rs` and `protocol_diagnostics.rs::tests`.

### 7. Generated docs and registry tables are coherent (confirmation)

- [docs/errors/SIFR-TYPE-0010.md](../docs/errors/SIFR-TYPE-0010.md:1) is a freshly generated `gen-error-docs` page that mirrors the registry entry: code, family `TYPE`, severity `Error`, owner `sifr_hir::lower::expressions`, message template byte-identical to [codes.rs:669](../crates/sifr_diagnostics/src/codes.rs:669), representative fixture path resolves, declared/dedupe args render in registry order. The "Generated by … Do not edit by hand." preamble matches every other generated doc page.
- [docs/errors/diagnostic-codes.md:57](../docs/errors/diagnostic-codes.md:57) inserts the row `| [SIFR-TYPE-0010](SIFR-TYPE-0010.md) | Error | TypeVar constraints are not satisfied by the inferred concrete type. |` between the `0009` (unpack shape mismatch) and `0901` (arithmetic overflow warning) rows. Insertion ordering is the existing id-numeric sort.
- [internal_docs/diagnostic_codes.md:86](../internal_docs/diagnostic_codes.md:86) inserts the matching expanded-row table entry with state `Active`, the registry's docs path, the representative fixture, the owner, the template, the declared/dedupe arg lists, and `false` for `fix_all_eligible`. Field-by-field this matches the pattern established by `0007`/`0008`/`0009` rows above.

All three are products of the docs-generator + sync-script gates the implementer ran; the diff stays inside the generated payload and does not edit any hand-curated text outside the table rows.

### 8. Inline emission vs. helper-module convention (minor — stylistic, not blocking)

Slices 2b.20 / 2b.21 / 2b.22 each routed their migrations through small helper functions in [crates/sifr_hir/src/lower/protocol_diagnostics.rs](../crates/sifr_hir/src/lower/protocol_diagnostics.rs:1) (`bound_not_satisfied`, `context_manager_missing`, `iterator_invalid_return_signature`) — three or four lines each, one per code. Slice 2b.23 instead inlines the `error_with_code` invocation directly inside the conditional block at [expressions.rs:1940-1948](../crates/sifr_hir/src/lower/expressions.rs:1940).

The codebase already has precedent for both styles: the inline emission for `TYPE_CONTAINER_ELEMENT_CONFLICT` at [expressions.rs:1986-1992](../crates/sifr_hir/src/lower/expressions.rs:1986), the inline emissions for `TYPE_UNPACK_SHAPE_MISMATCH` in [tuple_unpack.rs:62-79,166-170](../crates/sifr_hir/src/lower/tuple_unpack.rs:62), and the file-private helper for `TYPE_INVALID_ANNOTATION` (`invalid_typevar_shape` at [mod.rs:268-271](../crates/sifr_hir/src/lower/mod.rs:268), `invalid_type_annotation` at [typing_and_functions.rs:380-382](../crates/sifr_hir/src/lower/typing_and_functions.rs:380)) all coexist. For a *single* call site, inline is clearly the lighter choice; the `protocol_diagnostics.rs` pattern earned its keep because each of those slices wired multiple call sites (or set up infrastructure for future ones in the same family).

Two reasons to flag this as worth noting rather than as a blocker:

1. The protocol-family slices kept their helpers' co-located unit tests (`mod tests` inside `protocol_diagnostics.rs`) as the structured-identity gate. Slice 2b.23 places its structured-identity test in `expressions_tests.rs` instead. Either location is fine; this slice picks the heavier-traffic test file. There is no functional difference, only a discoverability difference: a future contributor grepping `protocol_diagnostics.rs` for the constraint-violation pattern won't find it (because it lives in a different file), while a grep for `TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED` finds both producer and test cleanly.

2. If a second emission site for `SIFR-TYPE-0010` ever appears (e.g., for class-constructor constraint conformance — see finding 9), promoting the inline emission to a helper at that point is a one-line refactor. Pre-emptively introducing the helper now would be scope creep against the slice's stated narrow scope.

Non-blocking. Two zero-risk follow-ups for a future pass; either is fine, neither is required:

1. Extract a small `typevar_constraint_not_satisfied(ctx, actual, constraints, type_param)` helper (in `expressions.rs` or a new sibling module like `type_diagnostics.rs`) for parity with the protocol family.
2. Leave inline; revisit only if/when a second call site appears.

### 9. Class-constructor constraint coverage is implicitly migrated but lacks a dedicated fixture (minor — pre-existing gap, not introduced)

The constraint check at [expressions.rs:1935-1949](../crates/sifr_hir/src/lower/expressions.rs:1935) is gated on `ctx.generic_functions.contains_key(&func_name)` ([expressions.rs:1870](../crates/sifr_hir/src/lower/expressions.rs:1870)). [classes.rs:606-611](../crates/sifr_hir/src/lower/classes.rs:606) registers generic class constructors in `generic_functions` keyed by the class name:

```rust
if let Some(type_params) = ctx.class_declared_type_params.get(&class_name).cloned() {
    if !type_params.is_empty() {
        ctx.generic_functions
            .insert(class_name.clone(), type_params);
    }
}
```

And [classes.rs:210-218](../crates/sifr_hir/src/lower/classes.rs:210) populates `type_param_bounds` for the class, including any `__constraint__:`-prefixed specs returned by `parse_typevar_bound_expr` for PEP 695 `class Pair[T: (int, str)]` declarations. This means the constraint-violation diagnostic now also fires (with `func_name = <ClassName>` rather than a function name) for class constructor calls that would violate constraints. The migration to `SIFR-TYPE-0010` therefore extends to that path automatically — which is correct and is the desired identity.

There is no fixture exercising the class-constructor constraint path: a repo-wide grep for `class.*\[T:.*(.*,.*)\]` in `crates/sifr/tests/` returns no fail fixtures. This is a pre-existing coverage gap that the slice inherits but does not introduce — the previous SIFR-TYPE-0001-bridge regime had the same shape (the legacy bridge would fire under both paths but only the function path had a fixture). Adding a `class_typevar_constraints_violation.sifr` fixture would tighten coverage for the new active code, but doing so in this slice would expand it beyond the stated narrow scope ("the existing typevar_constraints_violation fixture and the generic-call constraint check"). Worth absorbing into a follow-up before the `SIFR-TYPE-0001` bridge is finally removed (per the deferred-bridge-deletion line at [issues/...:59](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:59)).

Non-blocking; record-only observation.

### 10. Format string uses positional `{}` while registry template uses named placeholders (minor — documentation drift, not blocking)

The call site at [expressions.rs:1942-1947](../crates/sifr_hir/src/lower/expressions.rs:1942) uses positional placeholders:

```rust
"type '{}' does not satisfy constraints ({}) required by type parameter '{}'"
```

with three positional values in the order `concrete_ty.display_name(), constraints.join(", "), tv_name`. The registry template at [codes.rs:669](../crates/sifr_diagnostics/src/codes.rs:669) uses named placeholders:

```
"type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'"
```

mapped to `declared_args = [arg!("actual"), arg!("constraints"), arg!("type_param")]`. Post-substitution the two render byte-identically (verified by the unit test's `==` assertion in finding 6 and the e2e fixture's substring assertion in finding 5), so there is no functional drift today.

The pattern *across the migrated codes* is mixed. Slice 2b.20 (`PROTO_BOUND_NOT_SATISFIED`) and slice 2b.21 (`PROTO_CONTEXT_MANAGER_MISSING`) use named placeholders at both call site and registry; slice 2b.22 (`PROTO_INVALID_ITERATOR_SIGNATURE`) uses named placeholders at both layers as well; the older inline migrations (`TYPE_CONTAINER_ELEMENT_CONFLICT`, `TYPE_UNPACK_SHAPE_MISMATCH`) use a mix. The named-placeholder form is purely cosmetic at the call site (Rust's `format!` accepts both), but it has a small future-proofing benefit: if/when the structured-arg pipeline starts consuming `declared_args` at runtime (rather than treating them as documentation), the named-placeholder call site is the form the future builder API will most naturally lift from.

Non-blocking. Two zero-risk follow-ups, identical to slice 2b.10's R2:

1. Convert the call-site format to `format!("type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'", actual = ..., constraints = ..., type_param = ...)`. Pure refactor, no behavior change.
2. Leave it; the documentation-only nature of the registry's `declared_args` makes positional acceptable today.

### 11. Phase tracker bookkeeping (confirmation)

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:57-58](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:57) correctly:

- Flips slice 2b.22 from `[ ] implementation complete and reviewer-satisfied` to `[x] merged ... PR: https://github.com/sifr-lang/sifr/pull/1694` — consistent with `git log` showing `4eacdeae Migrate protocol signature diagnostic code (#1694)` already merged.
- Adds slice 2b.23 as `[ ]` in-progress with `PR: pending`, naming the active code (`SIFR-TYPE-0010`) and scope (TypeVar constraint application diagnostics with fixture coverage) in the same wording cadence as 2b.20-2b.22.

The deferred-bridge-deletion line at [:59](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:59) remains correctly marked `[x]` because the legacy `CompilePhase::TypeCheck` → `SIFR-TYPE-0001` mapping at [sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) is still in place and is still load-bearing for the many remaining unmigrated free-form `ctx.error(...)` calls in the HIR (180 sites in `expressions.rs` alone, plus calls in `mod.rs`, `statements.rs`, etc.). The bridge cannot be deleted until those are domain-migrated, which the line explicitly defers to later slices. No drift.

### 12. Diff is tightly scoped; no orphan fixtures, no stale callers (confirmation)

`git status` shows the exact expected eight-file delta:
- 7 modifications: `crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr`, `crates/sifr_diagnostics/src/codes.rs`, `crates/sifr_hir/src/lower/expressions.rs`, `crates/sifr_hir/src/lower/expressions_tests.rs`, `docs/errors/diagnostic-codes.md`, `internal_docs/diagnostic_codes.md`, `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`.
- 1 new file: `docs/errors/SIFR-TYPE-0010.md`.

Total diff stat: 42 insertions, 8 deletions across 7 modifications + 1 untracked file. The four `.rs`/`.sifr`/`.md` modifications outside the docs/issues set total 14 + 14 + 15 + 2 + 1 = 46-ish lines, all visibly in the migration-shaped insertion locations (registry entry, active-codes array entry, call-site replacement, unit-test addition, fixture line edit). No stray hunks, no baselines, no codegen output, no demos, no `verification/` edits, no schema-file edits. Repo-wide grep for any leftover `expect-error: SIFR-TYPE-0001` line that mentions "satisfy constraints" returns none, so there are no other fixtures still pinning the legacy bridge for this specific message.

The slice's diff is genuinely just the new constant + registry + active-codes membership, the call-site rewrite, the unit test, the fixture re-key, and the generated docs/registry tables.

### 13. Pass-side coverage of well-formed constrained TypeVars is unbroken (confirmation)

[crates/sifr/tests/e2e/pass/typevar_constraints_basic.sifr](../crates/sifr/tests/e2e/pass/typevar_constraints_basic.sifr:1) exercises `T = TypeVar("T", int, str)` with `int` and `str` arguments — both satisfy the constraints, so no `SIFR-TYPE-0010` should fire. A regression that, say, made the constraint check always fire on a satisfied constraint would surface in this pass fixture (it would no longer compile). No new pass fixture is needed; the existing one is sufficient regression coverage for the migration's "happy path" half.

The negative-shape fixtures from slice 2b.10 ([typevar_positional_constraint_shape.sifr](../crates/sifr/tests/e2e/fail/typevar_positional_constraint_shape.sifr:1) etc.) continue to assert `SIFR-TYPE-0007` for shape errors — those errors fire during `parse_typevar_declaration_specs` *before* the constraint application logic at the call site is even reached, so there is no risk of cross-contamination between `SIFR-TYPE-0007` and `SIFR-TYPE-0010`.

## Residual risks

### R1 — Class-constructor constraint path is migrated implicitly without a dedicated fixture

See finding 9. Pre-existing gap, not introduced by this slice; absorbing it would expand scope. Worth a tracked follow-up so the SIFR-TYPE-0001 bridge deletion doesn't silently lose this path's coverage. Non-blocking.

### R2 — Inline emission deviates from the helper-module convention sibling slices used

See finding 8. Stylistic; both inline and helper-module patterns coexist in the codebase. Non-blocking.

### R3 — Format-string positional placeholders vs. named placeholders in the registry template

See finding 10. Documentation drift only; no functional impact today. Non-blocking; same shape as slice 2b.10's R2.

### R4 — Pre-existing absence of unit-test coverage for the sibling argument-mismatch branches at lines 1886/1898

Those two branches still emit free-form messages via `ctx.error(...)`, route through the `SIFR-TYPE-0001` bridge, and have no `expect-error`-substring fixture coverage in `crates/sifr/tests/e2e/fail/` (a grep for `argument 1 ('` against `expect-error: SIFR-TYPE-0001` finds many *other* mismatch fixtures from non-generic call sites but none that exercise the *generic-call* re-check arms specifically — which fire only when `ft.is_generic()` is true and TypeVar substitution alters the expected type). Pre-existing gap; the slice does not worsen it. Will need a follow-up before bridge deletion. Non-blocking.

### R5 — `display_name()` rendering is locale/format-stable, but no test guards against future changes

The migration relies on `Type::Float.display_name() == "float"`, `Type::Int.display_name() == "int"`, `Type::Str.display_name() == "str"` for the test/fixture's rendered text to remain stable. [crates/sifr_type_system/src/types.rs:500-538](../crates/sifr_type_system/src/types.rs:500) hardcodes these mappings, and a change there would break this slice's exact-equality unit-test assertion (finding 6) — but that's a desirable failure mode (the equality check pins the public-facing message). No risk introduced; recording only because the new test uses `==` rather than `contains`, so it is more sensitive to formatting changes than the e2e harness's substring check.

### R6 — TypeVar bindings iteration order is `HashMap`-based

[expressions.rs:1912](../crates/sifr_hir/src/lower/expressions.rs:1912) iterates `&bindings` (a `HashMap<String, Type>`), and the constraint check inside the loop emits one diagnostic per offending TypeVar. For a function with multiple constrained TypeVars where multiple bindings violate their constraints simultaneously, the emitted-diagnostic order would be non-deterministic across runs. The new fixture and unit test only exercise a single TypeVar, so ordering does not affect their assertions. Pre-existing structural property; the migration does not change determinism semantics. Non-blocking; flag only because future fixtures with multiple constrained TypeVars in one call would need to use `contains` rather than ordered-equality assertions.

### R7 — Implementer's reported validation set did not include `scripts/run_all_tests.sh --profile quick`

The implementer's listed validation gates cover the surface this slice touches (docs sync, schema sync, HIR guardrails, targeted unit tests, e2e fail suite, full unit suite, clippy, `cargo fmt`). The CLAUDE.md authoritative gate is `scripts/run_all_tests.sh --profile quick`, which prior slice review records (e.g., 2b.10's `report_signature=e1bf653aaa770517`, `wall_time=75.98s`) cite. The individual scripts the implementer ran are a substantial subset of what `run_all_tests.sh --profile quick` aggregates, but not necessarily the full superset (the script also invokes the e2e pass suite, runs `cargo build --release` warm-up, and exercises a handful of additional targeted tests). Recommending the implementer either (a) re-validate with `scripts/run_all_tests.sh --profile quick` and record the report signature in the issue tracker for parity with prior slices, or (b) confirm that the listed targeted invocations are equivalent for this slice's surface. Non-blocking — the listed gates exercise every file this slice modifies — but worth recording as a process consistency note.

## Verification I performed

- Read the diff against `main` for all eight changed files via `git diff` (clean — no orphan hunks, no unrelated edits).
- Confirmed the new constant, registry entry, and `ACTIVE_DIAGNOSTIC_CODES` insertion are mutually consistent (id, severity, owner, fixture, template, args).
- Walked the call site at `expressions.rs:1935-1949` to confirm the message format string and runtime-substituted values match the registry template byte-identically post-substitution.
- Walked the `LoweringError` → `CompileError` → `CompilerDiagnostic` pipeline ([mod.rs:237-244](../crates/sifr_hir/src/lower/mod.rs:237) → [module_lowering.rs:47-51](../crates/sifr_driver/src/frontend/module_lowering.rs:47) → [diagnostics.rs:125-141](../crates/sifr_driver/src/diagnostics.rs:125)) to verify the `Some(DiagnosticCode)` branch is taken and the code identity is preserved.
- Confirmed the call-graph: the constraint check fires only inside the `ctx.generic_functions.contains_key(&func_name)` branch of `lower_call`, which is invoked once per call expression by `lower_expression`. No double-firing.
- Cross-checked all places where the constraint-violation message text appears in the repo — call site, registry, unit test, e2e fixture, generated docs row, internal docs row — and confirmed no orphan occurrences and no stale `SIFR-TYPE-0001` references for this specific message.
- Confirmed the `representative_fixture_path` resolves to a real file that emits exactly `SIFR-TYPE-0010` after the migration.
- Confirmed `DIAGNOSTIC_CODES` array membership for `TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED` at `codes.rs:1360` — no orphan-code risk for schema generation.
- Confirmed slice scope: arg-mismatch sibling branches at `expressions.rs:1885-1894` and `1897-1906` remain on `ctx.error(...)`, the protocol-bound branch at `expressions.rs:1924-1933` already routes through `SIFR-PROTO-0001` from slice 2b.20, and class-constructor calls share the same migrated check path.
- Confirmed phase-tracker bookkeeping at `issues/...:57-59` matches the merged-PR record in `git log` (`4eacdeae` for #1694) and follows prior-slice wording cadence.
- Did not re-run the implementer's validation gates; relied on the user's report that `gen-error-docs`, `cargo fmt`/`cargo fmt --check`, both `check_diagnostic_*_sync.py` scripts, `check_hir_maintainability_guardrails.py`, the targeted Cargo test invocations, the e2e fail suite, the full unit suite, and `cargo clippy --workspace -- -D warnings` all passed.

## Recommendation

Mergeable as-is. None of the findings are blockers; all are confirmations or minor non-blocking observations (helper-module convention, named placeholder convention, pre-existing class-constructor-coverage gap, pre-existing arg-mismatch-branch-coverage gap, ordering nit, validation-set parity nit). The slice is correctly scoped to a single emission point, the new active code is wired end-to-end (constant → registry → active-codes membership → call site → fixture → unit test → generated docs), the structured-identity unit test gates against future regressions on both code *and* exact message, and the legacy `SIFR-TYPE-0001` bridge is correctly left in place for the still-unmigrated sibling sites. R1, R2, and R4 should be folded into the bridge-deletion follow-up slice; R3, R5, R6, and R7 are watch-list-only.
