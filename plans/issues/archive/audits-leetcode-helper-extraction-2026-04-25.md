# LeetCode Audit Helper Extraction Report

Status: implementation complete, external review addressed
Owner: Codex
Created: 2026-04-25
Related:
- `issues/sifr-workspace-pyproject-import-resolution-2026-04-25.md` (workspace resolver — landed)
- `issues/leetcode-ws6-silent-fallback-remediation-2026-04-25.md` (silent-fallback cleanup — overlaps `unwrapInt`)
- `internal_docs/leetcode_fixture_helper_convention.md` (helper-content rules)

## Purpose

Now that `pyproject.toml` workspace resolution is implemented, fixture-driven helpers can move from inline duplication (and from the misplaced `lib/sifr/` stdlib namespace) into a shared `audits/leetcode/helpers/` directory consumable by both Python and Sifr fixtures. This report enumerates every extraction candidate identified by scanning `audits/leetcode/*.{sifr,py}` and notes the drift status of each.

Both languages need parallel helpers: Python pairs cannot import from a Sifr file, so each helper lands as two files (e.g. `helpers/list_node.sifr` and `helpers/list_node.py`).

## Pilot Status (Pre-Existing Issue)

`audits/leetcode/helpers/list_node.sifr` already exists from the workspace pilot but is **contaminated**: it contains `mergeTwoLists` (the algorithm under review for problem 0021), the test data builders `sampleListA` / `sampleListB` / `singleZeroList`, and a silent-fallback `unwrapInt` helper. These violate `internal_docs/leetcode_fixture_helper_convention.md` (helpers must not contain algorithm implementations or alternate solutions). **First action: clean the pilot helper to contain only `ListNode` + accessors before any further migration.**

## Drift Audit

A SHA-1 signature of every class body across all fixtures shows zero drift:

| Class | .sifr files | .sifr signature buckets | .py files | .py signature buckets |
| --- | ---: | ---: | ---: | ---: |
| `ListNode` | 23 | 1 | 25 | 1 |
| `TreeNode` | 35 | 1 | 35 | 1 |
| `Trie` | 3 | 1 (head verified, full body unverified) | 1 | n/a (only one fixture) |
| `UnionFind` | 2 (via `sifr.dsu` import) | 1 | 4 (inline) | unverified, likely drift |
| catch-all `Node` | 52 | unverified | 63 | unverified |

ListNode and TreeNode are byte-identical across every consumer in both languages. Bulk migration is safe.

## Extraction Candidates

### 1. `ListNode` (high priority)

**Sifr fixtures (23):** `0002, 0019, 0023, 0024, 0025, 0061, 0083, 0086, 0092, 0141, 0143, 0147, 0148, 0160, 0203, 0206, 0234, 0707, 0876, 1472, 1669, 1721, 2130`

**Python fixtures (25):** same set plus `0021, 0706` (Sifr-side 0021 already imports the contaminated helper; 0706 has only the Python pair using `ListNode`).

**Helpers used together with the class:**

- `nodeVal` (21 .sifr files)
- `nodeNext` (21 .sifr files)
- `hasNode` (21 .sifr files)
- `listNodeToString` (21 .sifr files)
- Python mirror: `tree_to_string`/list-to-string variants are not standardized — verify before extracting Python helpers.

**Target files:**

- `audits/leetcode/helpers/list_node.sifr`: only `ListNode` + the four accessors above.
- `audits/leetcode/helpers/list_node.py`: only `ListNode` + the canonical accessors.

**Architecture-boundary caveat:** `0141_linked_list_cycle` and `0160_intersection_of_two_linked_lists` are Category 4b in `verification/leetcode/leetcode_architecture_boundary_classification_20260424.md`. Their inline `ListNode` may need to stay inline if they encode an alternate ownership shape. Verify before migrating those two.

### 2. `TreeNode` (high priority)

**Sifr fixtures (35):** `0094, 0098, 0100, 0101, 0102, 0103, 0104, 0105, 0106, 0108, 0110, 0112, 0124, 0144, 0145, 0199, 0226, 0230, 0235, 0236, 0297, 0450, 0513, 0535, 0543, 0572, 0606, 0617, 0662, 0669, 0701, 0783, 0894, 1448, 1609`

**Python fixtures (35):** identical set.

