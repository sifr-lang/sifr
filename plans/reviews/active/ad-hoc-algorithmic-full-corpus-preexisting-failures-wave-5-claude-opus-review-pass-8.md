Review complete. Here is the standalone report.

---

# Independent Review — Wave 5, PR #3081 (pass 8)

**Range reviewed:** `f1c34cf9aaabadda546e670fca190decc580c935` (base, merged Wave 4) … `3fc566fd59a84354e27efea8471a1b878ba2118e` (head)
**Commits:** `2fc5c6f76`, `57ec6f110`, `912d7abb7`, `c18674571`, `a6b74ff9a`, `3fc566fd5`
**Working tree:** clean apart from two dirty submodule pointers (`third_party/ruff`, `verification/.../leetcode`) and the untracked empty pass-8 artifact — none in the diff. No files were modified by this review.

## Methodology

- Built the head compiler from the exact tree (`cargo build --release -p sifr`); reused the pass-7 base binary at `/private/tmp/wave5base` (verified `git rev-parse HEAD` = `f1c34cf9aa`), running each probe through **both** binaries on byte-identical sources with `SIFR_SYSROOT` pinned per binary.
- ~60 probes across `check`/`build`/`run` plus `emit` where generated Rust is the evidence; CPython used as the semantic oracle where behavior (not just compilability) was in question.
- Read the full Wave 5 diff, not just `3fc566fd5`: the new codegen module, the mutator interception in `collection_methods.rs`, the general list/set-literal ownership change in `lower_expr/`, and the lowering-side inference/provenance modules.
- Re-ran independently at head: focused lowering **12/12**, focused codegen **10/10**, `sifr_lowering --lib` **920 passed / 1 ignored**, `sifr_codegen --lib` **949/949**, `cargo clippy --workspace -- -D warnings` **clean**, `cargo fmt --all -- --check` **clean**, HIR maintainability guardrails **PASS**, file-size guardrails **PASS (3027 files, 900-line limit)**, capability fixture `defaultdict_order_independent_inference.sifr` check/build/run **exit 0**.
- Per instruction, the 679-fixture sweep was not rerun.
- Also mutation-tested the capability fixture (on a `/tmp` copy) to prove its assertions are live in the release build and can actually fail.

## Disposition of pass-7 findings

| Pass-7 finding | Disposition | Evidence |
|---|---|---|
| **F1** generally lowered `extend`/`update` iterables fall through to a cloned bucket | **Resolved for single-argument mutators** | slice → `2`, nested slice → `1`, concat → `3`, conditional → `2`, comprehension-over-slice → `2` (base rejected all at `check`: `SIFR-TYPE-0002`). Also correct for `str`, tuple, `dict`, `range`, `sorted()`, `reversed()`, `dict.values()`, and the set-side equivalents. **But see F1 below — the fail-open path is still reachable for multi-argument `intersection_update`/`difference_update`.** |
| **F2** same-map cross-bucket sources leaked `E0499` | **Resolved, and better than asked** | `d[2].append(7); d[1].extend(d[2])` → base built and printed `0` (silently wrong); head prints **`1`** (correct), no `E0499`. Set analogue same. Self-alias cases correct too: `d[1].extend(d[1])` on `[5]` → `2`; `symmetric_difference_update(d[1])` → `0`. |
| **F3** native coverage insensitive to the mutations it claims | **Resolved** | `defaultdict_order_independent_inference.sifr:60-100` now asserts an intermediate value after *every* mutator (`assert groups[1] == [3, 1, 2]` → `[1,2,3]` → `[3,2,1]` → `[4,3,2,1]` → `[4,3,1]`; set: `len==2` → `1` → `1` with `8 in`/`10 in` → `0` → …). Mutation test proves live: flipping `:65` to `[9,9,9]` panics `assertion failed: groups.entry(1_i64).or_insert(Vec::new()).clone() == vec![9,9,9]`. Codegen assertions are no longer vacuous — the set test now pins `let __sifr_defaultdict_bucket = groups.entry(1_i64).or_insert(HashSet::new());`. |
| **F4** ledger wording / unrecorded newly reachable classes | **Resolved** | The "owned-value conversion" claim is now true (`chunk = [text]; d[1].extend(chunk)` → `11`; `d[1].update({text})` → `1`; both were `E0308`/`E0271` before). The "asserts actual results" claim is now backed by F3. The newly-reachable paragraph (`:288-295`) records `sorted(<slice>)` `E0425`, set-comprehension/`set(<slice>)` invalid iterator calls, and the out-of-range `list.insert` runtime panic; I verified each reproduces **identically on base** through ordinary concrete paths (`E0425`, `E0599` on `set.update({setcomp})`, and the same `insertion index` panic). The `E0499` and borrowed-string classes were fixed rather than recorded, which is the stronger outcome. |
| **F5** borrowed non-Copy names in simple list/set literals | **Resolved** | `s = "ab"; items = [s]` → base `E0382`, head `22`; `{s}` → base `E0382`, head `12`. No aliasing-semantics change: `inner=[1,2]; outer=[inner]; inner.append(3)` → **`23` on base and head** (value semantics, consistent with the pre-existing `append` path). |

