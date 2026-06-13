

## Implementation-Readiness Review

### Verdict: **Implementation-ready with high-severity findings to address first**

The phase is well-scoped and actionably ordered. The core scope is correct. However, there are naming strategy conflicts, missing manifest enumerations, and one coverage gap that must be resolved before implementation.

---

### Severity: HIGH

**1. Naming Strategy Conflict Between Phase And Plan Review**

The phase uses `LinkedNode`/`TreeNode` throughout, but the plan review at `reviews/compiler-test-leetcode-refactor-plan-review.md` uses `Record`/`TwoChildNode`. These are incompatible naming conventions that would cause the implementation to make arbitrary choices mid-pass.

**Required fix** — Add to the Naming Rules section:
```
| Old LeetCode name | Chosen replacement |
| --- | --- |
| `ChainCell` | `LinkedNode` (confirmed, not `Record`) |
| `BinaryBranch` | `TreeNode` (confirmed, not `TwoChildNode`) |
```

Remove all references to `Record`, `LinkNode`, `TwoChildNode` as replacements unless the phase explicitly adopts them. The implementer should not have to choose between two contradictory naming guides.

---

**2. Manifest Entry Updates Are Listed As "update manifests" But Not Enumerated**

The phase correctly identifies that manifest references must be updated, but provides no concrete list of which entries change. An implementer working from this phase would need to reverse-engineer the full manifest delta.

**Required additions** to the Docs And Manifests section:

```markdown
| Manifest | Entry to update | From | To |
| --- | --- | --- | --- |
| `verification/generated_code_quality/manifest.json` | line 59, `id` field | `e2e-048-recursive-chain-cell` | `e2e-048-recursive-link-node` |
| `verification/generated_code_quality/manifest.json` | line 59, `source_path` | `recursive_chain_cell.sifr` | `recursive_linked_node.sifr` |
| `verification/validation_lanes/quick_e2e_manifest.json` | `"recursive_branch_traversal_runtime"` | — | rename to match new e2e filename |
| `verification/validation_lanes/pr_e2e_manifest.json` | `"recursive_branch_traversal_runtime"` | — | rename to match new e2e filename |
| `verification/validation_lanes/pr_e2e_manifest.json` | `"forward_ref_chain_cell"` | — | rename to match new e2e filename |
| `verification/validation_lanes/pr_e2e_manifest.json` | `"recursive_chain_cell"` | — | rename to match new e2e filename |
```

This ensures the implementer knows the full manifest surface and doesn't miss `pr_e2e_manifest.json` entries.

---

**3. `demos/nested_helpers/main.sifr` Problem-Adjacent Names Not Explicitly Named**

The phase lists `demos/nested_helpers/` under Demos And Companions and mentions "Letter combinations, N-Queens, redundant connection examples" in plain text, but does not explicitly call out the function names that need replacement:

- `demo_letter_combinations` → behavior name like `map_string_expand`
- `demo_total_n_queens` → behavior name like `count_configurations`
- `demo_redundant_connection` → behavior name like `detect_first_cycle`

The plan review explicitly lists these. The phase should match that specificity.

**Required addition** to the Demos And Companions table:
```markdown
| `demos/nested_helpers/main.sifr` | `demo_letter_combinations`, `demo_total_n_queens`, `demo_redundant_connection` | Replace with behavior-named helpers preserving dict-key guard, recursive set constraint counting, and union-find cycle detection coverage; regenerate/update companions. |
```

---

### Severity: MEDIUM

**4. E2E Fixture `nested_function_recursive_collection_backtracking.sifr` Has No Explicit Rename Entry**

The phase describes rewriting this fixture's `collect_budget_routes` but does not list the file in the E2E Fixtures table with a planned rename. The matching HIR test `test_recursive_nested_helper_infers_mutable_collection_param_from_usage` in `nested_function_tests.rs` is listed, but the e2e fixture itself is only mentioned in the description text.

**Required addition** to the E2E Fixtures table:
```markdown
| `crates/sifr/tests/e2e/pass/nested_function_recursive_collection_backtracking.sifr` | Same file | Rewrite `collect_budget_routes` to neutral captured-collection mutation fixture; preserve `copy`, `append`, `pop`, recursive inference coverage. |
```

---

**5. Validation Plan Missing Manifest-Update Verification Step**

The validation plan runs e2e and demo checks but does not include a step that confirms manifest entries resolve to existing files after rename.

**Required addition** to the Validation Plan section:
```bash
# Verify manifest entries reference existing files after rename
python3 verification/generated_code_quality/generated_code_quality.py e2e
python3 verification/generated_code_quality/generated_code_quality.py demos
```

These commands already exist in the plan but are listed under the quality gate, not under a "verify manifest consistency" checkpoint. Adding them as an explicit validation step near the top of the Validation Plan will prevent manifest-only changes from silently breaking the quality gate.

---

### Severity: LOW

**6. File-Size Guardrail Warning Is Implicit**

The phase mentions `collections_and_stdlib_codegen_tests.rs` "close to 900-line cap" but does not include a validation step that would catch a regression. The Validation Plan's `wc -l` command is adequate but should be called out as a guardrail check, not just a whitespace check.

**No change required** — the existing `wc -l crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs` command in the Validation Plan is sufficient.

---

**7. `collect_budget_routes` Fixture Content Suggests A New Behavior Name**

The HIR test `test_recursive_nested_helper_infers_mutable_collection_param_from_usage` at `nested_function_tests.rs:92` uses `collect_budget_routes` which mirrors combination-sum structure. The plan review recommends rewriting this to a "smaller captured-collection mutation fixture with neutral data." However, no suggested behavior name is provided.

**Proposed behavior name**: `accumulate_items` — captures the pattern (recursively accumulate list items to a captured collection) without problem framing.

**Required addition** to the Naming Rules:
```markdown
| `collect_budget_routes` | `accumulate_items` (or similar neutral accumulated-collection name) |
```

---

### Residual Risks (Post-Phase-Fixes)

Once the above items are addressed, the phase is implementation-ready. Residual risks:

- **Boundary judgment calls**: The phase correctly excludes `audits/**`, `third_party/**`, and stdlib parity demos (`demos/cloned_iterators/`, `demos/iterable_stdlib/`). The implementer must use judgment when encountering algorithm-adjacent names that aren't explicitly listed — the phase's "do not remove coverage simply because the old fixture was algorithm-shaped" guard is the correct guide.

- **`mirrored_sum` was a phantom**: Confirmed that no `mirrored_sum` identifier exists in the codebase. The phase correctly describes this test via `PairRecord`/`combined_value` behavior. No action needed.

- **Snapshot files**: Confirmed no external `.snap` files reference the renamed patterns. No snapshot updates are needed.

---

### Summary of Required Phase Changes

| # | Section | Change |
| --- | --- | --- |
| 1 | Naming Rules | Add explicit `LinkedNode`/`TreeNode` confirmation; remove `Record`/`TwoChildNode` alternatives |
| 2 | Docs And Manifests | Enumerate every manifest entry that changes with old→new values |
| 3 | Demos And Companions | Add explicit `demo_letter_combinations`/`demo_total_n_queens`/`demo_redundant_connection` rename entries |
| 4 | E2E Fixtures | Add explicit `nested_function_recursive_collection_backtracking.sifr` rewrite entry |
| 5 | Validation Plan | Add explicit manifest-resolve validation step near e2e/demo checks |
| 6 | Naming Rules | Add `collect_budget_routes` → `accumulate_items` (or similar) mapping |
