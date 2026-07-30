I have complete evidence. Writing the review.

---

# Independent Final Review — Wave 5, PR #3081 (`f1c34cf9aaabadda546e670fca190decc580c935…a6b74ff9a`)

**Method.** Head binary built from the exact head `a6b74ff9a` (tree clean apart from two dirty submodule pointers and the empty pass-7 artifact, none in the diff); base binary reused from `f1c34cf9aa` at `/private/tmp/wave5base`. 100+ probes were run through **both** binaries on byte-identical sources (`SIFR_SYSROOT` pinned per binary), `check`/`build`/`run`, plus `emit` where generated Rust is the evidence. Per instruction I did not rerun the 679-fixture sweep. Independently re-run at this head: focused lowering **12/12**, focused codegen **7/7**, `cargo clippy --workspace -- -D warnings` (0 warnings/errors), `cargo fmt --check` clean, `check_file_size_guardrails.py` **PASS (3026 files)**, `check_hir_maintainability_guardrails.py` **PASS**, and the capability fixture `crates/sifr/tests/e2e/pass/defaultdict_order_independent_inference.sifr` check/build/run exit 0. No files were modified.

---

## Pass-6 items: three of four resolved at the root, one incompletely

### P6-B1 — in-place bucket mutation → **fixed for strictly-lowerable arguments, still broken for generally-lowered ones** (see F1)

The receiver is now correct: `collection_methods.rs:553-572` builds `entry(k).or_insert(<default>)` once and every mutator arm reuses it. `emit` confirms `d.entry(1_i64).or_insert(Vec::new()).extend(…)` and `(*d.entry(1_i64).or_insert(HashSet::new())).extend(…)` — no `.clone()`. Behavior across the whole matrix is correct, and notably **repairs the "context, not blockers" list pass 6 recorded** (programs base compiled to silently wrong answers):

| probe | base `f1c34cf9aa` | head `a6b74ff9a` | correct |
|---|---|---|---|
| `append(3);append(1);sort()` → first·100+len | `302` | **`102`** | 102 |
| `append(1);append(2);reverse()` | `102` | **`202`** | 202 |
| `append(1);clear()` | `1` | **`0`** | 0 |
| `append(1);append(2);remove(1)` | `2` | **`1`** | 1 |
| `append(1);extend([2,3])` | `1` | **`3`** | 3 |
| `insert(0,4)` only | `0` | **`1`** | 1 |
| `add(7);update({8})` | `1` | **`2`** | 2 |
| `add(7);add(8);discard(7)` | `2` | **`1`** | 1 |
| `add(7);clear()` | `1` | **`0`** | 0 |
| `add(7);pop()` → v·100+len | `701` | **`700`** | 700 |

Newly reachable order (mutate-then-type) is correct for every supported mutator; base rejected all of these with `SIFR-TYPE-0002`:

`extend`→`2`; `sort`→`103`; `sort(reverse=True)`→`303`; `reverse`→`303`; `insert(0,9)`→`903`; `insert(len(values),9)`→`903`; `remove`→`601`; `pop()`→`501`; `pop(0)`→`401`; `pop(-1)`→`501`; `clear`→`0`; `update`→`2`; `intersection_update`→`1`; `difference_update`→`1`; `symmetric_difference_update`→`2`; `discard`→`1`; `set.pop()`→`700`; `set.clear`→`0`; membership after `update`→`1`; loop/`while`/nested-`nonlocal` forms→`302`/`301`/`2`; `list[str]`/tuple/`dict.values()`/`sorted()`/genexpr/`range`/`reversed` arguments all correct.

Receiver precedence is sound (`(*…entry().or_insert(…)).extend(…)` — `RustExpr::Paren(Deref(…))` at `:594-595`). No move/borrow regression found for parameters or locals (`extend(chunk)`→`303`, `extend(values)`→`202`, `update(other)`→`202`). Return narrowing is safe: the `unreachable!()` unwrap at `collection_methods.rs:351-364` is unreachable for buckets because bucket `pop()` never narrows to non-`Option` (`v: int = d[k].pop()` is rejected `SIFR-TYPE-0002` on base **and** head, same-key and cross-key), and `pop()` on a missing key yields `None` with no panic.

