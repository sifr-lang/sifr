Cleanup done: baseline worktree removed and pruned, probes deleted, tree byte-identical to how I found it (`git status` matches the starting snapshot).

# Wave 3 Review — pass 4 (exact working tree vs `b3f663a17`)

## CHANGES REQUIRED

One blocking finding. The happy path is genuinely correct and the post-pass-3 cleanups all landed and are net improvements — but the adoption gate accepts a *unified* hint type across heterogeneous writes, which converts four families of previously check-rejected programs into check-clean programs whose generated Rust does not compile.

Everything below is from my own builds. I built a `b3f663a17` baseline worktree and differentially compared `check` / `emit` / `run` on ~35 hand-written probes.

---

## Blocking

### 1. Unified hint adoption newly admits widening writes → check-clean, `rustc` E0308
`crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:94-116` (gate) · `crates/sifr_lowering/src/lower/statements/control_flow.rs:392-414` (adoption) · `crates/sifr_lowering/src/lower/container_literal_specialization.rs:104-105` (permissive write check)

The declaration hint comes from `FunctionEnv`, which **unifies** every subscript write (`FunctionEnv::bind_var` → `unify_types`, `nested_function_inference/state_collection.rs:95-103`). `should_adopt_inferred_binding_hint` only requires the hint to be concrete and shape-matching — it never requires the hint to *equal* each write's type. Once the declaration is pinned to the widened type, each individual write is validated with `value_ty.is_assignable_to(value_ty_expected)` (`container_literal_specialization.rs:105`), which accepts `int→float`, `Derived→Base`, and `T→T|None`. Codegen emits no coercion, so the narrower write becomes an E0308.

On `b3f663a17` this was unreachable: forward-only specialization pinned the *first* write's exact type, so the widening write was reported as a deterministic `SIFR-TYPE-0008`. Four confirmed families — **base rejects at `check`; this tree accepts at `check` and fails `sifr build`**:

| Repro | base `check` | this tree `check` | this tree `build` |
|---|---|---|---|
| `d={}` ; `d[1]=4` ; `d[2]=2.5` | `SIFR-TYPE-0008` (int vs float) | clean | `E0308: expected f64, found i64` |
| `d={}` ; `d["a"]=1` ; `d["b"]=2.5` | `SIFR-TYPE-0008` | clean | `E0308: expected f64, found i64` |
| `d={}` ; `for n in nums: d[n]=n` ; `d[0]=1.5` | `SIFR-TYPE-0008` | clean | `E0308: expected f64, found i64` |
| `d={}` ; `d[1]=Derived(1)` ; `d[2]=Base(2)` | `SIFR-TYPE-0008` (Derived vs Base) | clean | `E0308: expected Base, found Derived` |
| `d={}` ; `d[1]=Node(5)` ; `d[2]=make(flag)` (`Node \| None`) | `SIFR-TYPE-0008` | clean | `E0308: expected Option<Node>, found Node` |

Emitted body for the first row:
```rust
let mut d: HashMap<i64, f64> = HashMap::from([]);
d.insert(1_i64, 4_i64);      // <-- E0308
d.insert(2_i64, 2.5_f64);
```

This is the same defect class pass 1 raised as its blocking finding #2 (check-clean → `rustc` error, breaking "if it compiles, it works"), reached by a different route: pass 1's route was cross-binding name leakage, which the declaration-safety gate correctly closed; this route is *within a single binding*, via write-set unification. It also contradicts the wave contract clause "preserve deterministic conflicting-write `SIFR-TYPE-0008`" — five shapes that base flagged are now silently accepted.

Suggested fix, consistent with the wave's own design: refuse the new adoption unless every collected subscript write's key and value types are **exactly** the adopted hint's, i.e. reject when `unify_types` actually widened. `0001_two_sum` is unaffected (all writes are `int`/`int`), and the widening shapes fall back to base's forward specialization plus its deterministic `SIFR-TYPE-0008`. Fixing the underlying missing codegen coercion instead would be a separate, larger wave (pass 1 non-blocking §3) and is not needed to unblock this one.

