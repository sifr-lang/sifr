# Review: semantic-diagnostic-code-taxonomy — diag-9 stdlib primary ranges

**Slice**: milestone_diag_9 stdlib unsupported-surface primary ranges
**Phase tracker**: `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
**Branch**: `codex/diag-9-stdlib-primary-ranges`
**Review**: pass 1

---

## Changed files (git diff only)

| File | Change |
|------|--------|
| `crates/sifr_hir/src/lower/builtin_calls.rs` | 2 distinct hunk changes |
| `crates/sifr_hir/src/lower/expressions_tests.rs` | 1 test extended + 1 new test added |
| `crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr` | `expect-error` annotation gain `col=26` |
| `crates/sifr/tests/e2e/fail/tuple_dynamic_list_shape.sifr` | `expect-error` annotation gain `col=20` |

---

## Finding 1 — `lower_tuple_constructor_call` (builtin_calls.rs:194)

**Before:**
```rust
ctx.error_with_code(
    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
    "tuple() currently requires ...",
);
```

**After:**
```rust
ctx.error_with_code_at(
    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
    "tuple() currently requires ...",
    arg_expr.range(),
);
```

`arg_expr` is the single positional argument to `tuple(...)`. The e2e fixture
`tuple_dynamic_list_shape.sifr` has `tuple(nums)` where column 20 correctly
points at `tuple` (the call expression). The unit test
`test_tuple_constructor_rejects_dynamic_list_shape` additionally verifies
`STDLIB_UNSUPPORTED_SURFACE` code and that `primary_range` points at `nums`
via `range_for_after_anchor(source, "tuple(", "nums")`.

**Verdict: correct.**

---

## Finding 2 — `lower_defaultdict_constructor_call` (builtin_calls.rs:417-432)

**Before:**
```rust
if !call.arguments.keywords.is_empty() {
    ctx.error_with_code(
        DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
        "defaultdict() does not support keyword arguments",
    );
    return None;
}
```

**After:** now branches on two cases:

1. **Unpacked keyword** (`**{...}`) — `keyword.arg` is `None` → range is `keyword.range`
   (the entire `**{"default_factory": list}` expression).

2. **Named keyword** (`default_factory=...`) — `name.range()` points at the
   identifier `default_factory` which aligns with e2e annotation `col=26`.

The e2e fixture `defaultdict_keyword_constructor_unsupported.sifr` has the
named keyword form; column 26 in `    groups = defaultdict(default_factory=list)` is
0-indexed position 26 which lands exactly on `d` of `default_factory`.

The new unit test `test_defaultdict_unpacked_keyword_constructor_unsupported_has_stdlib_code`
covers the `**{...}` path separately.

**Verdict: correct.**

---

## Finding 3 — `expressions_tests.rs` coverage

Two unit tests were updated/added to assert `code == STDLIB_UNSUPPORTED_SURFACE`
and `primary_range` in addition to the message. These provide regression
anchors for the concrete ranges and are aligned with the established pattern
used by all other diagnostic-code-bearing tests in this file (e.g.
`test_builtin_sum_wrong_arity_has_call_code`, `test_sorted_unexpected_keyword_has_call_code`).

**Verdict: correct and consistent with existing test conventions.**

---

## Finding 4 — E2E fixture annotations

Both fixture files updated `expect-error` from bare to `expect-error[col=N]`.
The column values were verified manually:

- `defaultdict_keyword_constructor_unsupported.sifr`: `col=26` → 0-indexed
  position of `d` in `default_factory` ✓
- `tuple_dynamic_list_shape.sifr`: `col=20` → 0-indexed position of `t` in
  `tuple` ✓

**Verdict: correct.**

---

## Validation summary

| Check | Command | Result |
|-------|---------|--------|
| `cargo fmt --check` | user-reported clean | ✓ |
| `git diff --check` | user-reported clean | ✓ |
| `check_hir_maintainability_guardrails.py` | user-reported clean | ✓ |
| `cargo clippy --workspace -- -D warnings` | user-reported clean | ✓ |
| `cargo test -p sifr_hir defaultdict` | user-reported 2 tests pass | ✓ |
| `cargo test -p sifr_hir tuple_constructor_rejects_dynamic_list_shape` | user-reported pass | ✓ |
| `cargo test -p sifr --test e2e test_e2e_fail` (2 fixtures) | user-reported pass | ✓ |
| `scripts/run_all_tests.sh --profile quick` | user-reported e1bf653aaa770517 | ✓ |

---

## Conclusion

The slice attaches concrete `primary_range` source ranges to both
`STDLIB_UNSUPPORTED_SURFACE` diagnostic paths in `lower_tuple_constructor_call`
and `lower_defaultdict_constructor_call`. No fallback/raw diagnostics remain
in the touched paths. Unit tests and e2e fixture annotations are consistent
with established conventions.

**Satisfied.** No further pass required.
