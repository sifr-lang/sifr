

Now I have enough data for a comprehensive review. Let me deliver the verdict.

---

## Phase Review: `ad-hoc-surface-parity-and-api-cleanup-2026-04-07.md`

**Verdict: Not Ready**

### Critical Issues

#### 1. Scope undercounts by 16 fixtures

The document states 22 fixtures in scope, but the taxonomy contains **38 total failures** across 9 categories. The phase scope explicitly omits fixtures from `nonlocal_mutable_capture_not_supported`, `optional_none_flow_and_narrowing_gap`, `ownership_and_mutability_boundary`, `recursive_node_and_field_expression_surface`, `signature_invalid_fixture_surface`, and `codegen_runtime_build_gap` — all of which contain fixtures in categories the user listed as in-scope.

Specific fixtures in top-level requested categories that are absent from the scope:

| Fixture | Taxonomy category | First diagnostic |
|---|---|---|
| `0543_diameter_of_binary_tree` | `nonlocal_mutable_capture_not_supported` | recursive nested function cannot mutate captured state |
| `0673_number_of_longest_increasing_subsequence` | `nonlocal_mutable_capture_not_supported` | same |
| `0721_accounts_merge` | `optional_none_flow_and_narrowing_gap` | cannot index `list[int]` with `int \| None` |
| `0230_kth_smallest_element_in_a_bst` | `recursive_node_and_field_expression_surface` | attribute access `.right` not supported |
| `0707_design_linked_list` | `recursive_node_and_field_expression_surface` | attribute access `.next` not supported |
| `1849_splitting_a_string_into_descending_consecutive_values` | `signature_invalid_fixture_surface` | argument 2 got `Result[int, ParseError]` |
| `1930_unique_length_3_palindromic_subsequences` | `signature_invalid_fixture_surface` | str has no method `rfind` |
| `0018_4sum`, `0056_merge_intervals`, `0402_remove_k_digits`, `0442_find_all_duplicates_in_an_array` | `ownership_and_mutability_boundary` | add `mut` to parameter declaration |

`1029` is listed in the scope but is tagged `destructuring_and_assignment_target_surface_gap` — which IS one of the 4 user-requested top-level categories, but the fixture is not listed under that category in the summary counts (only `generic: 1` under destructuring). This means the scope is self-inconsistent.

**Recommendation**: Either expand scope to cover all fixtures in the 4 requested top-level categories, or explicitly move these to an "out-of-scope dependency" list with a separate phase handle.

---

#### 2. `1029_two_city_scheduling` resolution mode is incomplete

The resolution matrix says `adaptation` for list-row tuple destructuring, and the primary diagnostic (`for loop tuple target expects iterable elements of tuple type, got 'list[int]'`) supports that.

But the live rerun2 shows **additional unresolved diagnostics** for `1029`:
```
type error: unsupported operand type(s) for +: 'int' and 'int | None'
type error: unsupported operand type(s) for +: 'int' and 'int | None'
```

These `int | None` arithmetic errors are NOT addressed by the tuple-destructuring adaptation claim. They point to an optional/flow narrowing gap (`optional_none_flow_and_narrowing_gap`) that is a separate root cause.

**Recommendation**: `1029` should be `both` (compiler + adaptation) or the scope should explicitly call out that the `int | None` arithmetic errors require a separate closure before the fixture can pass.

---

#### 3. `1091_shortest_path_in_binary_matrix` root cause is misattributed

The phase doc attributes this to SP-7 (`adaptation`) citing "ambiguous `set((0, 0))` shape". But the actual live diagnostic is:
```
type error: 'not in' operator: element type 'tuple[int, int]' is not compatible with collection element type 'int'
```

This is a membership type mismatch where a tuple element is being checked against an `int`-typed collection — not primarily about set construction ambiguity. The ambiguity claim is plausible but the document does not reconcile it with the observed diagnostic, which points to `int` being the collection element type.

**Recommendation**: `1091` root cause attribution needs reanalysis. The membership type mismatch and the set-construction shape ambiguity may be the same bug viewed from different angles, but the document doesn't establish this chain.

---

#### 4. Five fixtures (`0049`, `0144`, `0145`, `0705`, `1137`) are RUN_ERROR codegen failures outside the 22-fixture scope

The taxonomy `codegen_runtime_build_gap` bucket contains:
- `0049_group_anagrams` — RUN_ERROR with runtime assertion failure (not a build failure; passes `cargo check`, fails at runtime assertion)
- `0144_binary_tree_preorder_traversal` — RUN_ERROR with actual Rust build failure (`expected TreeNode, found Option<TreeNode>`)
- `0145_binary_tree_postorder_traversal` — same pattern as 0144
- `0705_design_hashset` — RUN_ERROR build failure
- `1137_n_th_tribonacci_number` — `rust_missing_binding_emission`

These are NOT in the phase scope but share the same codegen-runtime-build-gap category as the correctly-scoped `0150`, `0297`, `1260`, `1383`, `1498`. If SP-8 is about codegen defects, these out-of-scope codegen fixtures represent a parallel unresolved codegen backlog.

**Recommendation**: Either include these 5 in the phase scope (they are the same class of defect) or explicitly call them out as a separate codegen backlog in the cross-bucket dependencies section.