### P6-B2 — ledger wording → **improved but two claims are still unsupported** (see F4)

### P6-B3 — `statement_dispatch.rs` headroom → **resolved.** 900 → **829** lines; `binding_hint_adoption.rs` is 72 lines. The moved block is byte-identical to the removed one (verified by diffing the extracted regions), so there is no behavior delta; `pub(in crate::lower)` / `pub(super)` visibilities are preserved (`binding_hint_adoption.rs:5,44,57`, re-exported at `statements.rs:63-64`), clippy is clean, and both focused suites pass. Largest touched file is `control_flow.rs` at 871 (pre-existing size, +35 this wave); `collection_methods.rs` is 795.

## Pass-5 items: all three still fixed at this head

| pass-5 finding | probe | base | head |
|---|---|---|---|
| F1 inline slice `append` | `d[1].append(values[0:2]); len(d[1][0])` | `SIFR-BUILD-0005`/`E0282` | **`2`** |
| F1 (other key first) | concrete write on key 2, then slice append on key 1 | `0` (silent wrong) | **`1`** |
| F1 (move) | `chunk=[1,2,3]; d[1].append(chunk); len(chunk)+len(d[1])` | `E0382` | **`4`** |
| F2 borrowed `str` set | `d[1].add(text); text in d[1]` | `1` | **`1`** |
| F2 (literal first) | `add("lit"); add(text); text in d[1]` | `E0308` | **`1`** |
| F3 nested `nonlocal` provenance | `inner()` appending `values[0]` | `1` | **`1`** (parity) |
| conflict determinism | list/set element conflicts | `SIFR-TYPE-0002` | `SIFR-TYPE-0002` |

`state_collection.rs:753-758` still propagates `lowering_inexact_bindings` outward for exactly the propagated names, and taint stays monotonic (`:148-152` insert-only).

---

## Actionable findings

### F1 — BLOCKING: `extend`/`update` bail to the cloned-bucket path, silently dropping the mutation

`collection_methods.rs:574-583` lowers the `extend` iterable with `registry_iterable_to_owned_iter_expr(self, iterable)?`, whose first statement is `try_lower_registry_expr_strict(expr)?` (`registry_helpers.rs:267`). When the argument is not strictly lowerable the `?` returns `None`, `try_lower_defaultdict_index_method_call_expr` returns `None`, and `try_lower_registry_method_call_expr` (`:108-115`) falls through to the generic rvalue lowering of `d[k]` at `recursive_exprs.rs:343-350`, which clones the bucket for every non-`int` value type. The mutation is applied to the temporary and discarded — no diagnostic. This is precisely the strict-bail that pass-5 F1 diagnosed for `append` and fixed with the two-stage fallback that the `append`/`add`/`insert`/`remove` arms still use (`:599-605`); the `extend` arm has no such fallback, and the set family at `:585-597` fails the same way when `try_lower_registry_set_method_call_expr` returns `None`.

All five programs below are **check-clean and build-clean on head, and print the wrong answer**; base rejected every one at `check` with `SIFR-TYPE-0002`, so this wave makes them newly reachable:

| probe | head prints | correct | head emit |
|---|---|---|---|
| `d[1].extend(values[0:2])` | **`0`** | `2` | `d.entry(1_i64).or_insert(Vec::new()).clone().extend({…slice…})` |
| `d[1].extend(values[0:2][0:1])` | **`0`** | `1` | same `.clone().extend(` |
| `d[1].extend(values + [9])` | **`0`** | `3` | same `.clone().extend(` |
| `d[1].extend(values[0:2] if len(values) > 1 else [1])` | **`0`** | `2` | `…or_insert(Vec::new()).clone().extend(if …` |
| `d[1].extend([v for v in values[0:2]])` | **`0`** | `2` | same `.clone().extend(` |

Full source of the first probe (`d[2].append(7)` supplies the later type evidence):

```python
from sifr.collections import defaultdict


def solve(values: list[int]) -> int:
    d = defaultdict(list)
    d[1].extend(values[0:2])
    d[2].append(7)
    return len(d[1])


def main():
    print(solve([1, 2]))
```

