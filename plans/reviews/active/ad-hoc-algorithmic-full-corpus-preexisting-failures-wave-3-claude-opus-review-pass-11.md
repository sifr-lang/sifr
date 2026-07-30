Baseline worktree and all probe files removed; `git status` is byte-identical to the session-start snapshot.

# Wave 3 Review — pass 12 (exact published head `ec5aab945` vs base `ea119724e`)

## Verdict

Zero actionable findings. The merge is clean, the mechanism is correct after the current-main merge, and every behavioural delta I could produce against a freshly built base is either identical or a strict improvement. All claims below are from my own runs against a `ea119724e` worktree I built and then removed.

## Merge and scope

- `ec5aab945` is a true merge (`69d7534d0` + `ea119724e`), not an evil merge: `git diff ea119724e ec5aab945 -- crates/` is exactly the 16 Wave 3 files, and the main-side delta it absorbed (`git diff 69d7534d0 ec5aab945`) touches only `verification/areas/rust_interop/**` plus two `crates/sifr_package/src/tests/*` interop files. Zero overlap with the compiler change, so the merge cannot perturb the mechanism passes 6/8 approved.
- Diff scope is `crates/**` + `plans/**` only; `git diff --check` clean; no `.gitmodules`, submodule pointer, matrix, stable-claim, or profile changes — consistent with the issue's own constraint at `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:301-303`.

## Correctness verification (own differential runs, ~40 probes)

**Order-independent exact-write inference.** `record_dict_write` (`state_collection.rs:650-660`) records shapes before any unification and collapses to `None` on the first disagreement; the gate at `statement_dispatch.rs:128-131` retains a candidate only when `binding_hints.get(name) == exact_dict_write_hints.get(name)`. Read-before-write, `in`-before-write, `.items()`-before-write, `setdefault`-before-write, and pass-to-typed-parameter-before-write all now adopt correctly (5 probes; base fails 4 of them at `build` or `check`). Literal-typed writes (`d[1] = 2`) collapse to `Int` before recording, so pass 6 §3's "literals silently disable adoption" concern does not materialise — `emit` shows `let mut d: HashMap<i64, i64>`.

**Lexical binding / shadowing safety.** Pass 1's two repros (sibling `if`/function-level, loop-local/function-level) are clean and produce correct results; base fails `build` on both. Pass 3 §1's nested-function shadowing false `SIFR-TYPE-0008` is *also* resolved by the exact-write gate — `d: dict[str,int]` outer + `d = {}`/`d[2]=3` in a nested `def` now checks clean and runs (`2`), where base fails `build`. `self.data[...]`, walrus, chained `a = b = {}`, tuple targets, module-level `table = {}`, and nested classes are all ineligible by construction and byte-identical to base.

**Deterministic conflict handling.** All conflict families preserve base behaviour exactly: hard `int`/`str` (`SIFR-TYPE-0008`), numeric widening `int`/`float` (`SIFR-TYPE-0008`), divergent `if`/`else` branch shapes (`SIFR-TYPE-0008`), unhashable-key widening (exactly one `SIFR-TYPE-0002` + the `0008`), and the nested-function-block key conflict (`SIFR-TYPE-0002` "not compatible", identical message on both trees).

**Missing-key / augassign semantics.** `disqualify_exact_dict_writes` (`state_collection.rs:662-664`) is sticky and propagates through every merge site — I confirmed all `clone → analyze → merge_env_types` sites (`state_collection.rs:594-683`, `compound_statement_inference.rs:158-242`) take the clone from the current parent with no intervening parent mutation, so the documented `source ⊇ target` invariant holds, and `try`'s `orelse` is covered (`compound_statement_inference.rs:177-179`). The word-tally loop form still reports `SIFR-TYPE-0005 unsupported operand type(s) for +: 'Any' and 'int'`, identical to base. `d[1]=2; d[1]+=1` and chained `d[1][2] += 1` are byte-identical to base.

**Declaration-local codegen types.** `local_binding_registry.rs:8-14` drops ambiguous names and clears the stale `none_widened` entry with them; every consumer degrades to `expr.ty()`. Same-named sibling declarations emit `HashMap<String, i64>` and `HashMap<i64, i64>` (pinned by the two codegen tests) and the int/str variant now builds where base emits `let v: i64 = "abc".to_string()`.

**Nearest-declaration patching.** Reverse iteration + `pending.remove` (`container_literal_specialization.rs:273-289`) is sound because patches drain after every statement (`statement_dispatch.rs:184-188`); I could not construct a mis-patch across sibling blocks, loop bodies, `try`/`match` arms, or a `NestedFunction` sibling.

**Real `0001_two_sum` native path.** `verification/areas/algorithmic_compatibility/corpora/leetcode/src/0001_two_sum.sifr` checks, builds, and runs (exit 0), emitting `let mut prevMap: HashMap<i64, i64> = HashMap::from([])`; base fails `build` with 3 rustc errors.

## Regression evidence

