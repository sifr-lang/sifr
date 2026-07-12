I have enough evidence to compose the verdict.

## Round 10 verification

Round 9's stated closures verified against the current diff and a rebuilt compiler at `target/release/sifr`.

### Round-9 reproductions — all now emit `SIFR-PYCTX-0001`

`SIFR_SYSROOT=$repo target/release/sifr check <file>` from `/tmp`:

| Case | File | Result |
|---|---|---|
| `return sink_list([transaction])` | `/tmp/wave3audit10/return_call_agg.sifr` | ✓ PYCTX-0001 at `[transaction]` |
| `return sink.collect([transaction])` | `/tmp/wave3audit10/return_method_agg.sifr` | ✓ PYCTX-0001 |
| `return Registry.store([transaction])` | `/tmp/wave3audit10/return_static_agg.sifr` | ✓ PYCTX-0001 |
| `stored.append(transaction)` (list) | `/tmp/wave3audit10/list_append.sifr` | ✓ PYCTX-0001 on outer method-call range |
| `stored.add(transaction)` (set) | `/tmp/wave3audit10/set_add.sifr` | ✓ PYCTX-0001 |
| `if is_ok([transaction]):` | `/tmp/wave3audit10/if_condition.sifr` | ✓ PYCTX-0001 |
| `assert is_ok([transaction])` | `/tmp/wave3audit10/assert_call.sifr` | ✓ PYCTX-0001 |
| `del data[compute([transaction])]` | `/tmp/wave3audit10/delete_key.sifr` | ✓ PYCTX-0001 |
| `raise LeakError([transaction], "…")` | `/tmp/wave3audit10/raise_call_agg.sifr` | ✓ PYCTX-0001 (bonus sibling) |
| `Holder(transaction)` (ConstructorCall) | `/tmp/wave3audit10/constructor_call.sifr` | ✓ PYCTX-0001 |
| `yield [transaction]` composite | `/tmp/wave3audit10/yield_composite.sifr` | ✓ PYCTX-0001 |

Mechanism: `reject_python_context_borrow_created_value` is now invoked at the tail of `lower_expr` (`expressions/core_and_calls.rs:96`), so every subexpression that yields an owned-aggregate/constructor/storing-method HirExpr is walked via `python_context_borrow_in_owned_expr` (`python_interop/context.rs:13-114`). Aggregate literals, `ConstructorCall`, `WalrusExpr`, `Lambda`, `ListComp`/`SetComp`/`DictComp`/`GeneratorExpr`, and `Call`/`PythonCall`/`IntrinsicCall`/`IteratorCall`/`SuperCall`/`Index`/`FieldAccess`/`Await` when their return type holds opaque, and `MethodCall` when the method stores (`append`/`insert`/`extend`/`add`/`update`/`setdefault`) or its return type holds opaque, are all treated as "created owned values" — and because the guard fires on every recursive `lower_expr`, the inner `[transaction]` is caught regardless of whether the outer statement is `return`, `assert`, `if`, `while`, `del`, `raise`, `Stmt::Expr`, `for … in`, `with … as`, or an ordinary assignment/annotation, none of which need bespoke wiring beyond calling `lower_expr`. `python_context_borrow_in_call_args` (`context.rs:116-126`) preserves the round-8 discipline of not inspecting direct-`Name` args for reader calls (so `read_ok(transaction)`, `compute_id(transaction)`, `if read_ok(transaction):` all remain valid — verified via `/tmp/wave3audit10/legit_direct_read.sifr`, `no errors found`), but toggles `inspect_direct_names=true` when the method stores or the return type carries opaque, which is exactly what catches `stored.append(transaction)` / `stored.add(transaction)`.

### Legitimate use preserved

- Direct borrowed reads (`read_ok(transaction)`, `compute_id(transaction)`, `if read_ok(transaction):`) — `no errors found` (`/tmp/wave3audit10/legit_direct_read.sifr`).
- Direct `Name` moves into `own`-parameter functions (test `python_context_entered_borrow_cannot_move_into_owned_parameter` at `python_interop_tests.rs:673-687`) still route through `mark_moved_with_flow` (`mod_context.rs:383-401`) — untouched by the diff.

### Native `with` regression

None. `python_context_borrows` is only populated inside the `entered_is_opaque_borrow == true` branch (`statement_dispatch.rs:436-442`); `HirWithItemKind::Native` never inserts, so `mark_moved_with_flow` and `python_context_borrow_in_owned_expr` short-circuit for non-Python contexts. `/tmp/wave3audit10/native_with.sifr` (a class-based `with … as value:`) checks clean.

### HIR consumer omissions

The `With { items: Vec<(String, HirExpr, bool)> }` → `Vec<HirWithItem>` migration in `sifr_ir` was applied comprehensively across `sifr_lowering` (all pattern matches at `cfg.rs`, `flow_graph.rs`, `flow_graph/effects.rs`, `nonlocal_support.rs`, `async_with.rs`, `hir_snapshot_tests.rs`, `name_resolution_snapshot_tests.rs`, and callers of `HirStmt::With` in `sifr_codegen` — `error_refs.rs`, `function_emitter/python_callback_bounds.rs`, `hir_analysis/traversal/traversal_impl.rs`, `lower_stmt/{candidate_and_validation,with_yield_and_match,yield_unpack_with_tests,simple_dispatch_and_bindings}.rs`, `stmt_support_emitter/{async_with_and_for,stmt_block,stmt_block_helpers,loops_try_finally}.rs`, and the two `lib_codegen_tests` fixtures). `cargo test -p sifr_lowering --lib` reports 665 passes (51 focused Python-interop), `cargo test -p sifr_codegen --lib` reports 752 passes.

### Duplicate/spurious diagnostics — observed but non-blocking

Nested/wrapped constructions emit the invariant multiple times:
- `sink([transaction])` as `Stmt::Expr` emits two `PYCTX-0001`s: one from the central rule at the inner ListLiteral range and one from `reject_python_context_borrow_discard` at `Stmt::Expr` at the outer range (`statement_dispatch.rs:256-260`), because the discard guard re-walks the whole Call and its args. Verified in `/tmp/wave3audit10/single_agg_stmt.sifr`.
- `sink([[transaction]])` produces three `PYCTX-0001`s — one per aggregate level plus the discard guard.
- `raise LeakError([transaction], "…")` produces two `PYCTX-0001`s — one from the inner ListLiteral, one from the outer ConstructorCall.

Every message names the same underlying binding and points at overlapping spans. This is UX noise, not a correctness escape — the invariant is enforced at least once for each shape and every reproduction is rejected. Round 9 did not require message deduplication as an acceptance gate, and the focused tests use `errors.iter().any(...)`. Worth polishing later (probably by having `reject_python_context_borrow_discard` short-circuit when the discarded expression is itself an owned aggregate/ConstructorCall/storing MethodCall that the central rule already handled, and by suppressing the outer-aggregate walk once an inner element already reported), but does not warrant a NOT SATISFIED verdict.

### Overall

Every round-9 concrete escape (return-through-Call/MethodCall/staticmethod, assert/if/while/del sub-call, `list.append`/`set.add`) is now caught by the centralized `reject_python_context_borrow_created_value` invocation. Legitimate direct borrowed reads pass. Native `with` is unaffected. HIR consumers were updated in lockstep. Duplicate diagnostics for nested/wrapped shapes are a UX polish item, not a safety regression.

VERDICT: SATISFIED
