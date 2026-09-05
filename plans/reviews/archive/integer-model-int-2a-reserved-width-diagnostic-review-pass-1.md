# Review: INT-2A — Reserved `int128` / `uint128` width diagnostic (`SIFR-INT-0003`)

Reviewer: agent
Date: 2026-05-05
Branch: `int-2a-reserved-128-width-diagnostic`
Phase: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), milestone INT-2A
Design source of truth: [internal_docs/integer_model.md](internal_docs/integer_model.md)

## Verdict: CHANGES REQUESTED (small)

The diff lands the right substrate for the INT-2A reserved-width slice: a new `SIFR-INT` diagnostic family, a `SIFR-INT-0003` active code wired into `resolve_annotation_expr`, a unit test that locks the message text, code, and primary range against `int128`/`uint128` in alias and parameter annotation positions, and the matching auto-generated doc page. The implementation is small, focused, and correctly scoped to the milestone — no surface-level lowering of `int8`/`int16`/.../`uint64` or `isize`/`usize` (which is INT-2B's job), and no widening of `Type::Int` semantics.

The only blocking findings are two leftover **3-digit** `SIFR-INT-008` / `SIFR-INT-001..011` references inside the issue file that the rest of this PR is explicitly normalizing to 4-digit form. Once those two lines are normalized, the slice is ready to land.

---

## Files reviewed

- [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs) — adds `INT_RESERVED_WIDTH_NAME` constant, `INT` diagnostic family, reserved family base, `SIFR-INT-0003` active entry, and the `ACTIVE_DIAGNOSTIC_CODES` membership.
- [crates/sifr_hir/src/lower/typing_and_functions.rs](crates/sifr_hir/src/lower/typing_and_functions.rs:412) — adds `reserved_integer_width_name` helper and the early `int128`/`uint128` branch in `resolve_annotation_expr`.
- [crates/sifr_hir/src/lower/type_alias_tests.rs](crates/sifr_hir/src/lower/type_alias_tests.rs:113) — adds `test_reserved_integer_width_annotations_have_int_code` covering type-alias right-hand side and a parameter annotation.
- [docs/errors/diagnostic-codes.md](docs/errors/diagnostic-codes.md) — adds the new family row and the `SIFR-INT-0003` active row.
- [docs/errors/SIFR-INT-0003.md](docs/errors/SIFR-INT-0003.md) — auto-generated diagnostic page.
- [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md) — adds the schema row alongside the family table.
- [internal_docs/integer_model.md](internal_docs/integer_model.md) — normalizes `SIFR-INT-003` and `SIFR-INT-004` to 4-digit form, plus the `SIFR-INT-001..011` reservation table.
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) — partial 4-digit normalization across the milestone scopes.

---

## Scope check against INT-2A

INT-2A scope (`issues/...:128-133`) lists five items; this PR only addresses the last:

> Add `SIFR-INT-0003` for reserved `int128`/`uint128` names before support lands.

The first four (lossless integer literal lexeme capture, parser/AST→HIR shim, normalized literal representation across decimal/hex/octal/binary, malformed/over-budget literal diagnostics) are **not** delivered here. That is consistent with the user's framing — "current milestone slice: INT-2A reserved int128/uint128 width diagnostic" — and with the sequencing in the phase plan. The diff does not expand into adjacent slices, which is what we want.

INT-2A acceptance criterion 3 (`issues/...:139`) — "Reserved `int128`/`uint128` names produce a specific reserved-width diagnostic" — is satisfied for **every annotation context that flows through `resolve_annotation_expr`**:

| Context | Coverage path | Asserted in test |
| --- | --- | --- |
| Type-alias RHS, e.g. `type Wide = int128` | `lower::type_aliases` → `resolve_annotation_expr` | Yes (`int128` arm) |
| Parameter annotation, e.g. `def f(x: uint128)` | `build_function_type` → `resolve_annotation_expr` | Yes (`uint128` arm) |
| Return annotation, e.g. `-> int128` | `build_function_type` → `resolve_annotation_expr` | No (relies on the same code path; not asserted) |
| `let` annotation, e.g. `x: int128 = ...` | `lower::statements` → `resolve_annotation_expr` | No (relies on same code path) |
| Container-element annotation, e.g. `list[int128]`, `dict[int128, str]`, `int128 \| None` | `Subscript` / `BinOp` arms in `resolve_annotation_expr` recurse into the same `Expr::Name` branch | No (relies on recursion; not asserted) |

The third column shows what the test covers literally; the second column shows what is structurally guaranteed to flow through the new branch. The unasserted contexts are real, but every one of them recurses into the same `Expr::Name` arm — there is no second annotation resolver to keep in sync — so the milestone scope is met. See N1 below for an optional tightening.

