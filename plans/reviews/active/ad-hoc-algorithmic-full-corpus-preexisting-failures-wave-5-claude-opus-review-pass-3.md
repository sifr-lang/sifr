I have sufficient evidence. Here is the re-review.

## Re-review — Wave 5 PR #3081 (`57ec6f110` vs `f1c34cf9a`)

### Method
Differential probing of a head release binary against a freshly built base binary at `f1c34cf9a` (`/private/tmp/wave5base`), both driven with `SIFR_SYSROOT` at their own tree. Every probe below was run through both binaries on identical source.

### Pass-1 findings: correction status

| ID | Status | Evidence |
|---|---|---|
| F1 — adopted hints disagree with lowering | **NOT corrected (partial)** | pass-1 shapes fixed; equivalent shapes still regress — see below |
| F2 — no `Type::Tuple` arm | ✅ corrected | `defaultdict_refinement.rs:52` now uses canonical `body.contains_unknown_or_any()`, which recurses `Tuple`/`Union`/`Intersection`/`Alias`/`Set`/`Iterable`/`Result` (`type_rendering.rs:422-450`). `u1.sifr` (tuple key with `p.x`): head `no errors found`, builds, runs `1`; no `Unknown` leak |
| F3 — incomplete types escape return gate | ✅ corrected | gate is now recursive (`state_collection.rs:400`) with a structured `replace_inference_holes_with_any` recovery. `r1.sifr`: head emits **exactly 1** `SIFR-TYPE-0004`, no cascade, no raw rustc at `run` |
| F4 — ledger cites wrong code | ✅ corrected | ledger now states `SIFR-TYPE-0008` and records the `set.add` tightening |
| F5 — test gaps | ✅ corrected | 4 new negatives/positives added incl. `lowering_inexact_index_elements_do_not_force_declaration_hints`, `tuple_key_with_unresolved_member_is_not_adopted`, `incomplete_defaultdict_nested_return_reports_missing_annotation`, `nested_defaultdict_shadow_does_not_merge_with_outer_hint`; sibling test now pins both declaration types. Focused: **10/10** lowering, **2/2** codegen reproduced |
| F6 — placement/duplication | ✅ corrected | shared census extracted to neutral `lower/declaration_hint_safety.rs`; both `empty_plain_dict_inference.rs:7` and `defaultdict_refinement.rs:10` call `safe_direct_assignment_names` |

Also verified clean: seeded aliases excluded (`n2.sifr`, 2-arg `defaultdict(list, {...})` untouched); HIR constructor/declaration consistency pinned by `binding_and_constructor_types`; conflict cardinality exactly 1 for both key conflict (`c1`) and element conflict (`c2`); lexical/sibling shadow isolation holds in both source orders (`v1`, `u3`, `s1` — `s1` runs `2` correctly).

---

## F1-R — BLOCKING: the provenance gate covers only subscripts, so the same inference/lowering disagreement still rejects valid programs

`defaultdict_shape_expr_is_lowering_exact` (`nested_function_inference/defaultdict_inference.rs:22-39`) taints exactly two things: a non-slice `Expr::Subscript`, and a `Name` already in `lowering_inexact_bindings`. Any **call** whose inference model differs from lowering is invisible to it. `dict.get` and `list.pop` are exactly that: `expression_inference.rs:499-517` returns `*elem_ty` for `pop`, and `.get` likewise yields `V`, while real lowering yields `T | None`. So the hint is adopted from a type lowering will never produce, and the lowered write is then checked against it — pass-1 F1's root cause, unchanged.

All four regress identically to pass 1. Base **checks, builds, and runs**, printing `1`; head rejects at `check`:

```python
# g1.sifr — element side, direct call
d = defaultdict(list)
d[1].append(m.get(5))          # head: SIFR-TYPE-0002 'int | None' vs element type 'int'

# g5b.sifr — element side, list.pop
d[1].append(values.pop())      # head: SIFR-TYPE-0002 'int | None' vs element type 'int'

# g6.sifr — key side, name-propagated
k = m.get(5); d[k].add("a")    # head: SIFR-TYPE-0008 key conflict: expected 'int', got 'int | None'

# h2.sifr — key side, loop-scoped (group-by-lookup idiom)
for k in m:
    v = m.get(k); d[v].add(k)  # head: SIFR-TYPE-0008 key conflict: expected 'int', got 'int | None'
```

