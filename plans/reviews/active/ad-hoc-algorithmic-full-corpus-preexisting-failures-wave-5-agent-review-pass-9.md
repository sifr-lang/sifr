Review complete. No files were modified.

---

# Independent Review — Wave 5, PR #3081 (pass 9)

**Range reviewed:** `f1c34cf9aaabadda546e670fca190decc580c935` (base, merged Wave 4) … `ee8e4b86c42ce90b2aba2b811903f028fe345137` (head)
**Commits:** `2fc5c6f76`, `57ec6f110`, `912d7abb7`, `c18674571`, `a6b74ff9a`, `3fc566fd5`, `ee8e4b86c`
**Pass-8 head was `3fc566fd5`**, so the entire pass-8 remediation is the single commit `ee8e4b86c` ("fix(codegen): close defaultdict mutator fallbacks").
**Working tree:** unchanged from session start — two dirty submodule pointers (`third_party/ruff`, `verification/.../leetcode`) and the untracked empty pass-9 artifact, none in the diff. This review created only `/tmp` files.

## Methodology

- Built the head compiler from the exact tree (`cargo build --release -p sifr`); reused the base binary at `/private/tmp/wave5base` (verified `git rev-parse HEAD` = `f1c34cf9aa`), running probes through **both** binaries on byte-identical sources with `SIFR_SYSROOT` pinned per binary.
- ~60 `check`/`build`/`run` probes plus `emit` where generated Rust is the evidence; **CPython used as the semantic oracle** for every behavioral question.
- Read the full Wave 5 diff (not just `ee8e4b86c`): the new `defaultdict_iterable_mutations.rs`, the fail-closed interception in `collection_methods.rs`, the `methods::is_in_place_collection_method` allowlist, the two `?`-propagating call sites, `registry_iterable_to_owned_iter_expr_from_lowered`, and the `methods/list.rs` mutator lowerings that the entry receiver now flows into.
- Deliberately hunted for reachability of the new fail-closed `CodegenError` (~20 targeted shapes) and mutation-tested both new native assertions on a `/tmp` copy to prove they can fail.
- Per instruction, the 679-fixture sweep was not rerun.

## Disposition of pass-8 findings

| Pass-8 finding | Disposition | Evidence |
|---|---|---|
| **F1** BLOCKING — multi-arg `intersection_update`/`difference_update` silently drops the mutation | **Resolved** | Guard narrowed to `symmetric_difference_update` only (`defaultdict_iterable_mutations.rs:84`), and the retain now loops over every materialized arg (`:135-160`). Both pass-8 reproducers now correct: `update({1,2}); intersection_update({1,2,3},{2,3})` → **`1`** (was `2`, CPython `1`); `update({1,2,3}); difference_update({1},{2})` → **`1`** (was `3`). `emit` shows two `__sifr_defaultdict_bucket.retain(` calls and **no** `.or_insert(HashSet::new()).clone().retain(`. Verified for same-map cross-bucket args (`d[1].intersection_update(d[2], d[3])` → `0`, CPython `0`; single-arg `d[2]` → `2`), self-alias (`difference_update(d[1], {9})` → `0`), duplicate-arg (`update(d[2], d[2])` → `1`), generally-lowered slices (`intersection_update(nums[0:3], nums[1:4])` → `2`), and zero-arg no-ops (`update()`/`intersection_update()` → `2`, correct). `symmetric_difference_update` with 2 args is rejected upstream by `SIFR-CALL-0001`, so the retained arity guard is unreachable/defensive. |
| **F2** MEDIUM — iterable evaluated before the subscript key | **Resolved as specified** | A `__sifr_defaultdict_key` temporary is now bound first (`:37-42` list, `:89-94` set) and the entry receiver is built from it (`collection_methods.rs:580-593`). Pass-8 reproducer now prints **`1`** (was `2`); CPython `['k','g']`. Set analogue with variadic args also `1`. `emit`: `let __sifr_defaultdict_key = key(&mut log).clone();` precedes `let __sifr_defaultdict_items = …` precedes `let __sifr_defaultdict_bucket = d.entry(__sifr_defaultdict_key)…`. Args materialize left-to-right (`__sifr_defaultdict_set_items_0`, `_1`), destination borrow last, and same-map cross-bucket args stay borrow-clean (no `E0499`). **But the entry's *default-insertion* side effect still runs after the arguments — see F1 below.** |
| **F3** LOW — fail-open interception structure | **Resolved** | `try_lower_registry_method_call_expr` now returns `Result<Option<_>, CodegenError>` and pre-classifies bucket mutators via `registry_defaultdict_alias_parts` + `is_in_place_collection_method`, erroring rather than falling through (`collection_methods.rs:56-88`). The generic path is walled off in `try_lower_registry_method_call_expr_unchecked` with no defaultdict interception. Both call sites propagate with `?` (`print_calls.rs:109`, `stmt_expr_method_and_question_mark.rs:147`); `grep` confirms there are no others. I could not reach the error with any valid source (see "Verified clean"). |
| **F4** (pass-8 coverage ask) tests insensitive to variadic results and key-before-arg order | **Resolved** | `variadic_set_bucket_updates_never_fall_back_to_cloned_receivers` pins ≥4 `__sifr_defaultdict_bucket.retain(` and forbids `.or_insert(HashSet::new()).clone().retain(` — under pass-8 code it emitted 2 cloned retains, so it fails there. `iterable_mutation_evaluates_key_before_arguments_and_bucket_borrow` asserts `key < items < bucket` by string offset. Native fixture adds `variadic_set_bucket_updates()` (asserts `len==1`/`2 in`/`len==1`/`3 in`) and `iterable_mutation_evaluation_order()` (`assert log == ["key","items"]`). **Mutation-tested live:** flipping the expected variadic result to `2` panics `assertion failed: (variadic_set_bucket_updates() == (2_i64))`; flipping the order to `["items","key"]` panics `assertion failed: log == vec!["items".to_string(), "key".to_string()]`. |

