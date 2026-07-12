I have enough evidence to compose the verdict.

---

## Round 8 verification

Round-8's stated closure is honored for `Stmt::Expr`:
- `sink.collect([transaction])`, `Registry.store([transaction])`, `sink.collect_tuple((transaction, 1))`, `sink.collect_set({transaction})` as bare statements all fire `SIFR-PYCTX-0001 … cannot be discarded`. The centralization in `python_context_borrow_in_call_args` at `crates/sifr_lowering/src/lower/python_interop/context.rs:106-117` correctly walks non-Name args for `Call`/`PythonCall`/`IntrinsicCall`/`IteratorCall`/`SuperCall`/`MethodCall`, and `type_can_hold_python_opaque` at `context.rs:299-323` now covers `Set`/`Iterable`/`Iterator`/`Awaitable`/async wrappers.
- `python_context_entered_borrow_cannot_move_through_temporary_method_argument` at `crates/sifr_lowering/src/lower/python_interop_tests.rs:612-629` pins method + staticmethod aggregate coverage. All previous round-1..8 shapes (yield, unpack, chained/aug-assign, for-iterable, walrus, Stmt::Expr discard, match-capture) still fire when re-checked.

## Fresh finding — `Stmt::Return` bypasses the centralized call-arg borrow guard

**File:** `crates/sifr_lowering/src/lower/return_lowering.rs:13-124` (`lower_return`), reinforced by the shape of `transfer_return_ownership` at `return_lowering.rs:126-232`.

`lower_return` lowers the return value with `lower_expr` and then only calls `transfer_return_ownership`. The recursion at `return_lowering.rs:126-232` walks `Name`, aggregate literals, `ConstructorCall`, `IteratorCall`, `OkWrap`/`QuestionMark`/`ErrWrap`, `IfExpr`, and comprehensions/generators — **it never visits `HirExpr::Call`, `HirExpr::PythonCall`, `HirExpr::IntrinsicCall`, `HirExpr::SuperCall`, or `HirExpr::MethodCall`**, and `lower_return` itself never invokes `python_context_borrow_in_owned_expr`. Consequently, the centralized `python_context_borrow_in_call_args` analysis added in round 8 is unreachable from the return path. This is the exact analog of round-7 case (3), fixed for `Stmt::Expr` (via `reject_python_context_borrow_discard` at `statement_dispatch.rs:256-260`) but not for `Stmt::Return`.

Concrete reproductions — each returns `no errors found` on the current diff. All were verified via `SIFR_SYSROOT=$repo target/release/sifr check <file>` (from `/tmp` to bypass the package-boundary check):

1. **Regular call, aggregate arg, `None` return** — `/tmp/wave3audit9/return_call_agg.sifr`:
   ```python
   def sink_list(values: list[Transaction]) -> None: return None
   def leak() -> Result[None, PythonError]:
       try:
           with make_transaction() as transaction:
               return sink_list([transaction])       # temp list drops borrow before __exit__
       except PythonError as error:
           raise error
   ```
   Rewriting the body as `sink_list([transaction]); return None` immediately fires `SIFR-PYCTX-0001 … cannot be discarded`, confirming the divergence is specifically the return path.

2. **Method call, aggregate arg, `None` return** — `/tmp/wave3audit9/return_method_agg.sifr` with `return sink.collect([transaction])`. Verified: `no errors found`.

3. **`ClassName.method(...)` (staticmethod-shape), aggregate arg, `None` return** — `/tmp/wave3audit9/return_static_agg.sifr` with `return Registry.store([transaction])`. Verified: `no errors found`.

4. **Non-`None` return type (int)** — `return sink.score([transaction])` (`/tmp/wave3audit9/return_method_int.sifr`) and `return Registry.score([transaction])` (`/tmp/wave3audit9/return_static_int.sifr`), both with method returning `int`, both compile cleanly. This rules out the "return type must be opaque-holding" hypothesis: the escape is the temp-aggregate drop at the call site, independent of the callee's return type.

Trace for reproduction 1: `lower_return` at `return_lowering.rs:53-74` produces `HirExpr::Call { func: "sink_list", args: [HirExpr::ListLiteral { elements: [HirExpr::Name{name: "transaction", ..}], .. }], ty: Type::None }`. `transfer_return_ownership` matches `Call` — the arm at line 230 (`_ => {}`) — no error. `python_context_borrow_in_owned_expr` is never called on this shape from any return-adjacent guard. The temp `ListLiteral` holding the borrow is dropped when `sink_list` returns, closing the borrow before the enclosing with block's `__exit__` runs. Failure mode: `SIFR-PYCTX-0001 … cannot be moved or closed independently`, exactly the invariant wave-3 acceptance names.