### 2. Same root cause: duplicated hash-key diagnostic on unhashable key widening
`crates/sifr_lowering/src/lower/container_literal_specialization.rs:95-102`

`d={}` ; `d[1]="a"` ; `d[2.5]="b"` — base emits one `SIFR-TYPE-0002` "key type … unavailable for 'float'" plus the `SIFR-TYPE-0008` conflict; this tree pins the declaration to `dict[float, str]` and therefore emits the *same* unhashable-key error twice, once for each write, and loses the conflict diagnostic. Still safely rejected, so not independently blocking — but it is fixed by the same change as #1 and should be re-checked after it lands.

### 3. Test coverage does not reach the widening surface
`crates/sifr_lowering/src/lower/expressions_tests/empty_plain_dict_inference.rs:44-52`

The only conflict test uses a hard `int` vs `str` mismatch, which unification cannot absorb. Nothing pins behaviour when the write set is *assignable-but-unequal* — precisely the gap in #1. The `int`/`float` and `Derived`/`Base` repros should become lowering tests alongside the fix.

---

## Non-blocking

4. **Nested-function shadowing disposition — I concur it is non-blocking.** The ledger's `plans/issues/active/…-preexisting-failures.md:274-278` disposition is factually correct, and I verified both variants rather than taking it on trust: the annotated shape (`d: dict[str,int]` outer, `d={}` in a nested `def`) and the *unannotated* shape (`d={}` outer, `d={}` in a nested `def`) both fail `sifr build` on `b3f663a17` with generated-Rust `E0308`. This tree turns both into a false `SIFR-TYPE-0008` at check time — a wrong-reason rejection, but strictly not a regression on working code, and arguably closer to the language guarantee than emitting invalid Rust. Worth recording the actual mechanism when the follow-up is filed: pass 3's suggested remedy (restrict `inferred_binding_hint` to the innermost frame) would **not** fix it, because `infer_function_types` seeds the block's own env from `ctx.scope.visible_local_bindings()` (`nested_function_inference/state_collection.rs:139-148`), so the innermost frame is already polluted.

5. **Asymmetric capability in blocks containing a nested function.** `control_flow.rs:406` requires `!allow_general_hint`, so when the block has a nested `def`, the `Let` still adopts the hint via the pre-existing path but the `DictLiteral` type and `empty_dict_specializations` are not set. Verified benign today (`d={}` + a nested `def` + int writes builds and runs identically on both trees), but the wave's "both `Let` and `DictLiteral` are concrete" invariant holds only in nested-function-free blocks.

6. **`safe_hint_names_for_block` binding census is narrower than its name.** `crates/sifr_lowering/src/lower/empty_plain_dict_inference.rs:30-52` counts `Assign`/`AnnAssign`/`AugAssign`/`For`/`With`/`FunctionDef` but not `ClassDef`, `Import`/`ImportFrom`, `except … as`, or match-pattern captures. Unreachable today — each of those makes a later `d = {}` a rebinding (`HirStmt::Assign`) rather than a new `Let`, so adoption cannot mis-fire — but a comment stating that reasoning would keep the gate honest.

7. **Style:** `statement_dispatch.rs:100,102` calls `empty_collection_literal_kind(value_expr)` twice; bind it once.

8. **Ledger count off by one.** `plans/issues/active/…-preexisting-failures.md:307` says "all 894 lowering tests (plus one ignored)". Actual on this tree: 893 passed + 1 ignored = 894 total. Everything else in the Wave 3 row checks out against my runs; Wave 2 → `merged` matches `b3f663a17` / PR #3074.

9. **Housekeeping:** `plans/reviews/active/…-wave-3-claude-opus-review-pass-4.md` is present and zero bytes (recurring pass-1 §7 / pass-3 §7). I left it untouched per the no-write constraint.

