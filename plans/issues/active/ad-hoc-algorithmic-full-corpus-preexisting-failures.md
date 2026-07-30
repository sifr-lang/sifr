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

Those figures describe the corpus's check-only lane. A complete native-build
audit on 2026-07-29 also found a disjoint set of 23 fixtures that pass
`sifr check` but fail `sifr build`, partitioned across three root causes:
21 owned optional-class destructure failures, one empty-dictionary
specialization failure, and one recursive optional-class constructor-coercion
failure. That latent set is recorded separately below and is included in the
remediation acceptance gate; it is not added to or substituted for the
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
root-cause groups keyed to each fixture's first blocking diagnostic; the table
below adds two further root causes found only by the native-build audit. Some
fixtures can expose follow-on diagnostics after their first blocker is fixed,
so this is not permission to stop after one diagnostic disappears.

| Root cause | Ownership boundary | Fixtures | Remediation |
| --- | --- | ---: | --- |
| Recursive `list[T]` total-order capability is omitted even though generated `Vec<T>` has lexicographic `Ord` when `T: Ord` | lowering type-capability query | 6 | admit `list[T]` recursively for `list.sort()` while continuing to reject non-total-order element types; keep sets and dictionaries excluded because their language semantics are not total orders, regardless of incidental generated representation |
| Empty list literals in equality comparisons retain `list[Any]`, including nested empty literals | comparison lowering and literal specialization | 6 | specialize literal HIR recursively from the concrete opposite operand before structural-equality checks; do not weaken the type-system capability gate |
| An empty plain-dictionary declaration retains an `Any` value despite a later concrete subscript write | order-independent declaration-site inference | 1 latent build failure | infer compatible plain-dictionary writes within the enclosing function before lowering the declaration; preserve ordinary missing-key access and augassign semantics |
| `defaultdict(int)` subscript augassign preserves an `Any` key in HIR | container specialization at the augassign target | 4 | specialize the alias key from the concrete subscript while preserving defaultdict codegen semantics |
| `defaultdict(set)` is read before its first textual write, so forward-only refinement cannot establish its key/value types | order-independent declaration-site inference | 1 | infer compatible defaultdict access shapes within the enclosing function before lowering the declaration; reject conflicting shapes deterministically |
| Consuming recursive linked-list traversal is declared as a mutable borrow, and generated owned optional-class destructures omit required Rust mutability | fixture ownership plus optional-class codegen | 2 check failures plus 21 latent build failures | use `own mut` in the two check-failing fixtures and fix the generated owned optional-class destructure in codegen so it emits a mutable binding; the shared `helpers/list_node` module and its two local copies need no source change, and all 23 affected fixtures must build and run, not merely check |
| Recursive optional-class locals passed to constructors are emitted as `Option<T>` instead of the required `Option<Box<T>>` | recursive-class constructor argument codegen | 1 latent build failure | apply the same recursive-field storage coercion used by direct recursive constructor arguments to typed optional-class locals, with focused positive and non-recursive negative coverage |
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
- Empty plain-dictionary declaration refinement: `0001_two_sum`.
- `defaultdict(int)` augassign specialization:
  `0350_intersection_of_two_arrays_ii`, `0621_task_scheduler`,
  `0767_reorganize_string`, and
  `1481_least_number_of_unique_integers_after_k_removals`.
- Order-independent `defaultdict(set)` inference: `0036_valid_sudoku`.
- Ownership and owned optional-class codegen: `0002_add_two_numbers` and
  `0086_partition_list` are the check-failing members; the 21 latent
  build-failing members are enumerated in the native-build audit below.
- Recursive optional-class constructor argument coercion:
  `0894_all_possible_full_binary_trees`.
- Dead invalid fixture surface: `0377_combination_sum_iv`.

The complete native-build audit checked every pinned fixture and then built
each of the 391 check-passing fixtures:

```bash
target/debug/sifr check --isolated <fixture>
target/debug/sifr build --quiet --isolated -o <unique-output-dir> <fixture>
```

It produced exactly 411 terminal records: 20 `CHECK_FAIL`, 23 `BUILD_FAIL`,
and 368 `BUILD_PASS`. The 20 check failures are set-identical to the preserved
corpus lane. The 23 distinct latent build failures are:

