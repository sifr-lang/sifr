# Independent Review — Wave 5, PR #3081 (pass 11)

**Range reviewed:** `f1c34cf9aaabadda546e670fca190decc580c935` (base, merged Wave 4) … `5e079b1276b77a5e8cf4ee76c430f2cb5e051d54` (head)
**Commits:** `2fc5c6f76`, `57ec6f110`, `912d7abb7`, `c18674571`, `a6b74ff9a`, `3fc566fd5`, `ee8e4b86c`, `d819a5a82`, `5e079b127`
**Pass-10 head was `d819a5a82`**, so the entire pass-10 remediation is the single commit `5e079b127` ("test(codegen): pin defaultdict set insertion order") — test-and-ledger only, 2 non-artifact files, +28/−1 (no emitter or lowering change).
**Working tree:** unchanged from session start and at exit — two dirty submodule pointers (`third_party/ruff`, `verification/.../leetcode`) and the untracked **empty** pass-11 artifact, none in the diff. No repo file was modified; all scratch lived in `/tmp` and was removed.

## Methodology

- Reused the base binary at `/private/tmp/wave5base` (verified `git rev-parse HEAD` = `f1c34cf9aa`) and the head release binary from this exact tree, running byte-identical sources through both with `SIFR_SYSROOT` pinned per binary; ~45 `check`/`build`/`run`/`emit` probes.
- **CPython as the semantic oracle** for every behavioral question, via a scripted harness that generated a matched `.sifr`/`.py` pair per probe and diffed the outputs.
- Read the full Wave 5 diff independently rather than only the head commit: the codegen interception and `build_entry_expr`/`preinsert_entry_expr` split (`collection_methods.rs:548-700`), both emitters (`defaultdict_iterable_mutations.rs`), the fail-closed dispatch (`collection_methods.rs:56-88`), the general-path clone changes (`literal_and_intrinsic_exprs.rs`, `collections_and_comprehensions.rs`, `leaves_and_plain_calls.rs`), `is_in_place_collection_method`, and the whole lowering side (`defaultdict_inference.rs`, `type_unification.rs`, `state_collection.rs`, `declaration_hint_safety.rs`, `defaultdict_refinement.rs`, `binding_hint_adoption.rs`, `control_flow.rs`, `statement_dispatch.rs`, `subscript_type.rs`, `method_type_collections.rs`, `mod_context.rs`).
- **Mutation-tested both pre-insertion sites independently** in an isolated `/tmp` clone (never the working tree, its own target dir, ruff submodule copied in): removed the set-path push, rebuilt the compiler, ran the full codegen suite and the native capability fixture; then reverted and did the same for the list-path push.
- Per instruction, the 679-fixture native sweep was not rerun; I did run the in-repo `test_e2e_pass` suite (which builds and runs the capability fixture, including the new function).

## Disposition of pass-10 finding

| Pass-10 finding | Disposition | Evidence |
|---|---|---|
| **F1** MEDIUM — the **set**-path pre-insertion was load-bearing but pinned by no test in either suite, so deleting it shipped a silent wrong answer undetected | **Resolved at both levels, exactly as specified** | Codegen: `defaultdict_order_independent_codegen_tests.rs:132-146` now pins `let __sifr_defaultdict_key = 1_i64;` < `groups.entry(__sifr_defaultdict_key.clone()).or_insert(HashSet::new());` < `let __sifr_defaultdict_set_items_0 =` < `let __sifr_defaultdict_bucket = groups.entry(__sifr_defaultdict_key)`, mirroring the list assertion at `:162-176`. Native: `default_set_insertion_precedes_iterable_arguments` (fixture `:185-192`) uses the self-observing variadic update `groups[1].update({len(groups)}, {1})` and asserts `len == 2`, `1 in`, `2 in`. |

**Mutation matrix re-derived independently** (isolated clone, rebuilt compiler each time):

| mutant | `cargo test -p sifr_codegen --lib` | native capability fixture |
|---|---|---|
| remove **set** pre-insert (`defaultdict_iterable_mutations.rs:98`) | **950 passed / 1 FAILED** (`variadic_set_bucket_updates_never_fall_back_to_cloned_receivers`) | **panics**: `assertion failed: (groups.entry(1_i64).or_insert(HashSet::new()).clone().len() as i64) == (2_i64)` |
| remove **list** pre-insert (`:44`) | **950 passed / 1 FAILED** (`iterable_mutation_evaluates_key_before_arguments_and_bucket_borrow`) | **panics**: `assertion failed: groups.entry(1_i64).or_insert(Vec::new()).clone() == vec![1_i64]` |

