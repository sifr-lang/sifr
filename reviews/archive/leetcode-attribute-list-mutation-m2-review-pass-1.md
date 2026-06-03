

Based on my thorough review of all diffs, generated code, test results, and benchmark validation, here is my assessment:

---

## Review Summary

### Correctness and Ownership Soundness

**Clone suppression** (`field_and_stdlib_rewrites.rs`, `plain_call_args.rs`, `call_args_and_returns.rs`):

The `strip_redundant_borrowed_self_field_clone` function strips a trailing `.clone()` from `self.field.clone()` to produce `self.field` before the borrow is added. This is sound because:

- The HIR pattern match confirms the object is literally `self` (not a method result, not a parent field access).
- The Ref is then added to the bare `self.field`, so the borrow points to the actual field — no ownership is lost.
- The `method_call_needs_field_clone_suppression` reordering is a pure control-flow cleanup; it does not change which methods trigger clone suppression.

**Structured list lowering** (`field_assignment.rs`, `stmt_block.rs`):

Both structured lowering paths now match `Type::List(_) => ... build_list_subscript_assign_stmt(...)`. The `build_list_subscript_assign_stmt` in `subscript_assignment.rs:7-20` emits `get_mut` wrapped in bounds checking via `if __idx_norm >= 0 { if let Some(__elem) = self.history.get_mut(...) { ... } }`, which is the correct bounded pattern. The method returns a `RustStmt`, not a bare expression, matching both call sites.

### 1472 Metadata Removal Justification

The evidence is consistent:
- `slowness_seed.py` no longer contains `1472_design_browser_history`.
- The Sifr source now uses direct `self.history[self.i + 1] = str(url)` with no copy-and-reassign workaround.
- The generated Rust emits `self.history.get_mut(...)` — no clone, no workaround.
- Benchmark: Sifr faster than Python at all sizes (~4.9x to ~5.3x).
- Analyzer: measured-slower problems reduced from 65 to 64; 1472 removed.

### Missing Validation Gap

The new `Type::List` path in `field_assignment.rs:try_lower_structured_attribute_subscript_assign_stmt` (top-level structured lowering) has no dedicated regression test. The existing `test_structured_stmt_path_handles_attribute_list_subscript_assign_inside_if` tests only the `stmt_block.rs` path (nested inside an `If` statement body). This is a minor coverage gap — the two code paths share the same underlying `build_list_subscript_assign_stmt` function, so they're unlikely to diverge in practice, but it's worth noting.

### File Sizes

| File | Lines | Cap |
| --- | --- | --- |
| `field_assignment.rs` | 368 | ✅ |
| `plain_call_args.rs` | 842 | ✅ |
| `call_args_and_returns.rs` | 456 | ✅ |
| `field_and_stdlib_rewrites.rs` | 793 | ✅ |
| `stmt_block.rs` | 811 | ✅ |
| `collections_and_stdlib_codegen_tests.rs` | 811 | ⚠️ near cap |
| `structured_lowering_codegen_tests.rs` | 814 | ⚠️ near cap |

Both test files are within the 900-line limit but approaching it. The actual test additions are modest (32 and 36 lines respectively).

### Known Full-Test-Suite Failures

The 65 pre-existing failures are broad codegen string/snapshot/decomposition mismatches unrelated to this milestone's focused regression tests (both of which pass).

---

## Conclusion

**APPROVED WITH NITS**

The implementation is correct, ownership-sound, and validated against the benchmark. Two minor observations:

1. **Nit — Test coverage gap**: `field_assignment.rs`'s `try_lower_structured_attribute_subscript_assign_stmt` (top-level `AttributeSubscriptAssign` lowering, not inside a statement body) has no dedicated regression test. The existing nested test covers the `stmt_block.rs` path only. Consider adding a top-level test or expanding the existing test to also capture direct structured lowering.

2. **Nit — File sizes**: `collections_and_stdlib_codegen_tests.rs` (811 lines) and `structured_lowering_codegen_tests.rs` (814 lines) are both near the 900-line cap. The additions are small but the files have limited room for growth.

Neither nit blocks the milestone. The correctness gates all pass, the benchmark improvement is real and validated, and the clone suppression is carefully scoped to `self.field` patterns only.