INT-2A validation requires "Negative parser/frontend tests for malformed integer token text and reserved `int128`/`uint128`" (`issues/...:145`). The reserved-name half is delivered as a HIR-level unit test; the malformed-literal half belongs to a later INT-2A slice and is correctly out of scope here.

---

## Correctness review

### `resolve_annotation_expr` — new `int128` / `uint128` branch

[crates/sifr_hir/src/lower/typing_and_functions.rs:435-438](crates/sifr_hir/src/lower/typing_and_functions.rs:435):

```rust
if matches!(name.id.as_str(), "int128" | "uint128") {
    reserved_integer_width_name(ctx, &name.id, name.range());
    return Type::Any;
}
```

- The branch sits **after** the type-variable, type-alias, and class-types lookups. That ordering is deliberate: a user-defined `class int128: ...` or `type int128 = ...` will shadow the reserved diagnostic. For a pre-production language with no `bigint` migration to keep alive, that ordering is acceptable, but it is worth being explicit about (see N4).
- The branch sits **before** `resolve_type_annotation`, so the names short-circuit ahead of the generic `unknown type` fallback — exactly what the design doc requires (`internal_docs/integer_model.md:67`: "must produce `SIFR-INT-0003`, not a generic unresolved-name diagnostic").
- Returning `Type::Any` mirrors `unknown_type`'s existing recovery strategy (`typing_and_functions.rs:404-410`). That keeps the rest of the function/alias usable for downstream lowering and avoids cascading false-positive diagnostics on the same expression. ✓

### `reserved_integer_width_name` helper

[crates/sifr_hir/src/lower/typing_and_functions.rs:412-418](crates/sifr_hir/src/lower/typing_and_functions.rs:412):

```rust
fn reserved_integer_width_name(ctx: &mut LowerCtx, name: &str, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::INT_RESERVED_WIDTH_NAME,
        format!("reserved integer width name '{name}' is not supported yet"),
        range,
    );
}
```

- Signature matches the established `unknown_type` / `invalid_type_annotation` style — a free helper that takes `(ctx, name, range)`. Consistent with the file's conventions. ✓
- Uses `error_with_code_at`, so the diagnostic is tagged with both the code and the primary range. Both fields are checked by the test. ✓
- The single-quoted name in the rendered message (`'int128'`) does not match the registry template `reserved integer width name {name} is not supported yet` literally — the template has no quotes — but this is consistent with the established convention used by `SIFR-NAME-0003` ("unknown type: `{name}`" template, "unknown type: `'unknown'`" rendered message). The structured `args` map preserves the unquoted value for tooling; quotes are presentation-only in the human-readable string. ✓

### Diagnostic registry entry

[crates/sifr_diagnostics/src/codes.rs:742-752](crates/sifr_diagnostics/src/codes.rs:742):

```rust
active_entry!(
    "SIFR-INT-0003",
    "INT",
    "Reserved integer width name used before support lands.",
    Severity::Error,
    "crates/sifr_hir/src/lower/type_alias_tests.rs::test_reserved_integer_width_annotations_have_int_code",
    "reserved integer width name {name} is not supported yet",
    "sifr_hir::lower::typing_and_functions",
    [arg!("name")],
    ["name"]
),
```

- Severity `Error` matches the design doc and the constant declaration at line 62. ✓
- `arg!("name")` (i.e., `MessageAndJson` format) is correct because `{name}` does appear in the message template, so the `placeholders(template)` schema-test guard at codes.rs:1850-1869 will be satisfied. ✓
- Dedupe key `["name"]` is correct: two `int128` references in the same module would otherwise round-trip as duplicate diagnostics; deduping on `name` collapses them per the existing convention. ✓
- Owner module string `sifr_hir::lower::typing_and_functions` matches the actual call site. ✓
- Representative-fixture string matches a real file path so `check_diagnostic_code_coverage.py`'s `fixture_file_exists` guard passes; the script splits on `::` and only checks the file portion (script lines 111-113). ✓

### `INT_RESERVED_WIDTH_NAME` constant placement

[crates/sifr_diagnostics/src/codes.rs:62](crates/sifr_diagnostics/src/codes.rs:62) is inserted between the DECIMAL and CALL constant blocks, which **does** match the family declaration order (`PARSE → NAME → IMPORT → TYPE → DECIMAL → INT → CALL → ...`). ✓ The same placement is used in `ACTIVE_DIAGNOSTIC_CODES`. ✓

### Doc-table placement (non-blocking observation)