The pass-10 asymmetry is gone: each pre-insertion is now caught by both focused/full codegen **and** native coverage. The fixture assertion is genuinely result-changing, not just present — without the set pre-insertion `len(groups)` is `1`, so the set collapses to `{1}` (len 1) instead of `{1, 2}` (len 2); the `assert 2 in groups[1]` is the load-bearing element check.

Verifying the three specific requests:
1. **Set codegen test pins key < HashSet pre-insert < first materialized argument < bucket re-borrow** — yes, and confirmed against real `emit` output: `let __sifr_defaultdict_key = 1_i64;` → `d.entry(__sifr_defaultdict_key.clone()).or_insert(HashSet::new());` → `let __sifr_defaultdict_set_items_0 = …` → `let __sifr_defaultdict_set_items_1 = …` → `let __sifr_defaultdict_bucket = d.entry(__sifr_defaultdict_key).or_insert(HashSet::new());` → two `extend`s. Arguments still materialize left-to-right. All four `find()` calls resolve inside the first `update` statement (the only earlier statement, `groups[5].add(9)`, emits no key temporary and no `_i64 = 1` binding), so the ordering assertions are not accidentally satisfied by a later statement.
2. **Native fixture has a self-observing variadic set update whose asserted result changes if the pre-insertion is removed** — proven by mutation above.
3. **Either pre-insertion removal is caught** — proven by mutation above, at both the codegen and native levels.
4. No coverage was weakened by the rewritten test source: the old `retain(` count assertion still holds (`>= 4`, from two 2-argument `intersection_update`/`difference_update` calls) and the `!contains(".or_insert(HashSet::new()).clone().retain(")` no-fallback assertion is retained.

**All pass-1 through pass-9 behavioral fixes remain intact at this head.** Independently re-probed against CPython, all matching byte for byte: list `extend([len(d)])` → `[1]`; `extend([1 if 1 in d else 0])` → `1`; `extend(d.keys())` → `3`; `extend([len(d), len(d[2])])` → `32`; conditional key `d[len(d)].extend([len(d)])` → `22`; variadic `update({len(d)}, {1})` → `2`; `update(d.keys(), {99})`-style `intersection_update(d.keys())` → `12`; `symmetric_difference_update({len(d)})` → `11`; `difference_update({len(d)})` → `12`; loop form `for i in range(3): d[i].update({len(d)}, {i})` → `64`; non-Copy `String` key → `102`; tuple key `(2,"b")` → `12`; same-map cross-bucket `d[1].update(d[2])` → `12`; self-alias `d[1].update(d[1])` → `11` and `d[1].extend(d[1])` → `21`; zero-arg `d[7].update()` → `20` (key inserted); the full mutator chain `append,append,sort(reverse),insert,remove,reverse` → `39`; the whole live-receiver matrix (`sort`/`insert`/`remove`/`reverse`/`discard`/`remove`/`clear`) → `39`, with **zero** `or_insert(…).clone().<mutator>` occurrences in `emit`. Nested-function shadowing → `21`, `nonlocal` rebind → `1`, `dict.get` taint → `1`, all CPython-correct. Fail-closed dispatch confirmed at source (`collection_methods.rs:66-79` maps a `None` bucket-mutator lowering to `CodegenError`, never to a cloned receiver) and unreachable in practice — the only `None` arm I could aim at (`symmetric_difference_update` with 2 args) is rejected earlier by `SIFR-CALL-0001` with CPython-identical arity semantics.

## Findings

**None.** No actionable findings at this head.

## Verified clean (not findings)