- Twenty linked-list fixtures that fail with generated Rust `E0596` because
  `node.next.take()` is emitted from an owned destructure whose binding is not
  mutable. Eighteen import the shared `helpers/list_node.nodeNext` helper (the
  other two importers are the preserved check failures `0002` and `0086`):
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
- The remaining two linked-list fixtures contain local copies of the same
  owned `nodeNext` helper:
  `0141_linked_list_cycle` and
  `0160_intersection_of_two_linked_lists`.
- `0617_merge_two_binary_trees`, which fails with the same generated Rust
  `E0596` mechanism for owned `TreeNode | None` parameters whose recursive
  fields are consumed after optional destructuring.
- `0001_two_sum`, whose empty `prevMap = {}` remains
  `dict[int, Any]` despite the later `prevMap[n] = i`; generated Rust then
  fails with `E0277`/`E0308` around `Box<dyn Any>`.
- `0894_all_possible_full_binary_trees`, whose typed
  `TreeNode | None` locals are passed to a recursive-class constructor as
  `Option<TreeNode>` instead of `Option<Box<TreeNode>>`, producing generated
  Rust `E0308`.

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
Passes 8 and 9 were interrupted before producing reviewable output, and pass
10 failed at the reviewer API certificate boundary before producing a report;
their zero-byte outputs were discarded and are not evidence. A final rebased
audit then found the incomplete native-build inventory in
[`review pass 12`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-12.md);
the complete 411-fixture native-build audit, expanded waves, and explicit
passes-8-to-10 disposition above are the responses to both requested changes.
[`Review pass 13`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-13.md)
independently verified the audit counts and technical partition, then requested
the group-count, membership-map, progress-state, and evidence-continuity
corrections now recorded here.
[`Review pass 14`](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-claude-opus-review-pass-14.md)
rechecked every pass-13 correction and the full sweep ledger, then approved the
corrected diagnosis with zero actionable findings.

## Focused Remediation Waves

Each wave is implemented, locally validated, reviewed to satisfaction, merged,
and recorded here before the next wave starts:

1. Recursive list total-order support and positive/negative compiler coverage.
2. Contextual empty-list equality specialization, including nested literals
   plus mismatched-literal and variable-operand negative coverage.
3. Order-independent empty plain-dictionary declaration refinement, including
   the `0001_two_sum` native build/run and deterministic conflicting-write
   coverage.
4. `defaultdict(int)` subscript-augassign specialization with runtime counting
   coverage.
5. Order-independent `defaultdict` declaration inference with the existing
   deterministic `TYPE_CONTAINER_ELEMENT_CONFLICT` diagnostic and the
   resulting `0036_valid_sudoku` pass, with no fixture-side change.
6. Removal of the dead invalid `0377` Sifr fixture block while deliberately
   leaving the Python reference sibling unchanged as the upstream parity
   source.
7. Owned linked-list traversal fixture corrections plus the generated optional
   recursive-class extraction fix and build/run coverage for all 23 affected
   fixtures: the 22 linked-list fixtures plus
   `0617_merge_two_binary_trees`.
8. Recursive optional-class constructor argument coercion with focused
   compiler coverage and the `0894_all_possible_full_binary_trees` native
   build/run.
