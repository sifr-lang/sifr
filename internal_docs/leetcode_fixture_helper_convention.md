# LeetCode Fixture Helper Convention

Status: accepted for WS3 B1

## Decision

Use self-contained fixture boilerplate with a strict shared template for `ListNode` and `TreeNode` helpers.

Root LeetCode fixtures are not named `main.sifr`, so the CLI intentionally treats them as single-file entries. That means sibling fixture-helper imports are not available without changing frontend mode resolution. Until that compiler behavior changes, linked-list and tree fixtures keep their structural helpers inline.

The approved convention is:

- keep a single `ListNode` or `TreeNode` definition at the top of each structural fixture
- keep only the helper functions the fixture actually uses
- use the same helper names across fixtures (`nodeVal`, `nodeNext`, `hasNode`, `listNodeToString`, `treeToString`)
- delete unrelated catch-all `Node` scaffolding unless the fixture itself needs it
- do not hide algorithm logic in helpers

## Scope

Inline helpers may contain:

- canonical fixture data structures such as `ListNode` and `TreeNode`
- value/next accessors needed until narrowing and cursor ergonomics remove that ceremony
- assertion and serialization helpers used only by fixture tests

Inline helpers must not contain:

- algorithm implementations for a LeetCode problem
- alternate solutions
- hidden mutable global state
- ownership shortcuts or object-identity emulation

## Rationale

Self-contained duplication made early fixture generation simple, but inconsistent boilerplate obscures the actual algorithm deltas and makes linked-list/tree rewrites noisy. A strict inline template keeps structural scaffolding consistent while preserving each root fixture as the source of the algorithm under review.

The original preferred import-based approach is blocked by current single-file entry semantics for non-`main.sifr` files. The strict inline template keeps the phase moving without changing CLI mode resolution as a side effect of fixture cleanup. A later compiler/frontend slice can revisit shared imports for non-main fixture entries.

## Pilot

`0021_merge_two_sorted_lists.sifr` is the pilot migration. It keeps the current drain/sort/rebuild algorithm unchanged, retains only the helpers that algorithm uses, and removes unrelated catch-all `Node` scaffolding. Cursor and owned-chain algorithm improvements belong to later WS3 slices.
