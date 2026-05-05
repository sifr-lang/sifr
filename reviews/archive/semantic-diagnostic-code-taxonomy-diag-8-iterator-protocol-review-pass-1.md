# Review: `milestone_diag_8` slice 5 — Iterator/Reversible Protocol Diagnostics Migration

**Review round:** pass 1
**Reviewer:** Claude (implementation review)
**Branch:** `codex/diag-next-slice-original`
**Files changed:** `protocol_diagnostics.rs`, `classes.rs`, `diagnostic_emission_inventory.md`, `ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
**New fixture:** `invalid_iter_parameter_signature.sifr`

---

## Summary

Migrate residual iterator/reversible protocol diagnostics from raw `ctx.error(...)` to `SIFR-PROTO-0002` via the `protocol_diagnostics` helper module. Five distinct diagnostic categories are addressed: extra parameters on `__iter__`, `__next__`, and `__reversed__`; element-type mismatch between `__iter__` and `__next__`; and element-type mismatch between `__iter__` and `__reversed__`. The parameter-shape / return-signature cascading suppression is also confirmed fixed. No behavioral regressions, no fallback/compatibility paths, no scope creep.

---

## Change Analysis

### 1. `protocol_diagnostics.rs` — Two new helpers added

**`iterator_invalid_parameter_signature(ctx, type_name)`** (lines 48–53)
- Constructs `ctx.error_with_code(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE, ...)`
- Message: `"class '{type_name}' must not declare parameters besides self"`
- All three call sites (`__iter__`, `__next__`, `__reversed__`) pass the qualified method name (e.g., `"BadIterParam.__iter__"`) as `type_name`, preserving the existing message style

**`iterator_element_mismatch(ctx, class_name, left_method, left_type, right_method, right_type)`** (lines 55–69)
- Constructs `ctx.error_with_code(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE, ...)`
- Message: `"class '{class_name}' iteration protocol mismatch: '{left_method}' yields '{left_type}' but '{right_method}' yields '{right_type}'"`
- Replaces the prior raw `ctx.error(format!(...))` call for both `__iter__`/`__next__` and `__iter__`/`__reversed__` mismatch pairs

**Verdict:** Correct. Both helpers use the same `DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE` (`SIFR-PROTO-0002`) as the pre-existing `iterator_invalid_return_signature` helper, consistent with the iterator/reversible protocol family. No new code was introduced without corresponding unit test coverage.

---

### 2. `classes.rs` — `validate_iteration_protocol_methods` refactored

**Parameter-shape → return-signature cascading fix (all three methods)**

Before (for `__iter__`):
```rust
if !iter_ft.params.is_empty() {
    ctx.error(format!(...));   // parameter error
}
if class_iter_element_type(class_name, methods).is_none() {
    protocol_diagnostics::iterator_invalid_return_signature(...); // return error
}
```

After:
```rust
if !iter_ft.params.is_empty() {
    protocol_diagnostics::iterator_invalid_parameter_signature(...);
} else if class_iter_element_type(class_name, methods).is_none() {
    protocol_diagnostics::iterator_invalid_return_signature(...);
}
```

The `else if` chain means: if there are extra parameters, emit parameter diagnostic and **skip** the return-type diagnostic. Previously, both diagnostics could fire on the same method (e.g., `def __iter__(self, x: int) -> int`). The user's validation (`cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/invalid_iter_parameter_signature.sifr`) confirmed no duplicate cascade. This is correct behavior.

**Element-mismatch migration**

Before:
```rust
if !next_elem.is_assignable_to(&iter_elem) || !iter_elem.is_assignable_to(&next_elem) {
    ctx.error(format!("class '{class_name}' iteration protocol mismatch: '__iter__' yields '{}' but '__next__' yields '{}'", iter_elem.display_name(), next_elem.display_name()));
}
```

After:
```rust
if !next_elem.is_assignable_to(&iter_elem) || !iter_elem.is_assignable_to(&next_elem) {
    protocol_diagnostics::iterator_element_mismatch(
        ctx,
        class_name,
        "__iter__",
        iter_elem.display_name().as_str(),
        "__next__",
        next_elem.display_name().as_str(),
    );
}
```

Same transformation for the `__iter__`/`__reversed__` pair. No `ctx.error(...)` raw calls remain in `validate_iteration_protocol_methods`. The broader `classes.rs` still has `ctx.error(...)` calls for unrelated categories (parent class resolution, unsupported class body statements, field default expressions) — these are outside slice 5 scope.

**Verdict:** Correct. The `else if` chain is the right pattern for exclusive error alternatives on the same method. The element-mismatch path is independent (it fires based on `class_iter_element_type` and `class_next_element_type` results, which are computed separately) and cannot trigger a cascade from a parameter-shape diagnostic.

---

### 3. `protocol_diagnostics.rs` — Unit tests

Three new HIR unit tests added (lines 185–221):

| Test | Covers | Method |
|---|---|---|
| `invalid_iter_parameter_signature_has_proto_code` | Extra param on `__iter__` | `iterator_invalid_parameter_signature` |
| `iter_next_element_mismatch_has_proto_code` | `__iter__`/`__next__` element mismatch | `iterator_element_mismatch` |
| `iter_reversed_element_mismatch_has_proto_code` | `__iter__`/`__reversed__` element mismatch | `iterator_element_mismatch` |

Each test asserts both `error.message` equality and `error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)`. The existing pre-slice-5 tests for `__iter__` return shape (`invalid_iter_signature_has_proto_code`), `__next__` return shape (`invalid_next_signature_has_proto_code`), and `__reversed__` return shape (`invalid_reversed_signature_has_proto_code`) are all still present and unchanged.

**Verdict:** Correct. Good coverage of the new helpers. Each new diagnostic path has a dedicated unit test.

---

### 4. E2E fixture `invalid_iter_parameter_signature.sifr`

```sifr
# expect-error: SIFR-PROTO-0002

