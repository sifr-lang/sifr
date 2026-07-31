I inspected the full diff, both changed emitter paths, and all four sites that apply this coercion, and probed the compiler built at `f546f563b`.

## Verdict: REQUEST CHANGES — 5 actionable findings

### 1. HIGH — Regression: direct constructor with a shared-borrowed optional recursive argument now emits `Option<Box<Box<T>>>`
`crates/sifr_codegen/src/intrinsic_method_emitters/plain_call_args.rs:153`

The new `else if needs_box_inner && arg_is_option` fires at line 154, **before** the borrowed-name owned-arg clone at line 228. The resulting expression is `node.map(…).clone()`. The direct-constructor post-adapter (`expr_call_and_literal_helpers.rs:286`) then runs `ensure_option_box_inner_for_ir` again, and the new recognizer cannot see through the intervening `.clone()`, so it boxes a second time.

Reproduced (`/tmp/w8/p4.sifr`, compiler at exact head):
```python
def wrap(node: TreeNode | None) -> TreeNode:
    return TreeNode(5, node)
```
emits
```rust
TreeNode::new(5_i64, (node.map(|__sifr_option_value| Box::new(__sifr_option_value))).clone().map(|__sifr_option_value| Box::new(__sifr_option_value)))
```
→ `error[E0308]: expected Option<Box<TreeNode>>, found Option<Box<Box<TreeNode>>>`.

At base this same site emitted `(node).clone()` from the plain-call path and exactly one map from the post-adapter, i.e. it compiled. This is a new build break on the most ordinary Wave 8 shape (optional recursive parameter forwarded straight into a constructor). `0894` escapes it only because `cloneTree` forwards locals, never its own `node` parameter.

Root cause is placement, not the recognizer: the coercion is inserted before ownership adaptation, and the design relies on a syntactic self-recognizer instead of applying the coercion exactly once.

### 2. HIGH — New coercion moves out of a shared reference (`E0507`) on the nested path
`crates/sifr_codegen/src/intrinsic_method_emitters/plain_call_args.rs:154`

`Option::map` takes `self` by value, but `lowered_arg` at this point may be a shared-borrowed `&Option<T>` receiver. No `.clone()`/`.as_ref()` is applied first.

```python
def wrap_nested(node: TreeNode | None) -> list[TreeNode]:
    res: list[TreeNode] = []
    res.append(TreeNode(6, node))          # also with keyword form TreeNode(value=…, left=node)
    return res
```
emits `(node.map(|__sifr_option_value| Box::new(__sifr_option_value))).clone()` for `node: &Option<TreeNode>` →
`error[E0507]: cannot move out of *node which is behind a shared reference`.

This is the exact "named optional local inside `res.append(TreeNode(...))`" scenario the wave targets, only with a parameter instead of a local — so it is squarely in scope and unfixed. (This site was also broken at base, with `E0308`; the change swaps one build failure for another rather than resolving the class.) The `own`-convention variant does compile and run correctly, so the defect is specific to borrowed option arguments.

### 3. MEDIUM — Idempotency-by-recognizer is the wrong responsibility placement; the same coercion is now replicated at four divergent sites
`crates/sifr_codegen/src/stmt_support_emitter/print_calls.rs:459`, with duplicates at `plain_call_args.rs:117-155`, `call_args_and_returns.rs:77-114`, `expr_call_and_literal_helpers.rs:278-290` and `:339-356`

`is_option_box_map_expr_for_ir` exists only because the direct-constructor path applies the same adaptation twice from two modules. Making the adapter recognize its own output papers over that; findings 1–2 are the immediate consequence, since any adaptation inserted between the two applications defeats the recognizer. The correct fix is one shared "recursive-ctor option argument" adapter, applied once, after ownership/borrow adaptation.

The four copies have already drifted: `call_args_and_returns.rs:112` uses `else if needs_box_inner` while the new `plain_call_args.rs:153` uses `else if needs_box_inner && arg_is_option`; `plain_call_args.rs:146` still calls `registry_ensure_some_box_inner` while line 154 calls the shared `Self::ensure_option_box_inner_for_ir` — two parallel implementations of the same Some/Box coercion inside a single loop body.

### 4. LOW — Focused coverage misses the only shape that regresses
`crates/sifr_codegen/src/lib_codegen_tests/recursive_node_codegen_tests.rs:168`

Both new tests pass a local bound from a call (`left_copy`) or a non-recursive parameter. Neither exercises a **borrowed optional parameter** forwarded into a constructor — direct or nested — which is precisely where the double-box and `E0507` appear. The `!contains("…Box::new(…)).map(")` double-box assertion is anchored on the bare-name shape and does not detect the `.clone()`-interleaved form. There is no snapshot or test anywhere in `crates/` or `verification/` pinning `.clone().map(|__sifr_option_value| Box::new(…))`, which is why 964/964 stayed green.

### 5. LOW — Ledger overstates the guarantee and omits the PR link
`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:334`

"the shared option-box adapter recognizes its own canonical `.map(Box::new)` shape so repeated constructor adaptation is idempotent rather than producing `Option<Box<Box<T>>>`" is not accurate — repeated adaptation still produces `Option<Box<Box<T>>>` whenever a clone is interposed (finding 1). The status cell reads "parent PR pending" with no link, unlike every other wave row, which cites its PR (#3089).

### Verified as claimed
- Head is exactly `f546f563b`, one commit over `4c867d1cda`, four files, no fixture/baseline/waiver changes; corpus submodule and `third_party/ruff` untouched.
- Focused recursive-node tests: 16/16 pass (948 filtered of 964).
- `0894_all_possible_full_binary_trees` checks, builds a release binary, and runs successfully.
- The recursive field-access path is correct and not double-boxed: `(node.left).as_deref().cloned().map(|__sifr_option_value| Box::new(…))`, single layer, at both direct and nested sites.
- `None` literal, non-option argument, `own`-convention option parameter, and the non-recursive `Record(value)` negative all emit correctly.
- Recognizer false-positive risk is low: the only other producer of a `map`-with-`__sifr_option_value` closure (`class_upcasts.rs:24`) never emits `Box::new` as the converted body.

### Out of scope, pre-existing (not caused by this diff, unchanged by it)
An `own` recursive parameter whose child is extracted via a simple annotated declaration emits `node.left.take()` while the parameter is declared `fn consume(node: TreeNode)` without `mut` → `E0596`. This originates in the Wave 7 mutability gate, not in either file touched here.
