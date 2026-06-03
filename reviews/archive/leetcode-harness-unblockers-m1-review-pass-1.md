

Now I have a thorough understanding of the implementation. Let me write the review.

---

## M1 Review: Benchmark Harness Unblockers

**File reviewed:** `audits/leetcode/benchmarks/harnesses/generic.py` (892 lines)

### H1: Owned Result Rendering — **Correct**

The fix is in `sifr_expected_check` (lines 819–822) and `sifr_wrong_result_expr` (lines 832–835):

```python
# sifr_expected_check for list_node_int:
return f"actual_text: str = listNodeToString({result_name})\nif actual_text != expected_text.strip():"
# sifr_wrong_result_expr for list_node_int / tree_node_int:
return "actual_text"
```

The rendered string is captured into a local `actual_text` variable by the expected check, then reused by the wrong-result print via the `sifr_wrong_result_expr` returning the literal `"actual_text"`. The checksum path (`sifr_checksum_expr`, lines 847–850) calls `listNodeToString` / `treeToString` fresh on the loop result, so each use is independent.

Verified in generated runner `0206_reverse_linked_list_runner.sifr`:
```sifr
result: ListNode | None = reverseList(_build_list_node(_bench_tokens, 0, len(_bench_tokens)))
actual_text: str = listNodeToString(result)
if actual_text != expected_text.strip():
    print("wrong result: " + actual_text)
    exit(1)
```

No double-use of a moved value. No `Display` required on `ListNode`. The approach is clean and does not hide ownership problems.

### H2: Owned Tree Inputs Rebuilt Per Call — **Correct**

The fix is in `sifr_call` (lines 881–882):
```python
elif binding and binding["type"] == "balanced_tree[int]":
    rendered_args.append(f"_build_balanced_tree({arg}_values, 0, len({arg}_values) - 1)")
```

For every call site in the generated runner body, the tree is rebuilt inline from the values list. The binding variable (`root: TreeNode | None = _build_balanced_tree(...)`) is still emitted in the bindings section, but the call sites (initial `result: ... = {call}` and each loop iteration `loop_result: ... = {call}`) expand to fresh `_build_balanced_tree(...)` calls.

Verified in generated runner `0226_invert_binary_tree_runner.sifr`:
```sifr
root: TreeNode | None = _build_balanced_tree(root_values, 0, len(root_values) - 1)  # binding (unused after)
result: TreeNode | None = invertTree(_build_balanced_tree(root_values, 0, len(root_values) - 1))  # fresh...
for _loop in range(0, loops):
    loop_result: TreeNode | None = invertTree(_build_balanced_tree(root_values, 0, len(root_values) - 1))  # fresh```

Verified in generated runner `0617_merge_two_binary_trees_runner.sifr` (two-tree input):
```sifr
result: TreeNode | None = mergeTrees(_build_balanced_tree(p_values, ...), _build_balanced_tree(q_values, ...))
for _loop in range(0, loops):
    loop_result: TreeNode | None = mergeTrees(_build_balanced_tree(p_values, ...), _build_balanced_tree(q_values, ...))
```

Rebuilding all `balanced_tree[int]` inputs is appropriate because:
1. Every H2 problem consumes owned tree arguments (verified by phase doc evidence patterns).
2. The `_bench_tokens` and `_build_balanced_tree` helpers are pure and cheap for the fixture sizes used in build/correctness validation.
3. Matching the Python runner's `fresh_input_each_call` behavior (line 206–207), which already rebuilds for both `list_node[int]` and `balanced_tree[int]` input types.

The binding variable (`root`) is redundant but harmless — it's never used after the call sites are replaced. This is a minor style issue, not a correctness bug.

### H3: Wrong-Result Formatting Uses `treeToString` / `listNodeToString` — **Correct**

Both `list_node_int` and `tree_node_int` use the same `actual_text` mechanism (line 833):
```python
if expected_type in ("list_node_int", "tree_node_int"):
    return "actual_text"
```

Verified in generated runner `0226_invert_binary_tree_runner.sifr`:
```sifr
actual_text: str = treeToString(result)
if actual_text != expected_text.strip():
    print("wrong result: " + actual_text)
```

This eliminates the `Display` requirement on `TreeNode` that caused the original `error[E0277]: TreeNode doesn't implement std::fmt::Display` failures.

### Additional Observations

**Helper import injection is correct.** `missing_helper_imports` (lines 431–439) guards on `ListNode` and `TreeNode` presence. All H1/H2/H3 algorithm sources already import `listNodeToString` / `treeToString` from their respective helpers, so the generated runners get all needed imports. No algorithm that uses structured types omits the helper import.

**No `mutates_single` with structured types.** I verified that no problem in the registry uses `call.mode == "mutates_single"` with `expected.type` of `list_node_int` or `tree_node_int`. The H1 approach is complete for all current registry entries.

**Python harness parity.** The Python runner (`run_python`, lines 93–132) already handles structured types correctly via `list_node_to_text` / `tree_node_to_text` formatters and `fresh_input_each_call` for rebuilding. The Sifr runner now matches this behavior.

**One minor style note:** The binding variable for `balanced_tree[int]` (e.g., `root: TreeNode | None = _build_balanced_tree(...)`) is emitted but never used, since all call sites inline the rebuild. This is dead code in the generated runner. It doesn't affect correctness, compilation, or runtime, but could be suppressed by returning `""` from `sifr_binding_code` for `balanced_tree[int]` (mirroring the `list_node[int]` suppression at line 791). Not required for M1.

### Summary

All three harness fixes are correctly implemented. The approach:
- Does not hide ownership bugs (no clones added to `ListNode`/`TreeNode`, no `Display` impl required).
- Rebuilds all owned tree inputs for every call, matching Python harness behavior.
- Renders structured results exactly once into a local variable and reuses that variable for comparison, wrong-result printing, and checksum computation.
- M1 is ready to merge.
