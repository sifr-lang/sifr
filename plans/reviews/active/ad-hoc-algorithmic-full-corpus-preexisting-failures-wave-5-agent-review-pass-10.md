Review complete. No files were modified.

---

# Independent Review — Wave 5, PR #3081 (pass 10)

**Range reviewed:** `f1c34cf9aaabadda546e670fca190decc580c935` (base, merged Wave 4) … `d819a5a823ff70624ed349664ae5b3260c45f2c7` (head)
**Commits:** `2fc5c6f76`, `57ec6f110`, `912d7abb7`, `c18674571`, `a6b74ff9a`, `3fc566fd5`, `ee8e4b86c`, `d819a5a82`
**Pass-9 head was `ee8e4b86c`**, so the entire pass-9 remediation is the single commit `d819a5a82` ("fix(codegen): preserve defaultdict insertion order") — 5 non-artifact files, +40/−11.
**Working tree:** unchanged from session start — two dirty submodule pointers (`third_party/ruff`, `verification/.../leetcode`) and the untracked **empty** pass-10 artifact, none in the diff. This review created only `/tmp` scratch (removed).

## Methodology

- Built the head compiler from the exact tree (`cargo build --release -p sifr`); reused the base binary at `/private/tmp/wave5base` (verified `git rev-parse HEAD` = `f1c34cf9aa`), running probes through both binaries on byte-identical sources with `SIFR_SYSROOT` pinned per binary.
- ~40 `check`/`build`/`run` probes plus `emit` where generated Rust is the evidence; **CPython used as the semantic oracle** for every behavioral question.
- Read the full Wave 5 diff independently, not just `d819a5a82`: `collection_methods.rs` interception + `build_entry_expr`/`preinsert_entry_expr`, `defaultdict_iterable_mutations.rs`, the lowering-side provenance (`state_collection.rs`, `defaultdict_inference.rs`, `type_unification.rs`), the shared census (`declaration_hint_safety.rs`), `defaultdict_refinement.rs`, the adoption lifecycle in `statement_dispatch.rs`, `control_flow.rs`, `subscript_type.rs`, `mod_context.rs`.
- **Mutation-tested the pass-9 remediation itself** in an isolated `/tmp` clone of the repo (never the working tree, separate `CARGO_TARGET_DIR`): removed the list-path pre-insert, then the set-path pre-insert, rebuilt the compiler, and re-ran the full codegen suite, the full lowering suite, and the native capability fixture against each mutant.
- Per instruction, the 679-fixture sweep was not rerun.

## Disposition of pass-9 finding

| Pass-9 finding | Disposition | Evidence |
|---|---|---|
| **F1** MEDIUM — the bucket's implicit default-insertion happened *after* the arguments, so self-observing arguments read a stale map | **Resolved at the root, exactly as specified** | `collection_methods.rs:594-601` builds a separate `preinsert_entry_expr` from `__sifr_defaultdict_key.clone()`, threaded into both emitters and pushed as its own statement (`defaultdict_iterable_mutations.rs:44` list, `:98` set). `emit` shows the required five-step order verbatim: `let __sifr_defaultdict_key = …;` → `d.entry(__sifr_defaultdict_key.clone()).or_insert(Vec::new());` → `let __sifr_defaultdict_items = …;` → `let __sifr_defaultdict_bucket = d.entry(__sifr_defaultdict_key).or_insert(Vec::new());` → `__sifr_defaultdict_bucket.extend(…)`. All four pass-9 reproducers now match CPython: `d[1].extend([len(d)])` → **`[1]`** (was `[0]`); `d[1].extend([1 if 1 in d else 0])` → **`[1]`** (was `[0]`); `d[5].add(9); d[1].update({len(d)},{1})` → **`2`** (was `1`); `d[1].extend(d.keys())` after two other keys → **`3`** (was `2`). |

All five remediation requirements from the request verified independently:

