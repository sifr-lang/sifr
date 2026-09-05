## Clarified Pass-4 Report — Wave 5, PR #3081

**Scope note on the artifact.** `…wave-5-agent-review-pass-4.md` contains only 3 lines: a note that a background e2e sweep reported `SWEPT 679` with one diff (`defaultdict_order_independent_inference.sifr`, base `SIFR-TYPE-0005` → head clean), and a one-sentence verdict naming F1/F2/F3 with no probe sources, no base/head transcripts, and no code paths. `…pass-5.md` and all four `*.agent.log` files are zero bytes. Everything below I re-derived and re-reproduced myself at the exact head; nothing is carried over from the pass-4 text except the three finding labels.

**Method.** Head binary built from `912d7abb7` (`target/release/sifr`, clean worktree at that commit); base binary at `f1c34cf9a` (`/private/tmp/wave5base/target/release/sifr`). Each driven with `SIFR_SYSROOT` pointing at its own tree. Every probe below was run through both binaries on byte-identical source (`/tmp/w5p4`), `check` then `run`. Per instruction I did not rerun the 679-fixture native sweep or any full library suite.

---

## F1 — BLOCKING: an inline list-slice element makes `d[k].append(...)` silently drop the element

### Probes (full source)

`z1.sifr`
```python
from sifr.collections import defaultdict


def solve(values: list[int]) -> int:
    d = defaultdict(list)
    d[1].append(values[0:2])
    return len(d[1])


def main():
    print(solve([3, 4, 5]))
```

`z2.sifr`
```python
from sifr.collections import defaultdict


def solve(values: list[int]) -> int:
    d = defaultdict(list)
    d[1].append(values[0:2])
    return len(d[1][0])


def main():
    print(solve([3, 4, 5]))
```

### Exact behavior

| Probe | Base `check` | Base `run` | Head `check` | Head `run` |
|---|---|---|---|---|
| `z1` | `no errors found` | `SIFR-BUILD-0005`, two `E0282` (`type annotations needed for HashMap<_, _>` at `src/main.rs:99`; closure param at `src/main.rs:111`) | `no errors found` | exit 0, prints **`0`** (correct: `1`) |
| `z2` | `no errors found` | `SIFR-BUILD-0005`, three `E0282` (`src/main.rs:99`, `:111`, `:115`) | `no errors found` | exit 0, prints **`0`** (correct: `2`) |

Head produces a wrong answer with no diagnostic at all. Base refused to produce a binary.

### Root cause chain

1. **Admission (Wave 5, this commit).** `crates/sifr_lowering/src/lower/nested_function_inference/defaultdict_inference.rs:47` — the new exact-expression allowlist admits a subscript whose slice is `Expr::Slice(_)` and recurses only into the *value*. `expression_inference.rs:616-622` models a list slice as `list[T]`, so the collected element type is `list[int]` and the declaration hint `__sifr_defaultdict_list[dict[int, list[list[int]]]]` is recursively complete and gets adopted. Head emits `let mut d: HashMap<i64, Vec<Vec<i64>>> = HashMap::new();` (`z1` emit line 98).
2. **Codegen bail.** `crates/sifr_codegen/src/intrinsic_method_emitters/collection_methods.rs:511` (`try_lower_defaultdict_index_method_call_expr`) is the only emitter that produces the correct `entry(k).or_insert(Vec::new()).push(v)` shape, but line 529 requires `try_lower_registry_exprs_strict(args)`. The strict registry path handles `HirExpr::Slice` **only when the sliced object is `Type::Str`** (`intrinsic_method_emitters/recursive_exprs.rs:802-818`), so a *list* slice returns `None` and the defaultdict-aware emitter aborts.
3. **Unsound fallback.** Lowering then falls through to the generic dict-indexed-list-append path (`crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_method_and_question_mark.rs:85-138`; the same shape also exists at `crates/sifr_codegen/src/string_char_cache.rs:138-197`), which emits `if let Some(__elem) = d.get_mut(&k) { __elem.push(v); }` — **no default-bucket insertion**. Key `1` is absent, so the append is a no-op. Head `z1` emit, lines 100-107:
   ```rust
   if let Some(__elem) = d.get_mut(&(1_i64)) {
       __elem.push({ /* slice block */ });
   }
   ```