---

#### 5. Tuple lexicographic Comparable — classification is correct

The document classifies tuple lexicographic `Comparable` as `compiler` (WS3, SP-5). This is correct:

- `type_bounds.rs:100-104` explicitly limits `Comparable` to primitives only: `Type::Int | Type::Float | Type::Str | Type::Bool | Type::BigInt`
- `Type::Tuple` is absent with no recursive element check
- No architecture document forbids tuple ordering
- Fixture `1851` uses `(size, right)` heap tuples where every element is comparable — this is a natural, statically-safe extension

**Classification stands**: `compiler`, WS3. No correction needed.

---

#### 6. Run-stage fixtures (0150, 0297, 1260, 1383, 1498) — correctly grouped as codegen/compiler blockers

The taxonomy labels these `RUN_ERROR` with `other_type_surface_and_api_mismatch`, which is misleading. Their actual failure mode is:

- `0150`, `0297`: `cargo build` fails with `can't compare 'String' with 'Option<String>'` — same Option-narrowing codegen bug family
- `1260`: `cargo build` fails with `Option<i64>` in index normalization arithmetic
- `1383`, `1498`: `cargo build` fails with `expected identifier, found keyword 'mod'` — reserved keyword escaping gap

The overflow **warnings** are a red herring (secondary noise). The actual blocker is a hard Rust build failure at the `run` stage. The phase doc correctly identifies this distinction and groups them as `compiler`.

**Classification stands**: `compiler`, SP-8. Correct.

---

#### 7. WS3 placement in execution order is questionable

The document orders WS1 → WS2 → WS4 → WS3 → WS5 → WS6, with the rationale:
> "WS4 must land before several iterator and compat cases stop degenerating into `Unknown`"

But WS3 (iterator consumer stabilization) **depends on** concrete element types that WS1 and WS2 establish. The rationale for WS4 before WS3 is stated, but the rationale for WS3 after WS4 (rather than after WS1/WS2 which produce the types WS3 needs to stabilize) is not established.

WS4 empty-container specialization preventing degeneration into `Unknown` makes sense for containers, but the `sorted()` output type stabilization in WS3 should follow directly from WS1/WS2 establishing concrete types, not from WS4.

**Recommendation**: Justify WS3 → WS4 ordering with concrete reasoning, or swap them if the dependency chain actually runs WS1/WS2 → WS3 → WS4.

---

#### 8. Implementation-readiness checklist is incomplete

The checklist at lines 543-550 has only 5 items and all are unchecked/pending. No workstream has confirmed compiler loci beyond the document's own claim. The checklist items themselves are correct but are not actionable without evidence.

**Recommendation**: Before declaring ready, each workstream needs:
- A concrete file path + function/struct name for every primary locus
- At least one pre-existing test that exercises the failing path (even if xfail)
- A clear acceptance criterion that is binary (pass/fail), not a spectrum

---

### Summary of Disagreements

| Fixture | Document says | Should be | Reason |
|---|---|---|---|
| `1029_two_city_scheduling` | `adaptation` | `both` (compiler + adaptation) | Secondary `int \| None` arithmetic errors are unresolved |
| `1091_shortest_path_in_binary_matrix` | SP-7 root cause (set shape ambiguity) | Needs reanalysis | Actual diagnostic is membership type mismatch, not set construction |
| 5 codegen fixtures (`0049`, `0144`, `0145`, `0705`, `1137`) | Out of scope | Should be called out explicitly | Same codegen-runtime-build-gap class as scoped SP-8 fixtures; represent parallel backlog |
| Scope | 22 fixtures | Undercounts by 16 | Omits fixtures from `nonlocal_mutable_capture_not_supported`, `optional_none_flow_and_narrowing_gap`, `ownership_and_mutability_boundary`, `recursive_node_and_field_expression_surface`, `signature_invalid_fixture_surface` |

### What Is Correct

- Tuple `Comparable` as `compiler` / WS3: **correct**
- Run-stage SP-8 fixtures as `compiler` codegen blockers (not policy issues): **correct**
- SP-2 range membership grouping with all 5 fixtures: **correct** (re-confirmed via live results)
- SP-6 parse-safety as `adaptation` (`0241`, `0682`): **correct**
- SP-7 canonical shape as `adaptation` (`0012`, `1029`, `1091` primary): **correct**
- Execution order (WS1/WS2 before WS3) dependency chain: **directionally correct**
- `WS6` adaptation sweep coming last: **correct**
e analysis is sound.
- **SP-9 (mixed fixtures):** Correctly identifies residual adaptation needs after compiler parity closes.
- **WS1-W5 workstream loci:** Plausible and traceable to actual files.
- **WS6 sweep target list:** Correctly enumerates adaptation-owned fixtures.
- **Resolution-mode split:** `adaptation: 5` and `both: 7` are internally consistent. `compiler: 10` is correct per the matrix (10 unique compiler-only fixtures).

---

**Bottom line:** Mostly ready. Fix the `1091` classification, clarify the keyword-escaping sub-lane for 1383/1498, resolve the `0212` secondary-blocker ambiguity, and fill the checklist. The taxonomy-vs-root-cause analysis in SP-8 is the strongest part of the document and should be preserved verbatim.
