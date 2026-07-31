# Wave 8 Correction Review (Pass 3) — Standalone Report

**Target:** branch `codex/algorithmic-recursive-option-constructor`, exact head `a73b14b85fea671c9206fef8342258cd591033e3`, base `4c867d1cda`, draft PR #3089
**Scope:** entire PR diff — 11 files, +535/−126 (3 commits: `f546f563b`, `a1633ea4f`, `a73b14b85`)
**Files modified by me:** none. `git status --porcelain` at end is identical to session start (` M third_party/ruff`, ` M .../leetcode` — untracked `.DS_Store`/`__pycache__` only, plus one `src/sifr_output/` build directory left by my own `sifr run` of `0894`; no tracked file in either submodule differs, and the submodule-ownership guardrail passes). The 0-byte untracked `…wave-8-parent-claude-opus-review-pass-3.md` placeholder was present at session start and is untouched.

## Verdict: APPROVE — 0 actionable findings

---

## 1. Pass-2 requested changes: all four verified implemented

### Request 1 — focused coverage + narrowed negative ✅
`crates/sifr_codegen/src/lib_codegen_tests/recursive_node_codegen_tests.rs:168`, `:288`

All four requested shapes are pinned in `test_nested_recursive_constructor_maps_named_optional_arguments_to_boxes`, and each assertion string matches what the compiler at head actually emits (I reproduced every one against `./target/debug/sifr emit`):

| Requested shape | Fixture fn | Pinned assertion |
|---|---|---|
| non-option recursive value | `wrapChild` | `TreeNode::new(9_i64, Some(Box::new((child).clone())))` + `!contains("Some(Box::new((Some(Box::new((child).clone()))")` |
| `own`-convention nested forward | `wrapOwnedNested` | `TreeNode::new(7_i64, node.map(|__sifr_option_value| Box::new(…)))` |
| field-projection nested forward | `wrapFieldNested` | `TreeNode::new(8_i64, (node.left).as_deref().cloned().map(…))` |
| keyword form | `wrapKeyword` | `TreeNode::new(10_i64, (node).clone().map(…))` |

The pass-2 line-230 negative is correctly narrowed: it is now `!contains("TreeNode::new(5_i64, node.map(") && !contains("TreeNode::new(6_i64, node.map(")` — anchored to the two borrowed call sites (`wrapBorrowed`/`wrapBorrowedNested`) and therefore able to coexist with the `own` positive at value `7`. The unnarrowed form pass 2 rejected would have failed this very test; it does not.

The non-recursive negative (`test_nested_non_recursive_constructor_keeps_named_optional_arguments_unboxed`) pins `records.push(Record::new(value));` plus absence of the map, which I independently confirmed by probe (`Holder::new(local)` for a non-recursive `Plain | None` field is emitted untouched).

### Request 2 — ledger evidence ✅
`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:334`

Every figure the row cites, I reproduced at this exact head:

| Ledger claim | My independent result |
|---|---|
| "all 964 codegen tests" | `cargo test -p sifr_codegen` → **964 passed; 0 failed** |
| "complete native e2e suite passes 686/686 with signature `96d2681cf0c5ac5c`" | `verification/runner/e2e/run_e2e_pass.sh` → `686 pass tests completed (686 passed, 0 failed)`, `[sifr-e2e] report_signature=96d2681cf0c5ac5c` — **exact match** |
| "a 30-fixture recursive-node corpus sweep builds 30/30" | I went further: base-vs-head emit differential over **all 56** corpus fixtures naming `ListNode`/`TreeNode`/`TrieNode` → exactly **1** changes (`0894`), correctly; 55 byte-identical |
| "`0894…` also checks, builds as a native release binary, and runs" | `check` → `no errors found`; `run` → release build finished, exit 0 |
| "affected Clippy, rustfmt, maintainability, file-size … checks pass" | `cargo clippy --workspace -- -D warnings` exit 0; `cargo fmt --check` clean; maintainability `PASS`; file-size `PASS (3066 files, limit 900)`; submodule-ownership `PASS`; `baseline_hygiene.py` exit 0; `git diff --check` clean |

The row's narrative also accurately describes the current design ("a successful adaptation directly returns the completed Rust expression, so strict registry lowering and constructor fallback each suppress their terminal clone without a redundant status payload") and correctly records that the syntactic idempotence recognizer was removed. The Wave 7 row's promotion to `merged … at 4c867d1cda` matches this PR's base commit.

