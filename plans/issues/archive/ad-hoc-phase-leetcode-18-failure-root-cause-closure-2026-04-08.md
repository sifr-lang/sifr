# Ad-hoc Phase: LeetCode 18-Failure Root-Cause Closure (2026-04-08)

Status: done
Owner: phase_ad_hoc_leetcode_18_failure_root_cause_closure
Source run artifact: `verification/leetcode/full_corpus_current_results_20260408_live_rerun3.json`
Source taxonomy artifact: `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3.json`

## Closure Snapshot

Final corpus summary from closure artifacts:

- total cases: `411`
- failing cases: `0`
- pass: `208`
- no oracle: `203`

Closure artifacts:

- `verification/leetcode/ad_hoc_phase_leetcode18_targeted_after_all_fixes.json`
- `verification/leetcode/full_corpus_current_results_20260408_live_rerun3.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3.json`
- `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3.md`
- `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3_delta_vs_rerun1.md`

Merged PR ledger:

- `WS1_codegen_soundness_run_stage`: `https://github.com/sifr-lang/sifr/pull/1605`
- `WS3_canonical_adaptation_sweep`: `https://github.com/sifr-lang/sifr/pull/1606`
- `WS2_field_surface_and_optional_flow_core` + `WS4_closure_and_reclassification`: `https://github.com/sifr-lang/sifr/pull/1607`

## Scope Snapshot

Corpus summary from the live rerun artifacts:

- total cases: `411`
- failing cases: `18`
- categories: `7`

Category counts:

- `codegen_runtime_build_gap`: `6`
- `ownership_and_mutability_boundary`: `4`
- `nonlocal_mutable_capture_not_supported`: `2`
- `recursive_node_and_field_expression_surface`: `2`
- `signature_invalid_fixture_surface`: `2`
- `optional_none_flow_and_narrowing_gap`: `1`
- `other_type_surface_and_api_mismatch`: `1`

Resolution-lane split for these 18 failures:

- `compiler`: `7`
- `adaptation`: `5`
- `both`: `6`

Category-to-lane crosswalk (to avoid taxonomy/lane confusion):

| Taxonomy category | Fixture count | Lane impact |
|---|---:|---|
| `codegen_runtime_build_gap` | 6 | `compiler` + `both` |
| `ownership_and_mutability_boundary` | 4 | `adaptation` + `both` |
| `nonlocal_mutable_capture_not_supported` | 2 | `adaptation` |
| `recursive_node_and_field_expression_surface` | 2 | `both` |
| `signature_invalid_fixture_surface` | 2 | `compiler` + `adaptation` |
| `optional_none_flow_and_narrowing_gap` | 1 | `both` |
| `other_type_surface_and_api_mismatch` | 1 | `compiler` |

## Architecture Lock (Explicit)

`nonlocal` mutable capture is intentionally unsupported and remains unsupported in this phase.

- reference: `internal_docs/architecture.md` (`Python Divergences` table, `global` / `nonlocal` row)
- policy decision: keep explicit data flow (arguments/returns/state objects), no hidden mutable closure capture
- implication: no compiler feature work to add nonlocal mutation; affected fixtures must be adapted

## Fixture-Level Breakdown And RCA