This is **not** a pre-existing general-path defect: the same arguments on a plain local list are correct on base and head — `out.extend(values[0:2])` → `2`, `out.extend(values + [9])` → `3`. `[v for v in values]` (no slice) also works on the bucket path (`2`), which pins the failure to the strict-lowering bail rather than to comprehensions. The defect is the same class and the same severity as pass-5 F1 and pass-6 B1: a check-clean, build-clean silent wrong answer that base refused to compile.

Required fix, matching the pattern already applied twice in this file: give the `extend`/`update` arms the same generally-lowered fallback as `:599-605` (or make the bucket-mutator interception fail closed — once `is_bucket_mutator` is true, never return `None` into a path that clones the bucket). Coverage should pin `d[k].extend(<slice>)` / `<concat>` / `<conditional>` against `.clone().extend(` in codegen plus a value-asserting native case.

Related maintainability risk in the same code: `is_defaultdict_list_bucket_mutator` / `is_defaultdict_set_bucket_mutator` (`:11-31`) are hand-maintained allowlists duplicating knowledge held in the lowering method tables. Any future in-place mutator omitted here silently reverts to the cloned-bucket wrong-result path rather than failing loudly — the same failure mode as F1, one name away.

### F2 — MEDIUM: same-dictionary cross-bucket `extend`/`update` now fails to build with a leaked `E0499`

With the in-place receiver, the `&mut` borrow of the map lives across the call, so an argument that also touches the same map cannot borrow-check:

| probe | base | head |
|---|---|---|
| `d[2].append(7); d[1].extend(d[2]); len(d[1])` | `0` (built, silently wrong) | `check` clean → `SIFR-BUILD-0005` + `error[E0499]: cannot borrow 'd' as mutable more than once at a time` |
| `d[2].add(len(values)); d[1].update(d[2]); len(d[1])` | `0` (built, silently wrong) | `check` clean → `SIFR-BUILD-0005` + `E0499` |

Distinct dictionaries are fine (`e[2].append(7); d[1].extend(e[2])` → `1`), and the self-aliasing family already leaks borrow errors pre-existing (`d[1].append(len(d[2]))` → `E0502` on base **and** head), so the family is not new — but this diff converts these two shapes from "builds, wrong answer" into "checks, then leaks a raw rustc borrow error." No correct program regressed, yet a check-clean program failing at build with an unmapped rustc diagnostic is exactly the failure mode this issue's waves treat as a defect. The localized fix is to materialize the argument into a temporary before taking the entry receiver, which makes the shape both correct and borrow-clean; if that is out of scope, it belongs in the ledger's newly-reachable section (`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:279-286`) beside the `E0596` note.

### F3 — MEDIUM: the new native coverage cannot detect the defect it was added for

`crates/sifr/tests/e2e/pass/defaultdict_order_independent_inference.sifr:60-70` and `:73-85` both end with `clear()` followed by a single `append(5)` / `add(13)`, and `main` asserts only `== 1` (`:98-99`). If **every** mutation before the `clear()` were silently dropped — the exact regression this commit fixes — the final state would still be `[5]` / `{13}` and both assertions would still pass. The assertions are provably insensitive to `extend`, `sort`, `reverse`, `insert`, `remove`, `update`, `intersection_update`, `symmetric_difference_update`, `difference_update` and `discard`; only `clear` and the trailing add/append are actually pinned. Additionally:

- The set codegen test's positive assertion is vacuous: `assert!(rust_code.contains(".entry(1_i64).or_insert(HashSet::new()))` (`lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:69`) matches the buggy `.clone().extend` form too (and the trailing `len()` read). Only the negative assertion at `:70` has force.
- Nothing in either suite pins the receiver for `sort`, `reverse`, `insert`, `remove`, `pop`, `discard` or the `*_update` family — the bulk of the newly added matrix — in codegen or by value. All are correct today (verified above by native probe), but a regression would be caught by no test.

Suggested minimum: reorder the fixture so a length- or first-element-sensitive assertion follows the ordering mutators (e.g. assert the post-`sort`/`reverse` first element before any `clear()`), and add one value-sensitive native case per mutator family.