All pass-5/6/7 findings re-verified fixed at this head: general-iterable extend/update for slices (`2`), concatenation (`3`), conditional (`2`), comprehension (`2`), `str` (`3`), `range` (`3`), `reversed` (`3`), `sorted` (`3`), `dict.values()` (`2`), `dict.items()`/`zip`/`enumerate` tuple sources (`2`/`2`/`3`), nested lists (`2`); same-map cross-bucket (`1` vs base `0`), self-alias (`2` vs base `1`); in-place `sort(reverse=True)` (`[3,2,1]` vs base `[1,3,2]`), `remove` (`[1]` vs base `[3,1]`), `pop(0)`, `discard`, `insert` (`[9,3]` vs base `[3]`); borrowed non-Copy storage (`1|1|val`, base check-rejected); ownership preserved (`3|3`, `2|2`); no aliasing change (`2|3`).

## Findings

### F1 — MEDIUM (actionable): the bucket's implicit default-insertion still happens *after* the arguments, so arguments that observe the same map read a stale map

The pass-8 F2 fix binds the **key expression** before the arguments, but the entry lookup — which is the Python-observable `defaultdict.__missing__` insertion — is fused with the destination borrow and therefore still emitted **last** (`defaultdict_iterable_mutations.rs:43-58` for lists, `:96-117` for sets). CPython's order for `d[k].extend(args)` is: evaluate `d`, evaluate `k`, **`d.__getitem__(k)` inserts the default**, then evaluate the arguments.

`emit` for the reproducer is unambiguous:

```rust
fn solve() -> String {
    let mut d: HashMap<i64, Vec<i64>> = HashMap::new();
    {
    let __sifr_defaultdict_key = 1_i64;
    let __sifr_defaultdict_items = (vec![d.len() as i64]).into_iter().collect::<Vec<_>>();   // reads d BEFORE insertion
    let __sifr_defaultdict_bucket = d.entry(__sifr_defaultdict_key).or_insert(Vec::new());   // insertion happens here
    __sifr_defaultdict_bucket.extend(__sifr_defaultdict_items.into_iter());
```

Two **newly reachable** instances — base rejects each at `check`, head builds cleanly and silently prints the wrong answer:

| probe | head | CPython | base |
|---|---|---|---|
| `d = defaultdict(list); d[1].extend([len(d)]); d[2].append(7); str(d[1])` | **`[0]`** | `[1]` | `SIFR-TYPE-0002` at `check` |
| `d = defaultdict(list); d[1].extend([1 if 1 in d else 0]); d[2].append(7); str(d[1])` | **`[0]`** | `[1]` | `SIFR-STDLIB-0001`/`SIFR-TYPE-0002` at `check` |