| Fixture | Stage | Primary failure | Lane | Root cause summary |
|---|---|---|---|---|
| `0049_group_anagrams` | run | assertion panic | compiler | Dict value mutation lowered through `groups.get(...).cloned().push(...)`, mutating a clone instead of map entry; semantic mis-lowering. |
| `0144_binary_tree_preorder_traversal` | run | Rust type mismatch | compiler | Option/reference normalization in `while cur or stack` lowering produces inconsistent `cur` types (`&Option<TreeNode>` vs `Option<TreeNode>`). |
| `0145_binary_tree_postorder_traversal` | run | Rust type/field errors | compiler | Stack/pop lowering wraps Option layers incorrectly (`Option<Option<TreeNode>>`) and then applies field access to wrong level. |
| `0286_walls_and_gates` | run | Rust type mismatch/E0282 | compiler | Nested list indexing helper emits `Option<Vec<int>>` into `Vec<int>` typed locals and generates under-constrained closures in `and_then` chains. |
| `0705_design_hashset` | run | Rust Any/Clone/eq failures | both | Empty class field init (`self.hashset = []`) stays as `list[Any]` and lowers to `Vec<Box<dyn Any>>`; compile-safety hole plus fixture missing explicit field type. |
| `0973_k_closest_points_to_origin` | run | Rust type mismatch/E0282 | compiler | Same nested index normalization defect as `0286`; warning about possible `i64` overflow is non-blocking and policy-consistent. |
| `1137_n_th_tribonacci_number` | run | missing binding (`Memo`) | compiler | Module-global dict writes are emitted to unresolved symbol (`Memo`) while reads use synthesized `__const_Memo()`: global binding emission inconsistency. |
| `0543_diameter_of_binary_tree` | check | nonlocal mutable capture | adaptation | Intentional divergence: recursive nested function mutates captured state via `nonlocal`; must be rewritten to explicit return/state passing. |
| `0673_number_of_longest_increasing_subsequence` | check | nonlocal mutable capture | adaptation | Same intentional divergence; additionally contains optional-index arithmetic that should be removed via canonical rewrite. |
| `0721_accounts_merge` | check | `int | None` index usage | both | Optional/index narrowing gap in union-find/dict flow plus fixture relies on permissive Pythonic shape propagation without explicit guards/typing. |
| `0018_4sum` | check | immutable param + Optional arithmetic | both | Parameter mutability contract violated in nested helper; list indexing safety (`Option`) not narrowed in arithmetic-heavy loop. |
| `0056_merge_intervals` | check | immutable param + Optional flow | both | In-place mutation requires explicit `mut`; optional element flow leaks through interval indexing and append/result typing. |
| `0402_remove_k_digits` | check | immutable param + truthiness/or shortcut | adaptation | Uses int truthiness and `a or b` value shortcut; Sifr requires explicit bool conditions and typed fallback expressions. |
| `0442_find_all_duplicates_in_an_array` | check | immutable param mutation | adaptation | Canonical mutability contract breach only; add `mut` parameter for in-place sign marking algorithm. |
| `0230_kth_smallest_element_in_a_bst` | check | field access expression unsupported | both | Tree field reads (`.left/.right/.val`) require compiler surface parity; fixture also needs explicit `mut k` and total return path. |
| `0707_design_linked_list` | check | field access + invalid fixture surface | both | Field-expression surface gap plus fixture-local typing holes (missing annotations, undefined names) requiring adaptation cleanup. |
| `1849_splitting_a_string_into_descending_consecutive_values` | check | `Result[int, ParseError]` misuse | adaptation | `int(str)` intentionally returns `Result`; fixture assumes exception-style direct `int`. Must handle parse result explicitly. |
| `1930_unique_length_3_palindromic_subsequences` | check | `str.rfind` missing | compiler | Missing stdlib/string API parity (`rfind`) causes cascade (`first` undefined after failed destructuring assignment). |

### Mixed-Lane Ownership Split (`both`)

| Fixture | Adaptation-owned portion | Compiler-owned portion |
|---|---|---|
| `0018_4sum` | explicit `mut` on rebinding parameters and canonical Optional handling style | Optional/index narrowing precision in bounded loops |
| `0056_merge_intervals` | explicit `mut` and canonical result construction | Optional/index narrowing and list element flow precision |
| `0230_kth_smallest_element_in_a_bst` | explicit `mut k` and total return path completion | field expression parity for `.left/.right/.val` |
| `0705_design_hashset` | explicit typed field declaration for container shape | class-field empty-list inference and Any-lowering soundness |
| `0707_design_linked_list` | fixture-local typing cleanup (annotations/locals) | field expression parity on node links |
| `0721_accounts_merge` | explicit canonical typing/guards for map/index flows | Optional/index narrowing through dict/union-find flow |

## Category-Level Root Causes

### C1 `codegen_runtime_build_gap` (6 fixtures)

Compiler-owned run-stage soundness defects:

- mutable dict value update via borrowed lookup is lowered incorrectly (`0049`)
- Option/reference normalization across loop states is inconsistent (`0144`, `0145`)
- nested index normalization emits wrong target types and inference-hostile closures (`0286`)
- class-field empty-container inference drifts into unsound `Any` runtime lowering (`0705`)
- module-global mutable binding writes are not emitted consistently (`1137`)

Language adjustment decision:

- no language-rule change required
- this is compiler correctness and codegen consistency work

### C2 `ownership_and_mutability_boundary` (4 fixtures)

Mixed ownership and Optional-flow issues:

- strict mutable-parameter contract is intentional and should stay (`0018`, `0056`, `0402`, `0442`)
- Optional index flow in arithmetic-heavy loops still needs better narrowing (`0018`, `0056`)