4. **Why base was safe.** Base never adopted the hint, so the declaration stayed `let mut d = HashMap::new();` and rustc rejected it (`E0282`). The unsound fallback existed on base but was unreachable for this shape.

The get_mut fallback itself is pre-existing, and I proved that on **base**:

`z7.sifr`
```python
from sifr.collections import defaultdict


def solve(values: list[int]) -> int:
    d = defaultdict(list)
    d[2].append([9])
    d[1].append(values[0:2])
    return len(d[1])


def main():
    print(solve([3, 4, 5]))
```
Base: `check` clean, builds, prints **`0`** (correct `1`). Head: identical, prints **`0`**. The literal first write gives base's own hint path a concrete declaration, which reaches the same silent drop.

So: the *root* defect is codegen (missing default insertion / strict-only arg lowering); Wave 5's contribution is admitting the shape into hint adoption, which converts a hard build error into a **silent wrong result** for the plain `d = defaultdict(list)` idiom.

### Boundary probes (all run on both binaries)

| Probe | Source (body of `solve`) | Base | Head |
|---|---|---|---|
| `y1` | `d = defaultdict(list)`; `d[1].append(5)`; `return len(d[1])` | `1` | `1` — emits `d.entry(1_i64).or_insert(Vec::new()).push(5_i64)` (correct path) |
| `y3` | `chunk = values[0:2]`; `d[1].append(chunk)`; `return len(d[1][0])` | `2` | `2` — a named temp lowers through the strict path, so the correct emitter fires |
| `y4` | `d[1].append([values[0], values[1]])`; `return len(d[1][0])` | `2` | `2` — list literal is strict-lowerable |
| `z4` | `text: str`; `d[1].append(text[0:2])`; `return len(d[1])` | `1` | `1` — str slices *are* strict-lowerable (`recursive_exprs.rs:802`) |
| `z5` | `text: str`; `d = defaultdict(set)`; `d[1].add(text[0:2])`; `return len(d[1])` | `1` | `1` |
| `z6` | `d[1].append([9])`; `d[1].append(values[0:2])`; `return len(d[1])` | `2` | `2` — same key, so the bucket already exists and get_mut succeeds |
| `k1` | `d[text[0:2]].append(1)`; `return len(d[text[0:2]])` | `1` | `1` — slice in *key* position is unaffected |

The failure is exactly: inline **list** (non-str) slice as the `append`/`add` argument, on a key that does not yet exist.

### Severity

**Blocking.** It is a silent wrong answer under a check-clean, build-clean compile — the strongest possible breach of "if it compiles, it works." `d[k].append(items[i:j])` (window/chunk grouping) is a normal algorithmic idiom. The 679-fixture sweep passing is consistent with this: no corpus fixture appends an inline list slice into a `defaultdict(list)`.

### Required fix

Root cause first, in codegen (this also fixes the pre-existing `z7` shape):
- Make `try_lower_defaultdict_index_method_call_expr` (`collection_methods.rs:511-556`) able to lower arguments that the strict registry path rejects — fall back to `lower_stmt_expr_for_ir` for the value argument instead of `?`-bailing on `try_lower_registry_exprs_strict`; **and/or**
- Make the two generic dict-indexed append fallbacks (`stmt_expr_method_and_question_mark.rs:114-134`, `string_char_cache.rs:180-196`) use `entry(k).or_default()` when the indexed object's type resolves to a `__sifr_defaultdict_*` alias, keeping `get_mut` only for plain dicts.