And one that was already wrong on base and is still wrong (differently) at head:

| probe | head | CPython | base |
|---|---|---|---|
| `d[5].add(9); d[1].update({len(d)}, {1}); len(d[1])` | **`1`** | `2` | `0` |
| `d[2].append(7); d[3].append(8); d[1].extend(d.keys()); len(d[1])` | **`2`** | `3` | `0` |

Reproducer:

```python
from sifr.collections import defaultdict


def solve() -> str:
    d = defaultdict(list)
    d[1].extend([len(d)])
    d[2].append(7)
    return str(d[1])


def main():
    print(solve())   # head: [0], CPython: [1]
```

This is specific to the new materialization, and the contrast proves it: the **`append` path is order-correct**, because there the argument is lowered inline into `d.entry(k).or_insert(Vec::new()).push(arg)` where Rust evaluates the receiver (and thus the insertion) first. `d[1].append(len(d))` leaks a raw `E0502` **identically on base and head** — that is a separate pre-existing leak, not this class.

Fix (localized, one statement, preserves both the `E0499` fix and the pass-8 key-order fix): after binding `__sifr_defaultdict_key`, emit the insertion as its own statement so the borrow ends before the arguments are evaluated, then re-borrow for the mutation:

```rust
let __sifr_defaultdict_key = <key>;
d.entry(__sifr_defaultdict_key.clone()).or_insert(Vec::new());   // observable insertion, borrow ends here
let __sifr_defaultdict_items = <args>;                            // args now see the inserted entry
let __sifr_defaultdict_bucket = d.entry(__sifr_defaultdict_key).or_insert(Vec::new());
__sifr_defaultdict_bucket.extend(__sifr_defaultdict_items.into_iter());
```

(The clone is only needed for non-`Copy` keys; `registry_defaultdict_key_arg` already distinguishes those.) Coverage should pin the emitted pre-insertion statement in codegen and add a native case whose argument reads `len(d)`, since nothing in either suite exercises a self-observing argument today.

## Verified clean (not findings)

- **Fail-closed error is not reachable with valid source.** I probed every `None` return in `try_lower_defaultdict_index_method_call_expr` and its callees: heterogeneous `tuple[int, str]` and `deque[int]` sources are rejected upstream (`SIFR-PROTO-0002`), non-`Clone` class elements by `SIFR-TYPE-0002`, `symmetric_difference_update` arity by `SIFR-CALL-0001`, and `lower_sort`/`lower_pop`/`lower_remove`/`lower_insert`/`lower_clear` arity mismatches are all pre-rejected by the type checker. Complex keys (call, ternary, `a + b`, tuple, `i % 2`, borrowed `str` param), `nonlocal` buckets, nested-function shadows, loop/`while`/`try` bodies, `print(d[1].pop())`, and repeated same-block mutations all lower successfully. Residual non-blocking note: if this path ever *does* become reachable, the message is an uncatalogued `CodegenError` string rather than a `SIFR-*` diagnostic — acceptable given fail-closed was explicitly requested, but worth a catalogued code eventually.
- **Block scoping / no temporary collision.** Each mutation emits its own braced `RustExpr::Block`, so two `d[make()].extend(...)` statements in one function each get a private `__sifr_defaultdict_key` (verified in `emit`). Argument-side bucket *reads* emit `d.entry(k).or_insert(...).clone()` inline and introduce no competing binding.
- **Receiver mutability / no cloned bucket** for every mutator: `push`, `insert`, `extend`, `retain`, `*bucket = …` (symmetric difference via `Deref` assign on `&mut HashSet`), `sort`/`reverse`/`remove`/`pop` all applied to the live entry. No `.clone()` in any mutation position.
- **Deterministic diagnostics:** `SIFR-TYPE-0008` for key conflicts (`d[1]`/`d["x"]`) and set-element conflicts (`add(5)`/`add("x")`), one diagnostic each at stable locations; `extend` element mismatch → single `SIFR-TYPE-0002`.
- **Generated-runtime panic risk in scope:** the new module emits only `entry`/`or_insert`/`extend`/`retain`/`symmetric_difference`/`cloned`/`collect` — no indexing, `unwrap`, or `expect`.
- **Inference/provenance boundaries unchanged and base-identical:** `extend`/`update` do not seed element-type inference (only `set.add`/`list.append` do, exactly as the ledger states) — `d[1].update({1,2,3})` alone is rejected on **base and head** alike with the same `SIFR-TYPE-0002`. Branch-shadowed rebinds, nested-function shadows, and `nonlocal` buckets all match base or improve on it.
- **Pre-existing, unchanged (not Wave-5 regressions):** key expressions are evaluated **twice** for `sort(reverse=True)`, `remove(v)`, and `pop(i)` because `methods::list.rs` duplicates the receiver IR (`lower_sort:139-166`, `lower_remove:278-313`, `lower_pop:224-274`) — `d[bump(log)].sort(reverse=True)` logs 2 entries on **base and head** (CPython 1). `d[1].append(len(d))` leaks `E0502` on **base and head**. Neither is newly reachable.
- **Ledger accuracy:** the rewritten Wave-5 row is precise on every claim I could test, including "bind the key before evaluating their arguments" (true as written — it does not claim full CPython insertion-order fidelity), "support variadic intersection/difference updates", "fail closed at codegen", and the coverage list. Counts verified: focused codegen 12/12, focused lowering 12/12, full codegen 951/951, full lowering 920+1 ignored.
- **Responsibility boundaries and guardrails:** mutation emitters isolated in a 199-line module; the allowlist centralized in `methods/mod.rs:21-41`; `collection_methods.rs` 821, `statement_dispatch.rs` 829, `leaves_and_plain_calls.rs` 885 — all under 900. Minor non-blocking nit: the four-name iterable-mutator list is spelled out three times inside `try_lower_defaultdict_index_method_call_expr` (`:570-579`, `:595`, `:607-614`) plus once more in the emitter's `match`; a shared predicate would prevent drift.

