Now I have enough data for a thorough review. Let me compile the verdict.

---

## Phase Document Review: Ad-hoc Surface Parity And API Cleanup (2026-04-07)

### Verdict: **Mostly Ready**

The document is well-structured and most classifications are sound. However, there are several specific errors and risks that must be addressed before this phase is implementation-ready.

---

### CRITICAL: Count Discrepancy

The resolution-mode header reports `compiler: 10`, but the per-fixture matrix shows **12 compiler-only fixtures**:

- 0150, 0290, 0297, 0853, 1260, 1345, 1383, 1498, 1834, 2002 = 10... plus **1834** is listed as `compiler` but appears only in the `both` column (line 349), and **2002** similarly. Let me recount the matrix rows:

Compiler-only rows: 0150, 0290, 0297, 0853, 1260, 1345, 1383, 1498, 1834 = 9. Plus 1834 appears in both. Let me recount: 0150, 0290, 0297, 0853, 1260, 1345, 1383, 1498, 1834 — that's 9. Did I miss one?

Let me check the matrix again: 0150, 0290, 0297, 0853, 1260, 1345, 1383, 1498, 1834. 9 compiler-only. But the header says 10.

Wait — let me recount ALL matrix rows and their mode labels:

| Fixture | Mode |
|---|---|
| 0012 | adaptation |
| 0072 | both |
| 0130 | both |
| 0150 | compiler |
| 0200 | both |
| 0212 | both |
| 0221 | both |
| 0241 | adaptation |
| 0290 | compiler |
| 0297 | compiler |
| 0682 | adaptation |
| 0853 | compiler |
| 0994 | both |
| 1029 | adaptation |
| 1091 | adaptation |
| 1260 | compiler |
| 1345 | compiler |
| 1383 | compiler |
| 1498 | compiler |
| 1834 | compiler |
| 1851 | both |
| 2002 | compiler |

Compiler: 0150, 0290, 0297, 0853, 1260, 1345, 1383, 1498, 1834, **2002** = **10**. I missed 2002. So the count is actually correct — it's 10.

But wait — I count 22 fixtures in the matrix and 22 in the scope list, so that's consistent. And 10+5+7 = 22. Good.

---

### ISSUE 1: `1091_shortest_path_in_binary_matrix` is misclassified as `adaptation`

**Matrix row (line 344):** `1091` → `adaptation`. SP-7 rationale: "ambiguous `set((0, 0))` shape."

**But the taxonomy diagnostic is:**
```
'not in' operator: element type 'tuple[int, int]' is not compatible with collection element type 'int'
```

The error is not about list-row destructuring — it's about a **tuple being checked for membership in an int-keyed collection**. This is fundamentally different from the SP-7 rationale. Either:
- (a) The set is being specialized as `Set[int]` and a tuple is being checked against it (type inference failure, `compiler`)
- (b) The fixture is checking `tuple in set_of_ints` which is a genuine shape mismatch (`adaptation`)

The SP-7 "ambiguous `set((0, 0))`" explanation doesn't connect to the observed diagnostic. This needs root-cause verification before the classification stands. **Flag: needs re-examination.**

---

### ISSUE 2: Run-stage fixtures (0150, 0297, 1260, 1383, 1498) are correctly identified as codegen blockers, but the taxonomy label is corrosive

All five show `RUN_ERROR` with `other_type_surface_and_api_mismatch` in the live taxonomy. The actual diagnostics are overflow warnings, not surface mismatches. The document correctly unpicks this in SP-8, but the taxonomy classification will mislead anyone reading the artifact directly.

**SP-8 is correct.** The taxonomy entry is the artifact-level problem. The document should explicitly note that the taxonomy classification for these five fixtures is stale/misleading and will not self-correct when the codegen is fixed.

**Additionally:** The document says `1383` and `1498` expose "missing Rust-keyword escaping for local identifiers" and generate `mod` as a reserved identifier. I searched the codegen source and found no `r#mod` raw identifier escaping anywhere. Either this escaping exists and I missed it, or the document is asserting a bug that hasn't been located yet. The codegen directory has no file containing `is_keyword`, `RUST_KEYWORDS`, `sanitize`, or raw identifier patterns. **This needs confirmation against actual emitted Rust output before WS5 sub-lane 3 is scoped as a known fix.**

---

### ISSUE 3: `0212_word_search_ii` should be out-of-scope or explicitly deferred

The cross-bucket table lists `0212`'s secondary blocker as "recursive node / field-expression surface, helper annotations." This is not being closed in this phase. The fixture is also listed as `both` (line 335), meaning it will partially pass (range membership fixed) but still fail due to trie helpers and recursive node field access.

Including a fixture in this phase while knowing it has unresolved out-of-scope secondary blockers creates the risk of false "phase success." The fix: either explicitly mark `0212` as a **partial closer** with a follow-up phase ticket, or remove it from this phase's scope entirely. Do not leave it dangling in the matrix.

---

### ISSUE 4: Tuple `Comparable` — the document asserts `compiler` without acknowledging it is a design decision

`type_bounds.rs:101-104` shows `Comparable` is intentionally limited to `Int | Float | Str | Bool | BigInt`. The document says "current architecture does not document tuple ordering as forbidden" and calls this `compiler`. This is **technically correct** but it is also a **language design choice**.