- **Set-path materialization is borrow-clean and CPython-exact.** Every same-map/self-alias shape I could build after the pre-insert/re-borrow split compiles and runs correctly; no `E0499`/`E0502`, no `SIFR-BUILD-0005`. The pre-insert's `&mut` borrow ends at the statement boundary, which is what makes this sound.
- **Diagnostics are deterministic and single-per-site.** `SIFR-TYPE-0008` fires exactly once per conflicting subscript (`defaultdict key type conflict: expected 'int', got 'str'`) and once per conflicting `set.add`; a program with four conflict sites (loop key, augassign key, two read-position keys) produced exactly four diagnostics with a byte-identical, run-to-run stable ordering (md5-identical across three runs).
- **The new `set.add` element validation does not over-reject.** Its only behavioral delta versus base is `set[int].add(True)` → `SIFR-TYPE-0008`; Sifr already rejects `bool`→`int` everywhere else (`x: int = True`, `list[int].append(True)` both error identically on base and head), so this is consistency, not narrowing. `set[float].add(2)` and nominal-class adds behave identically on base and head.
- **The general-path clone additions fix a pre-existing leak rather than creating one.** `xs = [name, name]`, `{name}`, and `items = [a]` followed by `a.value()` all **failed to build on base** (`SIFR-BUILD-0005`) and now run correctly at head with CPython-matching answers. `clone_non_copy_name_expr_for_ir` returns early on `contains_affine_resource()` and clones only `HirExpr::Name` of non-Copy type, so no resource is duplicated and no move semantics are loosened for anything observable (element-mutation-through-container is not expressible in Sifr — `for mut it in …` is a parse error and `items[0].bump()` is a layout error, so the clone is observationally equivalent).
- **The `Expr::Compare | Expr::BoolOp => {}` non-walking arm is sound by construction, not just unreached.** Both forms always yield `bool` in inference *and* in lowering, so an inexact binding nested inside one cannot make inference disagree with the lowered type. Probed as an `append` argument (`v is None`), as a `BoolOp` (`v is not None and v > 0`), as a subscript key (`a < b`, `v == 1`): all check-clean, all CPython-correct on head, all identical to base.
- **The `bind_var`/`bind_call_result` short-circuit on unresolved defaultdict aliases is correct, not a lost refinement.** Replacing (rather than unifying) the seed at the declaration is what lets each fixpoint iteration re-accumulate the block's evidence order-independently; a second direct binding disqualifies adoption in the census anyway.
- **Adoption scoping is shadowing-safe.** `push_defaultdict_hint_adoption`/`pop_defaultdict_hint_adoption` are paired in `lower_stmts` (`statement_dispatch.rs:53`, `:126`), `can_adopt_defaultdict_hint` reads only `.last()`, and `safe_direct_assignment_names` requires exactly one direct binding plus no nested-block binding. `nested_block_binds_name` deliberately returns `false` for `FunctionDef`/`ClassDef`; I probed the residual case that suggests — a nested `nonlocal` rebind of the adopted name — and head is correct (`nonlocal` reset → `1`, CPython `1`), while a conflicting `nonlocal` rebind (`groups["a"]` against an `int` key) is turned into a clean `SIFR-TYPE-0008` at check where **base leaked `SIFR-BUILD-0005`/`E0308`**. Strictly better.
- **Ledger accuracy.** The head commit's only wording change — "…key-before-argument evaluation order, and default insertion before self-observing arguments **on both list and set paths**" — is now true as written, and I proved both halves by mutation. The pass-10 row is a fair summary of that review. The pre-existing-defect notes are accurate: I reproduced the `set(<slice>)` invalid-iterator-call class (`E0599: no method named collect found for struct Vec<i64>`) **identically on base and head** through an ordinary concrete `t = set(src[1:])`, exactly as the ledger describes; the general iterable list path (`src[1:]`, comprehension, concatenation, conditional) runs CPython-correct at head (`2343`).
- **Unseeded-alias boundaries fail closed, not open.** `extend`/`update` deliberately contribute no element-type evidence (only `add`/`append` do), so an alias with only `update` evidence is rejected at `check` with `SIFR-TYPE-0002` rather than guessed — deterministic, no wrong answer, and the same shape with one `add` compiles and runs correctly.
- **Generated-runtime panic surface unchanged.** The pre-insertion statement emits only `entry`/`or_insert`/`clone` — no indexing, `unwrap`, or `expect`.
- **Responsibility boundaries and file sizes.** Pre-insert construction lives once in `collection_methods.rs` and is threaded to both emitters. `collection_methods.rs` 833, `defaultdict_iterable_mutations.rs` 203, `control_flow.rs` 871, `statement_dispatch.rs` 829, `defaultdict_inference.rs` 289, codegen test file 177, fixture 213 — all under 900; guardrail PASS across 3027 files.

### Non-blocking observations (explicitly not findings; no action required for approval)

