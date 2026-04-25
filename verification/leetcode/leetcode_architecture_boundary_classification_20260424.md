# LeetCode Architecture Boundary Classification

Date: 2026-04-24
Source phase: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`
Source review: `reviews/leetcode_divergence_category_review_20260421_category4_arch_boundary.md`

## Scope

This artifact records fixtures whose remaining divergence is intentional under Sifr's current architecture boundaries. These are not canonical rewrite backlog items unless a separate safe arena/handle or ownership feature is approved.

## Category 4a: Mutable Nonlocal Capture Boundary

### `0673_number_of_longest_increasing_subsequence`

Python uses mutable `nonlocal` closure state for recursive DP accumulators. Sifr intentionally does not support mutable `nonlocal` capture, so the fixture keeps explicit accumulator state.

Current status:

- Primary Category 4a classification remains correct.
- The previous linear `valueAt` collection workaround is tracked separately as collection/index ergonomics pressure, not as a reason to relax the nonlocal boundary.

### Pattern Continuity Fixtures

These below-cutoff fixtures demonstrate the same nonlocal-to-explicit-state pattern. They are not escalated rewrite items in this phase, but if a future scan promotes them they retain Category 4a classification:

- `0052_n_queens_ii`
- `0543_diameter_of_binary_tree`
- `0783_minimum_distance_between_bst_nodes`
- `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero`

## Category 4b: Object Identity And Shared Ownership Boundary

### `0133_clone_graph`

Canonical Python graph cloning relies on object identity and a memo from original node object to cloned node object. Sifr does not currently expose a safe object-identity graph model with shared mutable aliases. Keep this classified as an architecture boundary until a safe arena/handle design exists.

### `0138_copy_list_with_random_pointer`

Canonical Python preserves arbitrary `random` pointers across heap nodes. Sifr's single-ownership model cannot directly encode the aliased object graph without an approved handle/arena representation. Value-semantic alternates must not be claimed as canonical object-identity parity.

### `0141_linked_list_cycle`

Cycle detection is boundary-limited because a true cyclic linked list requires shared ownership or a handle graph. Tests that only exercise acyclic lists are smoke checks, not evidence of canonical cycle-input support.

Helper extraction note: keep this fixture's `ListNode` shape inline. Importing the shared linked-list helper would obscure that the fixture is intentionally smoke-only under the current ownership model.

### `0160_intersection_of_two_linked_lists`

The canonical problem is about two list heads sharing tail node identity. Sifr-owned chains cannot share the same tail node under two owners. A value-equality or copied-tail fixture is a supported alternate, not canonical object-identity parity.

Helper extraction note: keep this fixture's `ListNode` shape inline. The local definition documents the shared-tail identity boundary rather than a generic linked-list algorithm helper.

### `0894_all_possible_full_binary_trees`

Python can memoize and reuse subtree objects across many generated parent trees. Sifr must clone subtrees to preserve single ownership. This is the intended ownership boundary; do not introduce shared ownership or interior mutability just to emulate Python subtree aliasing.

Helper extraction note: keep this fixture's `TreeNode` shape inline. The cloned-subtree construction is the ownership boundary under review, so the fixture should remain self-describing.

## Closure Rule

These fixtures remain closed for this phase when:

- the classification is linked from the execution report,
- no Category 4 item is listed as unresolved canonical rewrite debt,
- future work that wants canonical object identity first proposes a safe arena/handle model,
- future work that wants mutable closure state first proposes a sound nonlocal-capture design.