## Validation evidence (re-run independently at this exact head)

| Gate | Result |
|---|---|
| `cargo test -p sifr_codegen --lib` | **951 passed; 0 failed** |
| `cargo test -p sifr_lowering --lib` | **920 passed; 0 failed; 1 ignored** |
| focused codegen (`defaultdict_order_independent`) | **12/12** |
| focused lowering (`defaultdict_order_independent`) | **12/12** |
| `cargo clippy --workspace -- -D warnings` | **clean** |
| `cargo fmt --all -- --check` | **clean** |
| `scripts/check_hir_maintainability_guardrails.py` | **PASS** |
| `scripts/check_file_size_guardrails.py` | **PASS (3027 files, limit 900)** |
| capability fixture `defaultdict_order_independent_inference.sifr` | `check`/`build`/`run` **exit 0**; new assertions mutation-proven live |
| `0036_valid_sudoku.sifr` | `check`/`build`/`run` **exit 0** |
| 679-fixture native sweep | not rerun per instruction; accepted from the stated authoritative evidence |

Every figure matches the authoritative evidence supplied with the request.

## Verdict: **CHANGES REQUESTED**

The pass-8 remediation is genuine and complete on its own terms. All four remediation requirements are met: multi-argument `intersection_update`/`difference_update` now materialize every source and retain against the real bucket with no cloned receiver anywhere in `emit`; the key is bound to a temporary before the arguments, arguments materialize left-to-right, the destination borrow comes last, and same-map cross-bucket sources stay borrow-clean; the interception is genuinely fail-closed and I could not reach the error with any valid source; and both new tests are provably sensitive to variadic results and to key/argument order. Every pass-5/6/7 finding remains fixed, several base programs that silently printed wrong answers now print the CPython answer, and every workspace gate is clean at this exact head.

It is not approvable because **F1** leaves one branch of the evaluation-order class open: the bucket's implicit default-insertion is still emitted after the arguments, so an argument that reads the same map — `d[1].extend([len(d)])`, `d[1].extend([1 if 1 in d else 0])` — is check-clean, build-clean, newly reachable (base rejects both at `check`), and silently wrong (`[0]` instead of `[1]`). That is exactly the class this wave exists to close, and the `append` path's correctness on the same shape shows it is a property of the new materialization rather than a pre-existing limit. The fix is one additional emitted statement in `defaultdict_iterable_mutations.rs` plus one codegen assertion and one native case whose argument observes `len(d)`.