Do **not** fix this by merely dropping the `Expr::Slice` arm from the allowlist: that hides the head symptom while `z7` stays silently wrong on both sides.

### Required tests

- Codegen assertion that `d = defaultdict(list); d[1].append(values[0:2])` emits `entry(…).or_insert(Vec::new()).push(…)` and never `get_mut` — pinned for both the list-slice and str-slice element forms.
- Native e2e case asserting the *value*, not just absence of errors: `groups[1].append(values[0:2])` then `assert len(groups[1]) == 1 and len(groups[1][0]) == 2`. The existing fixture `defaultdict_order_independent_inference.sifr` asserts only shapes whose bucket is created by a strict-lowerable write, so it cannot catch this.
- A regression for the pre-existing `z7` shape (missing key after a concrete seed on a different key).

---

## F2 — Raw rustc leak, newly reachable: `str` parameter as a `defaultdict(set)` element / membership operand

### Probes (full source)

`m2.sifr`
```python
from sifr.collections import defaultdict


def solve(text: str) -> int:
    d = defaultdict(set)
    d[1].add(text)
    if text in d[1]:
        return 1
    return 0


def main():
    print(solve("abc"))
```

`m3.sifr` — same, with a literal element written first
```python
from sifr.collections import defaultdict


def solve(text: str) -> int:
    d = defaultdict(set)
    d[1].add("x")
    d[1].add(text)
    if text in d[1]:
        return 1
    return 0


def main():
    print(solve("abc"))
```

### Exact behavior

| Probe | Base `check` | Base `run` | Head `check` | Head `run` |
|---|---|---|---|---|
| `m2` | `no errors found` | builds, prints **`1`** | `no errors found` | `SIFR-BUILD-0005`: `E0308 mismatched types … insert(text)` expected `String`, found `&String` (`src/main.rs:103`) **and** `E0277 the trait bound String: Borrow<&String> is not satisfied … contains(&(text))` (`src/main.rs:106`) |
| `m3` | `no errors found` | `SIFR-BUILD-0005`, byte-identical `E0308` at `src/main.rs:107` | `no errors found` | same `E0308` |

### Root cause

The generated bodies are *character-identical* except for the declaration:

base `m2`:
```rust
fn solve(text: &String) -> i64 {
    let mut d = HashMap::new();
    { d.entry(1_i64).or_insert(HashSet::new()).insert(text); () };
    if d.entry(1_i64).or_insert(HashSet::new()).contains(&(text)) { return 1_i64; }
    0_i64
}
```
head `m2`:
```rust
fn solve(text: &String) -> i64 {
    let mut d: HashMap<i64, HashSet<String>> = HashMap::new();
    …identical body…
}
```

The defaultdict intrinsic emitters pass a borrowed `str` parameter straight through with no owned conversion: `collection_methods.rs:546-552` (`add` → `insert(value)`) and `collection_methods.rs:557-595` (`try_lower_defaultdict_index_contains_expr` → `contains(&(element))`). On base the declaration was untyped, so rustc inferred `HashSet<&String>` and the program happened to compile; head's adopted concrete `HashSet<String>` exposes the missing `.clone()`/`to_string()`. `m3` proves the defect is pre-existing and reachable on base as soon as base's own hint path yields `HashSet<String>`.

### Severity

The root cause is pre-existing codegen, not Wave 5 inference — but the user-visible delta at this head is a regression: `m2` **ran correctly on base and fails to build on head**, with `check` reporting `no errors found` and two raw rustc errors surfacing at build. That breaches "no raw rustc leaks." The pass-4 one-liner classified F2 as ledger recording only; my evidence supports a stronger classification, because the comparable pass-3 item (`F4-R`) was already rejected at `check` on base, whereas this one worked on base.

### Required fix (preferred) or ledger record (minimum)