Language adjustment decision:

- keep explicit `mut` policy unchanged
- improve compiler narrowing where proofs are local and sound
- adapt fixtures that rely on implicit mutation/truthiness idioms

### C3 `nonlocal_mutable_capture_not_supported` (2 fixtures)

Intentional divergence surface:

- recursive nested mutable captures via `nonlocal` are not part of the supported architecture

Language adjustment decision:

- do not add nonlocal mutable capture
- adaptation only

### C4 `recursive_node_and_field_expression_surface` (2 fixtures)

Node-heavy algorithms require attribute-read parity:

- field expression reads are missing for typed class/object nodes (`0230`, `0707`)
- some fixture-local issues remain adaptation-owned (`0707`, `0230` `mut`/return completeness)

Language adjustment decision:

- add compiler field-expression parity for typed objects
- no change to mutability/ownership principles

### C5 `optional_none_flow_and_narrowing_gap` (1 fixture)

Residual optional union/index narrowing instability (`0721`):

- key/index flows that are semantically total in the algorithm are still typed as `T | None`

Language adjustment decision:

- keep safe indexing/Option model
- improve narrowing precision and require explicit adaptation where proof is non-local

### C6 `signature_invalid_fixture_surface` (2 fixtures)

Two distinct roots:

- parse-safe conversion policy violation (`1849`) -> adaptation
- missing API parity (`1930` `rfind`) -> compiler

Language adjustment decision:

- keep `int(str) -> Result[int, ParseError]`
- add `str.rfind` parity

### C7 `other_type_surface_and_api_mismatch` (1 fixture)

`0973` is taxonomy-owned by this bucket and currently fails due Optional/index lowering instability that overlaps WS1 defect family; overflow warnings are advisory and consistent with integer policy.

Language adjustment decision:

- no language change needed for this failure

## Ready-To-Implement Workstreams

1. `WS1_codegen_soundness_run_stage`
- Fixtures: `0049`, `0144`, `0145`, `0286`, `0973`, `1137`
- Goal: zero run-stage build/assert failures from lowering/codegen defects.
- Exit criteria:
  - all six fixtures pass `sifr run`
  - targeted regression tests added for each defect family

2. `WS3_canonical_adaptation_sweep`
- Fixtures: `0402`, `0442`, `0543`, `0673`, `1849`, plus adaptation portions of mixed fixtures (`0018`, `0056`, `0230`, `0705`, `0707`, `0721`)
- Goal: rewrite fixtures to canonical Sifr patterns where divergence is intentional and close adaptation-owned parts of mixed fixtures first.
- Exit criteria:
  - no remaining failures caused by intentional-divergence patterns (`nonlocal`, unchecked parse `Result`, implicit int truthiness, implicit parameter mutation)
  - mixed-fixture adaptation preconditions are complete for WS2 compiler validation

3. `WS2_field_surface_and_optional_flow_core`
- Fixtures: `0230`, `0707`, `0018`, `0056`, `0721`, `1930`
- Goal: close compiler-owned check-stage gaps (field expressions, Optional flow precision, `str.rfind`) on top of WS3-adapted fixture forms.
- Exit criteria:
  - compiler diagnostics disappear for compiler-owned parts on all six fixtures
  - no relaxation of Option/Result safety contracts

4. `WS4_closure_and_reclassification`
- Goal: rerun full corpus, regenerate taxonomy, and close the phase with updated counts.
- Exit criteria:
  - `verification/leetcode/full_corpus_current_results_<date>_...json` regenerated
  - `verification/leetcode/full_corpus_failure_taxonomy_<date>_...json` regenerated
  - any residual failures are re-categorized with fresh RCA and explicit owner lane

## Reviewer Loop

Reviewer: agent

Artifacts:

- `reviews/ad-hoc-leetcode-18-root-cause-review-pass1-cli.md`
- `reviews/ad-hoc-leetcode-18-root-cause-review-pass2-cli.md`

Reviewer conclusions applied:

- apply reviewer pass-1 criticals:
  - reorder workstreams to `WS1 -> WS3 -> WS2 -> WS4`
  - clarify `0973` as C7 taxonomy owner with WS1 secondary dependency
  - add mixed-lane ownership split table
 - reviewer pass-2 verdict: `READY TO IMPLEMENT` with no remaining critical corrections

Final readiness verdict: `DONE`