9. Full-corpus closeout: capability-named demo, a complete 411-fixture native
   build/run audit, complete nightly lane,
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
`sorted(..., key=lambda ...)` can lower and then fail generated Rust name
resolution even for flat integer lists; `list[None]` literals can lower and
then emit a generated Rust option/unit mismatch even without sorting;
`min`/`max` list ordering has separate generated-Rust failure paths and remains
outside this issue's sort-specific Wave 1; `.values()` uses an over-broad
unknown-key capability guard; and
there is no standalone unreachable-code diagnostic. These findings are not
used as fallbacks or exclusions for this issue and do not broaden its focused
remediation waves. Wave 1 widens the element types that can reach the
pre-existing reverse-sort stability gap but does not change reverse sorting.
An enclosing concrete dictionary binding can also pollute nested-function
inference for a same-named empty dictionary declaration; current main already
emits invalid generated Rust for that shadowing shape, and Wave 3 deliberately
does not broaden nested-function inference semantics to absorb this adjacent
pre-existing problem.
Blocks containing a nested function also retain the pre-existing general
empty-collection hint path; assignable-but-unequal dictionary writes on that
path can still reach invalid generated Rust exactly as on current main. Wave
3's exact-write gate neither expands nor claims to repair that pre-existing
nested-function behavior.
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
| Failure diagnosis and root-cause grouping | approved; [PR #3064](https://github.com/sifr-lang/sifr/pull/3064) | exact current-main 411/20 check reproduction plus a complete 411-fixture native-build audit: 20 check failures, 23 distinct latent build failures, and 368 native-build passes; pass 14 approved the corrected diagnosis with zero actionable findings |
| Wave 1: recursive list total order | approved; [PR #3068](https://github.com/sifr-lang/sifr/pull/3068) | recursive `list[T]` generated-Rust `Ord` capability; focused nested positive/negative lowering tests including a list-returning `sorted` key; capability e2e; all six affected corpus fixtures build and run; [Opus pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-claude-opus-review-pass-1.md) requested the added `sorted` coverage, [Opus pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-claude-opus-review-pass-2.md) approved the completed wave, and [Opus pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-claude-opus-review-pass-3.md) approved the first rebased implementation; [pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-claude-opus-review-pass-4.md) returned an incomplete mid-sweep status and is not approval evidence, [pass 5](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-claude-opus-review-pass-5.md) approved the completed current-base implementation, and [pass 6](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-1-claude-opus-review-pass-6.md) approved the exact GitHub head/base prospective merge with zero actionable findings |
| Wave 2: contextual empty-list equality | merged; [PR #3074](https://github.com/sifr-lang/sifr/pull/3074) | recursive literal-only HIR specialization from the concrete opposite operand without changing the structural-equality gate; canonical recursive `Unknown`/`Any` query; exact-type preservation for unchanged concrete elements; explicit generated-Rust typing for concrete empty lists; focused empty-leading and trailing-empty nested positives in both operand positions, mismatched-literal and named-variable negatives; native capability e2e; all six affected corpus fixtures check, build, and run; `create-pr` profile passed with 131/131 selected native e2e fixtures and every blocking budget green; [Opus pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-claude-opus-review-pass-1.md) requested genuine nested left-side coverage, recursion into concrete outer literals, and the canonical type query; [Opus pass 2](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-claude-opus-review-pass-2.md) independently verified every correction and approved the complete wave with zero actionable findings; [Opus pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-claude-opus-review-pass-3.md) reviewed the exact pushed implementation head, independently ran the full 676/676 e2e suite and workspace checks, and approved with zero blocking findings; [Opus pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-2-claude-opus-review-pass-4.md) verified the documentation-only head delta, re-ran full e2e and workspace checks, and approved the complete PR with no blocking issues |
| Wave 3: empty plain-dictionary declaration refinement | merged; [PR #3077](https://github.com/sifr-lang/sifr/pull/3077) | function-level binding inference refines an empty plain-dictionary declaration from compatible later subscript writes only when the current lexical block has one unshadowed binding for that name and every collected write has the exact adopted key/value shape; any subscript augassign disqualifies the new adoption path so the existing missing-key behavior is not expanded; the existing empty-list/set/deque hint boundary is unchanged; the declaration and literal HIR receive the same concrete type, including eligible blocks containing nested functions; deferred container patches resolve only the nearest declaration; codegen keeps declaration-local types when same-named lexical bindings differ; incompatible and assignable-but-unequal writes on the new path preserve `SIFR-TYPE-0008`; focused lowering, codegen, and native capability coverage pins read-before-write inference, deterministic hard and widening conflicts, unhashable-key diagnostic cardinality, missing-key augassign-before-write rejection, no-evidence fallback, same-name sibling-scope isolation, nested-function-block HIR consistency, and loop-local/function-level isolation; all 898 lowering tests pass with one additional ignored test and all 934 codegen tests pass; Clippy, rustfmt, maintainability, file-size, and diff-hygiene checks pass; the capability e2e and `0001_two_sum` both check, build, and run; [Opus pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-1.md) found whole-block name-keyed adoption leaking across sibling bindings, and the declaration-safety gate, nearest-declaration patching, declaration-local codegen registry, and native regressions are the response; pass 2 timed out without reviewable output and its zero-byte artifact was discarded; [Opus pass 3](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-3.md) independently ran the full 677/677 native e2e suite and approved before its non-blocking cleanups were applied; [Opus pass 4](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-4.md) independently reran 677/677 e2e and found unified assignable-but-unequal writes could admit generated-Rust errors, and the exact-write-shape gate plus numeric, nominal-class, and unhashable-key boundaries are the response; pass 5 exceeded the 40-minute reviewer bound with zero output and its empty artifact was discarded; [Opus pass 6](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-6.md) independently verified the widening fix, then found subscript augassign was invisible to the gate and could expand the pre-existing wrong-result surface, and explicit augassign disqualification plus looped missing-key regression coverage are the response; pass 7 also exceeded the 40-minute bound with zero output and its empty artifact was discarded; [Opus pass 8](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-8.md) independently verified the augassign correction in direct, loop, nested control-flow, and try forms and approved the complete wave with zero actionable findings; the authoritative create-PR profile then passed every blocking lane and all 131 selected native e2e fixtures after isolated reruns confirmed two host-sensitive first-attempt timeouts in the readonly Python doctor and LSP shutdown smoke; implementation commit `1ad7389dd` is the first published head |
| Wave 4: `defaultdict(int)` augassign key specialization | review; [PR #3079](https://github.com/sifr-lang/sifr/pull/3079) | the first concrete subscript-augassign key refines only the unkeyed `defaultdict(int)` alias, widens literal keys to mutable base types, patches both declaration and constructor-call HIR, and preserves the alias-backed `entry(...).or_insert(0)` codegen path; initialized aliases keep their declared key type and conflicting later keys remain rejected; deferred patches now inspect only direct declarations in their lexical block, verify that the declaration expression matches the requested specialization before changing its type, and leave same-named nested `defaultdict` and scalar bindings independent; focused lowering is 9/9, focused codegen is 4/4, full lowering is 907 passed with one ignored, full codegen is 938/938, affected Clippy/rustfmt/maintainability/file-size checks pass, the expanded capability e2e passes, all four affected corpus fixtures check/build/run, and the complete native e2e suite passes 678/678; [Opus pass 1](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-4-claude-opus-review-pass-1.md) found the original name-keyed recursive patch could retarget a nested shadow and requested the added shadowing, alias-boundary, unhashable-key, and plain-dictionary negative coverage, all of which are addressed in the current implementation |
| Waves 5-8 | pending | start sequentially after Wave 4 merges |
| Full-corpus closeout | blocked | starts after every remediation wave merges; includes restoring `leetcode-full` to the release profile |

Wave 3's exact prospective merge `ec5aab945` includes current `main`
`ea119724e`; the authoritative create-PR profile passed again on that exact
state with all 131 selected native e2e fixtures. Reviewer pass 10 returned
only a transient HTTP 529 overload response and is not review evidence.
[Opus pass 11](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-11.md)
then independently compared the published head and base, ran the complete
677-fixture e2e suite, compared all 411 corpus checks, ran all 58 corpus
fixtures containing an empty dictionary through the native path, and approved
the complete Wave 3 implementation with zero actionable findings.
[Opus pass 12](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-12.md)
verified the documentation-only published-head delta and requested only that
the pass-11 report's self-label be corrected from pass 12 to pass 11; the
report heading now matches its filename and ledger link.
The branch then incorporated current `main` `ca7731aa8` without file overlap;
the authoritative create-PR profile passed on exact prospective merge
`7dbe8bd36` with every blocking budget green and 131/131 selected e2e
fixtures. [Opus pass 14](../../reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-claude-opus-review-pass-14.md)
approved that exact head/base pair with zero actionable findings. Wave 3
subsequently merged in [PR #3077](https://github.com/sifr-lang/sifr/pull/3077)
at merge commit `789b359737`; this merge record supersedes the earlier
pre-merge table status.

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
      all 23 corpus fixtures exercising the affected owned optional-class
      extraction pattern build and run after the ownership/codegen fix, and
      `0894_all_possible_full_binary_trees` builds and runs after recursive
      constructor coercion is corrected.
- [ ] All 411 pinned corpus fixtures pass a complete native build/run audit at
      closeout; the check-only corpus lane is not sufficient evidence for this
      criterion.
- [ ] Every associated demo uses a capability-based name containing no phase
  number or phase name.
- [ ] The authoritative create-PR and merge profiles, Clippy, rustfmt,
  maintainability, file-size, and diff-hygiene gates pass locally.
- [ ] Review rounds are satisfied and all remediation PRs are merged.
