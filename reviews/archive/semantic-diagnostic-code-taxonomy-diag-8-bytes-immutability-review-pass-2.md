# Review: milestone_diag_8 slice 6 — Pass 2

## Verdict: SATISFIED

---

## 1. Pass-1 Template/Emission Mismatch — Fully Fixed

**Pass-1 finding:** A single registry entry `SIFR-OWN-0007` covered both plain `bytes[i] = x` and augmented `bytes[i] += x`, but the two code paths emitted semantically distinct messages under the same code.

**Fix applied:** `SIFR-OWN-0008` was introduced for augmented subscript assignment. `SIFR-OWN-0007` now exclusively covers plain subscript assignment.

Evidence:
- `codes.rs:70-72` — `OWN_IMMUTABLE_BYTES_ASSIGNMENT` (0007) and `OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT` (0008) are two distinct constants with distinct codes.
- `ownership_diagnostics.rs:81-93` — `immutable_bytes_subscript_assignment` emits 0007; `immutable_bytes_augmented_subscript_assignment` emits 0008. No shared template.
- `codes.rs:884-905` — Two separate `active_entry!` invocations with distinct `message_template`, `owner_module`, and `representative_fixture_path`. The templates are `"bytes is immutable; subscript assignment is not supported"` (0007) and `"bytes is immutable; augmented subscript assignment is not supported"` (0008).

The mismatch is eliminated.

---

## 2. Taxonomic Soundness — Both Codes Are Active, Documented, Fixture-Represented

### SIFR-OWN-0007

| Property | Value |
|---|---|
| Code | `SIFR-OWN-0007` |
| Registry state | Active (`DiagnosticState::Active`) |
| Owner module | `sifr_hir::lower::statements` |
| Message template | `"bytes is immutable; subscript assignment is not supported"` |
| Fixture | `bytes_subscript_assignment_unsupported.sifr` → expects `SIFR-OWN-0007` |
| Docs page | `docs/errors/SIFR-OWN-0007.md` |
| Constant | `DiagnosticCode::OWN_IMMUTABLE_BYTES_ASSIGNMENT` (codes.rs:70) |
| Active list | `ACTIVE_DIAGNOSTIC_CODES` (codes.rs:1406) |
| Emission site | `ownership_diagnostics::immutable_bytes_subscript_assignment` called at statements.rs:1415, 1443, 1471 |

### SIFR-OWN-0008

| Property | Value |
|---|---|
| Code | `SIFR-OWN-0008` |
| Registry state | Active (`DiagnosticState::Active`) |
| Owner module | `sifr_hir::lower::aug_assign_lowering` |
| Message template | `"bytes is immutable; augmented subscript assignment is not supported"` |
| Fixture | `bytes_augmented_subscript_assignment_unsupported.sifr` → expects `SIFR-OWN-0008` |
| Docs page | `docs/errors/SIFR-OWN-0008.md` |
| Constant | `DiagnosticCode::OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT` (codes.rs:71-72) |
| Active list | `ACTIVE_DIAGNOSTIC_CODES` (codes.rs:1407) |
| Emission site | `ownership_diagnostics::immutable_bytes_augmented_subscript_assignment` called at aug_assign_lowering.rs:105, 183, 241 |

Both codes are taxonomically sound: distinct codes, distinct semantics, distinct owners, distinct fixtures, both documented.

---

## 3. No Raw `ctx.error` Fallback for Immutable Bytes Assignment Paths

All four Bytes-type check sites in the two relevant files now route through the typed diagnostic functions:

**statements.rs (plain subscript assignment):**
- Line 1414–1417: `if matches!(obj_ty.resolve_alias(), Type::Bytes)` → `immutable_bytes_subscript_assignment(ctx)`
- Line 1442–1445: attribute subscript Bytes check → `immutable_bytes_subscript_assignment(ctx)`
- Line 1470–1473: plain subscript Bytes check → `immutable_bytes_subscript_assignment(ctx)`

**aug_assign_lowering.rs (augmented subscript assignment):**
- Line 104–107: nested subscript Bytes check → `immutable_bytes_augmented_subscript_assignment(ctx)`
- Line 182–185: attribute subscript Bytes check → `immutable_bytes_augmented_subscript_assignment(ctx)`
- Line 240–243: plain subscript Bytes check → `immutable_bytes_augmented_subscript_assignment(ctx)`

Zero raw `ctx.error("...")` calls found on any Bytes subscript path. The fallback is absent.

---

## 4. Fixture and HIR Test Coverage

### E2E Fixtures

**`bytes_subscript_assignment_unsupported.sifr`**
```sifr
# expect-error: SIFR-OWN-0007
def main() -> None:
    payload: bytes = b"abc"
    payload[0] = 65
```

**`bytes_augmented_subscript_assignment_unsupported.sifr`**
```sifr
# expect-error: SIFR-OWN-0008
def main() -> None:
    payload: bytes = b"abc"
    payload[0] += 1
```

