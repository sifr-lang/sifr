# Independent Final Review — Wave 5, PR #3081 (`f1c34cf9a…c18674571`)

**Method.** Head binary built from the exact head `c18674571` (worktree clean apart from two dirty submodule pointers and the empty pass-6 artifact, neither in the diff); base binary built from `f1c34cf9a` in `/private/tmp/wave5base`. Every probe below was run through both binaries on byte-identical source in `/tmp/w5p7`, `check` then `run`, plus `emit` where the generated Rust is the evidence. Per instruction I did not rerun the 679-fixture native sweep. I re-ran the focused suites (`sifr_lowering defaultdict_order_independent` → **12 passed**; `sifr_codegen defaultdict_order_independent` → **5 passed**), `cargo clippy --workspace -- -D warnings` (clean, exit 0), `cargo fmt --check` (clean), and `scripts/check_file_size_guardrails.py` (**PASS**, 3025 files). No files were modified.

---

## Pass-5 findings: all three are fixed at the root

### F1 — inline list-slice `d[k].append(...)` silently dropping the element → **FIXED**

`crates/sifr_codegen/src/intrinsic_method_emitters/collection_methods.rs:529-535` replaces the `try_lower_registry_exprs_strict(args)?` bail with a single-argument strict attempt falling back to `lower_stmt_expr_for_ir`, then routes the result through the repo's established `clone_owned_append_arg_expr_for_ir` (`stmt_support_emitter/nested_subscript_assignment_helpers.rs:20-31`), the same helper used by every other append/insert emitter (`collection_methods.rs:110,208`, `recursive_exprs.rs:117,194`, `stmt_expr_method_and_question_mark.rs:112,217`, `string_char_cache.rs:179`). The `entry(k).or_insert(...)` shape is therefore preserved instead of falling through to the `get_mut` path.

| Probe | Base | Head |
|---|---|---|
| `z1` (`d[1].append(values[0:2])`, `len(d[1][0])`) | `SIFR-BUILD-0005` / `E0282` | **2** (correct) |
| `z7` (concrete write on key `2` first, then slice append on key `1`) | **0** (silent wrong) | **1** (correct) |

The pre-existing `z7` shape pass 5 explicitly warned about is fixed too — the fix was made in codegen, not by narrowing the allowlist. Emitted head Rust for `z1` is `d.entry(1_i64).or_insert(Vec::new()).push({…slice block…})`, with no duplicated statements and no double-`clone()` from the two-stage lowering. Bonus, verified: `b2` (`chunk = [1,2,3]; d[1].append(chunk); len(chunk)+len(d[1])`) was `E0382 borrow of moved value` on base and prints **4** on head.

### F2 — borrowed `str` in a concrete `defaultdict(set)` → **FIXED**

`collection_methods.rs:535` gives owned storage (`insert(text.clone())`) and `:586-597` passes a borrowed-parameter `Name` element straight to `contains` instead of double-borrowing it.

| Probe | Base | Head |
|---|---|---|
| `m2` (`d[1].add(text)`, `text in d[1]`) | ran, `1` | **1** (was `E0308` + `E0277` at the pre-fix head) |
| `m3` (literal element first) | `SIFR-BUILD-0005` / `E0308` | **1** |

Head emit: `d.entry(1_i64).or_insert(HashSet::new()).insert(text.clone())` and `.contains(text)`. Boundary probes all clean on both binaries: `n1` (`str` param into `defaultdict(list)` + `in`), `b1`, `b3` (borrowed `list[str]` param appended and membership-tested), `n4`/`n5` (`mut` parameter present in scope, exercising the `mut_borrowed_params` arm) — all `1`/`3` as expected, no raw rustc.

### F3 — nested `nonlocal` provenance asymmetry → **FIXED**

