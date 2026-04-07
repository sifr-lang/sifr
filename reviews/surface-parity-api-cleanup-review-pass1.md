# Review (Pass 1): Ad-hoc Surface Parity And API Cleanup Phase (2026-04-07)

Reviewed doc: `issues/ad-hoc-surface-parity-and-api-cleanup-2026-04-07.md`
Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun2.json`
Source results: `verification/leetcode/full_corpus_current_results_20260407_live_rerun2.json`
Reviewer: implementation-readiness pass

## TL;DR

- **Verdict: Mostly Ready.**
- **Fixture inventory is correct.** All 22 in-scope fixtures map cleanly to the three requested categories (10 + 11 + 1). The "stale callable_argument_contract_mismatch=1" correction is accurate.
- **Compiler vs adaptation classification is largely sound.** All 9 root-cause groupings (SP-1..SP-9) line up with what the code actually does. I verified the underlying compiler state for variadic min/max, `range` membership, `Comparable` bound, and the 5 run-stage codegen failures.
- **The 5 run-stage fixtures (0150, 0297, 1260, 1383, 1498) are correctly grouped as codegen blockers**, not policy/warning issues. The taxonomy's "first diagnostic" is the noisy overflow warning; the actual `cargo build` errors are real Rust codegen defects (`String == Option<String>`, `Option<i64>` arithmetic in index normalization, unescaped `mod` keyword).
- **Tuple lexicographic `Comparable` is correctly classified as a compiler feature**, not adaptation. It is consistent with Sifr's existing auto-derive principle for `Comparable` and with both Python and Rust semantics. The current `type_bounds.rs` only hardcodes primitives, so this is a genuine surface gap.
- **Three fixtures should arguably be reclassified as not-closable-in-this-phase**: `0241_different_ways_to_add_parentheses`, `0212_word_search_ii`, and `1345_jump_game_iv`. The doc partially flags the latter two in cross-bucket dependencies but does not change exit expectations. `0241` is silently understated.
- **Workstream loci are real and accurate.** All cited files exist (some Glob lookups fail spuriously but `ls` confirms them). Acceptance criteria for WS1, WS2, WS4, WS5 are concrete and testable. **WS3 and WS6 acceptance criteria are too soft** to gate phase exit cleanly.
- **The execution order is mostly defensible** but two ordering points (WS5 placement, WS3 sub-scope split) deserve attention.

---

## 1. Fixture Inventory Validation

Cross-checked against `full_corpus_failure_taxonomy_20260407_live_rerun2.json`:

| Category | Doc count | Taxonomy count | Match |
|---|---|---|---|
| `python_stdlib_and_builtin_parity_gap` | 10 | 10 | ✅ |
| `other_type_surface_and_api_mismatch` | 11 | 11 | ✅ |
| `destructuring_and_assignment_target_surface_gap` | 1 | 1 | ✅ |
| `callable_argument_contract_mismatch` | 0 | n/a (category absent) | ✅ |
| **Total in scope** | **22** | **22** | ✅ |

The doc's "stale callable_argument_contract_mismatch (1)" correction is technically right but slightly misleading: that category is not just empty, it has been **renamed/refactored** in the live taxonomy into `signature_invalid_fixture_surface (2)` (fixtures `1849_splitting_a_string_into_descending_consecutive_values` and `1930_unique_length_3_palindromic_subsequences`). Neither is in scope here, but the reader should know the category did not just shrink to zero — it was restructured. **Non-blocking nit; flag in revision.**

All 22 fixture slugs in the doc match the failure list exactly.

---

## 2. Verifying Compiler State Behind The Root Causes

I read the actual sources to confirm that each compiler-side claim is accurate.

### SP-1 — variadic `min` / `max` (compiler)

`crates/sifr_hir/src/lower/expressions.rs:1134-1207` only handles `min`/`max` with exactly 1 or 2 arguments and falls through to `ctx.error("min() takes 1 or 2 arguments".to_string())` / `"max() takes 1 or 2 arguments"`. Variadic scalar form is not implemented. **Claim verified.** Compiler classification is correct. The `crates/sifr_hir/src/lower/min_max_validation.rs` locus exists.

### SP-2 — `range` membership and compat-mapping membership (compiler)

`crates/sifr_type_system/src/types.rs:1047` `contains_element_type()` only matches `List | Set | Dict | Str | Bytes`. `Range` is a first-class `Type::Range` variant in `types.rs:30` and already has iteration metadata at line 875 (`element_type = Int`), but is not wired into `contains_element_type`. Wiring it is essentially a one-line addition. **Claim verified.** Compiler classification is correct.

The compat `__compat_defaultdict_list` membership gap is also verified through the diagnostic on `1345_jump_game_iv` and is consistent with the absence of compat-wrapper handling in `contains_element_type`.

### SP-3 — iterator/list consumer interoperability (compiler)

Confirmed via `0853` and `1834` source reads. Both fixtures lose static element type at `enumerate(...)`/`zip(...)` boundaries inside list comprehensions. `0853` additionally fails on `pair.sort(reverse=True)` — see issue (a) below.

### SP-4 — empty container specialization (compiler)

`0290_word_pattern` uses `charToWord = {}` followed by `if c in charToWord` (str key) and `charToWord[c] = w`. The diagnostic `'in' operator: element type 'str' is not compatible with collection element type 'int'` shows the empty dict was specialized to `int` keys somewhere upstream. The `container_literal_specialization.rs` locus exists. **Claim verified.**

### SP-5 — tuple lexicographic `Comparable` (compiler) — answering the explicit question

`crates/sifr_hir/src/lower/type_bounds.rs:96-121`:

```rust
match bound {
    "Comparable" => matches!(
        ty,
        Type::Int | Type::Float | Type::Str | Type::Bool | Type::BigInt
    ),
    ...
```

`Tuple` is not in this set. **The doc's claim is verified.**

Whether this should be a compiler feature or stay adaptation-only:

- **`internal_docs/architecture.md:770-772`** says: "`Display`, `Hashable`, `Comparable` are auto-derived for classes where all fields implement the corresponding Rust trait." The auto-derive principle is stated as a general rule, not a class-only rule.
- **Rust** auto-derives `Ord` for tuples when every component implements `Ord`. That is exactly what the doc proposes.
- **Python** supports lexicographic tuple ordering by default.
- **Sifr `Range` lowering** already maps to `std::ops::Range<i64>`, and the existing heap/sort generic bounds in `13_type_system_completion.md:575-580` use `T: Comparable` without restricting the shape of `T`.
- **No phase doc forbids tuple comparability.** I grepped `internal_docs/` for `tuple.*Comparable|tuple.*Ord|lexicographic|tuple.*compare|tuple.*sort` and got nothing prohibitive.

**Verdict on tuple comparability: this should be a compiler feature, exactly as the doc proposes.** It is the principled extension of an existing rule, it matches Python and Rust, and it removes an asymmetry (classes get auto-derived Comparable but tuples don't).

**One subtlety the doc misses:** `type_bounds.rs:103` lists `Type::Float` in the `Comparable` set, even though `02_type_system_power.md:281` and `architecture.md:772` explicitly say `float` is **not** Comparable (because of NaN). That is a pre-existing implementation drift unrelated to this phase, but it has direct implications for WS3 acceptance:

- A tuple `tuple[float, int]` should **not** satisfy `Comparable`, yet under the current (buggy) primitive set it would.
- WS3 should explicitly state: tuple Comparable holds iff every element type satisfies Comparable **under the documented spec**, not under the current `type_bounds.rs` set. Either fix the float-in-Comparable drift first, or have WS3 carry an explicit per-element exclusion of `Float`.

**Action: WS3 acceptance criteria must call this out.** Otherwise WS3 will land working but logically inconsistent with the language spec, and `list[tuple[float, int]].sort()` will silently start working.

### SP-6 / SP-7 — adaptation lanes

I read `0241`, `0682`, `0012`, `1029`, `1091` directly. The classifications are correct in spirit:

- `0682`: `int(o)` -> `Result[int, ParseError]`, plus arithmetic on `score_stack[-1]` Optional access. Adaptation. ✅
- `0012`: heterogeneous `[["I", 1], ...]` rows. Adaptation. ✅
- `1029`: `for c1, c2 in costs` where `costs: list[list[int]]`. Adaptation. ✅
- `1091`: `set((0, 0))` ambiguous shape. Adaptation. ✅
- `0241`: see issue (b) below — **classification understated.**

### SP-8 — codegen defects (compiler) — answering the explicit question

I pulled the actual `cargo build` stderr for all 5 run-stage fixtures from `full_corpus_current_results_20260407_live_rerun2.json`. Verified:

| Fixture | Real cargo error | Doc claim | Match |
|---|---|---|---|
| `0150_evaluate_reverse_polish_notation` | `error[E0277]: can't compare `String` with `Option<String>`` at `if true && (first == Some(("-".to_string()).clone()))` | "narrowed value vs `Option` compare lowering" | ✅ same defect; framing is approximate (see issue (g)) |
| `0297_serialize_and_deserialize_binary_tree` | identical `String == Option<String>` defect on a `"-"` literal compare | same family as `0150` | ✅ |
| `1260_shift_2d_grid` | `error[E0308]: mismatched types ... expected Option<i64>, found integer` on `let __oi_norm = if __oi_raw < 0 { (res.len() as i64) + __oi_raw } else { __oi_raw }` | "incorrect index normalization lowering for values from list-returning helpers" | ✅ Real bug: the codegen-introduced `__oi_raw` is `Option<i64>` (because it came through indexed access of a list returned by a helper) but the normalization arithmetic treats it as `i64`. |
| `1383_maximum_performance_of_a_team` | `error: expected identifier, found keyword `mod`` at `let mod: i64 = (10 as i64).pow(...)` | "missing Rust-keyword escaping for local identifiers" | ✅ |
| `1498_number_of_subsequences_that_satisfy_the_given_sum_condition` | identical `mod` keyword escape failure | same family as `1383` | ✅ |

**All 5 are confirmed real codegen defects, not policy/warning issues.** The phase doc's grouping is correct: these belong with the compiler workstreams, not in `WS6` adaptation, and the overflow warnings preceding them are incidental noise.

---

## 3. Fixture-by-Fixture Issues To Address

### (a) `0853_car_fleet` — WS3 scope is incomplete

The diagnostics on this fixture are:

1. `cannot iterate over type 'Iterator[tuple[int, int]]'`
2. `for-loop iterable must have a statically-known element type, got 'Unknown'` (cascade from #1)
3. `sort() got an unexpected keyword argument 'reverse'`

The doc puts (3) under **WS3 iterator stabilization** ("`sort(reverse=...)` stabilization"), but `sort(reverse=True)` is not an iterator stabilization concern — it is a missing keyword-argument surface in `list.sort()`. `02_type_system_power.md:278` explicitly documents `list.sort(reverse=True)` as a milestone_generics feature, so it is intended parity.

**Recommendation:** move "support `list.sort(reverse=True)` (and `list.sort(key=...)` if also missing)" into **WS1 (builtin parity)** or as a fourth sub-lane of WS3. WS3 acceptance must explicitly include "no `unexpected keyword argument 'reverse'` diagnostics remain in scope". As written, WS3 acceptance is satisfied by stabilizing iterators alone, which would leave 0853 still failing.

### (b) `0241_different_ways_to_add_parentheses` — adaptation classification is **silently understated**

The fixture uses **`eval(f'{l}{c}{r}')`**. Sifr does not have `eval`. The doc lists root cause as "Pythonic `or` truthiness + parse-safety mismatch" and prescribes "keep explicit control flow" as the canonical Sifr response. This is correct as far as it goes, but **does not address the dominant blocker**, which is that fixing the truthiness alone leaves `eval(...)` as an unsupported call.

To pass this fixture under canonical Sifr, the rewrite has to replace `eval(f'{l}{c}{r}')` with an explicit arithmetic dispatch on `c`:

```python
if c == "+": res.append(l + r)
elif c == "-": res.append(l - r)
elif c == "*": res.append(l * r)
```

That is closer to a substantive algorithm rewrite than a "canonicalization sweep," and WS6 as written gives the implementer no signal that this is required.

**Recommendation:** either
- (i) pull `0241` out of the in-scope list and mark it "out of scope: requires `eval`-free rewrite, not within adaptation surface area," or
- (ii) call out the eval rewrite explicitly in WS6 with a per-fixture acceptance note.

Option (i) is cleaner, given that the rewrite is non-trivial and there is no compiler payload here.

### (c) `0212_word_search_ii` — should not be exit-gating

The fixture has, in addition to `range` membership:

- A separate `Node` class with five `Node | None` fields used as a recursive linked node (`recursive_node_and_field_expression_surface` family).
- A `TrieNode` class **defined after** `findWords` uses it (forward reference).
- Untyped `__init__`, `addWord`, `removeWord` helpers on `TrieNode`.
- Untyped nested `dfs(r, c, node, word)`.
- Dynamic `cur.children[c]` and `node.children[board[r][c]]` field/dict expressions.

The doc correctly flags this in cross-bucket dependencies ("recursive node / field-expression surface, helper annotations") and marks it `both`. But putting it in scope with no exit caveat means WS2 will land, the fixture will still fail, and the phase will look incomplete.

**Recommendation:** keep `0212` listed in scope but explicitly **carve it out of phase exit criteria**. Acceptance should read "WS2 makes the `range` membership diagnostic disappear from `0212`'s diagnostic list; remaining diagnostics are tracked in the recursive_node phase." Otherwise this fixture will distort the phase scoreboard.

### (d) `1345_jump_game_iv` — should not be exit-gating either

In addition to compat-defaultdict membership and empty-container specialization, this fixture has:

- A nested `getUnqueuedNeighbors` defined **before** `seen` is bound (lines 17-34 reference `seen`, which is bound at line 37). Forward reference into a captured local list.
- Mutation of the captured `seen` list (`seen[i - 1] = True`).
- Implicit fallthrough on `minJumps` (no return at the bottom of the function), producing `function 'minJumps' must return a value of type 'int' on all control-flow paths`.

The doc flags the return-path issue in cross-bucket. But none of the workstreams own "fix forward-captured local list scoping" or "fix totality analysis on this control-flow shape." After WS2 + WS4, this fixture will still fail.

**Recommendation:** same as `0212` — keep in scope, carve out of phase exit criteria, with the explicit note that closure depends on a separate phase or on the user fixing the fixture itself (the totality issue is real Sifr-rejection-of-Python's-undefined-behavior, not a compiler bug).

### (e) `1834_single_threaded_cpu` — root cause description is slightly off

The diagnostic `cannot iterate over type 'Iterator[tuple[int, list[int]]]'` does not come from a heap operation — it comes from the comprehension `sorted([(t[0], t[1], i) for i, t in enumerate(tasks)])`, where `tasks: list[list[int]]`, so `enumerate(tasks)` yields `Iterator[tuple[int, list[int]]]` and the comprehension's `for i, t in ...` cannot destructure it inside the comprehension form.

After that destructure works, the resulting `tasks` becomes `list[tuple[int, int, int]]`, then the heap is `(tasks[i][1], tasks[i][2])` which is `tuple[int, int]`, and **then** tuple `Comparable` becomes load-bearing for `heappush`/`heappop`.

So the dependency chain is: **comprehension iterator destructure (compiler) → tuple `Comparable` (compiler) → fixture passes**. Both depended-on fixes are in WS3, so the classification is right, but **WS3 acceptance criteria need to mention "destructure inside comprehension iterator" specifically**, not just the bare for-loop case. As written, the test set could pass for bare for-loops while leaving the comprehension form broken.

### (f) `1851_minimum_interval_to_include_each_query` — already mostly fine, but the adaptation half is significant

Even after tuple `Comparable` lands, this fixture still has `intervals[i][0]` etc. producing `int | None`, then `r - l + 1` arithmetic on optionals. The adaptation half is real and the doc captures it as "row shape/optional extraction." Just confirm that WS6 has explicit acceptance for that adaptation half.

### (g) SP-8 sub-lane 1 framing is speculative

The doc says `0150` and `0297` "share an Option-comparison lowering bug **around narrowed character access**." Looking at the actual generated Rust:

```rust
if true && (first == Some(("-".to_string()).clone())) {
```

The defect is that the **literal** `"-"` was wrapped in `Some(...)` on the RHS of a `String == ?` comparison. This is **not necessarily** narrowed-character-access lowering — it could be a literal-wrapping bug in an `==` codegen path that mistakenly thinks the LHS is `Option<String>` (perhaps because the original Sifr value was `tokens.get(0)` or similar). The `if true && (...)` pattern also suggests the `if` was lowered in two parts and one side dropped narrowing.

**Recommendation:** WS5 sub-lane 1 should be described as "diagnose and fix the `String == Option<String>` codegen on the `0150`/`0297` shape; root-cause TBD." Committing now to "narrowed character access" pre-decides the fix and may waste implementer time.

---

## 4. Workstream Implementation-Readiness Per Lane

### WS1: variadic `min` / `max` — **Ready**

- Loci verified: `expressions.rs:1134-1207`, `min_max_validation.rs` exists, `expressions_tests.rs` exists.
- Acceptance criterion ("no remaining `min()/max() takes 1 or 2 arguments` diagnostics in scoped rerun" + explicit 3-arg/4-arg tests) is concrete and testable.
- **Verdict: Ready.**
- Suggested addition: also gate on the language-policy decision being preserved — `min(a, b, c)` where any operand is `Option`-typed should still error per the existing 2-arg validator. Make sure variadic doesn't accidentally bypass `validate_two_arg_min_max_operands`.

### WS2: `range` and compat membership — **Ready**

- Loci verified: `types.rs::contains_element_type` (`crates/sifr_type_system/src/types.rs:1047`), `expressions.rs`, `compat_imports.rs`.
- Acceptance criterion is concrete: zero `range`/`__compat_defaultdict_list` diagnostics + dedicated tests.
- **Verdict: Ready.** This is the simplest workstream — `Range => Some(Type::Int)` is essentially a one-liner, plus a similar wiring for typed compat wrappers.

### WS3: iterator stabilization + tuple `Comparable` — **Mostly Ready, needs scope tightening**

Issues:
- Acceptance only says "`0853`, `1834`, and compiler-owned part of `1851` stop failing in focused reruns." This is too coarse and conflates several distinct sub-features.
- Does not call out **comprehension-form** iterator destructure (issue (e)).
- Does not call out **`list.sort(reverse=...)`** (issue (a)).
- Does not address the `Float` in `Comparable` drift (see SP-5 verification above).

**Recommendation: split WS3 acceptance into explicit clauses:**
1. `for tup in zip(a, b)` and `for i, t in enumerate(xs)` preserve concrete tuple element types in both bare for-loop **and comprehension** form.
2. Tuple `T = tuple[T1, ..., Tn]` satisfies `Comparable` iff each `Ti` satisfies `Comparable` **under the documented spec** (not the current type_bounds.rs primitive set).
3. New e2e pass test: `heap` of `tuple[int, int]` sorted lexicographically.
4. New e2e fail test: `heap` of `tuple[float, int]` is rejected (or handle the float-Comparable drift first).
5. `list.sort(reverse=True)` and `list.sort(key=...)` accepted on lists whose element type satisfies `Comparable` (per (2)).

### WS4: empty-container specialization — **Mostly Ready**

- Locus `container_literal_specialization.rs` exists, plus the related `defaultdict_refinement.rs` and `empty_collection_refinement.rs` files in the same directory may also be relevant.
- Acceptance ("`0290` stops failing; `1345` loses container-specialization blocker; new tests for paired forward/reverse writes and membership checks") is concrete.
- **Verdict: Mostly Ready.** Ensure the test fixture coverage includes both `dict[K] = V` followed by `K in dict[K]` (ordering matters for the bug).

### WS5: scoped codegen defect closure — **Ready**

- 3 sub-lanes are individually concrete.
- Acceptance ("focused run reruns produce no Rust build error") is the right gate.
- **Verdict: Ready**, with the caveat from issue (g) that sub-lane 1 should not pre-decide root cause.
- Loci are intentionally broad (`crates/sifr_codegen/src`) which is fine since the precise file is unclear until diagnosis.

### WS6: canonical Sifr adaptation sweep — **Not Ready (acceptance is too soft)**

- "rewrites preserve algorithmic intent and stay within current Sifr rules" is a quality criterion, not a measurable acceptance gate.
- No per-fixture target for what "canonical Sifr" looks like.
- Silently asks the implementer to handle `eval()` in `0241` without flagging it.
- Does not specify whether `0212`/`1345` are exit-gating or carve-outs.

**Recommendation: rewrite WS6 acceptance as a per-fixture table:**

| Fixture | Canonical rewrite target | Pass criterion |
|---|---|---|
| `0012` | tuple rows `list[tuple[str, int]]` | `check + run` clean |
| `0241` | replace `eval` with explicit op dispatch; explicit bool guard | `check + run` clean — or move OUT of scope |
| `0682` | unwrap `int(o)` via `match`; explicit length guards | `check + run` clean |
| `1029` | indexed access `costs[i][0]`/`[1]` instead of for-loop unpack | `check + run` clean |
| `1091` | `set[tuple[int, int]]()` + literal insert | `check + run` clean |
| residual `0072` | `int` sentinel (e.g. `len(word1) + len(word2)`) instead of `float("inf")` | passes after WS1 |
| residual `0130` | `mut board`, drop tuple-return shape | passes after WS2 |
| residual `0200` | pick one variant; tuple direction list; mark `mut grid` if needed | passes after WS2 |
| residual `0221` | `int` totality on `max(cache.values())` via explicit fallback | passes after WS1 |
| residual `0994` | `mut grid` | passes after WS2 |
| residual `1851` | unwrap optional-row extractions | passes after WS3 |

This is the level of specificity that lets WS6 actually exit cleanly.

---

## 5. Execution Order

Doc order: WS1 → WS2 → WS4 → WS3 → WS5 → WS6.

- **WS1 → WS2 → WS4 first** is correct: each removes a class of false-`Unknown` cascades that would otherwise mask whether WS3's iterator stabilization is real or apparent.
- **WS4 before WS3** is correct, for the reason the doc gives.
- **WS5 placement (after WS3)** is defensible but not load-bearing. The 5 codegen defects are completely independent of the checker side and can run in parallel with anything. The doc's reasoning ("isolated by layer; should run after checker-side type surfaces are cleaner") is fine but conservative — WS5 could be parallelized to compress the schedule.
- **WS6 last** is correct so adaptation rewrites do not paper over compiler defects. Strongly agree.

**Verdict: order is correct, no changes required, but WS5 could optionally run in parallel with WS1-WS4.**

---

## 6. Cross-Bucket / Out-of-Scope Reclassification Recommendations

The doc's "Cross-Bucket Dependencies" table is honest but does not change the in-scope set. I would push three fixtures the rest of the way:

| Fixture | Recommendation | Reason |
|---|---|---|
| `0241_different_ways_to_add_parentheses` | **Move out of scope** (or require explicit `eval`-rewrite acceptance in WS6) | Uses `eval(f'{l}{c}{r}')`. Adaptation lane as written cannot close it. |
| `0212_word_search_ii` | **Keep in scope, carve out of phase exit criteria** | Recursive `Node`/`TrieNode` field-expression surface and forward-reference are both in `recursive_node_and_field_expression_surface` territory, which is a different phase. |
| `1345_jump_game_iv` | **Keep in scope, carve out of phase exit criteria** | Forward-captured local list (`seen`) and totality on `minJumps` are separate from compat-membership and specialization. |

The other 19 fixtures can plausibly close in this phase under the workstreams as scoped (with the WS3/WS6 tightening above).

---

## 7. Implementation-Readiness Checklist (revised)

- [x] Each compiler workstream has explicit, real loci.
- [x] Each adaptation lane is policy-aligned (no Optional weakening, no truthiness shortcuts, no tuple-target relaxation, no exception-flow back-channel).
- [x] Trigger-label vs actual-root-cause mismatch is documented for the 5 run-stage fixtures.
- [ ] Cross-bucket blockers are called out — **partial**: doc names them but does not adjust exit expectations. Fix per §6.
- [ ] WS3 acceptance covers `sort(reverse=...)`, comprehension-form iterator destructure, and the `Float`-in-`Comparable` drift. **Not yet.**
- [ ] WS6 has per-fixture canonical-target table with measurable pass criteria. **Not yet.**
- [ ] `0241` is either out of scope or carries an explicit `eval`-rewrite acceptance. **Not yet.**
- [ ] Phase exit criteria explicitly carve out `0212` and `1345` so they don't distort the scoreboard. **Not yet.**

---

## 8. Final Verdict

**Mostly Ready.**

The taxonomy work, the compiler-vs-adaptation triage, the codegen-blocker identification, and the tuple `Comparable` decision are all sound, and the bulk of the workstream loci are real and testable. The phase is implementable as written and will produce useful compiler closure.

To move from "Mostly Ready" to "Ready" the doc needs four edits:

1. **Reclassify `0241`** — either drop from scope or commit WS6 to the `eval` rewrite.
2. **Carve `0212` and `1345` out of phase exit criteria**, since both have load-bearing blockers in other categories.
3. **Tighten WS3 acceptance** to cover (a) `list.sort(reverse=...)`, (b) comprehension-form iterator destructure, and (c) the `Float`-in-`Comparable` spec drift.
4. **Replace WS6's soft acceptance with a per-fixture canonical-target table** that gives the implementer concrete pass criteria.

None of these are architectural rework. They are exit-criteria edits.