Both fixtures are lexically ordered (plain before augmented), syntactically minimal, and directly exercise the specific code path. The comments are accurate and match the expected code.

### HIR Unit Tests (expressions_tests.rs)

- `test_bytes_subscript_assignment_has_ownership_code` (line 511): asserts `error.code == Some(OWN_IMMUTABLE_BYTES_ASSIGNMENT)` and the exact message for plain subscript assignment.
- `test_bytes_augmented_subscript_assignment_has_ownership_code` (line 525): asserts `error.code == Some(OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT)` and the exact message for augmented subscript assignment.

Both tests verify the code field is populated and the message matches the registry template, confirming round-trip fidelity from lowering through error emission.

### Inventory Coverage (diagnostic_emission_inventory.md)

- Line 320: `SIFR-OWN-0007 | immutable bytes subscript assignment | assignment lowering | ...fixture...`
- Line 321: `SIFR-OWN-0008 | immutable bytes augmented subscript assignment | augmented-assignment lowering | ...fixture...`

---

## 5. Regression Check

**Codegen consistency:** The two diagnostic messages differ by one word ("subscript assignment" vs "augmented subscript assignment"). The distinction is precise and unambiguous. Both use the "bytes is immutable; ..." prefix, which is consistent with the OWN family semantics.

**Registry skeleton test (`registry_skeleton_is_internally_consistent`):** Passes — each active entry's `message_template` placeholders are declared, `docs_path` matches canonical format, `declared_severity` is set, `owner_module` is set, and `representative_fixture_path` is set. Both 0007 and 0008 satisfy all invariants.

**Docs sync:** Both `SIFR-OWN-0007.md` and `SIFR-OWN-0008.md` exist under `docs/errors/`, matching the `active_diagnostic_docs_pages_exist_with_exact_casing` test.

**Active list sync:** `ACTIVE_DIAGNOSTIC_CODES` includes both `OWN_IMMUTABLE_BYTES_ASSIGNMENT` (1406) and `OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT` (1407). The `registry_skeleton_is_internally_consistent` test verifies the constant list and active registry entries are in sync.

**No fallbacks:** The three plain-assignment sites in `statements.rs` and three augmented-assignment sites in `aug_assign_lowering.rs` all route to the typed `error_with_code` path. No bare `ctx.error(...)` strings appear on any Bytes-related path.

---

## 6. Documentation Accuracy

- `docs/errors/SIFR-OWN-0007.md`: owner = `sifr_hir::lower::statements`, message = `"bytes is immutable; subscript assignment is not supported"`. Matches registry.
- `docs/errors/SIFR-OWN-0008.md`: owner = `sifr_hir::lower::aug_assign_lowering`, message = `"bytes is immutable; augmented subscript assignment is not supported"`. Matches registry.
- `internal_docs/diagnostic_codes.md` lines 104–105: both codes listed with correct owner modules and message templates.
- `docs/errors/diagnostic-codes.md` lines 80–81: index table has both codes with correct summaries.

All documentation is accurate and internally consistent.

---

## 7. Validation Results (User-Confirmed Passing)

All listed validations passed:
- `cargo run -q -p sifr_diagnostics --bin gen-error-docs`
- `cargo fmt --check`
- `git diff --check`
- `python3 scripts/check_diagnostic_docs_sync.py`
- `cargo test -p sifr_diagnostics`
- `cargo test -p sifr_hir bytes_subscript_assignment_has_ownership_code`
- `cargo test -p sifr_hir test_bytes_augmented_subscript_assignment_has_ownership_code`
- `cargo test -p sifr --test e2e test_e2e_fail -- bytes_subscript_assignment_unsupported`
- `cargo test -p sifr --test e2e test_e2e_fail -- bytes_augmented_subscript_assignment_unsupported`
- `cargo run -q -p sifr -- check ...bytes_subscript_assignment_unsupported.sifr` → exit 1, SIFR-OWN-0007
- `cargo run -q -p sifr -- check ...bytes_augmented_subscript_assignment_unsupported.sifr` → exit 1, SIFR-OWN-0008
- `cargo clippy -p sifr_diagnostics -p sifr_hir --no-deps -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` (wall_time 611.23s)

---

## Summary

| Concern | Status |
|---|---|
| Pass-1 template/emission mismatch fixed | Done — 0007 plain, 0008 augmented, no shared template |
| Both codes taxonomically sound | Done — distinct codes, owners, fixtures, docs |
| No raw `ctx.error` fallback on bytes paths | Done — all 6 sites use typed `error_with_code` |
| Fixtures and HIR tests provide coverage | Done — 2 e2e fixtures, 2 HIR unit tests |
| No regressions | Done — registry tests pass, docs in sync |
| All validations passed | Confirmed by user |

**Nothing left to fix.**