class BadIterParam:
    def __iter__(self, limit: int) -> Iterator[int]:
        return iter([1])

def main():
    _ = BadIterParam()
```

Correctly targets the parameter-shape diagnostic (`SIFR-PROTO-0002`). No `__next__` or `__reversed__` is present, so no element-mismatch diagnostic fires. No duplicate diagnostics. The fixture is minimal and focused.

The three pre-existing `SIFR-PROTO-0002` e2e fixtures (`invalid_iter_signature.sifr`, `invalid_next_signature.sifr`, `invalid_reversed_signature.sifr`) all carry `# expect-error: SIFR-PROTO-0002` and are unaffected.

**Verdict:** Correct. Single, clean fixture for the parameter-shape diagnostic category.

---

### 5. `diagnostic_emission_inventory.md` — Inventory updated

```diff
-| `SIFR-PROTO-0002` | invalid iterator/reversible protocol signature | protocol checking | invalid iterator/reversible fixtures |
+| `SIFR-PROTO-0002` | invalid iterator/reversible protocol signature | protocol checking | `crates/sifr/tests/e2e/fail/invalid_iter_parameter_signature.sifr`, `crates/sifr/tests/e2e/fail/invalid_iter_signature.sifr`, `crates/sifr/tests/e2e/fail/invalid_next_signature.sifr`, `crates/sifr/tests/e2e/fail/invalid_reversed_signature.sifr` |
```

The fixture list is now concrete rather than vague. Correct.

---

### 6. `ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` — Roadmap updated

```diff
+- [ ] `milestone_diag_8` slice 5 in progress: migrate residual iterator/reversible protocol parameter and element-mismatch diagnostics from raw `ctx.error(...)` transport to the existing `SIFR-PROTO-0002` helper path and add fixture coverage.
```

Entry is added with in-progress marker. Correct.

---

## Scope Creep Check

No out-of-scope changes detected. The changes are confined to:
- Two new helpers in `protocol_diagnostics.rs`
- Refactoring of `validate_iteration_protocol_methods` in `classes.rs`
- Three new unit tests in `protocol_diagnostics.rs`
- One new e2e fixture
- Two documentation updates

---

## Fallback/Compatibility Path Check

No fallback paths introduced. All five diagnostic categories now use `error_with_code(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE, ...)`. The only remaining `ctx.error(...)` calls in `classes.rs` are for: parent type/class resolution failures, unsupported class body statements, and field default expression failures — none of which are in slice 5 scope.

---

## Diagnostic Code Taxonomy Fit

`SIFR-PROTO-0002` covers "invalid iterator/reversible protocol signature". The migrated diagnostics fit precisely:
- Extra parameters on `__iter__`/`__next__`/`__reversed__` → invalid parameter shape → `SIFR-PROTO-0002`
- Element-type mismatch between `__iter__` and `__next__`/`__reversed__` → invalid signature semantics → `SIFR-PROTO-0002`

This is consistent with how `SIFR-PROTO-0002` was already used for return-type mismatches in the prior slice. The code family is coherent.

---

## Cascading/Duplicate Diagnostic Check

The `else if` chain is the correct anti-cascade pattern. For a method with both bad parameters AND bad return type (e.g., `def __iter__(self, x: int) -> int`), only the parameter diagnostic fires. This matches the stated scope requirement: "parameter-shape failures should avoid cascading into duplicate return-signature diagnostics for the same method."

The element-mismatch diagnostics are independent of parameter-shape diagnostics — they only fire when both `__iter__` and `__next__`/`__reversed__` are present and their element types are mutually non-assignable.

User's explicit validation confirmed: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/invalid_iter_parameter_signature.sifr` (expected nonzero user diagnostic; confirmed no duplicate return-signature cascade).

---

## Validation Summary

All user-reported validation passes are consistent with correct behavior:
- `cargo fmt --check` — formatting correct
- `git diff --check` — no whitespace errors
- `cargo test -p sifr_hir protocol_diagnostics::tests -- --nocapture` — all 10 tests pass (7 pre-existing + 3 new)
- `cargo test -p sifr --test e2e test_e2e_fail -- invalid_iter_parameter_signature --nocapture` — e2e fixture matches
- `cargo run -q -p sifr -- check ...` — single `SIFR-PROTO-0002` diagnostic, no duplicate cascade
- `cargo clippy -p sifr_hir --no-deps -- -D warnings` — clean
- `scripts/run_all_tests.sh --profile quick` — full quick profile pass

---

## Findings

**No actionable bugs or regressions found.** The slice is ready for commit.

Minor observation (non-blocking): The `iterator_element_mismatch` message says `'{left_method}' yields '{left_type}' but '{right_method}' yields '{right_type}'`. The word "yields" is appropriate for the element type being produced by the iterator, but the return-type annotations checked by the return-signature diagnostic use the word "return". This is a pre-existing message-style choice and not introduced by this slice; it is consistent across both pre-existing element-mismatch messages. No change needed.

---

**Reviewer recommendation:** Approve for commit.