1. **Nested-return strictness narrows one previously-compiling shape.** The pass-1-mandated `is_unknown()` → `contains_unknown_or_any()` change in `finalize_nested_function_types` means a nested function inferred as `tuple[int, list[Unknown]]` (`def inner(): return (1, [])`) now errors `SIFR-TYPE-0004`, where base compiled and ran it. This is intended, documented, deterministic, and has a clean escape hatch (annotating `-> tuple[int, list[int]]` works; `return (1, [2])` works), and the two neighbouring shapes it also newly rejects (`return []`, `return {}`) were **build-failures with raw-rustc leakage on base**. Net fail-closed improvement.
2. **`binding_hint_adoption.rs` carries a pre-existing dead arm verbatim.** The second `Expr::Call` arm (the `collections.deque()` case) is shadowed by the first, which has an identical guard and returns `None` internally — so `"deque"` is unreachable. The code was moved byte-identically out of `statement_dispatch.rs`, so this is not a Wave 5 defect; worth folding into the first arm whenever that file is next touched.
3. **`list.remove` of an absent element is a silent no-op** where CPython raises `ValueError` (`[1,2].remove(5)` → len `2` on **base and head alike**). Wave 5 makes it newly reachable on defaultdict buckets, so by the same standard the ledger already applies to `sorted(<slice>)`, `set(<slice>)`, and out-of-range `list.insert`, one more sentence in the pre-existing-defect note would make that list complete. The wave claims nothing about this shape and the fixture only removes present elements, so nothing in the PR is inaccurate.

## Validation evidence (re-run independently at this exact head)

| Gate | Result |
|---|---|
| focused codegen (`defaultdict_order_independent`) | **12 passed; 939 filtered** |
| focused lowering (`defaultdict_order_independent`) | **12 passed; 909 filtered** |
| `cargo test -p sifr_codegen --lib` | **951 passed; 0 failed** |
| `cargo test -p sifr_lowering --lib` | **920 passed; 0 failed; 1 ignored** |
| `cargo test -p sifr -- test_e2e_pass` | **1 passed** (builds/runs the capability fixture incl. the new function) |
| `cargo clippy --workspace -- -D warnings` | **clean** |
| `cargo fmt --all -- --check` | **clean** |
| `scripts/check_hir_maintainability_guardrails.py` | **PASS** |
| `scripts/check_file_size_guardrails.py` | **PASS (3027 files, limit 900 lines)** |
| capability fixture `defaultdict_order_independent_inference.sifr` | `check`/`build`/`run` **exit 0**; new set assertion mutation-proven live |
| `0036_valid_sudoku.sifr` | `check`/`build`/`run` **exit 0**, no fixture change |
| set-pre-insert mutant | **1 codegen failure + native assertion panic** |
| list-pre-insert mutant | **1 codegen failure + native assertion panic** |
| 679-fixture native sweep | not rerun per instruction; accepted from the stated authoritative evidence |

Every figure matches the authoritative evidence supplied with the request.

## Verdict: **APPROVE**

The pass-10 remediation is precisely the minimum correct fix and it is complete: the set path now has the same emitted-order assertion as the list path (key → `HashSet::new()` pre-insert → first materialized argument → bucket re-borrow) plus a native self-observing variadic update whose asserted value — not merely its length — flips when the pre-insertion is deleted. I re-derived the mutation matrix from scratch in an isolated clone and both emitter paths now fail loudly at both the codegen and native levels, closing the exact asymmetry pass 10 raised, with no behavioral change and no weakening of the pre-existing assertions in the rewritten test source.

Reviewing the wave end to end rather than just the delta: every pass-1 through pass-9 fix holds, the full self-observing/same-map/self-alias/non-Copy-key/variadic/loop/conditional matrix matches CPython byte for byte, live bucket receivers are preserved for every mutator with zero cloned-receiver mutations in `emit`, the interception fails closed to a `CodegenError`, the lowering-side provenance and adoption census are shadowing-safe and monotonic, diagnostics are single-per-site and deterministic, the general-path clone additions repair rather than introduce raw-rustc leakage, and the three shapes where head differs from base outside the wave's scope are all base-leaking-to-head-diagnostic improvements. Every documented pre-existing-defect note reproduces on base as described, the ledger's newly corrected coverage sentence is now true as written, and every workspace gate is clean at this exact head. Zero actionable findings.
