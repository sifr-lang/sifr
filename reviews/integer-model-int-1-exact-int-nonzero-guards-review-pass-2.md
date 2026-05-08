

---

## Review: INT-1 exact-int nonzero guard follow-up (elif + nested guards)

**Verdict: APPROVED**

No blocking findings. The implementation is sound.

---

### Blocking Findings
none

---

### Non-Blocking Findings

**1. No test for `elif` with `else` clause when all guards exit**

The current test `test_exact_int_division_after_elif_zero_guard_early_exit_lowers` uses `elif` with no `else`. A variant with a bare `else:` clause where all guards exit would exercise the else-body save/restore cycle in addition to the elif chain:

```python
if flag: return
elif divisor == 0: return
else: pass
result = 10 // divisor
```

This is minor — the save/restore pattern is identical to the existing code and the e2e fixture covers similar shapes.

**2. `all_previous_branches_exit` mutation during elif iteration**

At lines 1777–1780, `all_previous_branches_exit` is both read (`if all_previous_branches_exit && elif_body_exits`) and written (`all_previous_branches_exit &= elif_body_exits`). This is correct but worth noting: the flag must be updated *after* the condition check so the current branch's exit status does not influence its own guard collection. The ordering is sound because `elif_body_exits` is computed from the body, and the guard collection uses `all_previous_branches_exit` (pre-update) to determine whether to propagate. No issue — just worth a comment for future maintainers.

**3. Guard collection in elif uses `saved_nonzero_integer_bindings` not `elif_saved`**

At line 1753, each elif iteration restores from `saved_nonzero_integer_bindings` (the pre-if snapshot) rather than the elif-specific save. This is correct — the elif is building on the false-branch of the if (and all previous elifs), so it must start from the original state. The elif-specific `elif_saved` (line 1762) is used only for narrowing within that elif's true branch, not for the outer restore. Confirmed correct.

---

### Soundness Analysis

| Property | Analysis | Status |
|---|---|---|
| **elif chain propagation** | `all_previous_branches_exit` initialized from `then_body_always_exits(&then_body)` and updated with `&=` per elif. Guards collected only when previous branches all exit. | ✓ |
| **Nested guard composition** | `detect_true_nonzero_integer_guards` handles `not (X or Y)` by delegating to `detect_false_nonzero_integer_guards`, which propagates through `BoolOp::Or` to collect both operands. | ✓ |
| **Save/restore discipline** | Every branch (if, elif, else) properly saves before body and restores after. | ✓ |
| **Post-chain application** | `post_if_false_nonzero_guards` accumulated during processing, applied once at end (line 1846). | ✓ |
| **Reassignment invalidation** | `invalidate_rebound_binding_facts` calls `clear_proven_nonzero_integer_binding` (line 992). Unit test validates. | ✓ |
| **Augassign unconditional clear** | Correct — clearing a non-member is a no-op; no incorrect suppression. | ✓ |
| **Diagnostic suppression surface** | Only `HirExpr::Name` nodes reach `is_proven_nonzero_integer_binding`; literals are handled separately via `is_proven_nonzero_integer_expr` matching on `IntLiteral`/`LargeIntLiteral`. | ✓ |
| **Path sensitivity for if/elif/else** | Guard is applied after the full chain, not after the if alone. Nested guard marks both operands non-zero inside the branch. Correct. | ✓ |

---

### Validation Notes

All validation commands passed:
- `cargo test -p sifr_hir exact_int -- --nocapture`: 13 tests pass
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/exact_int_nonzero_elif_and_nested_guards.sifr`: correct Rust codegen
- `scripts/run_all_tests.sh --profile quick`: 24 e2e pass tests, report signature `e1bf653aaa770517`

The e2e fixture exercises the two new shapes:
- `guarded_elif`: `if flag: return` + `elif divisor == 0: return` proves `divisor` non-zero after chain
- `guarded_nested`: `not (left == 0 or right == 0)` proves both `left` and `right` non-zero inside branch

---

### Residual Risks

Acceptable. The proven-nonzero binding set is a `HashSet<String>` with save/restore at every if/elif/else/while boundary. The follow-up correctly handles sequential guards in elif chains and logical composition in nested guards. The save/restore pattern is consistent with the prior pass.

---

### Recommendation

**This review is satisfied.** The implementation is sound and the coverage is adequate. Proceed to PR.