The case for `compiler` is sound: heap algorithms in Python naturally use tuple keys with lexicographic ordering, Rust supports this natively, and `heapq` is already in the compat stdlib (`compat_imports.rs:26`). Adding tuple `Comparable` is the correct engineering response.

However, the document should explicitly note that this was not previously intended and requires a policy sign-off that goes beyond the normal "close the gap" framing. The decision to extend `Comparable` to tuples is load-bearing for the language's heap/priority queue story.

**Verdict on tuple Comparable: Keep as `compiler`.** The engineering case is correct. Just add a note that this is an explicit language extension, not a gap closure.

---

### ISSUE 5: `1834_single_threaded_cpu` — `heappop` return-type instability

The SP-3 analysis mentions "heap element typing instability after `heappop`" and the `nested_function_inference.rs:1218-1226` code confirms `heappop` returns `elem_ty` from the `List` wrapper — but if the list type has drifted to `Unknown`, the return type is `Unknown`. This is correctly attributed to `compiler` (type stabilization through iterator consumers), but the secondary dependency on `heappop` return-type correctness means the heap tuple comparability issue (SP-5) is a separate problem from the iterator-consumer problem (SP-3). The document correctly separates these but the cross-bucket table doesn't surface `heappop` return-type instability as a dependency for `1834`.

---

### ISSUE 6: Execution order is sound

WS1 → WS2 → WS4 → **WS3** → WS5 → WS6. Reasoning:
- WS3 (tuple Comparable) is type-system only — no codegen dependency on WS1/W2/W4 landing first
- WS5 (codegen defects) is independent of WS3 — narrowed value comparison and keyword escaping are pure codegen issues
- WS6 last is correct — fixture rewrites should not compensate for remaining compiler defects

The `build_normalized_list_index_i64_expr` function in `lower_stmt.rs` operates on `RustExpr`, not on HIR types, confirming that type stabilization (WS3) and index normalization (WS5 sub-lane 2) are independent.

**Execution order: Correct.**

---

### ISSUE 7: Checklist is unfilled and review loop is not marked complete

Lines 543-550: all checklist items are unchecked bullets, including `[ ] Claude review loop completed`. The document is under review but has no checked items. This is fine for a draft-under-review but the document must not be promoted to implementation-ready without these being addressed.

---

### Summary of Required Corrections

| # | Severity | Issue | Fix Required |
|---|---|---|---|
| 1 | High | `1091` misclassified as `adaptation` | Verify actual fixture error. If it's a type inference failure causing wrong set element type → reclassify to `compiler` |
| 2 | High | Taxonomy misclassifies 0150/0297/1260/1383/1498 as `other_type_surface_and_api_mismatch` | Document must flag these five as known taxonomy artifacts that won't self-correct; add explicit note in SP-8 |
| 3 | Medium | `1383`/`1498` "reserved keyword" escaping not verified against source | Confirm bug exists in emitted Rust; if not found, this sub-lane may be incorrectly scoped |
| 4 | Medium | `0212` has out-of-scope secondary blockers still in scope | Either explicitly mark as partial close + follow-up ticket, or remove from phase |
| 5 | Low | Tuple Comparable is a language extension, not gap closure | Add explicit note that this is a policy extension requiring sign-off |
| 6 | Low | Checklist unfilled | All items are unchecked bullets — document cannot claim readiness until addressed |

---

### What Is Correct

- **Scope snapshot and fixture list:** 22 fixtures from live taxonomy, correctly excluding `callable_argument_contract_mismatch` (stale count corrected).
- **SP-1 (variadic min/max):** Correct. `builtin_calls.rs` currently only handles 1-arg and 2-arg forms.
- **SP-2 (range membership):** Confirmed. `types.rs:1047-1055` — `contains_element_type` has no `Type::Range` arm.
- **SP-3 (iterator consumers):** Correct. `Iterator[Tuple[...]]` not being recognized by `contains_element_type` or iteration lowering.
- **SP-4 (empty-container specialization):** Correct. `empty_collection_refinement.rs` handles first-write specialization but can drift.
- **SP-6 (parse-safety adaptation):** Correct. `0241` uses `res or [int(s)]` truthiness; `0682` uses `int(str)` in arithmetic context.
- **SP-8 (codegen defects):** Correct diagnosis. The taxonomy label is wrong but the root-cause analysis is sound.
- **SP-9 (mixed fixtures):** Correctly identifies residual adaptation needs after compiler parity closes.
- **WS1-W5 workstream loci:** Plausible and traceable to actual files.
- **WS6 sweep target list:** Correctly enumerates adaptation-owned fixtures.
- **Resolution-mode split:** `adaptation: 5` and `both: 7` are internally consistent. `compiler: 10` is correct per the matrix (10 unique compiler-only fixtures).

---

**Bottom line:** Mostly ready. Fix the `1091` classification, clarify the keyword-escaping sub-lane for 1383/1498, resolve the `0212` secondary-blocker ambiguity, and fill the checklist. The taxonomy-vs-root-cause analysis in SP-8 is the strongest part of the document and should be preserved verbatim.