### Request 3 — PR #3089 body ✅
`gh pr view 3089` now reads "one shared post-ownership coercion that clones borrowed options before `Option::map`" and "apply that coercion exactly once across strict registry and constructor-fallback lowering, **without syntax recognition or duplicate boxing helpers**". The rejected "structurally idempotent" recognizer language is gone. The Validation list carries 964/964, 686/686 with signature `96d2681cf0c5ac5c`, 30/30, `0894`, and the new capability fixture — i.e. it describes `a73b14b85`, not `f546f563b`.

### Request 4 — adapter result contract ✅ (behaviour-preserving)
`crates/sifr_codegen/src/stmt_support_emitter/recursive_constructor_args.rs`

`struct RecursiveOptionConstructorArg` and its `consumed_owned_borrowed_name` field are deleted; the signature is now `-> Option<RustExpr>` and callers read the contract directly as `!recursive_option_adapted` (`plain_call_args.rs:224`, `call_args_and_returns.rs:152`). I verified this is exactly equivalent, not merely similar: the old flag was `convention.is_owned() && borrowed_name_arg`, so `!flag` was always false when the caller's own guard `convention.is_owned() && borrowed_name_arg` held — identical to unconditional suppression. The one divergent branch (the `NoneLiteral` early return, which set the flag `false`) is unreachable in combination, because `borrowed_name_arg` only matches `HirExpr::Name`. No ownership behaviour changed.

---

## 2. Single-application routing: independently re-verified

`ensure_option_box_inner_for_ir` (`print_calls.rs:429`) is **not** idempotent on a `.map(…)` receiver, so "exactly once" is load-bearing. The three call sites are mutually exclusive:

- `plain_call_args.rs:119` (strict registry). Its caller `expr_call_and_literal_helpers.rs:264` now `continue`s for every option param, so the registry post-pass can no longer re-box — the boxing decision there is delegated wholesale.
- `call_args_and_returns.rs:82` (nested/print path; reached from `print_calls.rs:88`, `expr_call_and_literal_helpers.rs:738`, `recursive_exprs.rs:275`) — none of these routes is inside the constructor macro's second loop.
- `expr_call_and_literal_helpers.rs:335` (constructor fallback) — reached only after the registry branch returns at line 281.

`registry_is_some_ctor`, `registry_is_some_expr`, and `registry_ensure_some_box_inner` are deleted; `grep` confirms no remaining references. The registry post-pass and fallback loop both correctly bail before their `Box::new` wrapper for option params, and the fallback loop now guards on `ctor_params.get(idx)` before indexing (line 303) rather than after.

I checked the one divergence risk this delegation creates: the registry site keys recursion off `func.strip_suffix("::new")` (i.e. `registry_ctor_key`) whereas the removed post-pass keyed off `class_name`, and `registry_ctor_key` may be `emitted_class_name::new` (`expr_call_and_literal_helpers.rs:91-98`) while `recursive_fields`/`class_field_order` are keyed by source `class.name` (`field_analysis_helpers.rs:148,164`). I could not make this diverge: generic `Node[T]` falls back to `source_ctor_key` and boxes correctly, and the 56-fixture corpus differential — which includes real cross-module `helpers/list_node` imports of a recursive `ListNode` — shows zero change, so no boxing is lost on the import/alias path.

## 3. Boundary probes: 24 shapes, base-vs-head differential, all build and run at head

Every probe below was emitted by both `4c867d1cda` and `a73b14b85` and diffed in full. Head is byte-identical or strictly corrective everywhere; **I could not construct a regression.**

