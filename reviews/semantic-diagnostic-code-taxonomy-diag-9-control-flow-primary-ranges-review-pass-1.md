# Review: `milestone_diag_9` Slice 1 — Source-Span Completion: Primary Range Transport

**Review pass:** 1 of 1
**Date:** 2026-05-02
**Branch:** `codex/diag-next-slice-original`
**Files reviewed:**
- `crates/sifr_hir/src/lower/mod.rs`
- `crates/sifr_hir/src/lower/flow_diagnostics.rs`
- `crates/sifr_hir/src/lower/control_flow_conditions.rs`
- `crates/sifr_hir/src/lower/statements.rs`
- `crates/sifr_hir/src/lower/diagnostic_transport_tests.rs`
- `crates/sifr_hir/src/lower/expressions_tests.rs`
- `crates/sifr_driver/src/frontend/module_lowering.rs`
- `crates/sifr/tests/e2e/fail/elif_condition_numeric_truthiness.sifr`
- `internal_docs/diagnostic_emission_inventory.md`
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`

---

## 1. `LoweringError` primary_range transport is clean

**Verdict: Satisfied.**

`LoweringError` (mod.rs:99–106) carries `primary_range: Option<TextRange>` as an additive `Option` field with no default, no fallback, and no compatibility shim. Three error constructors exist:

- `error(...)` — `primary_range: None` (legacy/uncoded path, intentionally spanless).
- `error_with_code(...)` — `primary_range: None` (coded path without a span).
- `error_with_code_at(...)` — `primary_range: Some(range)` (the new ranged path).

The `Display` impl (mod.rs:107–115) falls back to raw message when `line`/`col` are absent; it does **not** attempt to synthesize a span from `primary_range`. This is correct — span rendering is deferred to the driver/CLI layer in later slices.

`module_lowering.rs` passes `primary_range: None` in its test helper (line 85), which is appropriate since the driver transport for `primary_range` is not yet wired. No fallback behavior was introduced.

No `unwrap()`, `expect()`, or cascade-on-missing-logic was added to any of the three constructors.

---

## 2. `if`/`elif`/`while` SIFR-FLOW-0005 diagnostics use the AST test expression range correctly

**Verdict: Satisfied.**

`validate_control_flow_condition` (control_flow_conditions.rs:6–40) takes an `Option<TextRange>` and calls through to the ranged helper when present:

```rust
if let Some(range) = primary_range {
    super::flow_diagnostics::invalid_condition_type_at(ctx, keyword, actual.as_str(), range);
} else {
    super::flow_diagnostics::invalid_condition_type(ctx, keyword, actual.as_str());
}
```

Call sites in `statements.rs`:

| Location | Construct | Range passed | Correct? |
|---|---|---|---|
| Line 1688 | `if` test | `if_stmt.test.range()` | Yes |
| Line 1733 | `elif` test | `test.range()` | Yes |
| Line 1977 | `while` test | `while_stmt.test.range()` | Yes |

All three use the `.test` expression's `range()`, which is the correct span for the condition that violates the type requirement. No other statement types pass a range here.

---

## 3. Adding `elif` validation is semantically correct and not an accidental regression

**Verdict: Satisfied.**

`elif` branches were handled in `lower_if` (statements.rs:1721–1758) but **no** `validate_control_flow_condition` call existed for `elif` test expressions prior to this slice. A bare `if 1` / `while 1` would error, but `elif 1` would silently pass.

The fix (line 1733):
```rust
validate_control_flow_condition(&cond, "elif", Some(test.range()), ctx);
```

The keyword argument is correctly `"elif"` so the message template produces the correct diagnostic text: `"elif condition must be bool or collection/string truthiness, got 'int'"`.

The narrowing and scope/state management around the `elif` clause is unchanged; `validate_control_flow_condition` is purely diagnostic and does not affect control flow or type narrowing. No regression risk.

---

## 4. Tests are sufficient for this slice

**Verdict: Satisfied.**

### Unit tests — `diagnostic_transport_tests.rs`

Three tests covering the transport layer:
- `error_with_code_records_structured_identity` — verifies `primary_range: None` with a code.
- `error_with_code_at_records_primary_range` — verifies `primary_range: Some(range)` with a code.
- `legacy_error_records_no_structured_identity` — verifies uncoded path carries `code: None` and `primary_range: None`.

All three are deterministic, isolated, and directly exercise the new constructor path.

### Unit tests — `expressions_tests.rs`

Three focused regression tests with **range assertions**:

| Test | Source | Expected range |
|---|---|---|
| `test_if_condition_rejects_numeric_truthiness` | `if 1: pass` | `range_for(source, "1")` |
| `test_while_condition_rejects_numeric_truthiness` | `while 1: return` | `range_for(source, "1")` |
| `test_elif_condition_rejects_numeric_truthiness_with_primary_range` | `elif 1: pass` | `range_for(source, "1")` |

All three assert `primary_range == Some(expected_range)` alongside the code and message checks. The `range_for` helper resolves string position to `TextRange`.

### E2E fixture

`elif_condition_numeric_truthiness.sifr`:
```
# expect-error: SIFR-FLOW-0005
def main(flag: bool) -> None:
    if flag:
        pass
    elif 1:
        pass
