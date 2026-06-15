## Findings — Pass 2

### Blocking
None.

### On the new unclaimed-matrix-fixture guard

The follow-up at `verification/areas/core_language/checks/lowering_layer_inventory.py:213-243` (`validate_matrix_fixtures_are_claimed`) is correct and closes both pass-1 items 3 and 4. Specifics:

- The `claimed` set is built from inventory row `source_fixture` values; the matrix-side `fixture_ref` is built as `f"{matrix_path.relative_to(REPO_ROOT)}#{collection}/{fixture_id}"`. This exactly matches the canonical form already used in `data/lowering_layer_inventory.json:9,30,51,72`. Verified by inspection against the four current inventory rows.
- It iterates the same `MATRIX_COLLECTIONS` list used by the existing inventory→matrix check, so it uniformly covers both `syntax_parser_lexer_matrix.shape_snapshots` and `hir_lowering_snapshot_matrix.hir_snapshots`. Wave 5.1's two shape fixtures are both claimed; Wave 5.2's two HIR fixtures are both claimed.
- Skips matrix rows lacking the expected_field (no spurious failure on non-snapshot fixtures); requires `id` to be a non-empty string before comparing (degrades cleanly on malformed input); reports a clean error if the matrix file is unreadable.
- The combined effect: **adding** a matrix fixture without an inventory row → fails here; **removing** a matrix fixture while leaving an inventory row → still fails via `validate_source_fixture`. Both directions are now enforced.

### Non-blocking (optional, do not block merge)

1. **Path-canonicalization implicit assumption.** Equality between inventory `source_fixture` and `f"{matrix_path.relative_to(REPO_ROOT)}#..."` relies on both sides using identical relative path strings. POSIX-only repo, so safe today; defensive option is `matrix_path.relative_to(REPO_ROOT).as_posix()`. Mention only — not worth a follow-up PR.

2. **`MATRIX_COLLECTIONS` and `EXPECTED_FIELD_BY_SNAPSHOT_KIND` carry overlapping knowledge.** A future snapshot kind has to be added in both places. A small assertion that `{c[2] for c in MATRIX_COLLECTIONS} <= set(EXPECTED_FIELD_BY_SNAPSHOT_KIND.values())` could catch divergence. Trivial; defer to Wave 5.3+ when a third layer lands.

3. **Pass-1 item 1 (projection field omissions) intentionally deferred.** Reading `crates/sifr_lowering/src/hir_snapshot_tests.rs:67-83,91-93,594-606` confirms `is_async`, `method_kind`, `decorators`, `type_params`, `is_mutable`, `convention`, `default`, `keyword_only`, and `HirTupleTarget.rebind_existing` are still dropped. Neither fixture exercises any of these, so the assertion `assert_eq!(&actual, expected)` is still meaningful evidence for what the current inventory rows claim. File size is 817/900 (90.7%); deferring projection expansion until projection helpers are split out is the correct ordering. Pass-1 item 2 (`project_module` only walks `module.functions`) is similarly still open and similarly safe today.

4. **File-size headroom unchanged.** No new lines in the Rust test; the inventory checker grew from ~187 to 247 lines — still well under any cap and well-decomposed.

### Verdict

The pass-1 follow-up is correctly scoped and correctly implemented. The new guard validates the matrix↔inventory pairing in both directions and works uniformly across the parsed-source and HIR-lowering matrix collections. Deferring the HIR projection-field expansion is defensible given file-size headroom and the current two-fixture scope — those gaps remain open as worth-doing-before-broader-rows items, but neither weakens the evidence for the two fixtures shipped here.

**No blocking concerns remain for this Wave 5.2 slice. PR/merge is ready as scoped. Another review round is not required.**
