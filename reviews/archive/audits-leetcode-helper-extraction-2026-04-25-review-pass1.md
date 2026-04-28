# Pass-1 Review: LeetCode Audit Helper Extraction (2026-04-25)

Reviewer date: 2026-04-25
Reviewer angle: implementation completeness and convention conformance for an external reviewer
Issue under review: [issues/audits-leetcode-helper-extraction-2026-04-25.md](../issues/audits-leetcode-helper-extraction-2026-04-25.md)
Branch: `audits-leetcode-helper-extraction`
Cross-checked against:
- [internal_docs/leetcode_fixture_helper_convention.md](../internal_docs/leetcode_fixture_helper_convention.md)
- [internal_docs/leetcode_trie_helper_design.md](../internal_docs/leetcode_trie_helper_design.md)
- [verification/leetcode/leetcode_architecture_boundary_classification_20260424.md](../verification/leetcode/leetcode_architecture_boundary_classification_20260424.md)
- [verification/leetcode/full_corpus_current_results_20260425_helper_extraction.json](../verification/leetcode/full_corpus_current_results_20260425_helper_extraction.json)
- [verification/leetcode/leetcode_pair_diff_scan_20260425_helper_extraction.json](../verification/leetcode/leetcode_pair_diff_scan_20260425_helper_extraction.json)

## Summary

The implementation faithfully executes the seven-PR sequence (PR-A through PR-G) the issue prescribes, lands the helper convention update, scrubs the misplaced `sifr.dsu` stdlib entry plus its e2e/unit coverage, and ships a focused compiler enablement (project exports retain class constructors; project codegen now preloads cross-module function/method signatures and class field metadata) so workspace-imported classes get correct constructor calls and `Box<T>` recursive-field wrapping.

Validation evidence is strong:
- Full LeetCode corpus rerun ([full_corpus_current_results_20260425_helper_extraction.json](../verification/leetcode/full_corpus_current_results_20260425_helper_extraction.json)): 411/411 cases, 208 PASS / 203 NO_ORACLE, **zero CHECK_ERROR / RUN_ERROR / TIMEOUT** (issue exit criterion).
- Pair-diff scan ([leetcode_pair_diff_scan_20260425_helper_extraction.json](../verification/leetcode/leetcode_pair_diff_scan_20260425_helper_extraction.json)): 395 paired cases, 16 sifr_only `_v2` placeholders (matches the documented non-goal), 1 py_only (`batch_convert`).
- New regression test [crates/sifr_driver/src/tests/project_build_check.rs:114](../crates/sifr_driver/src/tests/project_build_check.rs:114) (`test_build_project_preserves_imported_class_constructors_and_signatures`) directly exercises imported-class construction.

## Readiness Verdict

**No blockers.** The core requirements (helpers contain only canonical structures + permitted accessors/serializers; no algorithms, test data, or silent-fallback `unwrapInt`/`unwrapStr`/`nodeValue`; no misplaced `sifr.dsu` in any code path; architecture-boundary inline justifications captured in the classification doc; compiler changes scoped to enabling helper imports) all hold up under inspection. Several non-blocking items below are worth folding into a follow-up.

## Verification of stated guarantees

### Helpers contain no algorithms, test data, or silent fallbacks — verified