| Shape | Base | Head |
|---|---|---|
| borrowed option → direct ctor | `(node).clone().map(Box::new)` | identical |
| borrowed option → nested (`append`) | `(node).clone()` → **E0308** | `+ .map(Box::new)` |
| `own` option → direct | `node.map(Box::new)` | identical |
| `own` option → nested | `node` → **E0308** | `node.map(Box::new)` |
| recursive optional field → direct | `(node.left).as_deref().cloned().map(…)` | identical |
| recursive optional field → nested | `.as_deref().cloned()` → **E0308** | `+ .map(Box::new)` |
| non-option recursive → direct | `Some(Box::new((Some(Box::new((child).clone()))).clone()))` **double box** | `Some(Box::new((child).clone()))` |
| non-option recursive → nested | `(Some(Box::new((child).clone()))).clone()` | `Some(Box::new((child).clone()))` |
| keyword `left=node` → direct / nested | direct ok; nested **E0308** | both `+ .map(Box::new)` |
| `None` literal → direct / nested | `None` | identical (untouched) |
| local option from call → direct / nested | direct ok; nested **E0308** | both `local.map(Box::new)` |
| mutually recursive `A`/`B` → direct / nested | nested **E0308** | `+ .map(Box::new)` |
| generic `Node[int]` → direct / nested | nested **E0308** | `+ .map(Box::new)` |
| conditional-branch ctor, dict-value assign, `print` statement, nested-subscript field base | **E0308** on nested forms | all `+ .map(Box::new)` |
| loop variable over `list[TreeNode \| None]` | `TreeNode::new(6_i64, n)` **E0308** | `n.map(Box::new)` |
| nested constructor call as the option arg | `Some(Box::new(TreeNode::new(2,None)))` | identical, one layer |
| non-recursive-class field of a recursive class (`w.node`) | `Some(Box::new(w.node.clone()))` | identical |
| non-recursive option param (`Record`, `Holder`) | untouched | identical (negative holds) |
| ctor param order ≠ field order | `T::new((n).clone(), 1_i64)` | **byte-identical** (pre-existing) |
| recursive *container* param through nested path | `Outer::new(2_i64, (ks).clone(), …)` | container arg **byte-identical**; only the option arg gains the map |

The new e2e fixture `crates/sifr/tests/e2e/pass/recursive_constructor_option_forwarding.sifr` is a genuine regression guard, not decoration: it exits 0 at head and fails at base with `E0308` (`SIFR-BUILD-0005`), and its `childValue` assertions read the boxed child back at runtime rather than only type-checking. It is one of the 686 discovered `e2e/pass/*.sifr` files, so the suite count confirms it ran.

## 4. Pre-existing, out of scope (verified byte-identical between `4c867d1cda` and `a73b14b85`)

Recording these so they are not mistaken for regressions; none is newly reachable and none is attributable to this PR.

- **Constructor parameter order ≠ field declaration order** still mis-indexes `class_field_order`; the new adapter inherits base's positional `fields.get(context.index)` lookup verbatim. Full emit diff for this probe is empty.
- **Recursive *container* parameter through the nested-call path** loses its `Box::new` (`call_args_and_returns.rs` has no `is_recursive_container_param` post-pass; the direct path does). Only the *option* argument on that line differs between base and head.
- **An `Option` local consumed by two constructor calls** yields raw `E0382`. This is a general Sifr hole, not a recursive-path one: I reproduced the identical `E0382` on a non-recursive `Holder::new(local)` twice, and base fails the recursive form too (with `E0308`). It does affect the `cloneTree` source inside the new focused test — that program does not build end-to-end — but the assertions there pin emit *shape*, and both halves of the shape are correct; fixing the double-move is a separate, wider concern than Wave 8.
- **Subclass inheriting a recursive field**: `Derived::new(tag, parent: Option<Base>)` calls `Base::new(tag, parent)` which expects `Option<Box<Base>>` → `E0308`. Full emit diff between base and head for this probe is empty.
- **`&root.left` as a call argument** emits `root.left.take()` → `E0596` on a non-`mut` binding. Base-identical (Wave 7 territory).
- The ledger and PR body both cite "diff-hygiene guardrails", for which no script of that name exists in the repo (`baseline_hygiene.py` is the only match). This wording is inherited verbatim from the already-merged Wave 5 and Wave 7 rows, and the substance holds (`git diff --check` is clean), so I am not raising it as a finding against this PR.

## 5. Responsibility, docs, workflow

`recursive_constructor_args.rs` is 81 lines and holds exactly one responsibility (the shared adapter), exported through `stmt_support_emitter.rs:45-46`; file-size and HIR maintainability guardrails pass. The three call sites now hold routing only, not policy. Both review artifacts (`…pass-1.md`, `…pass-2.md`) are committed and their ledger characterisations match their contents. The wave row's status (`implementation complete; parent PR #3089 in review`) is accurate for a draft PR whose create-PR profile has not yet been claimed — consistent with the Wave 5/6/7 workflow, where the exact-head profile is recorded at merge time.

**Verdict: APPROVE.** All five pass-1 findings and all four pass-2 requested changes are implemented and independently verified; 964/964 codegen, 686/686 native e2e with the exact claimed signature, 56/56 recursive corpus fixtures free of regression, all guardrails green, and 24 hand-built boundary probes show head strictly improves on base with zero regressions.