`h2` is the mainstream "group by a looked-up value" idiom, so the surface is comparable in width to the `d[k].append(items[i])` case pass 1 cited.

**Required fix:** either verify the adopted hint against the types real lowering computes for the same key/element expressions (pass-1's root-cause option), or extend the taint to every expression form whose prepass type can diverge from lowering — at minimum optional-returning calls (`dict.get`, `list.pop`, and an audit of the rest of `infer_attribute_call_type`). An expression-shape denylist that enumerates only `Subscript` cannot be sound; if the denylist approach is kept it needs an allowlist of provably-exact shapes instead.

## F2-R — BLOCKING: exactness taint is cleared by a later rebinding, while the lowered declaration type persists

`record_lowering_inference_exactness` (`state_collection.rs:148-154`) *removes* the name from `lowering_inexact_bindings` when a later assignment is exact. But lowering keeps the declaration's widened type from the first assignment, so the taint must be monotonic per binding.

```python
# g2.sifr
d = defaultdict(list)
x = values[0]     # x tainted  -> lowering declares x as 'int | None'
x = 5             # taint CLEARED, but x is still 'int | None' at lowering
d[1].append(x)    # head: SIFR-TYPE-0002 'int | None' vs element type 'int'
```
- base: checks, builds, runs → `1`
- head: rejected at `check`

This is a defect inside the chosen correction design, independent of F1-R's coverage gap. **Required fix:** make the taint monotonic (never remove), or key it to the binding's actual lowered declaration type rather than to a name-keyed boolean.

## F3-R — MINOR: no test covers either residual class

The new negatives pin only the `Subscript` provenance path (`lowering_inexact_index_elements_do_not_force_declaration_hints`). Nothing covers a divergent *call*-derived key/element (F1-R) or taint-clearing on rebind (F2-R) — the two classes that still break. Add both as lowering negatives asserting the declaration is *not* adopted.

## F4-R — MINOR (note, pre-existing root cause): a raw rustc leak is newly reachable without an annotation

```python
# n1.sifr
rows = defaultdict(set)
rows[1].add("a")
def peek(k: int):
    return rows[k]
return len(peek(1))
```
- base: rejected at `check` (`len(...) got 'Any'` + `SIFR-TYPE-0004`)
- head: `check` → `no errors found`; `run` → `SIFR-BUILD-0005` / raw `error[E0596]: cannot borrow 'peek' as mutable`

I verified this is **not** a Wave 5 codegen defect: the byte-identical E0596 reproduces on **base** as soon as the return type is written explicitly (`n4.sifr`, `def peek(k: int) -> set[str]`). Wave 5 only removes the check-time error that was masking a pre-existing closure-mutability codegen bug. Not blocking for this wave, but it should be recorded in the ledger as newly-reachable so it isn't mistaken for a Wave 5 regression later.

## F5-R — MINOR: ledger overstates the F1 resolution

`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` (Wave 5 row) describes the gate accurately ("depends on an indexed expression") but presents it as the response to pass-1's "indexed-expression inference/lowering disagreement" finding. Given F1-R and F2-R, the row should not claim that disagreement is resolved until the gate covers call-derived shapes and monotonic taint.

---

### Validation reproduced at this head
Focused lowering `10/10`, focused codegen `2/2`, both green. Head builds and runs correctly for `x1`, `u1`, `v1`, `u3`, `s1`, `g7`, `n2`. Per instruction I did not rerun the 679-fixture native suite. Note that the suite passing is consistent with F1-R/F2-R: no corpus fixture happens to feed a `dict.get`/`list.pop` result into a `defaultdict` key or element, so the suite does not cover this class.

## Verdict: **CHANGES REQUESTED**

Two blocking findings. F1-R is pass-1 F1 unfixed at the root — the correction narrowed the disagreement to `Subscript` provenance, while `dict.get` and `list.pop` diverge identically and still cause valid programs (`g1`, `g5b`, `g6`, `h2`) that base compiles and runs to be rejected at `check`. F2-R is a new defect in the correction itself: the exactness taint is non-monotonic, so a benign rebinding re-opens the same rejection (`g2`). F2, F3, F4, F5, F6 from pass 1 are all genuinely corrected, and I found no regression in the recursive-completeness, shadow-isolation, cardinality, seeded-alias, HIR-consistency, or census-placement work.