- Fix: apply the existing owned-conversion helper used elsewhere for `str` arguments to the two defaultdict emitters — `insert(text.clone())` / `contains(text.as_str())` (or route the element through `clone_owned_append_arg_expr_for_ir`-equivalent handling). This is a small, local change and removes the leak on both base-reachable (`m3`) and head-reachable (`m2`) forms.
- If deferred: record in `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` next to the existing `E0596` closure-mutability note (lines 279-286 of that file) that concrete `defaultdict(set)` adoption also makes a pre-existing **borrowed-`str`-element** codegen defect reachable, with the `m2`/`m3` pair as evidence that it reproduces on base once the declaration is concrete — so it is not mistaken for a Wave 5 inference defect later. State explicitly that the base-vs-head outcome differs for `m2`.
- Test either way: codegen assertion that `d[k].add(param_of_type_str)` emits an owned element, plus a native e2e case.

---

## F3 — MINOR: nested-function provenance asymmetry (types propagate out, taint does not)

### Probe (full source)

`p1.sifr`
```python
from sifr.collections import defaultdict


def solve(values: list[int]) -> int:
    d = defaultdict(list)
    x = 5

    def inner():
        nonlocal x
        x = values[0]

    inner()
    d[1].append(x)
    return len(d[1])


def main():
    print(solve([3, 4, 5]))
```

### Exact behavior

- Base: `check` → `no errors found`; `run` → `SIFR-BUILD-0005`, `E0308 mismatched types … __sifr_index_list.get(__sifr_index_norm).copied()` expected `i64`, found `Option<i64>` (`src/main.rs:106`).
- Head: `check` → `error[SIFR-TYPE-0002]: list.append() argument type 'None | int' is not compatible with list element type 'int'` at `p1.sifr:13:17`; `run` reports the same error.

### Root cause

`crates/sifr_lowering/src/lower/nested_function_inference/state_collection.rs:715-755` handles `Stmt::FunctionDef` with a cloned `nested_env`. Two things are asymmetric:

- **Outbound types do propagate:** lines 743-753 unify every refined nested type back into the enclosing env via `unify_name_binding`, including `nonlocal` names (the `local_bindings && !nonlocal_names` guard at line 750 deliberately lets nonlocals through).
- **Outbound provenance does not:** `nested_env.lowering_inexact_bindings` is discarded when the clone dies. The new inbound clearing loop added by this commit (lines 730-732) only removes inherited taint for nested locals/params.

So in `p1` the taint recorded for `x` inside `inner` (from `values[0]`, an inexact subscript) never reaches the outer env; the outer env still treats `x` as exact `int`, the hint `dict[int, list[int]]` is adopted, and real lowering computes `x: None | int` — precisely the pass-1 F1 / pass-3 F1-R class, re-entered through the nested-function boundary.

### Why MINOR, with the exact limit of what I could show

I could not construct a shape in this class where base compiles-and-runs correctly and head rejects, because every `nonlocal` rebinding shape is independently broken in codegen on both sides:

- `p3.sifr` (`nonlocal x; x = 7` — fully exact, no defaultdict involvement): base and head both `check` clean and both fail `run` with `E0594 cannot assign to 'x', as it is not declared as mutable` (`x = 7_i64;` against `let mut x: i64 = 5_i64;`, generated `src/main.rs:102`).

So today F3 is a latent soundness hole in the provenance invariant, not a user-visible regression: for `p1` head trades base's raw `E0308` at build for a deterministic `SIFR-TYPE-0002` at check, which is better diagnostics for a program that is broken either way. It becomes a real regression the moment the closure/nonlocal codegen defects are fixed.

Two controls confirming the boundary is otherwise intact:

- `p4.sifr` (`x = values[0]`; `d[1].append(x)`; `return len(d[1])`, no nesting): base `1`, head `1` — monotonic taint works in the flat case.
- `p5.sifr` (outer `x = values[0]`, nested `def inner(x: int): d[1].append(x)`, then `d[2].append(x)`) and `p7.sifr` (`d[1].append(7)`; `d[2].append(values[0])`): base and head emit the **identical** `SIFR-TYPE-0002` at the identical span. Base's own hint path already adopts from the first exact write, so "one exact write plus one inexact write" is pre-existing behavior, not a Wave 5 change.