**Helpers used together with the class:**

- `treeToString` / `tree_to_string` (33 fixtures each)

**Target files:**

- `audits/leetcode/helpers/tree_node.sifr`
- `audits/leetcode/helpers/tree_node.py`

**Architecture-boundary caveat:** `0894_all_possible_full_binary_trees` is Category 4b. Verify shape before migrating.

### 3. `Trie` (medium priority — already partly cleaned up)

**Background:** previously lived in `lib/sifr/trie.sifr` as a stdlib registry entry; correctly removed because Python has no `trie` stdlib module. Currently triplicated inline.

**Sifr fixtures (3):** `0208_implement_trie_prefix_tree`, `0211_design_add_and_search_words_data_structure`, `0212_word_search_ii`.

**Python fixtures:** only `0208` defines `class Trie:` directly. `0211` and `0212` Python pairs use dict-based or other in-place trie shapes — verify whether a shared Python helper is wanted or whether each Python fixture should keep its own canonical shape.

**Target files:**

- `audits/leetcode/helpers/trie.sifr`
- `audits/leetcode/helpers/trie.py` (only if all three Python pairs benefit; otherwise skip the Python side)

**Pre-existing design note:** `internal_docs/leetcode_trie_helper_design.md` is **stale** — it still says *"Sibling helper imports are currently not available for non-`main.sifr` LeetCode root fixtures"*, which is no longer true. Update during this extraction.

### 4. `UnionFind` (medium priority — currently mis-located in stdlib)

**Background:** `lib/sifr/dsu.sifr` is registered as `sifr.dsu` in the stdlib registry, but Python has no `dsu` stdlib module. Same category error as the previous `sifr.trie`.

**Sifr fixtures consuming `sifr.dsu` (2):** `0261_graph_valid_tree`, `0323_number_of_connected_components_in_an_undirected_graph`.

**Python fixtures with inline `class UnionFind:` or `class DSU:` (4):**
- `0323_number_of_connected_components_in_an_undirected_graph.py`
- `0721_accounts_merge.py`
- `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree.py`
- `2709_greatest_common_divisor_traversal.py`

**Drift status:** Python inline definitions are unverified — likely drift across the four Python files. Audit before merging into a single canonical helper. Sifr side has no drift (only two consumers, both via the shared registry import).

**Target files:**

- `audits/leetcode/helpers/dsu.sifr` (move from `lib/sifr/dsu.sifr`)
- `audits/leetcode/helpers/dsu.py`

**Side effects:**

- Delete the `sifr.dsu` entry in `crates/sifr_driver/src/stdlib/registry.rs`.
- Delete the `cargo test -p sifr_driver stdlib_dsu_exports_union_find_class` test in `crates/sifr_driver/src/tests/stdlib_exports.rs`.
- Decide whether `crates/sifr/tests/e2e/pass/stdlib_dsu.sifr` covers a language-level concern (keep + retarget) or is just an export smoke test (delete).
- The Sifr-only fixtures `0721`, `1489`, `2709` exist as `_v2` placeholders without Python pairs — they're tracked under the divergence-closure phase's *"sifr_only_v2 fixtures"* non-goal. Do not migrate those in this slice; they need a separate decision.

### 5. Catch-all `class Node` (DELETE, do not extract)

**Sifr fixtures with the dead `class Node`:** 52
**Python fixtures with the dead `class Node`:** 63

The catch-all class shape is:

```python
class Node:
    val: int
    next: Node | None
    random: Node | None
    left: Node | None
    right: Node | None
    neighbors: list[Node]
    key: int
    def __init__(self, ...): ...
```

This is leftover scaffolding from earlier fixture autogeneration. Inspection of every consumer shows the only references to `Node` are *within* its own class body. **Recommendation: delete from all 52+63 files in a separate cleanup PR.** This is independent of the workspace migration and removes ~30 lines per file from the pair-scan diff.

**Exceptions to verify before deleting:**
- `0133_clone_graph` — graph node with `neighbors`, genuinely needed.
- `0138_copy_list_with_random_pointer` — uses `next` and `random`, genuinely needed.

These two should keep their own specialized `Node` shapes inline. They are Category 4b architecture-boundary fixtures and the inline definition documents the boundary.

### 6. Silent-Fallback Helpers (DELETE, tracked elsewhere)