`crates/sifr_lowering/src/lower/nested_function_inference/state_collection.rs:753-758` propagates `lowering_inexact_bindings` outward for exactly the names whose refined types are propagated, i.e. the pass-5 required fix verbatim. Taint remains monotonic (`state_collection.rs:148-152` only ever inserts; the inbound clearing at `:730-732` touches only the nested clone). `p1` at head now `check`s clean and fails at build with base's own `E0308` — i.e. **identical to base**, the hint is no longer adopted. The regression test pass 5 asked for exists: `expressions_tests/defaultdict_order_independent_inference.rs:183`.

### Other verified-clean behavior at this head

`u1` (conflicting element types) → deterministic `SIFR-TYPE-0002` on both, no union declaration is adopted. `c1` (function-call key) → `1` on both, no silent drop through a non-strict *index*. `s1`, `e1`, `e2`, `e3`, `e5`, `g1`, `g3`, `h1`, `h2`, `k1`, `k2`, `b4` → byte-identical outcomes on base and head. Refactor quality is good: `declaration_hint_safety.rs` is a genuine de-duplication of the plain-dict census (`empty_plain_dict_inference.rs` shrinks 109→10 lines), and `defaultdict_inference.rs` / `type_unification.rs` are clean responsibility splits.

---

## Actionable finding

### B1 — BLOCKING: order-independent adoption makes silently-dropped `d[k].extend(...)` / `d[k].update(...)` newly reachable

`try_lower_defaultdict_index_method_call_expr` intercepts exactly two shapes — `("__sifr_defaultdict_list", "append")` at `collection_methods.rs:546` and `("__sifr_defaultdict_set", "add")` at `:554`. Every **other** in-place bucket mutation falls through to the generic rvalue lowering of `d[k]` at `intrinsic_method_emitters/recursive_exprs.rs:343-350`, which for non-`int` buckets emits `entry(k).or_insert(<default>).clone()`. The method then mutates that **temporary clone**, and the mutation is discarded with no diagnostic.

That codegen defect is pre-existing. What this diff changes is *reachability*: base only adopted a concrete declaration when an element-typing write preceded the mutation, so the "mutate first, type later" order was rejected at `check`. Head's order-independent collection supplies the element type from a later write, so the program now compiles.

**Probe `e6.sifr` (simplest form):**
```python
from sifr.collections import defaultdict


def solve() -> int:
    d = defaultdict(list)
    d[1].extend([1, 2])
    d[2].append(7)
    return len(d[1])


def main():
    print(solve())
```
| | `check` | `run` |
|---|---|---|
| base `f1c34cf9a` | `error[SIFR-TYPE-0002]: list.extend() requires elements with generated Rust Clone support` | same error, no binary |
| head `c18674571` | `no errors found` | exit 0, prints **`0`** (correct: `2`) |

**Probe `e4.sifr`** (`d[1].extend([1, 2])`; `d[2].append(len(values))`; `return len(d[1])`) — base `SIFR-TYPE-0002`; head prints **`0`** (correct `2`). Head emit:
```rust
let mut d: HashMap<i64, Vec<i64>> = HashMap::new();
d.entry(1_i64).or_insert(Vec::new()).clone().extend((vec![1_i64, 2_i64]).into_iter());
```

**Probe `g2.sifr`** — same class on the set side:
```python
def solve(values: list[int]) -> int:
    d = defaultdict(set)
    d[1].update({7})
    d[2].add(len(values))
    return len(d[1])
```
base: `error[SIFR-TYPE-0002]: set element type 'Any' does not have a statically known hash/equality capability`; head: `no errors found`, prints **`0`** (correct `1`). Head emit: `d.entry(1_i64).or_insert(HashSet::new()).clone().extend(…)`.

