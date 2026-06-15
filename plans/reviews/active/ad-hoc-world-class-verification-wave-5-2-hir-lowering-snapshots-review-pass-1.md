I reviewed the Wave 5.2 changeset against the tracker scope, inventory checker, HIR projection, and merge-gate plumbing. Findings below ordered by severity. No file modifications were made.

## Findings

### Blocking
None. The HIR snapshot suite is real executable evidence (`cargo test -p sifr_lowering ... matrix_matches_lowered_module_shape`) that runs in both `create-pr` and `merge` profiles via `verification/profiles/{create-pr,merge}.json:43` where `sifr_lowering` is `blocking` and `executed_in_merge: true`. The inventory rows are honestly scoped (2 active HIR rows), and the tracker entry does not overclaim Wave 5.2 completion.

### Non-blocking (worth a follow-up before broader HIR rows are added)

1. **Silently dropped HIR fields in the projection.** Several stable HIR semantics are eaten by `..` patterns or skipped fields, so two distinct lowerings could produce the same snapshot:
   - `crates/sifr_lowering/src/hir_snapshot_tests.rs:91-93` — `HirStmt::Let { .. }` drops `is_mutable`.
   - `crates/sifr_lowering/src/hir_snapshot_tests.rs:67-83` — `HirFunction` projection ignores `is_async`, `method_kind`, `decorators`, `type_params`.
   - `crates/sifr_lowering/src/hir_snapshot_tests.rs:73-79`, `crates/sifr_lowering/src/hir_snapshot_tests.rs:594-606` — `HirParam` projection drops `convention` (ownership!), `default`, `keyword_only`.
   - `crates/sifr_lowering/src/hir_snapshot_tests.rs:182-194` — `HirTupleTarget.rebind_existing` is dropped.
   None of these affect the two current fixtures, but ownership convention and `is_async` are exactly the Sifr-distinctive bits HIR snapshots are supposed to lock down. Recommend either projecting them, or adding a comment listing the deliberately omitted fields per the `hir-kind-only` normalizer contract.

2. **`project_module` only walks `module.functions`** (`crates/sifr_lowering/src/hir_snapshot_tests.rs:57-65`). It silently drops `classes`, `imports`, `constants`, `generic_functions`, and `type_param_bounds`. Fine for the current scope, but a future inventory row whose source emits any of those would yield a snapshot that says "no top-level interest" without failing — i.e., a fixture-shape mismatch the test can't catch. Worth widening the projection before the inventory adds a class- or import-bearing row. (The two `HashMap` fields would also need deterministic ordering once projected.)

3. **No matrix↔inventory cross-check.** `lowering_layer_inventory.py:158-187` walks inventory→matrix and fails if a referenced fragment is missing, but nothing fails when a fixture is added to `hir_lowering_snapshot_matrix.json` without a matching inventory row. The Rust test still asserts the new fixture, but coverage-matrix accounting goes silently out of sync. A simple counter check (matrix HIR rows == inventory `hir_lowering` rows) would close this.

4. **No automated stale-fixture detection** for the matrix itself: removing a fixture but leaving the inventory row would fail on missing fragment (good); leaving a fixture but removing the inventory row would not. Same family as (3); single guardrail closes both.

5. **File-size headroom.** `crates/sifr_lowering/src/hir_snapshot_tests.rs` is 817/900 (90.7%) before Waves 5.3–5.5 add their projections. The projection helpers (`project_expr`, `project_stmt`) dominate the size and are the natural growth axis. Splitting `project_expr` / pattern helpers into a sibling module before the next wave keeps the guardrail comfortable. Not blocking today.

6. **Diff quality.** `assert_eq!(&actual, expected, ...)` (`crates/sifr_lowering/src/hir_snapshot_tests.rs:34`) prints `serde_json::Value` Debug output on mismatch. Readable, but inferior to `insta` JSON snapshot diffs the rest of the project uses and the wave description ("snapshot output is reviewable") implies. A pretty-printed diff (e.g., `serde_json::to_string_pretty` on both sides before assert) would meaningfully improve regression triage; converting to `insta` is the bigger lift.

7. **Inventory checker enum coverage.** `verification/areas/core_language/checks/lowering_layer_inventory.py:22-25` validates `snapshot_kind` only via the `EXPECTED_FIELD_BY_SNAPSHOT_KIND` lookup. That works, but adding an explicit `ALLOWED_SNAPSHOT_KINDS` set (mirroring `ALLOWED_LAYERS`) would put parsed-source/HIR/future-layer kinds under the same enum-style validator and make Waves 5.3+ extensions one-line additions. Minor.

## Verdict

No blocking findings; the wave is ready for PR/merge as scoped. Tracker accurately reflects in-progress status, the snapshot is exact-match executable evidence wired into merge through `sifr_lowering`, the inventory checker correctly differentiates `statement_tree` vs `hir_module_shape` rows, and the file-size guardrail is satisfied today.

A second review round is not required before merging Wave 5.2. The follow-ups above (especially items 1 and 2 — projection field coverage and module-shape coverage) should be resolved before the inventory expands beyond the two current arithmetic/control-flow fixtures, since the `hir-kind-only` projection currently can't distinguish ownership conventions, async-ness, or non-function module contents.