**`unwrapInt`** appears in 46 .sifr fixtures. **`unwrapStr`** appears in 2 .sifr fixtures. **`nodeValue`** (silent-fallback variant) appears in 3 .sifr fixtures.

These are the silent-fallback anti-pattern already chartered for removal under `issues/leetcode-ws6-silent-fallback-remediation-2026-04-25.md`. **Do not migrate them into shared helpers** — that would lock the anti-pattern into a single canonical place. They belong in the WS6 follow-up, which closes them by either fixing the underlying narrowing gap or rewriting the fixture to avoid the optional access.

## Suggested PR Sequence

1. **PR-A: Fix the pilot.** Move `mergeTwoLists`, `sampleListA`, `sampleListB`, `singleZeroList` back into `audits/leetcode/0021_merge_two_sorted_lists.sifr`. Drop `unwrapInt` from the helper. Reduce `audits/leetcode/helpers/list_node.sifr` to only `ListNode` + `nodeVal` + `nodeNext` + `hasNode` + `listNodeToString`. Add `audits/leetcode/helpers/list_node.py` with the canonical Python shape. Update `internal_docs/leetcode_fixture_helper_convention.md` to record the workspace-based convention. Targeted `check`/`run` for `0021`. Regenerate pair scan.

2. **PR-B: Bulk `ListNode` migration.** 23 Sifr fixtures + 25 Python fixtures replace inline `class ListNode:` plus the four accessors with `from helpers.list_node import ...`. Skip `0141` and `0160` pending architecture-boundary review. Targeted `check`/`run` per fixture. Regenerate pair scan.

3. **PR-C: `TreeNode` extraction.** Add `audits/leetcode/helpers/tree_node.{sifr,py}`. 35 Sifr + 35 Python fixtures migrate. Skip `0894`. Targeted `check`/`run`. Regenerate pair scan.

4. **PR-D: Trie extraction.** Add `audits/leetcode/helpers/trie.sifr`. Decide on a Python helper — likely skip if `0211`/`0212` Python pairs use different shapes. Migrate `0208`, `0211`, `0212` Sifr-side. Update or retire `internal_docs/leetcode_trie_helper_design.md`. Targeted `check`/`run`. Regenerate pair scan.

5. **PR-E: DSU stdlib unmount.** Move `lib/sifr/dsu.sifr` → `audits/leetcode/helpers/dsu.sifr`. Add `audits/leetcode/helpers/dsu.py` reconciling drift across the four Python consumers. Delete `sifr.dsu` from the stdlib registry, the export test, and the e2e smoke test (or retarget). Migrate `0261`, `0323` Sifr-side. Migrate `0323`, `0721`, `1489`, `2709` Python-side to import the shared helper. Targeted `check`/`run`. Regenerate pair scan.

6. **PR-F: Dead `Node` cleanup.** Delete the catch-all `class Node` block from 50+ Sifr and 60+ Python fixtures (preserving specialized shapes in `0133` and `0138`). Pure deletion, no semantic change. Regenerate pair scan — expect a substantial drop in `changed_*` lines purely from removing dead scaffolding.

7. **PR-G: Architecture-boundary review.** Decide per-fixture whether `0141`, `0160`, and `0894` migrate to the shared helpers or keep their inline shape. If they keep inline shapes, document why in the architecture-boundary classification doc.

## Validation

Each PR must:

- Run targeted `python3 audits/leetcode/<fixture>.py` for every Python fixture it touches.
- Run `cargo run -q -p sifr -- check audits/leetcode/<fixture>.sifr` and `... run ...` for every Sifr fixture it touches.
- Regenerate `verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json` and confirm only the expected fixtures move.
- Pass `scripts/run_all_tests.sh --profile quick`.

The DSU-related PR (E) must additionally pass `scripts/run_all_tests.sh` (full profile) before merge because it touches stdlib registration.

## Non-Goals

- Do not migrate the `_v2` Sifr-only fixtures in this slice. They remain under the divergence-closure phase non-goal.
- Do not consolidate the silent-fallback helpers into a shared file. Their fix is the WS6 remediation work, not the helper extraction work.
- Do not introduce new helper APIs during migration. Each PR moves *existing* helpers verbatim. New API surface (e.g. a richer `ListNode` builder) is a separate proposal.
- Do not change architecture-boundary classifications. Category 4b fixtures keep their inline shapes unless a separate design approves a workspace-shared boundary helper.