1. **Order** — key → observable insertion → left-to-right argument materialization → re-borrow → mutate. Confirmed in `emit` for the list path and for all four set mutators; arguments still materialize as `__sifr_defaultdict_set_items_0`, `_1`, … in source order.
2. **Self-observing arguments match CPython** — `extend([len(d)])` `[1]`; membership `extend([1 if 1 in d else 0])` `[1]`; `extend(d.keys())` `3`; variadic `update({len(d)},{1})` `2`; variadic `update(d.keys(), {99})` **`4`** (CPython `4`); `update({len(d)})`→`1001`, `intersection_update({len(d),2})`→`1102`, `difference_update({len(d)})`→`1001`, `symmetric_difference_update({len(d)})`→`1001` — every one byte-identical to CPython. Loop form `for i in range(3): d[i].extend([len(d), i])` and conditional form `d[len(d)].extend([len(d)])` → `[1]` also correct.
3. **Borrow-clean after the split** — same-map cross-bucket `d[1].extend(d[2])` → `1`; self-alias `d[1].append(5); d[1].extend(d[1])` → `2`; `d[1].update(d[1], d[2])` → `2`; `d[1].extend([len(d), len(d[2])])` → `22`. No `E0499`/`E0502`, no `SIFR-BUILD-0005`. The pre-insert's `&mut` borrow ends at the statement boundary, which is what makes this work.
4. **Non-Copy keys cloned only where required** — `String` key: `let __sifr_defaultdict_key = key.clone();` (source name still usable afterward — verified by returning it), then exactly one `.clone()` for the pre-insert and a move for the re-borrow. `d[text].extend([len(d)])` → `[1]k`; tuple key `d[(1,"a")]` → `12`; both match CPython. No move/duplication error, no extra clone beyond the one the pre-insertion requires.
5. **Assertions sensitive to the corrected behavior** — the codegen assertion (`defaultdict_order_independent_codegen_tests.rs:150-160`) searches for `groups.entry(__sifr_defaultdict_key.clone()).or_insert(Vec::new());`, a string that does not exist in pass-8 output, and additionally orders `key < preinsert < items < bucket`. **Mutation-proven:** removing the list-path pre-insert makes `iterable_mutation_evaluates_key_before_arguments_and_bucket_borrow` FAIL (950 passed / 1 failed). The native assertion `assert groups[1] == [1]` is likewise live — flipping it to `[0]` panics `assertion failed: groups.entry(1_i64).or_insert(Vec::new()).clone() == vec![0_i64]` in the release build. (`return len(groups[1])` alone is *not* sensitive — `[0]` and `[1]` both have length 1 — so the inner `==` assertion is the load-bearing one, and it is genuine.)

**All pass-1/3/5/6/7/8 findings remain fixed at this head.** The capability fixture — which now pins intermediate values across list `append`/`extend`/`sort`/`reverse`/`insert`/`remove`/`clear`, set `add`/`update`/`intersection_update`/`symmetric_difference_update`/`difference_update`/`discard`/`remove`/`clear`, generally lowered iterables, cross-bucket same-map sources, borrowed-string storage, variadic set updates, and key-before-argument order — checks/builds/runs at exit 0. Spot-reprobed independently: full mutator chain `append,append,sort(reverse=True),insert,remove,reverse` → **`39`** (CPython `39`); nested-function shadowing → `11`; branch-shadowed rebind → `1`/`10`; key conflict → a single deterministic `SIFR-TYPE-0008` at a stable location.

## Findings

### F1 — MEDIUM (actionable): the **set**-path pre-insertion is load-bearing but pinned by no test in either suite, so removing it ships a silent wrong answer undetected

The pre-insertion is pushed in two independent places — `defaultdict_iterable_mutations.rs:44` (list) and `:98` (set). Only the list one is covered. Every assertion that mentions the pre-insert names the list path:

```
crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:151
    .find("groups.entry(__sifr_defaultdict_key.clone()).or_insert(Vec::new());")
```

`grep -rn "__sifr_defaultdict_key" crates/sifr_codegen/src/lib_codegen_tests/` returns five hits; none constrains a `HashSet::new()` pre-insert. The native fixture's set functions (`materialized_cross_bucket_iterables`, `borrowed_string_iterables`, `variadic_set_bucket_updates`) use only literal/constant sources or keys that a prior `add` already inserted, so the set pre-insert is a no-op in every one of them; the only self-observing native case, `default_insertion_precedes_iterable_arguments` (`:177-182`), uses `list.extend`.

**Mutation-proven asymmetry** (isolated `/tmp` clone, rebuilt compiler, separate target dir):

| mutant | `cargo test -p sifr_codegen --lib` | `cargo test -p sifr_lowering --lib` | native capability fixture |
|---|---|---|---|
| remove **list** pre-insert (`:44`) | **950 passed / 1 FAILED** | — | — |
| remove **set** pre-insert (`:98`) | **951 passed / 0 failed** | **920 passed / 1 ignored** | **exit 0** |