The active-entries table in [docs/errors/diagnostic-codes.md:64-67](docs/errors/diagnostic-codes.md:64) and the registry array now place `SIFR-INT-0003` **between** `SIFR-TYPE-0902` and `SIFR-DECIMAL-0001`, even though the family-summary table just above lists `INT` **after** `DECIMAL`. Concretely:

- Family order (codes.rs:269-282 / diagnostic-codes.md:11-22): `… DECIMAL → INT → CALL …`
- Active-entry order (codes.rs:742 / diagnostic-codes.md:65-68): `… SIFR-TYPE-0902 → SIFR-INT-0003 → SIFR-DECIMAL-0001 …`

The active entries do not group cleanly by family in the rendered table. This is a low-impact cosmetic inconsistency — when 0001/0002 land in INT-2B and beyond, future entries should be inserted contiguously somewhere, and the cleaner anchor is "after the last DECIMAL active entry, before the first CALL active entry" (i.e., right before the existing SIFR-CALL-0001 entry around codes.rs:977-ish). Not a blocker. See N3.

### Test coverage — `test_reserved_integer_width_annotations_have_int_code`

[crates/sifr_hir/src/lower/type_alias_tests.rs:113-135](crates/sifr_hir/src/lower/type_alias_tests.rs:113):

```rust
let source = "type Wide = int128\n\ndef take(value: uint128) -> int:\n    return 0\n";
```

- Uses the existing `range_for_after(source, anchor, needle)` helper. The computed ranges are correct: `int128` lands at byte offset 12 (after `type Wide = `), `uint128` at byte offset 37 (after `def take(value: `). Both match what the parser emits as the `Name` expression range. ✓
- Three assertions:
  1. The `int128` diagnostic exists with the expected message, code, and range.
  2. The `uint128` diagnostic exists with the expected message, code, and range.
  3. **No** error in the bag carries `DiagnosticCode::NAME_UNKNOWN_TYPE`. This is the contract that `internal_docs/integer_model.md:67` requires ("must produce `SIFR-INT-0003`, not a generic unresolved-name diagnostic") — and the test asserts it directly. ✓
- The test does **not** assert the total error count, so a future regression that emits an extra incidental diagnostic (e.g., a downstream type-mismatch on the `Wide` alias once int128 returns something other than `Type::Any`) would not be caught here. See N1 for an optional tightening.
- The test does **not** exercise return-type, `let`-annotation, generic-arg, union-arm, or `dict[int128, V]` positions. As noted in the scope-check table, those are structurally guaranteed by recursion through the same `Expr::Name` arm, so they are not strictly required — but a single parametric assertion across positions would lock the recursion guarantee against future refactors. See N1.

### Documentation sync

- Auto-generated [docs/errors/SIFR-INT-0003.md](docs/errors/SIFR-INT-0003.md) is consistent with the registry entry (code, family, severity, message template, owner, fixture, args). ✓
- The new family row in [docs/errors/diagnostic-codes.md](docs/errors/diagnostic-codes.md) and [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md) carries the same summary string as the registry. ✓
- All four diagnostic-doc/schema/coverage scripts pass per the user's pre-review validation. ✓
- The reserved family base `SIFR-INT-0000` is registered both as a family entry and a `reserved_family_base(...)` row in `DIAGNOSTIC_REGISTRY`, matching how every other family is wired. ✓

---

## Blocking findings

### B1. Two `SIFR-INT-008` / `SIFR-INT-001..011` references in the issue were not normalized to 4-digit form

The PR explicitly normalizes the `SIFR-INT-NNN` design shorthand to canonical 4-digit `SIFR-INT-NNNN` form across the design doc and the issue. Two lines in the issue were missed:

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:289](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:289) — `Emit \`SIFR-INT-008\` for fixed-width array/tensor/dataframe arithmetic …` should be ``SIFR-INT-0008``.
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:337](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:337) — `Reserve and document the \`SIFR-INT-001..011\` diagnostic families …` should be ``SIFR-INT-0001..0011``.

These are the only two lines in `internal_docs/`, `docs/`, and `issues/` that still hold the 3-digit form, per:

```
rg -n 'SIFR-INT-(00[1-9]|01[0-1])\b' internal_docs docs issues
```

Why blocking: the rest of the same issue file in this PR is now self-consistent at 4-digit (the milestone scopes for INT-2B/INT-3/INT-4/INT-5 use `SIFR-INT-0004`/`0005`/`0006`/`0007`/`0009`/`0010`/`0011`). Leaving 0008 and the `001..011` range string in the older 3-digit form makes the issue look half-migrated and breaks the canonical-code invariant the design doc states explicitly at integer_model.md:67 and 101. A grep-and-replace that mirrors the rest of this PR fixes both.