```

The harness expects exactly `SIFR-FLOW-0005` with no column marker, which is appropriate for this slice (column rendering is a later milestone).

### Driver test — `module_lowering.rs`

`lowering_error_code_or_internal` correctly handles `primary_range: None` by routing uncoded errors to `INTERNAL_COMPILER_PANIC`. The test helper creates errors with `primary_range: None`, which is correct for the driver's current transport state.

---

## 5. Missing updates or risks identified

**No required fixes found.** The following are noted as informational for the next slices:

1. **`module_lowering.rs` does not yet surface `primary_range`** — `lowering_error_to_diagnostic` reads `error.code` and `error.message` but not `error.primary_range`. The driver/CLI span rendering pipeline is deferred to future slices. This is explicitly called out in the slice contract and is not a gap.

2. **`LoweringError` still carries `line`/`col`** (mod.rs:104–105) — these were pre-existing fields. They are not used by any error constructor in this slice. Their eventual removal or reconciliation with `primary_range` is an orthogonal cleanup item.

3. **No deduplication or recovery logic changes** — this slice only adds span transport; no diagnostic deduplication, taint tracking, or recovery behavior is modified. Consistent with the slice contract.

4. **`diagnostic_transport_tests.rs` does not test `Display` with `primary_range`** — the `Display` impl shows `line:col` when available but falls back silently when only `primary_range` is set. Since span rendering is deferred, this is acceptable.

---

## 6. Verdict

**Satisfied.**

All review focus items pass:

1. `LoweringError.primary_range` transport is clean, additive, and introduces no fallback or compatibility layer.
2. `if`/`elif`/`while` SIFR-FLOW-0005 diagnostics correctly use `AST.test.range()` for the primary span.
3. Adding `elif` validation closes a gap (not a regression), and the semantic behavior is sound.
4. Test coverage is sufficient: transport unit test, if/while/elif range assertions, elif e2e fixture. All tests pass locally.
5. No required fixes identified. Driver span rendering is intentionally deferred to later slices per the milestone contract.
6. Local validation passed:
   - `cargo fmt --check`
   - `git diff --check`
   - `cargo test -p sifr_hir diagnostic_transport_tests -- --nocapture`
   - `cargo test -p sifr_hir condition_rejects_numeric_truthiness -- --nocapture`
   - `cargo test -p sifr_driver frontend::module_lowering -- --nocapture`
   - `cargo test -p sifr --test e2e test_e2e_fail -- elif_condition_numeric_truthiness --nocapture`
   - `cargo clippy -p sifr_hir -p sifr_driver --no-deps -- -D warnings`
   - `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=535.54s`)

**Recommended action:** Open PR for `milestone_diag_9` slice 1.