This is the same defect class and the same severity as pass-5 F1 — a check-clean, build-clean silent wrong answer that base refused to compile — and it is a direct consequence of this wave's order-independent adoption. The `append`/`add` fix cured two members of the family; `extend` and `update` remain. Preferred fix, consistent with the one already applied: extend the alias-aware emitter (or intercept at the receiver) so that **any** in-place mutation of a `__sifr_defaultdict_{list,set}` bucket keeps the `entry(k).or_insert(...)` receiver instead of the `.clone()` rvalue form. Required coverage: a codegen assertion that `d[k].extend(...)` / `d[k].update(...)` never emits `.clone().extend(`, plus a native e2e case asserting the resulting length.

*Context, not blockers* (identical on base and head, so pre-existing and not expanded by this diff): `s2`, `e3`, `e5` (`append` then `extend` → `1`, correct `3`); `g1`, `g3` (`insert` dropped); `k1` (`sort()` dropped — prints `31`, correct `13`); `k2` (`clear()` dropped). These share B1's root cause, so fixing B1 at the receiver would clear them; if the fix is scoped narrowly to `extend`/`update`, the remainder should be recorded in the ledger's newly-reachable-defect section (`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:270-286`) the way the `E0596` closure-mutability note is.

### B2 — MINOR: ledger wording overstates the emitter fix

`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:322` states "the defaultdict mutation emitter preserves `entry(...).or_insert(...)` default insertion when a stored argument needs general expression lowering". That is true only for `append`/`add`; as B1 shows, other bucket mutations do not preserve it and are newly reachable. The Wave 5 row should scope that claim to `list.append` / `set.add` and record the `extend`/`update` reachability delta.

### B3 — MINOR: `statement_dispatch.rs` is exactly at the 900-line cap

`crates/sifr_lowering/src/lower/statements/statement_dispatch.rs` is 900 lines after this diff's `+4` (the guardrail fails at `> 900`, `scripts/check_file_size_guardrails.py:13,171`, and does pass today). It is now the largest file in the diff and has zero headroom; the next line added there fails the gate. Worth a small responsibility split in the follow-up rather than treating it as a fresh problem later.

---

## Not findings

- `p1`'s post-fix outcome — check-clean, then base's own `E0308` at build — is a pre-existing `nonlocal`/closure codegen defect at exact parity with base, not a regression; the F3 fix correctly restores non-adoption.
- The dead `if crate::helpers::is_option_type(ty) { … }` branch at `recursive_exprs.rs:351-354` (both arms identical) predates this wave (`5f9404c9d3`); the file is untouched by this diff.
- The new `set.add` element-conflict diagnostic (`expressions/method_type_collections.rs:616-628`) mirrors the existing `list.append` check and is gated on `is_assignable_to` with an `Any`/`Unknown` escape; `u1` confirms deterministic behavior and the full suites pass. Its code (`SIFR-TYPE-0008`) differs from `list.append`'s (`SIFR-TYPE-0002`) for an equivalent error — cosmetic, and the ledger declares it intentional.
- Test adequacy for the *fixed* findings is sound: `lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:23,33,43` pin both the entry/or_insert shape and the absence of `get_mut`, and the e2e fixture asserts values rather than mere absence of errors (`crates/sifr/tests/e2e/pass/defaultdict_order_independent_inference.sifr:38,45,52` with `:67-69`).

---

## Verdict: **CHANGES REQUESTED**

All three pass-5 findings are genuinely fixed at the root, verified against the base binary, with adequate focused and native coverage; the wave's inference work, provenance monotonicity, alias boundaries, diagnostics, refactor placement, and workspace gates are clean. The blocker is B1: this wave's order-independent adoption newly admits `d[k].extend(...)` and `d[k].update(...)` programs that base rejected at `check`, and they compile to a mutation of an `entry(...).or_insert(...).clone()` temporary — a silent wrong answer (`e6` prints `0` instead of `2`, `g2` prints `0` instead of `1`) under a check-clean, build-clean compile. That is the same defect family and the same severity as pass-5 F1, cured for `append`/`add` but not for the sibling mutators; it must be fixed at the receiver (or explicitly recorded) before merge, together with the B2 ledger scoping and, ideally, the B3 headroom cleanup.