## Findings

### F1 — BLOCKING: multi-argument `intersection_update` / `difference_update` on a `defaultdict` bucket silently drops the mutation

`defaultdict_iterable_mutations.rs:76` bails when a non-`update` method has more than one argument:

```rust
if method != "update" && args.len() != 1 {
    return None;
}
```

`collection_methods.rs:553-563` `return`s that `None` unconditionally, so `try_lower_defaultdict_index_method_call_expr` returns `None` and `try_lower_registry_method_call_expr` falls through to the generic rvalue lowering of `d[k]`, which **clones the bucket**. CPython's `set.intersection_update` and `set.difference_update` are variadic, so this is reachable source.

Both programs below are check-clean *and* build-clean on head and print the wrong answer; base rejected both at `check`, so Wave 5 makes them newly reachable:

| probe | head prints | CPython | base |
|---|---|---|---|
| `d[1].update({1,2}); d[1].intersection_update({1,2,3}, {2,3}); len(d[1])` | **`2`** | `1` | `SIFR-TYPE-0002` |
| `d[1].update({1,2,3}); d[1].difference_update({1}, {2}); len(d[1])` | **`3`** (mutation dropped entirely) | `1` | `SIFR-TYPE-0002` |

`emit` shows the exact cloned-bucket form pass 7 called out:

```
103: let __sifr_defaultdict_bucket = d.entry(1_i64).or_insert(HashSet::new());
109: d.entry(1_i64).or_insert(HashSet::new()).clone().retain(|__item| __set_arg_0.contains(__item));
111: d.entry(1_i64).or_insert(HashSet::new()).clone().retain(|__item| __set_arg_1.contains(__item));
```

This is **not** a pre-existing general-path gap: the identical calls on an ordinary concrete set are correct on base *and* head — `s: set[int] = {1,2}; s.intersection_update({1,2,3},{2,3})` → `1`; `s = {1,2,3}; s.difference_update({1},{2})` → `1`. The defect is specific to the new bucket emitter.

Reproducer:

```python
from sifr.collections import defaultdict


def solve() -> int:
    d = defaultdict(set)
    d[1].update({1, 2})
    d[1].intersection_update({1, 2, 3}, {2, 3})
    d[2].add(7)
    return len(d[1])


def main():
    print(solve())   # head: 2, CPython: 1
```

Fix: fold multiple arguments into successive `retain`s over each materialized set (the loop already exists for `update`), and — as pass 7 asked and this residual demonstrates — make the interception **fail closed**: once `methods::is_in_place_collection_method(value_ty, method)` is true (`collection_methods.rs:532`), no path may return `None` into a lowering that clones the bucket. Coverage should pin a multi-arg `intersection_update`/`difference_update` against `.clone().retain(` in codegen plus a value-asserting native case; nothing in either suite exercises the multi-arg shape today.

### F2 — MEDIUM: `extend`/`update` now evaluate the iterable **before** the subscript key, inverting Python's side-effect order

The materialization that fixed pass-7 F2 emits the items `let` before the bucket `let` (`defaultdict_iterable_mutations.rs:33-52` for lists, `:83-101` for sets), so the key expression's side effects run second.

```python
def k(mut log: list[str]) -> int:
    log.append("k"); return 1
def g(mut log: list[str]) -> list[int]:
    log.append("g"); return [1]

def solve() -> int:
    log: list[str] = []
    d = defaultdict(list)
    d[k(log)].extend(g(log))
    d[2].append(7)
    first = log[0]
    if first == "k":
        return 1
    return 2
```

Head prints `2`; CPython's `log` is `['k', 'g']` (verified) so the answer is `1`. `emit` shows the inversion directly:

```
111: let __sifr_defaultdict_items = (g(&mut log)).into_iter().collect::<Vec<_>>();
112: let __sifr_defaultdict_bucket = d.entry(k(&mut log).clone()).or_insert(Vec::new());
```