| Check | Result |
|---|---|
| Full `verification/runner/e2e/run_e2e_pass.sh` on head | **677 passed, 0 failed** (`report_signature=981c6b2203ccc554`) |
| `cargo test -p sifr_lowering --lib` | **898 passed, 1 ignored** — matches the ledger claim exactly |
| `cargo test -p sifr_codegen --lib` | **934 passed** |
| Focused wave-3 modules | 13/13 lowering, 2/2 codegen |
| `clippy --workspace -- -D warnings`, `cargo fmt --check` | exit 0 / clean |
| HIR maintainability / file-size guardrails | PASS / PASS (3010 files, limit 900) |
| Full 411-fixture corpus `check` lane, head vs base | **403 pass / 8 fail on both, zero set differences** |
| All 58 corpus fixtures containing `= {}`, native `run`, head vs base | **head 58/58 OK; base 57/58** — the single difference is `0001_two_sum` |

`clippy --all-targets -p sifr_codegen` fails on both trees (14 head / 15 base) in files this diff does not touch; the authoritative workspace gate is green.

Touched-file sizes: `statement_dispatch.rs` 890, `scope_and_function_types.rs` 866, `control_flow.rs` 858, `mod_context.rs` 779, `state_collection.rs` 735, `container_literal_specialization.rs` 350, test file 162, `empty_plain_dict_inference.rs` 109, `local_binding_registry.rs` 47 — all under cap. Extracting `register_local_body_binding_types` into its own module is responsibility-based, not line-count chunking.

## Ledger accuracy

The Wave 3 row is accurate on every claim I checked: status `review`, PR #3077, the gate description, the augassign disqualification, the unchanged list/set/deque boundary, the `SIFR-TYPE-0008` preservation, 898/1-ignored and 934, the pass 1→8 narrative including the discarded passes 2/5/7, and the two newly documented pre-existing boundaries at `:274-283`. I independently confirmed the second one: `d = {}` + a nested `def` + `d[1]=nested()` + `d[2]=2.5` fails `build` with `E0308` **identically on base and head**, so the row's caveat is factually correct rather than defensive.

---

## Non-blocking observations

1. **Gate treats "absent from both hint maps" as eligible.** `statement_dispatch.rs:128-131` retains a name when both `binding_hints` and `exact_dict_write_hints` return `None`, while `inferred_binding_hint` (`mod_context.rs:405-410`) reverse-searches *enclosing* frames. Safety rests on `analyze_block` binding every candidate name, but `analyze_block` breaks early on `inference_stmt_always_exits` (`state_collection.rs:422-424`), whose notion of exiting is not literally the lowering CFG's. I could not construct a reachable divergence — every form where inference breaks (`Return`/`Raise`/exhaustive `If`/`With`/`Match`/`Try`) is also always-exiting in `cfg.rs:81-233`, so the following statement is skipped as unreachable during lowering too. Adding an explicit `binding_hints.contains_key(name)` to the `retain` would make the gate independent of that alignment.
2. **Container-patch order within one compound statement** (pass 3 §5) still has no invariant comment. With `pending.remove`, only the first-visited branch of a single `If` is patched when both branches declare the name. Verified masked: `if/else: xs = []` (and the dict variant) followed by a later write fails `build` identically on base and head, because that shape lacks a `let` hoist on both trees.
3. **Dead match arm** at `statement_dispatch.rs:62-74` (pass 1 §4, still open). The first `Expr::Call` arm's guard matches any zero-arg call and returns `None` for a non-`Name` callee, so `"deque"` is never produced. Behaviourally irrelevant — a list/set/`deque()`/`defaultdict(int)` fixture emits byte-identical Rust on both trees — but it means the "deque boundary unchanged" clause holds vacuously.
4. **Redundant arm** at `empty_plain_dict_inference.rs:351-352`: `Stmt::FunctionDef(_) | Stmt::ClassDef(_) => false` followed by `_ => false`. Reads as intentional documentation; a comment would carry that better than a duplicate arm.
5. **Codegen ambiguity gate is type-agnostic but only dict-tested.** `local_binding_registry.rs:8-14` changes behaviour for any same-named `Let` pair with differing types; the int/str sibling probe fails `build` on base and works on head — a genuine improvement with no test pinning it. The two new codegen tests cover only the dict shapes.
6. **Ledger continuity for the next update:** the row says "implementation commit `1ad7389dd` is the first published head" and does not yet record the current-main merge `ec5aab945` (the head actually under review) or pass 10's transient-529 disposition — both postdate the doc commit, and this issue's own precedent names discarded passes explicitly.
7. **Untracked artifacts** in `plans/reviews/active/`: zero-byte `*.claude.log` files for passes 1–8/10/11 and a zero-byte pass-11 `.md`. Not part of the diff, so not a PR defect, but the recurring pass-1 §7 / pass-3 §7 / pass-6 §4 note stands.
8. **Pre-existing, unchanged:** an empty dict with *no* subscript evidence (`d = {}` + `len(d)`) still emits `Box<dyn Any>` and fails `cargo build` with 2 rustc errors, identically on base and head. The `empty_plain_dict_without_concrete_evidence_remains_dynamic` test pins the lowering half only; the generated-Rust half remains a separate gap.

APPROVED
