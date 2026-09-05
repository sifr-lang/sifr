**CHANGES REQUIRED**

## Verdict

The wave's happy path works — `0001_two_sum` emits `HashMap<i64, i64>`, builds, and runs; the four focused tests, the 889-test `sifr_lowering` suite, clippy, fmt, and the HIR guardrail all pass on my independent runs; the new e2e fixture runs clean. But the mechanism the wave reuses is keyed on **variable name across the whole block**, not on binding identity, so the hint leaks between distinct bindings that happen to share a name. That turns previously-valid programs into compile errors and, in one shape, into non-compiling generated Rust.

---

## Blocking findings

### 1. Empty-dict hint leaks across disjoint bindings → false `SIFR-TYPE-0008` on previously-valid code
`crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:103`

`should_adopt_inferred_binding_hint` now adopts dict hints unconditionally, but the hint source (`infer_function_types` → `env.vars`, `nested_function_inference/state_collection.rs:135-190`) is a single name-keyed environment merged over the entire block. `analyze_stmt`'s `If`/`For`/`While` arms merge branch envs back into the parent (`state_collection.rs:~470-560`), so evidence gathered for one `x = {}` is visible at a *different* `x = {}` in a sibling scope. Previously this was unreachable for plain dicts because the gate required a nested function in the block.

Repro (`check` is clean on main by construction — with no nested function present, main's `allow_empty_collection_hint` is `false`, so the second `a` stays `dict[Any, Any]` and specializes to `dict[int, int]` on its own first write):

```python
def solve(flag: bool) -> int:
    if flag:
        a = {}
        a["k"] = 1
        return len(a)
    a = {}          # distinct binding, if-body frame already popped
    a[2] = 3
    return len(a)
```
```
error[SIFR-TYPE-0008]: empty literal type conflict for 'a': expected key 'str' and value 'int', got key 'int' and value 'int'
  --> 7:5
```

Same failure with loop-body/function-level pairs:
```python
def solve(items: list[int]) -> int:
    total = 0
    for x in items:
        d = {}
        d[x] = x
        total += len(d)
    d = {}
    d["a"] = 1      # error[SIFR-TYPE-0008]: expected key 'int' ..., got key 'str' ...
    return total + len(d)
```

This violates the contract's "preserve ordinary plain-dict … behavior". The evidence walk needs to be anchored to the declaration being lowered (stop at rebinds / scope boundaries, or refuse the hint when the name is bound more than once in the block) rather than consuming the block-wide merged env.

### 2. Same leakage yields generated Rust that does not compile
`crates/sifr_lowering/src/lower/statements/control_flow.rs:402-404`

When the leaked type is *assignable-compatible* rather than conflicting, no diagnostic fires and the bad type reaches codegen:

```python
def solve(flag: bool) -> int:
    if flag:
        a = {}
        a[1] = 2.5
        return len(a)
    a = {}
    a[3] = 4
    return len(a)
```
`sifr check` → clean. `sifr emit`:
```rust
let mut a: HashMap<i64, f64> = HashMap::from([]);
a.insert(3_i64, 4_i64);
```
`sifr build` → `error[SIFR-BUILD-0005]: cargo build failed: error[E0308]: mismatched types … expected f64, found i64`.

On main the second `a` specializes to `dict[int, int]` and this builds. This is a check-clean → rustc-error path, i.e. a direct break of the "if it compiles, it works" guarantee. Note the `Int`-into-`dict[int, float]` codegen gap itself is **pre-existing** (see non-blocking #3); what is new is that a correctly-typed binding is being given the wrong element type.

### 3. Test coverage does not exercise the failure mode
`crates/sifr_lowering/src/lower/expressions_tests/empty_plain_dict_inference.rs:1-61`

All four tests use a single `{}` binding per function. There is no negative test asserting the hint does **not** cross into a same-named binding in another scope, and no test pinning the pre-existing nested-function gate path. Both repros above should become tests alongside whatever fix lands.

---

## Non-blocking findings

3. **Pre-existing (not this wave): uncoerced int into a float-valued dict.** `def solve(): a = {}; a[1] = 2.5; a[3] = 4` checks clean and emits `a.insert(3_i64, 4_i64)` into `HashMap<i64, f64>` → E0308. Reachable identically on main (`validate_subscript_assignment_target` accepts `Int.is_assignable_to(Float)` at `container_literal_specialization.rs:100-101` with no codegen coercion). Worth a separate issue.

4. **Pre-existing dead match arm adjacent to the touched function.** `empty_collection_literal_kind` — `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:54` and `:62`. The first `Expr::Call` arm's guard (`args.is_empty() && keywords.is_empty()`) matches *any* zero-arg call and returns `None` for non-`Name` callees, so the `collections.deque()` arm at `:62` is unreachable and `"deque"` is never produced. Harmless today (it only suppresses deque hint adoption), but the wave contract's "do not widen … deque" is satisfied vacuously rather than by design.

5. **Silent diagnostic-class change on the pre-existing nested-function path.** `control_flow.rs:429-434` now registers every refined empty-dict declaration in `ctx.empty_dict_specializations`. Programs that already adopted a dict hint via the nested-function gate previously reported conflicts as the specific `TYPE-0002` "dict subscript assignment key type … is not compatible …" messages and will now report `TYPE-0008` "empty literal type conflict". Defensible and arguably more consistent, but undocumented and untested.

6. **Fragile adoption proxy.** `control_flow.rs:402-404` infers "the hint was adopted" from `binding_ty != value_ty`. Correct today only because `binding_ty` is either the adopted hint or a clone of `value_ty`; it silently changes meaning if the fallback ever stops being an exact clone. Capturing the adoption decision from the `.filter(...)` directly would be more robust.

7. **Zero-byte review artifact in the tree.** `plans/reviews/active/ad-hoc-algorithmic-full-corpus-preexisting-failures-wave-3-agent-review-pass-1.md` is 0 bytes and untracked. Remove or populate before the PR.

8. **Ledger.** `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:301-302` — Wave 2 → `merged` matches main (`b3f663a17`, PR #3074) ✓; the Waves 4-8 renumbering is correct ✓. The Wave 3 row's claim "without widening the existing empty-list/set/deque hint boundary" is literally true but omits the actual risk surface: the *dict* boundary is widened from "blocks containing a nested function" to *all* blocks. "Conflicting writes preserve `SIFR-TYPE-0008`" is true but incomplete — per finding #1 it now also fires on non-conflicting code.

---

## What I verified clean

- `cargo test -q -p sifr_lowering` → 889 passed, 1 ignored, 0 failed.
- `cargo clippy -q -p sifr_lowering -- -D warnings` → exit 0; `cargo fmt --check` → clean; `check_hir_maintainability_guardrails.py` → PASS.
- File sizes: `control_flow.rs` 851, `statement_dispatch.rs` 880, new test file 61 — all under 900.
- `0001_two_sum.sifr` builds and runs; new e2e fixture runs clean; emitted `HashMap<i64, i64>` with both the `Let` type and the `DictLiteral` HIR type concrete.
- Contract boundaries held: empty list/set/deque adoption unchanged; no `dict` annotation introduced; `defaultdict` is not classified as an empty literal so alias semantics are untouched; ordinary missing-key read still yields `int | None` (`SIFR-TYPE-0002` on unguarded return), and `d[k] += 1` on an evidence-free empty dict still reports `unsupported operand type(s) for +: 'Any' and 'int'`.
- Corpus scan: no fixture in the 411-file leetcode corpus reuses an empty-dict variable name within a file, so finding #1 does not regress the corpus — but it does regress user code.