### F4 — LOW: two Wave 5 ledger claims are still not evidence-backed

`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:322`:

1. *"…iterable-driven extend/update families, **apply owned-value conversion to stored values**…"* — the extend/update arms apply no owned conversion, unlike `append`/`add`/`insert`/`remove` which route through `clone_owned_append_arg_expr_for_ir`. Head evidence: `chunk = [text]; d[1].extend(chunk)` → `SIFR-BUILD-0005` + `E0308`; `d[1].update({text})` → `E0271` (`type mismatch resolving <IntoIter<&String> as IntoIterator>::Item == String`), both check-clean, both rejected by base at `check`. (The underlying defect is pre-existing — identical `E0271`/`E0308` on concrete `dict[int, list[str]]` / `dict[int, set[str]]` buckets on base **and** head — so this is a wording/reachability issue, not a new root defect.)
2. *"the expanded capability e2e … **asserts actual results across** list append/extend/sort/reverse/insert/remove/clear and set add/update/…"* — refuted by F3.

Also unrecorded in the newly-reachable section (`:279-286`, which lists only the `E0596` closure case): the borrowed-string `extend`/`update` leaks above; the `E0499` shapes in F2; `sorted(<slice>)` → `E0425`, `{v for v in <slice>}` → `E0599`, `set(<slice>)` → `E0599` (all pre-existing, verified identical on base and head for plain locals); and the pre-existing `list.insert` out-of-range **runtime panic** (`d[1].insert(5, 9)` panics `insertion index (is 5) should be <= len (is 0)` on base *and* head), whose reachable orderings this wave widens.

---

## Not findings

- `try`/`finally` around a bucket mutation → `E0425 cannot find type 'Error'`, and `d[k] += [1, 2]` → `E0368`: both reproduce identically on base and head for concrete `dict[int, list[int]]`. Pre-existing, unrelated to this diff; the emitted mutation line itself is correct.
- Inference does not collect element evidence from `extend`/`update` arguments (`d[1].extend([1, 2])` alone is `SIFR-TYPE-0002` on base and head). The emitter is deliberately broader than the inference; conservative and safe.
- The duplicated `Expr::Call` guard arms in the moved `empty_collection_literal_kind` (`binding_hint_adoption.rs:17-39`) make the `collections.deque()` arm dead, since the first arm's identical guard wins. Pre-existing and moved verbatim by this diff.
- `sort(key=lambda …)` on a bucket is `SIFR-TYPE-0005` at check — a deterministic diagnostic, not a leak.
- `is_defaultdict_*_bucket_mutator` living at the top of `collection_methods.rs` rather than in `registry_helpers.rs` beside the other alias-keyed helpers is a placement nit only; file sizes and both guardrails pass.

---

## Verdict: **CHANGES REQUESTED**

The wave's core work is sound and the pass-6 remediation is largely a genuine root-cause fix: the alias-aware receiver mutates the real bucket for every mutator I could reach, the newly reachable mutate-then-type orderings all produce correct values, ten shapes that base compiled to silently wrong answers are now correct, all three pass-5 corrections survive at this head, provenance and diagnostics are unchanged, `statement_dispatch.rs` headroom is genuinely resolved by a byte-identical extraction, and every workspace gate I re-ran is clean.

It is not mergeable as-is because of **F1**: the `extend`/`update` arms still `?`-bail into the cloned-bucket path whenever the iterable needs general lowering, so five newly reachable check-clean, build-clean programs — `d[k].extend(<slice>)`, `<nested slice>`, `<concat>`, `<conditional>`, `<comprehension over a slice>` — print `0` instead of `2`/`3`/`1`, with `emit` showing `.or_insert(Vec::new()).clone().extend(`. That is the same defect class, the same code path, and the same severity as pass-5 F1 and pass-6 B1, and the plain-local path handles the identical arguments correctly, so it is specific to this emitter rather than a pre-existing general gap. Fix it with the two-stage fallback already used one screen away (or make the interception fail closed), then address **F2** (materialize the argument, or record the `E0499` reachability), **F3** (make the native assertions sensitive to the mutations they claim to cover), and **F4** (scope the two ledger claims and record the newly reachable classes).