With the set pre-insert removed and every gate green, two check-clean, build-clean programs silently print the wrong answer:

| probe | head | set-preinsert-removed mutant | CPython |
|---|---|---|---|
| `d = defaultdict(set); d[5].add(9); d[1].update({len(d)}, {1}); len(d[1])` | `2` | **`1`** | `2` |
| `d = defaultdict(set); d[2].add(7); d[3].add(8); d[1].update(d.keys(), {99}); len(d[1])` | `4` | **`3`** | `4` |

Reproducer for the second:

```python
from sifr.collections import defaultdict


def solve() -> int:
    d = defaultdict(set)
    d[2].add(7)
    d[3].add(8)
    d[1].update(d.keys(), {99})
    return len(d[1])


def main():
    print(solve())   # head: 4 (correct); set pre-insert removed: 3
```

The behavior at head is correct — this is a regression-sensitivity gap, not a live defect. But it is exactly the class passes 7 (F3) and 8 (F4) raised and the wave accepted as actionable: a one-line deletion in the emitter reverts a fixed silent-wrong-answer class while 951 codegen tests, 920 lowering tests, and the capability fixture all stay green. Pass 9's own remediation instruction was to "pin the emitted pre-insertion statement in codegen and add a native case whose argument reads `len(d)`"; that was done once, for one of the two emitter paths.

Minimum fix: one codegen assertion pinning `<map>.entry(__sifr_defaultdict_key.clone()).or_insert(HashSet::new());` before `let __sifr_defaultdict_set_items_0 =` (mirroring `:150-160`), and one native case with a self-observing set source — `groups[1].update({len(groups)}, {1})` asserting `len == 2`, or `groups[1].update(groups.keys(), {99})` asserting `len == 4`. Both reproducers above are ready to use verbatim.

Relatedly, the ledger sentence "…variadic set updates, key-before-argument evaluation order, **and default insertion before self-observing arguments**" reads as covering the family; the coverage exists for the list path only. Adding the set case makes the wording true as written.

## Verified clean (not findings)

- **The `preinsert_entry_expr?` early-return cannot fail open.** `is_iterable_bucket_mutator` (`collection_methods.rs:570-579`) and the two branch guards (`:607`, `:620-628`) are textually identical predicates, so `Some` is guaranteed at both `?` sites; and even a future divergence degrades to `None` → `CodegenError` at `try_lower_registry_method_call_expr:66-79`, i.e. fail-closed, not a cloned bucket. I could not reach the error with any valid source.
- **No new base-object double-evaluation risk.** `lowered_object.clone()` is new, but the base of a `defaultdict`-alias subscript is always a local `Name`: a call-returning base is rejected at `check` (`SIFR-STDLIB-0001`, alias lost to `None | list[int]`), a class attribute is rejected (`SIFR-TYPE-0002`, `Any`), and a `list[dict[…]]` element loses the alias and takes the generic path — that shape leaks `E0599`/`E0624` **identically on base and head** (pre-existing, unrelated to this emitter). So the duplicated receiver is side-effect-free in every reachable case.
- **Generated-runtime panic surface unchanged.** The new statement emits only `entry`/`or_insert`/`clone` — no indexing, `unwrap`, or `expect`. No new panic path.
- **No new raw-rustc leakage.** Every borrow-heavy shape I could construct after the pre-insertion/re-borrow split builds cleanly. Zero-arg `update()`/`intersection_update()` still leave `__sifr_defaultdict_bucket` bound-and-unused in generated Rust (a warning-only artifact that predates this commit — the bucket `let` existed before), and `d[7].update()` correctly inserts the key: `len(d)*10 + len(d[7])` → `20`, CPython `20`.
- **Lowering-side inference/provenance is base-consistent and sound.** `bind_var`/`bind_call_result` short-circuit on unresolved defaultdict aliases; `lowering_inexact_bindings` is monotonic (insert-only, `state_collection.rs:148-150`), reset for nested locals/params (`:730-732`), and propagated outward only for the propagated `nonlocal` names (`:750-757`). The `Expr::Compare | Expr::BoolOp => {}` non-walking arm in `LoweringExactExprVisitor` (`defaultdict_inference.rs:60`) is a theoretical exactness hole; I probed it directly with an inexact `dict.get` binding inside a comparison as an `append` argument, as a `BoolOp`, and as a subscript key — **base and head identical and CPython-correct** in all three, so it is not reachable as a defect.
- **Adoption scoping is shadowing-safe and balanced.** `push_defaultdict_hint_adoption`/`pop_defaultdict_hint_adoption` are paired in `lower_stmts` (`statement_dispatch.rs:67`, `:126`); `can_adopt_defaultdict_hint` reads only `.last()`, so each lexical block is governed by its own census; `safe_direct_assignment_names` requires exactly one direct binding and disqualifies any name bound in a nested block (`declaration_hint_safety.rs:23-26`, `:71-103`). Verified by probe: nested-function shadow `11`, branch shadow `1`/`10`.
- **Deterministic diagnostics.** Mixed keys → exactly one `SIFR-TYPE-0008` ("defaultdict key type conflict: expected 'int', got 'str'") at `sub.slice.range()`. Non-adoptable shapes fall back to the pre-existing `SIFR-TYPE-0002`/`SIFR-TYPE-0005` rather than leaking.
- **Responsibility boundaries and file sizes.** Pre-insert construction lives once in `collection_methods.rs` and is threaded to both emitters — a good boundary. `collection_methods.rs` 833, `defaultdict_iterable_mutations.rs` 203, `control_flow.rs` 871, `statement_dispatch.rs` 829, `leaves_and_plain_calls.rs` 885 — all under 900. Non-blocking nit carried over from pass 9: the four iterable-mutator names are spelled out three times in `try_lower_defaultdict_index_method_call_expr` plus once in the emitter's `match`; a shared predicate would prevent the exact drift the `?` guards against.
- **Ledger accuracy.** The rewritten Wave-5 row is precise on every claim I could test, including the new "bind the key, perform the observable default insertion, evaluate and materialize every argument left-to-right, and then re-borrow the destination bucket" (true verbatim in `emit`) and every count. The only imprecision is the coverage claim discussed in F1.