Newly reachable: base rejects this program at `check` (`SIFR-TYPE-0002`). The `append` path is unaffected (`d[k(log)].append(g(log))` → `1` on base and head), and the concrete-dict analogue cannot reach `extend` at all, so there is no pre-existing precedent for the inversion. Check-clean, build-clean, silently wrong — the same class the wave exists to close, narrower only in that it needs a side-effecting key expression.

Localized fix: bind the key to its own temporary *before* materializing the iterable, then build the entry receiver from that temporary. Order is preserved and the `E0499` fix is retained.

### F3 — LOW: the fail-open structure that produced F1 is still unguarded

`methods::is_in_place_collection_method` (`methods/mod.rs:21-41`) correctly centralizes the mutator allowlist — a genuine improvement over the two hand-maintained lists pass 7 flagged. But the interception it guards is still fail-open: every `?`/`return None` downstream of `collection_methods.rs:532` silently degrades to the cloned-bucket rvalue path with no diagnostic. F1 is one instance; `try_lower_defaultdict_iterable_collection` returning `None` (both strict and general lowering failing, or `registry_iterable_to_owned_iter_expr_from_lowered` hitting its heterogeneous-tuple `None`) is another. I could not reach the latter with valid source, but the failure mode is a silent wrong answer rather than an error, so it will not surface as a test failure if it ever becomes reachable. Fixing F1 by relaxing the arg-count check alone leaves this class open.

## Verified clean (not findings)

- **Receiver mutability / no cloned bucket** for every single-arg mutator: `emit` shows `d.entry(k).or_insert(Vec::new()).push(...)`, `__sifr_defaultdict_bucket.extend(...)`, `(*...).retain(...)`, `*__sifr_defaultdict_bucket = ...` — no `.clone()` in the mutation position.
- **Ownership/moves:** `chunk=[1,2,3]; d[1].extend(chunk)` → `6` (source preserved); `other={1,2}; d[1].update(other)` → `22`; cross-dict `e[3]` source preserved (`11`).
- **Raw-rustc leakage improved, not worsened:** head *fixes* three base leaks — `E0282` on bucket `pop()` (base) → `42` (head), `E0308` on mixed defaultdict keys (base) → deterministic `SIFR-TYPE-0008` ×2 (head), `E0308` on `set.add` element conflict (base) → `SIFR-TYPE-0008` (head, matching the ledger claim). I found no *new* leak: the `E0061` shape from a same-name rebind to a different `defaultdict` factory reproduces **identically on base**, so it is pre-existing.
- **Deterministic diagnostics:** key and element conflicts emit `SIFR-TYPE-0008` at stable locations with stable cardinality.
- **Generated-runtime panics within scope:** the new module emits no indexing/unwrap. The `list.insert` out-of-range panic and the silent `remove`-missing no-op reproduce identically on base through concrete paths and are recorded/pre-existing.
- **Provenance & shadowing boundaries:** nested `nonlocal` bucket append, nested-function-local `defaultdict`, branch-shadowed rebinds, and loop-local shadows all give base/head parity. `Expr::Compare`/`BoolOp`/slice-bound holes in `LoweringExactExprVisitor` (`defaultdict_inference.rs:47-60`) are not reachable with type-valid source.
- **Guardrails:** all pass. `leaves_and_plain_calls.rs` is at 885/900 — tight, informational only.
- The `--all-targets` clippy warnings are in `sifr_ipc`/`sifr_runtime`/`sifr_stdlib_manifest`, untouched by this wave and outside the project's stated gate.
- Ledger wording at `:328` is accurate on every claim I could test, including the mutator list it says the fixture pins.

## Verdict: **CHANGES REQUESTED**

Wave 5's core work is sound and the pass-7 remediation is real: all five pass-7 findings are genuinely fixed at the root, several base programs that silently printed wrong answers or leaked raw rustc errors are now correct, the native fixture's assertions are provably sensitive to the mutations they cover, and every workspace gate I re-ran is clean at this exact head.

It is not mergeable as-is because **F1** leaves one branch of the very fail-open path pass 7 asked to close: multi-argument `intersection_update`/`difference_update` on a `defaultdict` bucket is check-clean, build-clean, newly reachable, and silently wrong (`2` and `3` instead of `1` and `1`), with `emit` showing `.or_insert(HashSet::new()).clone().retain(` — while the ordinary concrete-set path handles the same calls correctly on base and head. Fix that (and prefer the fail-closed form so the class cannot recur), then address **F2** (bind the subscript key before materializing the iterable, restoring CPython's evaluation order) and **F3**. Both fixes are localized to `defaultdict_iterable_mutations.rs` and want one codegen assertion and one value-asserting native case each.
