# Ad Hoc Issue: Algorithmic Full-Corpus Pre-Existing Failures

## Status

Active non-blocking follow-up created from the Rust-interop
`certification_0` validation on 2026-07-26. The failures predate that
milestone, are outside Rust interop and stable-release-governance scope, and do
not block `certification_0`, Phase 40, or stable-channel Rust-interop
certification.
The durable issue was established in
[PR #3029](https://github.com/sifr-lang/sifr/pull/3029).

No failure is suppressed or reclassified by the Rust-interop work. Phase 40
keeps the complete pinned corpus blocking in nightly and uses the already
blocking representative subset plus the taxonomy self-test for canonical
release qualification. Remediation belongs in separate focused PRs owned by
this issue. Restoring the full corpus to release qualification is part of this
issue's closeout, not a Phase 40 prerequisite.
The release-profile divergence expires on 2026-10-31. If remediation is not
complete by then, readiness fails closed until a separately reviewed decision
either restores the full corpus or renews the deadline with current evidence.

## Preserved Evidence

The exact-state nightly and release profiles both passed their complete
Rust-interop steps and then independently reported the same 20 blocking
failures in the 412-variant algorithmic-compatibility lane:

- `target/validation_lane_reports/nightly.latest.json` and its adjacent log:
  Rust interop passed in 4,161 ms; the algorithmic lane reported 20 failures.
- `target/validation_lane_reports/release.latest.json` and its adjacent log:
  Rust interop passed in 3,880 ms; the algorithmic lane reported the same 20
  failures.
- `target/verification/areas/algorithmic_compatibility/leetcode-full-taxonomy.json`
  records 20 failures among 411 corpus fixtures: 15
  `other_type_surface_and_api_mismatch`, 4
  `any_unknown_typing_and_container_specialization_gap`, and 1
  `signature_invalid_fixture_surface`.

The taxonomy artifact was generated on 2026-06-16 and contains 411 fixture
records. The later profile lanes report 412 area variants because their count
also includes the area-level `full-corpus-taxonomy-smoke` policy/runner
variant. The 20 failing fixture slugs are set-identical across all three
evidence sources.

Those figures describe the corpus's check-only lane. Diagnosis on 2026-07-29
also found a disjoint set of 20 fixtures that pass `sifr check` but fail
`sifr build`/`sifr run` through the same owned recursive-field extraction
codegen defect. That latent set is recorded separately below and is included
in the remediation acceptance gate; it is not added to or substituted for the
preserved 20 blocking check failures.

The failing fixture slugs are:

- `0002_add_two_numbers`
- `0036_valid_sudoku`
- `0056_merge_intervals`
- `0086_partition_list`
- `0094_binary_tree_inorder_traversal`
- `0144_binary_tree_preorder_traversal`
- `0145_binary_tree_postorder_traversal`
- `0252_meeting_rooms`
- `0350_intersection_of_two_arrays_ii`
- `0377_combination_sum_iv`
- `0435_non_overlapping_intervals`
- `0442_find_all_duplicates_in_an_array`
- `0452_minimum_number_of_arrows_to_burst_balloons`
- `0621_task_scheduler`
- `0767_reorganize_string`
- `1203_sort_items_by_groups_respecting_dependencies`
- `1383_maximum_performance_of_a_team`
- `1481_least_number_of_unique_integers_after_k_removals`
- `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`
- `2402_meeting_rooms_iii`

Phase 40 reproduced the set exactly on source
`c17f3c7d1ea1ed97ca125eb7a43344b30cf9413b`. The canonical release attempt
passed coverage, core guardrails, diagnostics, CPython differential, all 25
Python-interop variants, Rust interop, frontend syntax guardrails, all 48
developer-tooling variants, documentation, and `performance_budget_checks` in
`full` mode. It then reported the same 20 failures among 412 algorithmic area
variants. This independent reproduction is the evidence for separating the
nightly full-corpus remediation signal from stable-channel release
qualification.

Representative diagnostics include unknown `Any` hash/equality capability,
mutable-borrow representation changes, unavailable generated total `Ord`, and
structural-equality mismatches between concrete and `Any` containers. The
taxonomy artifact and profile logs remain the detailed ephemeral evidence;
this issue is the durable repository record.

## Current-Main Reproduction and Diagnosis

The complete corpus was reproduced locally on 2026-07-29 from
`649334330ce4f9c682b5aa8453ddad6ada737d40` with:

```bash
uv run --project verification --locked python -m sifr_verify areas run \
  --area algorithmic_compatibility --suite leetcode-full
```

The run checked all 411 pinned fixtures and reproduced exactly the preserved
20 blocking failures. Direct checks of all 20 fixtures established six
root-cause groups keyed to each fixture's first blocking diagnostic. Some
fixtures can expose follow-on diagnostics after their first blocker is fixed,
so this is not permission to stop after one diagnostic disappears.

| Root cause | Ownership boundary | Fixtures | Remediation |
| --- | --- | ---: | --- |
| Recursive `list[T]` total-order capability is omitted even though generated `Vec<T>` has lexicographic `Ord` when `T: Ord` | lowering type-capability query | 6 | admit `list[T]` recursively for `list.sort()` while continuing to reject non-total-order element types; keep sets and dictionaries excluded because their language semantics are not total orders, regardless of incidental generated representation |
| Empty list literals in equality comparisons retain `list[Any]`, including nested empty literals | comparison lowering and literal specialization | 6 | specialize literal HIR recursively from the concrete opposite operand before structural-equality checks; do not weaken the type-system capability gate |
| `defaultdict(int)` subscript augassign preserves an `Any` key in HIR | container specialization at the augassign target | 4 | specialize the alias key from the concrete subscript while preserving defaultdict codegen semantics |
| `defaultdict(set)` is read before its first textual write, so forward-only refinement cannot establish its key/value types | order-independent declaration-site inference | 1 | infer compatible defaultdict access shapes within the enclosing function before lowering the declaration; reject conflicting shapes deterministically |
| Consuming recursive linked-list traversal is declared as a mutable borrow, and generated owned optional-class destructures omit required Rust mutability | fixture ownership plus optional-class codegen | 2 check failures plus 20 latent build failures | use `own mut` in the two check-failing fixtures and fix the generated owned optional-class destructure in codegen so it emits a mutable binding; the shared `helpers/list_node` module and its two local copies need no source change, and all 22 affected fixtures must build and run, not merely check |
| Unreachable, unannotated nested `dfs` remains after the live iterative solution returns | fixture surface | 1 | remove the dead Sifr-only porting residue; continue type-checking reachable and unreachable declarations normally |

The fixture membership of those groups is:

- Recursive list total order: `0056_merge_intervals`,
  `0252_meeting_rooms`, `0435_non_overlapping_intervals`,
  `0452_minimum_number_of_arrows_to_burst_balloons`,
  `1383_maximum_performance_of_a_team`, and
  `2402_meeting_rooms_iii`.
- Contextual empty-list equality: `0094_binary_tree_inorder_traversal`,
  `0144_binary_tree_preorder_traversal`,
  `0145_binary_tree_postorder_traversal`,
  `0442_find_all_duplicates_in_an_array`,
  `1203_sort_items_by_groups_respecting_dependencies`, and
  `1489_find_critical_and_pseudo_critical_edges_in_minimum_spanning_tree`.
- `defaultdict(int)` augassign specialization:
  `0350_intersection_of_two_arrays_ii`, `0621_task_scheduler`,
  `0767_reorganize_string`, and
  `1481_least_number_of_unique_integers_after_k_removals`.
- Order-independent `defaultdict(set)` inference: `0036_valid_sudoku`.
- Ownership and owned optional-class codegen: `0002_add_two_numbers` and
  `0086_partition_list`.
- Dead invalid fixture surface: `0377_combination_sum_iv`.

The distinct latent build-failure set contains 20 fixtures that currently pass
the corpus check lane but fail build/run with the generated Rust error
`E0596` because `node.next.take()` is emitted from an owned destructure whose
binding is not mutable:

- Eighteen of the 20 shared `helpers/list_node.nodeNext` importers (the other
  two are the preserved check failures `0002` and `0086`):
  `0019_remove_nth_node_from_end_of_list`,
  `0021_merge_two_sorted_lists`, `0023_merge_k_sorted_lists`,
  `0024_swap_nodes_in_pairs`, `0025_reverse_nodes_in_k_group`,
  `0061_rotate_list`, `0083_remove_duplicates_from_sorted_list`,
  `0092_reverse_linked_list_ii`, `0143_reorder_list`,
  `0147_insertion_sort_list`, `0148_sort_list`,
  `0203_remove_linked_list_elements`, `0206_reverse_linked_list`,
  `0234_palindrome_linked_list`, `0876_middle_of_the_linked_list`,
  `1669_merge_in_between_linked_lists`,
  `1721_swapping_nodes_in_a_linked_list`, and
  `2130_maximum_twin_sum_of_a_linked_list`.
- Local copies of the same owned `nodeNext` helper:
  `0141_linked_list_cycle` and
  `0160_intersection_of_two_linked_lists`.

The original issue record was reviewed to satisfaction in passes 1-3. Claude
Opus then independently reproduced the current 20 failures and conditionally
approved this diagnosis with the mechanism corrections and build/run
requirement recorded above. The diagnosis reviews are preserved in
[`review pass 4`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-4.md),
[`review pass 5`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-5.md),
[`review pass 6`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-6.md),
and
[`review pass 7`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-7.md).
After the pass-7 findings were addressed, the complete diagnosis was approved
with zero actionable findings in
[`review pass 11`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-11.md).

## Focused Remediation Waves

Each wave is implemented, locally validated, reviewed to satisfaction, merged,
and recorded here before the next wave starts:

1. Recursive list total-order support and positive/negative compiler coverage.
2. Contextual empty-list equality specialization, including nested literals
   plus mismatched-literal and variable-operand negative coverage.
3. `defaultdict(int)` subscript-augassign specialization with runtime counting
   coverage.
4. Order-independent `defaultdict` declaration inference with the existing
   deterministic `TYPE_CONTAINER_ELEMENT_CONFLICT` diagnostic and the
   resulting `0036_valid_sudoku` pass, with no fixture-side change.
5. Removal of the dead invalid `0377` Sifr fixture block while deliberately
   leaving the Python reference sibling unchanged as the upstream parity
   source.
6. Owned linked-list traversal fixture corrections plus the generated optional
   recursive-class extraction fix and build/run coverage for all 22 affected
   fixtures: the 20 shared-helper importers plus the two local helper copies.
7. Full-corpus closeout: capability-named demo, complete nightly lane,
   restoration of `leetcode-full` to release qualification, complete release
   lane, final local merge gate, and full-implementation review.

The corpus runner is intentionally check-oriented. Remediation waves must also
build and run focused e2e coverage for corrected runtime surfaces so a green
corpus cannot hide generated-Rust failures. Each wave must check the complete
affected fixtures after the change, not merely confirm that the targeted first
diagnostic disappeared. Wave 1 and wave 2 tests belong in new focused modules
to preserve the 900-line source limit, and wave 2 must specialize literal HIR
in lowering without relaxing or editing the structural-equality gate in
`sifr_type_system/src/check.rs`. No wave may use a plain `dict`
annotation to erase a `defaultdict` alias: that changes missing-key augassign
codegen and can silently produce incorrect counts.

## Separately Tracked Findings

Diagnosis exposed pre-existing behavior outside the preserved 20-fixture set:
plain `dict` missing-key augassign can silently no-op instead of preserving the
approved error behavior; reverse sorting has an equal-element stability gap;
`min`/`max` list ordering remains intentionally narrower than `list.sort()` for
this issue; `.values()` uses an over-broad unknown-key capability guard; and
there is no standalone unreachable-code diagnostic. These findings are not
used as fallbacks or exclusions for this issue and do not broaden its focused
remediation waves. Wave 1 widens the element types that can reach the
pre-existing reverse-sort stability gap but does not change reverse sorting.
The missing-key wrong-result behavior is preserved in a separate correctness
issue,
[`ad-hoc-dict-missing-key-augassign-semantics.md`](./ad-hoc-dict-missing-key-augassign-semantics.md),
rather than worked around here.

## Scope

- Diagnose each failure against the current language contract.
- Group fixes by compiler concern and ownership boundary.
- Implement root-cause compiler or fixture corrections in focused PRs.
- Preserve the full-corpus gate as blocking in nightly; do not add baselines,
  exclusions, fallback behavior, or area-specific exceptions.
- Keep the release profile's representative subset and taxonomy self-test
  blocking until this issue closes.
- Name any associated demo after the capability it demonstrates. Demo names
  must not contain a phase number or phase name.
- This user-directed naming rule supersedes the project-workflow skill's
  generic `<milestone>_demo` example for every demo owned by this issue.
- Do not modify Rust-interop matrices, stable claims, crate pins, or profile
  registration unless a separately reviewed cross-area requirement is proven.

## Implementation Progress

| Item | Status | Evidence |
| --- | --- | --- |
| Failure diagnosis and root-cause grouping | satisfied; pending merge | exact current-main 411/20 check reproduction plus the distinct 20-fixture latent build-failure set; six first-diagnostic groups; Opus pass 11 approved the complete diagnosis with zero actionable findings after independently reproducing every count and inventory |
| Focused remediation PR waves | ready | seven sequential waves are defined above; starts after the satisfied diagnosis is merged |
| Full-corpus closeout | blocked | starts after every remediation wave merges; includes restoring `leetcode-full` to the release profile |

## Acceptance Criteria

- [ ] Every listed fixture passes the canonical full-corpus algorithmic suite.
- [ ] The current canonical corpus count passes without a baseline,
  suppression, exclusion, or non-blocking reclassification.
- [ ] The nightly profile passes the complete algorithmic-compatibility lane
  locally.
- [ ] After the corpus is green, `leetcode-full` is restored to the release
  profile and the release lane passes locally.
- [ ] Focused compiler tests cover each corrected root-cause category.
- [ ] Focused e2e tests build and run every corrected generated-Rust surface;
      all 22 corpus fixtures exercising the affected owned recursive-field
      extraction pattern build and run after the ownership/codegen fix.
- [ ] Every associated demo uses a capability-based name containing no phase
  number or phase name.
- [ ] The authoritative create-PR and merge profiles, Clippy, rustfmt,
  maintainability, file-size, and diff-hygiene gates pass locally.
- [ ] Review rounds are satisfied and all remediation PRs are merged.