## Validation evidence (re-run independently at this exact head)

| Gate | Result |
|---|---|
| `cargo test -p sifr_codegen --lib` | **951 passed; 0 failed** |
| `cargo test -p sifr_lowering --lib` | **920 passed; 0 failed; 1 ignored** |
| focused codegen (`defaultdict_order_independent`) | **12 passed; 939 filtered** |
| focused lowering (`defaultdict_order_independent`) | **12 passed; 909 filtered** |
| `cargo clippy --workspace -- -D warnings` | **clean** |
| `cargo fmt --all -- --check` | **clean** |
| `scripts/check_hir_maintainability_guardrails.py` | **PASS** |
| `scripts/check_file_size_guardrails.py` | **PASS (3027 files, limit 900)** |
| capability fixture `defaultdict_order_independent_inference.sifr` | `check` **0**, `run` **0**; new assertion mutation-proven live |
| `0036_valid_sudoku.sifr` | `check`/`build`/`run` **exit 0** |
| 679-fixture native sweep | not rerun per instruction; accepted from the stated authoritative evidence |

Every figure matches the authoritative evidence supplied with the request.

## Verdict: **CHANGES REQUESTED**

The pass-9 remediation is a genuine, complete root-cause fix, and it is the right one: the observable `defaultdict.__missing__` insertion is now its own statement between the key temporary and the argument materialization, the destination is re-borrowed afterward, and every self-observing shape I could construct — `len(d)`, membership, `d.keys()`, variadic set sources, non-Copy `String` and tuple keys, loop and conditional forms — now matches CPython byte for byte, with the same-map and self-alias cases staying borrow-clean and non-Copy keys cloned exactly once. Every pass-1 through pass-8 finding remains fixed, the whole prior mutator matrix still produces CPython answers, the lowering-side provenance and adoption scoping hold up under independent probing, and every workspace gate is clean at this exact head.

It is not approvable because of **F1**: of the two places the pre-insertion is emitted, only the list path is pinned. I proved the asymmetry by mutation — deleting `defaultdict_iterable_mutations.rs:44` fails a codegen test, while deleting `:98` leaves all 951 codegen tests, all 920 lowering tests, and the native capability fixture green, and silently regresses `d[1].update({len(d)}, {1})` from `2` to `1` and `d[1].update(d.keys(), {99})` from `4` to `3`. That is the same "coverage cannot detect the defect it was added for" class this wave has already accepted twice, and the request's own criterion — assertions sensitive to the corrected behavior, including variadic set sources — is met for only half the emitter. One codegen assertion on the `HashSet::new()` pre-insert plus one native self-observing set case (both reproducers above are ready to paste) closes it; no behavioral change is needed.