- [audits/leetcode/helpers/list_node.sifr](../audits/leetcode/helpers/list_node.sifr) holds only `ListNode`, `nodeVal`, `nodeNext`, `hasNode`, and `listNodeToString` — no `mergeTwoLists`, no sample builders, no `unwrapInt`. The pilot contamination called out in issue §17 has been undone: [audits/leetcode/0021_merge_two_sorted_lists.sifr:1](../audits/leetcode/0021_merge_two_sorted_lists.sifr) now redefines `mergeTwoLists`, `sampleListA`, `sampleListB`, and `singleZeroList` locally and imports only the canonical helper symbols.
- [audits/leetcode/helpers/list_node.py](../audits/leetcode/helpers/list_node.py) is the minimal Python mirror (`ListNode` + `list_node_to_string`).
- [audits/leetcode/helpers/tree_node.sifr](../audits/leetcode/helpers/tree_node.sifr) and [audits/leetcode/helpers/tree_node.py](../audits/leetcode/helpers/tree_node.py) hold only `TreeNode` + the serializer.
- [audits/leetcode/helpers/trie.sifr](../audits/leetcode/helpers/trie.sifr) is the index-backed trie data structure described in the design doc; reads return `int | None` / `bool` rather than sentinel ints, so it is not a silent-fallback shape. No Python mirror was added (consistent with the issue's note that Python `0211` / `0212` use different shapes).
- [audits/leetcode/helpers/dsu.sifr](../audits/leetcode/helpers/dsu.sifr) is the canonical UnionFind data structure (find, union, connected, component_count); [audits/leetcode/helpers/dsu.py](../audits/leetcode/helpers/dsu.py) reconciles the four Python consumers.

The `nodeVal`/`nodeNext` accessors in the Sifr helper return `0` / `None` for `None` input, which is shape-adjacent to the WS6 silent-fallback anti-pattern. The convention deliberately permits this under the "accessors needed until narrowing and cursor ergonomics remove that ceremony" carve-out ([leetcode_fixture_helper_convention.md:34-37](../internal_docs/leetcode_fixture_helper_convention.md)), distinguishing them from the standalone `unwrapInt` / sentinel-`nodeValue` shape that WS6 owns. Conformant.

### `sifr.dsu` is fully removed from code paths — verified

- [crates/sifr_driver/src/stdlib/registry.rs:67](../crates/sifr_driver/src/stdlib/registry.rs) drops the `("sifr.dsu", include_str!(...))` entry.
- [lib/sifr/dsu.sifr](../lib/sifr/dsu.sifr) is deleted.
- [crates/sifr_driver/src/tests/stdlib_exports.rs](../crates/sifr_driver/src/tests/stdlib_exports.rs) drops `stdlib_dsu_exports_union_find_class`.
- [crates/sifr/tests/e2e/pass/stdlib_dsu.sifr](../crates/sifr/tests/e2e/pass/stdlib_dsu.sifr) is deleted (issue §113 had asked for a "keep + retarget" vs "delete" decision; the implementer chose delete, which is consistent with treating it as an export smoke test now that DSU is no longer stdlib).
- `grep` for `sifr.dsu` and `from sifr.dsu` across `crates/`, `lib/`, and `audits/` returns zero hits.
- The two Sifr DSU consumers ([audits/leetcode/0261_graph_valid_tree.sifr](../audits/leetcode/0261_graph_valid_tree.sifr), [audits/leetcode/0323_number_of_connected_components_in_an_undirected_graph.sifr](../audits/leetcode/0323_number_of_connected_components_in_an_undirected_graph.sifr)) and the four Python consumers (`0323`, `0721`, `1489`, `2709`) all import from `helpers.dsu`.

### Architecture-boundary inline shapes are justified — verified

- [verification/leetcode/leetcode_architecture_boundary_classification_20260424.md](../verification/leetcode/leetcode_architecture_boundary_classification_20260424.md) gains explicit "Helper extraction note" lines for `0141_linked_list_cycle`, `0160_intersection_of_two_linked_lists`, and `0894_all_possible_full_binary_trees`, each tying the inline shape to the existing ownership/identity boundary already classified in that document.
- The convention doc's "Boundary Fixtures" list ([leetcode_fixture_helper_convention.md:46-55](../internal_docs/leetcode_fixture_helper_convention.md)) and the issue exit criteria match the actual inline-class survivors: only `0141`, `0160`, `0894` retain inline `ListNode`/`TreeNode`; only `0133`, `0138`, `0146`, `0622` retain a specialized `Node`. `grep -lE 'class Node:' audits/leetcode/*.sifr` returns no hits (Sifr `Node` was deleted everywhere) and the Python `Node` survivors are exactly the four documented carve-outs.
- The catch-all `Node` cleanup is verified clean.

### Compiler changes are appropriately scoped — verified

The compiler delta is small (≈150 changed lines across four crates) and each piece serves the imported-helper use case:

- [crates/sifr_driver/src/project/exports.rs:50](../crates/sifr_driver/src/project/exports.rs) — removes `.filter(|m| m.name != "new")` so workspace exports retain the synthesized constructor; this is the minimum needed for `from helpers.list_node import ListNode` to type-check `ListNode(...)` at the call site.
- [crates/sifr_codegen/src/lib.rs:181-247](../crates/sifr_codegen/src/lib.rs) — adds `StdlibCode::module_class_fields` and `module_func_signatures` / `module_class_fields` extractors; project codegen ([crates/sifr_codegen/src/lib.rs:757-770](../crates/sifr_codegen/src/lib.rs)) now clones `StdlibCode` and folds in per-module signatures + field maps before emitting any module, so cross-module call sites see correct param conventions and `Box<T>` recursion wrappers.
- [crates/sifr_codegen/src/field_analysis_helpers.rs:153-176](../crates/sifr_codegen/src/field_analysis_helpers.rs) — `register_external_class_fields` mirrors the existing intra-module recursive-field detection for imported classes (treats `(local_name, source_name)` as the SCC of size ≤2). Sufficient for `ListNode { next: ListNode | None }` and `TreeNode { left|right: TreeNode | None }`.
- [crates/sifr_driver/src/stdlib/bootstrap.rs:181, 270-277](../crates/sifr_driver/src/stdlib/bootstrap.rs) — populates `module_class_fields` from compiled stdlib modules so the new code path is consistent for stdlib-imported classes too. (No behavioral change for stdlib in practice; just structural symmetry.)

The new test [crates/sifr_driver/src/tests/project_build_check.rs:114-167](../crates/sifr_driver/src/tests/project_build_check.rs) covers the surface (workspace + lib helper, default-arg constructor, custom class without recursion, accessor) and was confirmed passing in §212.

## Non-blocking observations

### N1. Python DSU helper has a dual-mode shape with subtle inconsistency

[audits/leetcode/helpers/dsu.py](../audits/leetcode/helpers/dsu.py) supports two modes: dict-backed when `n is None` (`0323` consumer), list-backed when `n: int` (`0721`, `1489`, `2709`). In dict-mode, `union` always returns `True`, even when both args are already in the same component. List-mode correctly returns `False` in that case. Today no consumer reads dict-mode's return, so this does not affect validation, but a future fixture writing `if uf.union(a, b):` will silently behave incorrectly under dict-mode. Either align the dict-mode return contract or document the mode-dependent return.

The Python helper also keeps `count`, `rank`, and `size` as empty lists in dict-mode, so `uf.count` reads as `0` rather than the actual component count. Worth a 1-line comment near the constructor stating the mode contract; otherwise, follow-on consumers will grow ad-hoc workarounds.

### N2. Helper symmetry between Python and Sifr is intentional but undocumented

[audits/leetcode/helpers/list_node.py](../audits/leetcode/helpers/list_node.py) intentionally omits `node_val` / `node_next` / `has_node` mirrors because Python's narrowing handles `node is None` checks directly, whereas Sifr fixtures still need the accessors. Same asymmetry holds for DSU (`connected` / `component_count` exist only on the Sifr side). This is correct, but neither the convention doc nor the helper files note the rationale; a contributor migrating future fixtures may add Python mirrors out of misplaced symmetry. A two-line note in [leetcode_fixture_helper_convention.md](../internal_docs/leetcode_fixture_helper_convention.md) under "Approved Helpers" would forestall that.

### N3. `register_external_class_fields` only models self-recursion across the import boundary

[crates/sifr_codegen/src/field_analysis_helpers.rs:164-165](../crates/sifr_codegen/src/field_analysis_helpers.rs) constructs `same_class_names` as `{local_name, source_name}` — i.e. the SCC contains at most one logical class. This is sufficient for every helper shipped in this branch (`ListNode`, `TreeNode`, `Trie` self-only references). It will under-Box if a future helper adds mutually-recursive classes (e.g. `class A { b: B|None }; class B { a: A|None }`) imported together. Worth a TODO/comment near the helper noting the limitation and pointing at `detect_recursive_fields` as the in-module analog that handles the general case.

### N4. `exports.rs` filter removal is broader than "local helpers only"

[crates/sifr_driver/src/project/exports.rs:50](../crates/sifr_driver/src/project/exports.rs) drops `.filter(|m| m.name != "new")` unconditionally, so every workspace module — not just `audits/leetcode/helpers/` — now exports class constructors. That is the right behavior and matches what the new test asserts. But the issue's framing was specific to helper imports, and the removed filter may have predated a constraint that no longer applies. A one-line commit message or doc note explaining "constructors are now first-class exports" would help future readers; absent that, the change reads as incidental.

### N5. `nodeVal` / `nodeNext` accessors as canonical helpers

The convention rule permits these as accessors needed pending narrowing/cursor ergonomics. They are nevertheless sentinel-returning (`0` and `None` for absent input) and compose with `hasNode`-then-call patterns in the migrated fixtures. When the WS6 silent-fallback remediation lands narrowing improvements, `nodeVal` / `nodeNext` should be revisited for retirement; otherwise they will become an entrenched workaround under a "permitted accessor" label. Recommend tracking that revisit explicitly in [issues/leetcode-ws6-silent-fallback-remediation-2026-04-25.md](../issues/leetcode-ws6-silent-fallback-remediation-2026-04-25.md) so the carve-out has a defined sunset.

### N6. `0894` inline `TreeNode` is byte-identical to the helper

The helper-extraction note in [leetcode_architecture_boundary_classification_20260424.md:55](../verification/leetcode/leetcode_architecture_boundary_classification_20260424.md) says "the cloned-subtree construction is the ownership boundary under review, so the fixture should remain self-describing". The class shape itself, however, is identical to `helpers/tree_node.sifr`. The boundary lives in `cloneTree` / `allPossibleFBT`, not in the class declaration. The justification works as is, but a short inline `# boundary: ...` comment near the class in [audits/leetcode/0894_all_possible_full_binary_trees.sifr](../audits/leetcode/0894_all_possible_full_binary_trees.sifr) would communicate the boundary at the point of duplication, making the fixture genuinely self-describing rather than relying on the cross-document reference. Same comment applies to `0141` and `0160`.

### N7. `_v2` sifr-only fixtures are correctly scoped out

The pair scan reports 16 `sifr_only` cases, all `*_v2`. This matches the explicit non-goal in the issue (`§180`) and the divergence-closure phase's tracking. No action required; just confirming the artifact reflects the documented exclusion.

## Confirmation of validation artifacts

- [verification/leetcode/full_corpus_current_results_20260425_helper_extraction.json](../verification/leetcode/full_corpus_current_results_20260425_helper_extraction.json): 411 cases, summary `{"NO_ORACLE": 203, "PASS": 208}`, `{"in_scope": 411}`, no failure stages. Meets the issue's `Exit Criteria` line "Full LeetCode corpus rerun shows no CHECK_ERROR, RUN_ERROR, or TIMEOUT".
- [verification/leetcode/leetcode_pair_diff_scan_20260425_helper_extraction.json](../verification/leetcode/leetcode_pair_diff_scan_20260425_helper_extraction.json): 395 paired cases, 16 sifr_only (`_v2`), 1 py_only (`batch_convert`). Top-similarity entries (`1397`, `1203`, `0146`, `1489`) are all known boundary or below-cutoff cases; migrated fixtures (e.g. `0021` `changed_total_lines: 69`) sit well below those tops, consistent with helper extraction reducing fixture-local diff.

## Conclusion

The implementation is ready to ship. The non-blocking items above are documentation/comment-level polish and small future-proofing concerns; none requires re-work of this branch. Recommend addressing **N1** (DSU dual-mode return contract) opportunistically because it is the only one with a latent correctness exposure for future fixtures.