## Exit Criteria

- `audits/leetcode/helpers/list_node.{sifr,py}`, `tree_node.{sifr,py}`, `trie.sifr`, and `dsu.{sifr,py}` exist and contain only canonical helpers (no algorithms, no test data, no silent fallbacks).
- All non-architecture-boundary fixtures import from the helper modules instead of redeclaring the classes.
- `lib/sifr/dsu.sifr` is removed; the stdlib registry no longer registers `sifr.dsu`.
- Catch-all `class Node` scaffolding is removed; genuine local `Node` shapes remain in `0133`, `0138`, `0146`, and `0622`.
- `internal_docs/leetcode_fixture_helper_convention.md` and `internal_docs/leetcode_trie_helper_design.md` reflect the workspace-based convention.
- Full LeetCode corpus rerun shows no `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`.
- Pair scan shows the expected drop in `changed_*_lines` for migrated fixtures (helpers no longer count as fixture-local diff).

## Implementation Notes

Applied in branch `audits-leetcode-helper-extraction`.

Completed:

- Cleaned the pilot `helpers/list_node.sifr` so it contains only the canonical node shape plus accessors/serializer; moved `0021` sample builders and algorithm code back into the fixture.
- Added Python/Sifr helper mirrors for linked lists, trees, and DSU; added the Sifr-only trie helper under `audits/leetcode/helpers/`.
- Migrated all non-boundary `ListNode` and `TreeNode` fixtures to helper imports.
- Migrated Sifr trie fixtures `0208`, `0211`, and `0212` to `helpers.trie`.
- Moved DSU out of `lib/sifr/`, removed `sifr.dsu` registry/export coverage, and migrated DSU consumers to `helpers.dsu`.
- Deleted dead catch-all `Node` scaffolding while preserving genuine local `Node` shapes in `0133`, `0138`, `0146`, and `0622`.
- Kept `0141`, `0160`, and `0894` inline and documented the architecture-boundary reason.
- Fixed local helper import semantics in the compiler: project exports now retain class constructors, and project codegen preloads local helper call signatures plus recursive class field metadata.

Targeted validation completed before broad validation:

- Python: `0002`, `0021`, `0103`, `0208`, `0323`, `0721`, `1489`, `2709`.
- Sifr `check` + `run`: `0002`, `0021`, `0103`, `0208`, `0261`, `0323`.
- Rust regression: `cargo test -p sifr_driver test_build_project_preserves_imported_class_constructors_and_signatures -- --nocapture`.

Broad validation:

- `cargo fmt --check` PASS.
- `git diff --check` PASS.
- `cargo clippy --workspace -- -D warnings` PASS.
- `cargo test -p sifr_driver project_build_check:: -- --nocapture` PASS.
- `scripts/run_all_tests.sh --profile quick` PASS.
- `scripts/run_all_tests.sh` PASS.
- `python3 scripts/run_phase31_leetcode.py --manifest verification/leetcode/full_corpus_manifest_20260402_live.json --output verification/leetcode/full_corpus_current_results_20260425_helper_extraction.json --sifr-bin ./target/release/sifr --no-build-release-if-missing --timeout-seconds 30` PASS: 411 cases, 208 PASS, 203 NO_ORACLE, no CHECK_ERROR/RUN_ERROR/TIMEOUT.
- `python3 scripts/scan_leetcode_pair_diffs.py --output verification/leetcode/leetcode_pair_diff_scan_20260425_helper_extraction.json --top 80` PASS.

External review:

- `reviews/audits-leetcode-helper-extraction-2026-04-25-review-pass1.md` recorded no blockers.
- Followed up on the latent Python DSU helper contract concern: sparse dict-backed `union` now returns `False` when the two inputs are already connected, matching list-backed mode.
- Added notes documenting intentional Python/Sifr helper asymmetry, imported recursive-class codegen scope, and boundary-fixture inline class reasons.
- Post-review targeted validation: Python `0323`, `0721`, `1489`, `2709` PASS; Sifr `check` + `run` for `0141`, `0160`, `0894` PASS; `cargo fmt --check` and `git diff --check` PASS.