---

## Post-pass-3 cleanups — all verified applied and correct

- **Explicit `adopted_hint_ty`** ✓ `control_flow.rs:392-409`. The `binding_ty != value_ty` proxy is gone; adoption is captured from the `.filter(...)` decision and `binding_ty` derives from it, not the reverse.
- **Loop-local / function-level coverage** ✓ `expressions_tests/empty_plain_dict_inference.rs:102-106` plus `loop_scoped_maps` in the e2e fixture. I ran pass 1's exact loop repro independently — clean, and the fixture's `loop_scoped_maps([1,2]) == 3` assertion runs green natively.
- **Stale none-widening registry cleanup** ✓ `function_emitter/local_binding_registry.rs:12-20`. `widened_bindings.remove(name)` now runs with the `bindings.remove(name)`, so `none_widened_local_bindings` can no longer dangle. I traced every consumer (`intrinsic_method_emitters/collection_methods.rs:13,32`, `stmt_support_emitter/call_args_and_returns.rs:38`, `await_and_async_comprehension.rs:133`) — all degrade to `expr.ty()`. Differentially this is a **strict improvement**: three probes that emit invalid Rust on base now build and run correctly here (`let v: i64 = "ab".to_string()` → `let v: String`; `Option<Vec<i64>> = Some(HashMap::from(...))` → correct declaration-local types; sibling/shadowed empty lists getting the wrong element type).
- **Duplicated pending-patch cleanup** ✓ `control_flow.rs:439` hoisted above the `if`.
- **Abandoned nested-function expansion removed** ✓ No dead code or orphaned helpers; both widened visibilities (`collect_current_function_local_bindings`, `empty_collection_literal_kind`) have live callers, and `clippy -D warnings` is clean.
- **Reverse-walk / `remove` patch change** ✓ `container_literal_specialization.rs:273-289`. Sound because patches drain after every statement (`statement_dispatch.rs:170,181`), and it is the mechanism behind the sibling/shadow improvements above. I could not construct a case where nearest-declaration resolution picks the wrong `Let`.

## Independently verified clean

| Check | Result |
|---|---|
| `cargo test -p sifr_lowering` | 893 passed, 0 failed, 1 ignored |
| `cargo test -p sifr_codegen` | 934 passed, 0 failed |
| **`verification/runner/e2e/run_e2e_pass.sh`** | **677 passed, 0 failed** (`report_signature=981c6b2203ccc554`) |
| `cargo clippy -p sifr_lowering -p sifr_codegen -- -D warnings` | exit 0 |
| `cargo fmt --check` | clean |
| `check_hir_maintainability_guardrails.py` / `check_file_size_guardrails.py` | PASS / PASS (3008 files, limit 900) |
| `git diff --check` vs base | clean |
| `0001_two_sum.sifr` native `run` | exit 0 here; `E0277`/`E0308` on `b3f663a17` |
| `empty_plain_dict_write_inference.sifr` native `run` | exit 0, all 8 asserts pass |

File sizes on touched files: `statement_dispatch.rs` 886, `scope_and_function_types.rs` 866, `control_flow.rs` 859, `mod_context.rs` 779, `empty_plain_dict_inference.rs` 106, `local_binding_registry.rs` 47 — all under cap.

**Contract clauses held:** no `dict` annotation introduced; `defaultdict(int)` alias behaviour byte-identical to base; ordinary missing-key read still yields `int | None` (`SIFR-TYPE-0002` on unguarded return) and `d[k] += 1` on an evidence-free empty dict still reports `SIFR-TYPE-0005`; empty list/set/deque adoption unchanged; same-named sibling lexical declarations isolated in both lowering and codegen. **Contract clause not held:** "preserve deterministic conflicting-write `SIFR-TYPE-0008`" — see finding #1.