Sibling shapes (also unchecked; not the primary finding but reinforce the systematic gap): `Stmt::Assert(is_ok([transaction]))` (`/tmp/wave3audit9/assert_call.sifr`), `Stmt::Delete` subscript index `del data[compute([transaction])]` (`/tmp/wave3audit9/delete_key.sifr`), and `Stmt::If`/`Stmt::While` conditions `if is_ok([transaction]):` (`/tmp/wave3audit9/if_condition.sifr`) all return `no errors found` for the same reason — none of these statement-lowerers routes their sub-expressions through `python_context_borrow_in_owned_expr` / `reject_python_context_borrow_discard`. The `Stmt::Expr` fix in round 8 is a point solution; the general principle "any statement-borne expression that isn't visited by an entry to `python_context_borrow_in_owned_expr` is a candidate escape" (identified in round 5) recurs at every statement kind that skipped the sweep.

A secondary systematic gap outside the return path: `python_context_borrow_in_call_args` (`context.rs:106-117`) unconditionally skips direct-`Name` args when `type_can_hold_python_opaque(return_type, ctx) == false`. This is safe for a call that reads its arg by borrow, but consuming methods that move the arg into a caller-owned receiver (`list.append`, `set.add`, dict-`update`, etc.) also match this shape. Reproduction `/tmp/wave3audit9/list_append.sifr` — `def leak(mut stored: list[Transaction])` with `stored.append(transaction)` — returns `no errors found`; `transaction` (a borrow) is moved into the caller's `list[Transaction]`, which the caller retains after `__exit__` closes the borrow. `stored.add(transaction)` on `mut stored: set[Transaction]` reproduces identically (`/tmp/wave3audit9/set_add.sifr`). This is not caught anywhere else, because `lower_method_call` at `methods_lambdas_and_comprehensions.rs:279-284` never routes args through `mark_moved_with_flow`. This is HIGH but scope-secondary to the return-path bypass.

**Fix locus (unverified; do not implement in this pass):**

- Primary (return-path bypass): in `lower_return` at `return_lowering.rs:53-74`, after `lower_expr(val, ctx)?`, invoke `python_interop::python_context_borrow_in_owned_expr(&expr, ctx)` (or route via a new `reject_python_context_borrow_return` helper) and emit `PYCTX_INVALID_DECLARATION` when it returns `Some(_)`. Alternatively, extend `transfer_return_ownership` at `return_lowering.rs:126-232` to add a `HirExpr::Call | PythonCall | IntrinsicCall | SuperCall | MethodCall` arm that walks `args` (and for `MethodCall`, the receiver) using `python_context_borrow_in_call_args` semantics — mirroring the existing `IteratorCall` arm at lines 175-179. The chained-assign fix at `patterns_and_assignments.rs:482-486` and the round-8 method-aggregate fix in `context.rs:106-117` are the closest structural analogues.
- Secondary sweep (statement-condition bypasses): apply `reject_python_context_borrow_discard` (or a dedicated helper) to `Stmt::Assert.test`/`msg` at `statement_dispatch.rs:322-330`, to the subscript index in `Stmt::Delete` at `statement_dispatch.rs:309-312`, and to condition expressions in `lower_if`/`lower_while` at `control_flow.rs:446, 511, 648`. Also to `Stmt::Raise.exc` at `statement_dispatch.rs:338` (even though today's type guard rejects raising `Transaction` directly, a sub-call carrying the borrow via a temp aggregate is not covered).
- Tertiary (collection-mutation escape): change `python_context_borrow_in_call_args` at `context.rs:106-117` to always walk direct-`Name` args, or teach `lower_method_call` at `methods_lambdas_and_comprehensions.rs:168-183` to `mark_moved_with_flow` on args to mutating methods (`append`, `insert`, `extend`, `add`, `discard`, dict subscript-set, etc.) — those are the methods that consume-into-receiver.
- Regression coverage: add sibling tests to `python_interop_tests.rs` alongside `python_context_entered_borrow_cannot_move_through_temporary_method_argument` (`python_interop_tests.rs:612-629`) exercising `return sink_list([transaction])`, `return sink.collect([transaction])`, `return Registry.store([transaction])`, `return sink.score([transaction])`, `stored.append(transaction)`, `stored.add(transaction)`, `assert is_ok([transaction])`, `del data[compute([transaction])]`, and `if is_ok([transaction]):`.

Severity: HIGH — reproducible, no diagnostic emitted, and the failure mode is "context-scoped borrow moved into a temp aggregate that closes independently of `__exit__`" — exactly the "cannot be moved or closed independently" invariant wave-3 acceptance is meant to enforce, and a direct sibling of the round-7 case (3) that round 8 explicitly claimed to close.

VERDICT: NOT SATISFIED
