## APPROVED

Reviewed the complete working-tree diff (11 modified files + 5 new files) against `b3f663a17`. No blocking findings. I built both this branch and a `b3f663a17` baseline worktree and diffed behaviour on ~30 hand-written probes, so every claim below is from my own runs.

### Independently reproduced validation

| Check | Result |
|---|---|
| `cargo test -p sifr_lowering` | 892 passed, 0 failed, 1 ignored ✓ matches claim |
| `cargo test -p sifr_codegen` | 934/934 ✓ (bare run shows 3 failures — `registry_core_tests`/`lib_project_codegen` sysroot `NoCandidate`, environmental; green with `SIFR_SYSROOT` set) |
| `cargo fmt --check` | clean |
| `cargo clippy -p sifr_lowering -p sifr_codegen -- -D warnings` | exit 0 |
| `check_hir_maintainability_guardrails.py` / `check_file_size_guardrails.py` | PASS / PASS (3008 files, limit 900) |
| **Full `run_e2e_pass.sh`** | **677 passed, 0 failed** (751 s), including the new fixture |
| `0001_two_sum.sifr` native build+run | exit 0 |

File sizes on touched files: `statement_dispatch.rs` 886, `scope_and_function_types.rs` 866, `control_flow.rs` 858, `mod_context.rs` 779, new `empty_plain_dict_inference.rs` 106, `local_binding_registry.rs` 46 — all under cap.

### Mechanism verification

- **Scope identity.** `lower_stmts` (`statement_dispatch.rs:126-131,189`) is the *only* site that pushes `inferred_binding_hints`/`empty_collection_hint_adoption`, and the new `empty_plain_dict_hint_adoption` is pushed/popped in exactly the same place, so the stacks cannot desynchronize. `can_adopt_empty_plain_dict_hint` (`mod_context.rs:435`) reads only the innermost frame — correct per-block gating.
- **Patch consumption.** Reverse walk + `pending.remove` (`container_literal_specialization.rs:273-289`) is sound because patches are drained after *every* statement (`statement_dispatch.rs:181`), so `result` never contains a lexically-later declaration; inner blocks drain their own pending before the parent sees it. Verified against nested control flow and nested-function shadowing: outer `xs`/`d` get the right type and the nested function's shadowing local is untouched (`emit` on probes with `for`/`if`/`while` nesting and a shadowing closure local).
- **Codegen registry.** The ambiguity gate (`local_binding_registry.rs:11-20`) degrades safely — every consumer of `local_binding_types` uses `get(...).unwrap_or(ty)` or an `Option` guard (`lower_stmt/simple_dispatch_and_bindings.rs:56`, `stmt_support_emitter/call_args_and_returns.rs:38-50`, `intrinsic_method_emitters/collection_methods.rs:13-41`, `await_and_async_comprehension.rs:133-139`).
- **Pass-1 repros.** All three now clean: the `if/else` str-vs-int conflict, the compatible `float`/`int` pair (both in lowering + codegen + e2e), and the untested loop-body/function-level pair from pass 1 §1, which I ran manually — clean.
- **Boundaries preserved.** Empty list in sibling scopes, `set()`, `d[1] += 1` on evidence-free dict (still `SIFR-TYPE-0005`), and bare `deque()`/`defaultdict` all behave *identically to `b3f663a17`*.
- **Genuine improvements over main:** a shadowing-closure list case and a same-named sibling list pair emit correct Rust here where main emits `Vec<String>` + `push(1_i64)`; the `counts[ch] = counts[ch] + 1` counter dict compiles here and fails on main with `SIFR-TYPE-0005`.

### Non-blocking findings

1. **Enclosing-scope hints still leak into a nested function's shadowing declaration → false `SIFR-TYPE-0008`.** `can_adopt_empty_plain_dict_hint` guards only the innermost frame, but `LowerCtx::inferred_binding_hint` (`mod_context.rs:405-410`) reverse-searches *all* frames, and `FunctionEnv::bind` (`nested_function_inference/state_collection.rs:95-103`) unifies an `Any`/`Unknown` dict with the enclosing binding's concrete type. So:
   ```python
   def f() -> int:
       d: dict[str, int] = {"a": 1}
       def g() -> int:
           d = {}
           d[2] = 3      # error[SIFR-TYPE-0008]: expected key 'str' and value 'int', got key 'int' and value 'int'
           return len(d)
       return len(d) + g()
   ```
   Not a regression on working code — this shape (and the `dict[str,int]`/int and str/str variants) already fails on `b3f663a17` with `E0308` in generated Rust — but the failure mode moved from bad codegen to a false rejection, which is the pass-1 defect class. Suggested follow-up: also require the name to be absent from enclosing hint frames, or restrict the lookup to the innermost frame when the plain-dict gate is what authorises adoption. Worth a line in the issue rather than blocking this wave.
2. **Pass-1 non-blocking §6 unaddressed:** adoption is still inferred from `binding_ty != value_ty` (`control_flow.rs:398-403`) rather than captured from the `.filter(...)` decision.
3. **Coverage gaps:** pass-1's loop-body/function-level repro is fixed but untested; nothing pins the nearest-declaration patch ordering directly (the sibling test at `expressions_tests/empty_plain_dict_inference.rs:70` covers it only indirectly).
4. **Stale widening entry:** `local_binding_registry.rs:13-28` — if a `None`-widening `Assign` precedes a differently-typed sibling `Let`, the name stays in `none_widened_local_bindings` after its `local_binding_types` entry is dropped. Harmless today (all consumers fall back), and probes match main exactly; pruning it would be tidier.
5. **Patch order inside one compound statement:** `patch_stmt_container_specialization` visits `then_body` → elifs → `else_body` forward while the enclosing walk is reverse, so with `remove` semantics only the first-visited branch of a single `If` is patched when both branches declare the name. Currently masked — that shape (`if/else: xs = []` then `xs.append(1)`) emits `xs = ...` without `let` on *both* main and this branch. A comment stating the invariant would help.
6. **Pre-existing, out of scope but adjacent:** same-named different-typed siblings still break for non-dict shapes (missing `let` hoist for both-branch declarations; module-shadowing int/str pair; `None`-widened pair) — byte-identical failures on main. Worth its own issue.
7. **Housekeeping before the PR:** untracked `_probe3/` scratch dir (26 `.sifr` files) is still in the tree, plus zero-byte `…wave-3-…-pass-2.agent.log`, `pass-3.agent.log`, and `pass-3.md` (pass-1 §7 recurring).
8. **Style:** `control_flow.rs:437-443` — `pending_container_specialization_patches.remove(&name)` appears in both branches; hoist it above the `if`.

### Ledger accuracy

`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:301-303` is accurate: Wave 2 → `merged` matches `b3f663a17` (PR #3074); the Wave 3 row's status (`remediation validated; re-review pending`) is honest, the single-unshadowed-binding gate, nearest-declaration patching, declaration-local codegen registry, and preserved `SIFR-TYPE-0008` are all described as implemented, and the 892/934 and `0001_two_sum` claims check out; "Waves 4-8" renumbering is correct. I left every file unmodified and removed the baseline worktree and my probe directory, so the tree is exactly as I found it.