How to apply: change `SIFR-INT-008` → `SIFR-INT-0008` and `SIFR-INT-001..011` → `SIFR-INT-0001..0011`. No code, registry, or doc-table changes follow — the registry already uses canonical 4-digit codes throughout.

---

## Non-blocking findings

### N1. Test does not lock recursion through subscript / union / return / let positions

The implementation's correctness for `list[int128]`, `int128 | None`, `def f() -> int128:`, and `x: uint128 = ...` depends on every annotation site funneling through `resolve_annotation_expr` and recursing through the same `Expr::Name` arm. The current test only exercises type-alias RHS and parameter-annotation positions.

Suggestion (optional, can land here or in INT-2B):

- Add a small extra assertion that exercises a return type and a generic position (`-> int128` and `list[uint128]`), and assert the diagnostic count is exactly the number of reserved-name occurrences in the source. The latter — `assert_eq!(errors.len(), 4)` for a fixture containing four reserved-name uses — would also catch any future regression that emits an extra incidental diagnostic on top of the reserved-name one.

This is non-blocking because the structural guarantee already exists; the test would just lock it against refactors.

### N2. Verification inventory still uses the 3-digit shorthand

[verification/integer_model_implementation_inventory.md:46](verification/integer_model_implementation_inventory.md:46) reads:

> `crates/sifr_diagnostics/src/codes.rs`: add `SIFR-INT-001..011`; retire or migrate `TYPE_INT_BIGINT_MIXED`.

This file was created in commit `16819d84` ("Save new integer model with adhoc phase") and is not touched by this PR. The 3-digit form here is historical drift, not a contradiction — but if the project goal is "no 3-digit shorthand anywhere outside reviews," normalizing this single bullet (`SIFR-INT-001..011` → `SIFR-INT-0001..0011`) closes the loop. Pre-existing review files under `reviews/` are immutable closure artifacts and should be left alone.

### N3. `SIFR-INT-0003` active entry is interleaved between TYPE and DECIMAL active entries instead of after DECIMAL

The active-entry order in `DIAGNOSTIC_REGISTRY` is:

```
… SIFR-TYPE-0902, SIFR-INT-0003, SIFR-DECIMAL-0001, SIFR-DECIMAL-0002, …, SIFR-DECIMAL-0008, SIFR-CALL-0001, …
```

But the family-declaration order is `… DECIMAL → INT → CALL …`. The `INT_RESERVED_WIDTH_NAME` *constant* and its `ACTIVE_DIAGNOSTIC_CODES` membership both already sit between the DECIMAL and CALL groups (constant at codes.rs:62, list entry at codes.rs:1561), which is correct. The active-entry block at codes.rs:742-752 is the outlier.

Effect: the auto-generated active-codes table in `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` lists `SIFR-INT-0003` between the TYPE and DECIMAL rows rather than between the DECIMAL and CALL rows. Cosmetic but inconsistent with the family ordering elsewhere in the same files.

Suggestion (optional, low-priority): move the `active_entry!("SIFR-INT-0003", …)` block from codes.rs:742 to immediately before the first CALL `active_entry!` (around codes.rs:977-ish) on a follow-up. The doc tables will regenerate accordingly. Not necessary for this PR; a tracking note in INT-2B's TODO is enough.

### N4. Reserved-name check is shadowable by user-defined `class int128` / `type int128 = …`

Order in `resolve_annotation_expr` (`typing_and_functions.rs:422-442`):

1. type variables (TypeVars)
2. type aliases (`type int128 = ...` would resolve here)
3. class types (`class int128: ...` would resolve here)
4. **new**: reserved-width check
5. built-in resolution

A user who defines either a class or a type alias named `int128` would suppress the reserved diagnostic. The design-doc text "Using either reserved name before support lands must produce `SIFR-INT-0003`" (`internal_docs/integer_model.md:67`) does not say whether reservation should also reject *user redefinitions* of these names.

This is intentional given the existing scaffolding (built-in names like `int`, `bigint`, `decimal` are also shadowable through aliases/classes today), and INT-2B's "no user-facing `bigint`" cleanup is the natural place to take a global stance on reserved-name shadowing. Not a blocker; just call out the choice in the INT-2B follow-up.

### N5. No e2e fixture pair (negative `.sifr` file under `crates/sifr/tests/e2e/fail/`)

