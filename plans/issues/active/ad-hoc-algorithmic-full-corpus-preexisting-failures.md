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
| Failure diagnosis and root-cause grouping | pending | all 20 failures remain preserved above |
| Focused remediation PR waves | blocked | starts after diagnosis groups establish reviewable compiler ownership boundaries |
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
- [ ] Every associated demo uses a capability-based name containing no phase
  number or phase name.
- [ ] The authoritative create-PR and merge profiles, Clippy, rustfmt,
  maintainability, file-size, and diff-hygiene gates pass locally.
- [ ] Review rounds are satisfied and all remediation PRs are merged.
