# LeetCode Fixture Helper Convention

Status: accepted for workspace helper imports

## Decision

LeetCode audit fixtures may share canonical structural helpers from `audits/leetcode/helpers/`.
The repository root `sifr.toml` includes `audits/leetcode` as a source root, so non-`main.sifr`
fixtures can import helpers with paths such as:

```sifr
from helpers.list_node import ListNode, nodeVal, nodeNext, hasNode, listNodeToString
from helpers.tree_node import TreeNode, treeToString
```

Python fixture pairs use mirror helpers from the same directory:

```python
from helpers.list_node import ListNode, list_node_to_string
from helpers.tree_node import TreeNode, tree_to_string
```

## Approved Helpers

- `helpers/list_node.sifr` and `helpers/list_node.py`: canonical linked-list node shape and serialization/accessor helpers.
- `helpers/tree_node.sifr` and `helpers/tree_node.py`: canonical binary-tree node shape and serialization helpers.
- `helpers/trie.sifr`: LeetCode-only explicit trie helper backed by node indices.
- `helpers/dsu.sifr` and `helpers/dsu.py`: LeetCode-only disjoint-set helper.

The Python mirrors intentionally expose only the helpers Python fixtures use. Sifr-side accessors
such as `nodeVal`, `nodeNext`, `hasNode`, `connected`, and `component_count` exist to work around
current Sifr ownership/narrowing ergonomics; do not add Python mirrors just for symmetry.

## Rules

Shared helpers may contain:

- canonical fixture data structures such as `ListNode`, `TreeNode`, `Trie`, and `UnionFind`;
- accessors needed until narrowing and cursor ergonomics remove that ceremony;
- assertion/serialization helpers used by fixture tests.

Shared helpers must not contain:

- algorithm implementations for a LeetCode problem;
- fixture-specific sample builders or test data;
- alternate solutions;
- silent fallback helpers such as `unwrapInt`, `unwrapStr`, or sentinel-returning `nodeValue`;
- hidden mutable global state or object-identity emulation.

The Sifr linked-list accessors are temporary ergonomics helpers, not a precedent for new sentinel
unwrap helpers. Revisit them when WS6 narrowing/cursor work removes the need for optional accessors.

## Boundary Fixtures

Architecture-boundary fixtures keep inline shapes when the inline definition documents an ownership
or identity limitation:

- `0141_linked_list_cycle`
- `0160_intersection_of_two_linked_lists`
- `0894_all_possible_full_binary_trees`
- specialized `Node` shapes in `0133_clone_graph`, `0138_copy_list_with_random_pointer`,
  `0146_lru_cache`, and `0622_design_circular_queue`

Dead catch-all `Node` scaffolding should be deleted rather than moved into a helper.