### Required fix

Merge the nested env's `lowering_inexact_bindings` back into the enclosing env for exactly the names whose types are propagated out at `state_collection.rs:743-753` (i.e. non-param names that are either not nested-local or are declared `nonlocal`). That keeps provenance and type propagation symmetric and preserves the monotonic-taint property the pass-3 correction established.

### Required test

Lowering negative asserting the declaration is **not** adopted (`binding_and_constructor_types(...)` still `contains_unknown_or_any()`) for the `p1` shape — a nested `nonlocal` rebinding from an inexact subscript. The new `lowering_inexact_call_results_and_rebindings_do_not_force_declaration_hints` test (`expressions_tests/defaultdict_order_independent_inference.rs:151-174`) covers direct, name-propagated, loop-scoped and rebound forms but nothing crossing a nested-function boundary.

---

## Verified clean at this head (not findings)

- Focused suites reproduce the ledger's claim exactly: `cargo test -p sifr_lowering defaultdict_order_independent` → **11 passed**; `cargo test -p sifr_codegen defaultdict_order_independent` → **2 passed**.
- The new allowlist behaves as documented for the non-slice cases the pass-3 findings named: `y1`, `y3`, `y4`, `k1`, `z4`, `z5`, `p4` all agree between base and head and produce correct values.
- `p5`/`p7` show no change vs base for mixed exact/inexact writes on the same binding.
- Working tree is clean at `912d7abb7`; head binary was rebuilt from that commit before probing (the pre-existing `target/release/sifr` predated the final commit).
- Minor note, not a finding: union member order in the same diagnostic differs by path — `'None | int'` on the nonlocal-merged path (`p1`) vs `'int | None'` on the direct paths (`p5`, `p7`). Worth a determinism check if diagnostics are snapshotted.

## Required ledger changes

In `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md`, Wave 5 row:

1. The row currently claims the exact-expression allowlist admits "safe slices." That is false for list slices — it admits a shape that compiles to a silently dropped element (F1). Remove or qualify that claim until F1 is fixed.
2. Add the F2 note (borrowed-`str` element/membership defect newly reachable, base-vs-head outcome differs for `m2`, reproduces on base via `m3`) alongside the existing `E0596` closure-mutability note.
3. Do not represent pass-1 F1 / pass-3 F1-R as resolved: F3 shows the same inference/lowering disagreement still reachable across a nested-function boundary.
4. The three-line pass-4 artifact and the empty pass-5 artifact should be recorded the way earlier truncated passes were ("exceeded the bound with zero/partial output"), and this reconstruction referenced as the reviewable pass-4 content.

---

## Verdict: **CHANGES REQUESTED**

F1 is blocking on its own: at `912d7abb7`, `d[1].append(values[0:2])` checks clean, builds clean, and prints `0` instead of `2` where base refused to build — a silent wrong answer introduced by admitting list slices into hint adoption over an unsound codegen fallback whose root defect (`get_mut` with no default insertion, `try_lower_registry_exprs_strict` bail) is pre-existing and independently provable on base via `z7`. F2 is a check-clean → raw-rustc leak on a program base ran correctly; its root cause is pre-existing codegen, so a small owned-conversion fix or an explicit ledger record clears it, but it must not be left silent. F3 is minor today only because the nonlocal/closure codegen paths it needs are broken on both sides; the provenance asymmetry at `state_collection.rs:715-755` is real and should be closed in the same pass so the pass-3 monotonicity guarantee holds across nested functions. Everything else I probed at this head — the allowlist's non-slice behavior, monotonic taint in flat scopes, mixed exact/inexact writes, and both focused suites — is correct and unchanged from base.