Most active codes have a representative `.sifr` fixture under `crates/sifr/tests/e2e/fail/` (e.g., `bigint_int_mixed_arithmetic.sifr`, `decimal_invalid_literal_string.sifr`). `SIFR-INT-0003` uses a HIR-crate unit test instead. The coverage script accepts this (it only checks file existence after splitting on `::`), and INT-2A's validation criterion is satisfied either way. But for consistency with the rest of the registry — and so the e2e snapshot suite covers the rendered diagnostic envelope (URL, span column, JSON args) end-to-end — a small `crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr` fixture would round out the coverage. Optional follow-up; INT-2B is a natural place to bundle this with the other INT fixtures.

### N6. No test for `dict[int128, V]` / `int128 | None` recursion

Mechanically guaranteed via the recursion in `resolve_annotation_expr` (Subscript and BinOp arms both call back into it), but absent from the test. If N1's parametric assertion is added, this overlaps with it. Otherwise, a single extra `range_for_after`-driven assertion against `list[int128]` or `int128 | None` is enough to lock the recursion guarantee.

---

## Coherence with the design doc

Cross-checked against [internal_docs/integer_model.md](internal_docs/integer_model.md):

- `internal_docs/integer_model.md:67` — "Using either reserved name before support lands must produce `SIFR-INT-0003`, not a generic unresolved-name diagnostic." ✓ (`unknown_type` is bypassed for these two names; the test asserts no `NAME_UNKNOWN_TYPE` is emitted.)
- `internal_docs/integer_model.md:447-460` — diagnostic-family table now uses canonical 4-digit form for all of `SIFR-INT-0001..0011`. ✓
- `internal_docs/integer_model.md:60-66` — fixed-width type table is unchanged this slice. The runtime substrate from INT-1 already supports `i128` / `u128` conversion helpers, but the source-level reservation diagnostic is the right gate before INT-2B/INT-3 introduce source-level fixed-width types. ✓ (See `reviews/integer-model-int-1-runtime-fixed-width-conversions-review-pass-1b.md:163` for the rationale that runtime `try_to_i128`/`try_to_u128` are infrastructure, not source-level support.)

No design contradictions surfaced.

---

## Verification of provided commands

The user reports the following passed locally:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_hir test_reserved_integer_width_annotations_have_int_code`
- `cargo test -p sifr_diagnostics`
- `cargo clippy -p sifr_hir -p sifr_diagnostics -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_diagnostic_docs_sync.py`
- `python3 scripts/check_diagnostic_schema_sync.py`
- `python3 scripts/check_diagnostic_code_coverage.py`
- `scripts/run_all_tests.sh --profile quick`

Cross-checks against the diff:

- `check_diagnostic_code_coverage.py` validates that every active code has (a) a `docs/errors/<CODE>.md` page, (b) a fixture file that exists on disk, and (c) at least one non-test compiler-source `DiagnosticCode::<NAME>` use. The new entry satisfies (a) via [docs/errors/SIFR-INT-0003.md](docs/errors/SIFR-INT-0003.md), (b) via the existing `crates/sifr_hir/src/lower/type_alias_tests.rs` file, and (c) via the use in `typing_and_functions.rs:414`. ✓
- `check_diagnostic_docs_sync.py` and `check_diagnostic_schema_sync.py` regenerate from the registry; the registry, family list, and `ACTIVE_DIAGNOSTIC_CODES` are mutually consistent in the diff, so the doc/schema artifacts will round-trip cleanly. ✓
- `check_hir_maintainability_guardrails.py` enforces per-file size and complexity caps in `sifr_hir/src/lower/`. The new helper at `typing_and_functions.rs:412-418` is seven lines; the new test is twenty-three lines added to a file that already holds nine other tests in the same style. No guardrail nudged. ✓

None of these commands would catch B1 or N1–N6, which is expected — B1 is a doc-text grep concern, N1 is a test-strength concern, N2 is a doc-text grep concern outside the diff, and N3–N6 are stylistic / coverage-strength concerns.

---

## Recommended next steps

1. **Fix B1** — normalize the two remaining 3-digit `SIFR-INT-008` / `SIFR-INT-001..011` strings in `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`. Single-commit change, no other knock-on edits.
2. Optionally fold N2 (verification inventory) into the same commit if you want zero 3-digit shorthand outside `reviews/`.
3. Optionally tighten the unit test per N1 (parametric assertion across return-type / generic / union positions and an exact `errors.len()` lock) before INT-2B starts touching `resolve_annotation_expr`.
4. Track N3 (registry/doc placement of the active entry) and N5 (negative `.sifr` fixture) as INT-2B-side cleanups so they land alongside the next `SIFR-INT-*` codes rather than as standalone churn.
5. After B1 is fixed, this slice is ready to merge.

Once B1 lands, the verdict converts to **SATISFIED** with the listed non-blockers carried into INT-2B's checklist.
