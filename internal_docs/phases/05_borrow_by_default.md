# Borrow-by-Default (Original Phase 05 -- Superseded)

> **Note:** This phase was never executed. It has been superseded by **Phase 10: Borrow-by-Default** in the revised roadmap (`10_borrow_by_default.md`). The revised Phase 10 incorporates the content below plus additional milestones (escape analysis, consuming-self receivers, for-loop element semantics, stdlib ownership patterns) identified by post-Phase-07 audits. This file is preserved as a reference for the original detailed implementation plan, which Phase 10's `milestone_borrow_default` references.

This phase changes Sifr's function parameter passing from move-by-default to borrow-by-default. Function arguments are immutably borrowed (`&T`) by default, with opt-in `mut` (mutable borrow, `&mut T`) and `own` (ownership transfer, `T`) keywords. This eliminates "use-after-move" friction for the common case, unifies the two-tier system where built-in functions borrow and user-defined functions move, and establishes the ownership foundation required for fearless concurrency in the async phase.

## milestone_borrow_default: Borrow-by-Default Parameter Passing

status: superseded by Phase 10 (historical reference only)

**Goal:** Change Sifr's function parameter passing from move-by-default to borrow-by-default. Function arguments are immutably borrowed by default (`&T`), with opt-in `mut` (mutable borrow, `&mut T`) and `own` (ownership transfer, `T`) keywords. Copy types (`int`, `float`, `bool`) always pass by value. This unifies the existing two-tier system where built-in functions borrow (via a hardcoded `borrows_args` list) and user-defined functions move.

### 1. ParamConvention Enum and Signature Propagation

Add a `ParamConvention` enum to the type system with three variants:

- `Borrow` -- immutable borrow (default for Move types). Codegen: `&T`
- `MutBorrow` -- mutable borrow (`mut` keyword). Codegen: `&mut T`
- `Own` -- ownership transfer (`own` keyword). Codegen: `T`

Extend `FunctionType` to carry conventions alongside parameter types:

- `FunctionType.params`: change from `Vec<(String, Type)>` to `Vec<(String, Type, ParamConvention)>`
- `Callable` type variant: extend from `Callable(Vec<Type>, Box<Type>)` to `Callable(Vec<Type>, Vec<ParamConvention>, Box<Type>)`

This ensures conventions are available at every call site -- including cross-module imports, stdlib lookups, and `Callable`-typed variable calls. Without this, the codegen cannot determine whether to emit `&arg`, `&mut arg`, or `arg` for calls to functions defined outside the current compilation unit.

**Key files:** `crates/sifr_type_system/src/types.rs` (ParamConvention, FunctionType, Callable), all callers that construct FunctionType/Callable

### 2. Parser: `mut` and `own` Soft Keywords

Parse `mut` and `own` as soft keywords before parameter names in function definitions. These are not Python keywords, so they appear as identifiers and can be detected by peeking at the token before the parameter name.

```python
def process(items: list[int]) -> int:       # borrows items (default)
    return len(items)

def sort_it(mut items: list[int]):           # mutably borrows items
    items.sort()

def consume(own items: list[int]) -> int:    # takes ownership
    return len(items)
```

Add a `convention` field to the `Parameter` AST node.

**Key files:** `third_party/ruff/crates/ruff_python_parser/src/parser/statement.rs` (parse_parameter), `third_party/ruff/crates/ruff_python_ast/src/nodes.rs` (Parameter struct)

### 3. HIR: Convention on HirParam

Add a `convention: ParamConvention` field to `HirParam`. In `lower_function`, propagate the convention from each AST `Parameter` to the corresponding `HirParam`. Default convention: `Borrow` for Move types, `Own` for Copy types (Copy types are always passed by value regardless).

**Key files:** `crates/sifr_ir` (HirParam), `crates/sifr_lowering/src/lower/` (lower_function)

### 4. HIR: Delete `borrows_args` and Update All Call Paths

Delete the `borrows_args` match block in `lower.rs` that special-cases 25 built-in function names. Replace with convention-aware logic:

- Look up the called function's parameter conventions
- Only call `mark_moved(name)` if the corresponding parameter has `convention == Own` AND the argument type is `Move`
- For `MutBorrow` parameters: track that the variable is mutably borrowed (no move)
- For `Borrow` parameters: no move tracking needed

Apply this convention-aware logic to **all call paths** in `lower.rs`:

- Regular function calls (the main path)
- `Callable`-typed variable calls (extract conventions from the `Callable` type variant)
- Method calls (non-self parameters propagate conventions through `HirParam`)

**Note:** Constructor calls do not need convention changes -- constructors always take ownership of their arguments. Method `self` receivers continue to use auto-inference (`&self`/`&mut self` from body analysis).

**Key files:** `crates/sifr_lowering/src/lower/` (function call lowering, callable_info path, lower_method_call)

### 5. Codegen: Extend `func_signatures`, Register Class Methods, Emit `&T` / `&mut T` / `T`

Change the codegen-internal `func_signatures` map from `HashMap<String, (Vec<Type>, Type)>` to `HashMap<String, (Vec<(Type, ParamConvention)>, Type)>` so conventions are available at every call site. Register both top-level functions and class/static methods (under the `ClassName::method` key) during `collect_union_types`.

Update `emit_function` to emit parameter types based on convention:

- `Borrow` + Move type: emit `&T` (e.g., `&Vec<i64>`, `&String`)
- `Borrow` + Copy type: emit `T` (e.g., `i64`, `f64`, `bool`)
- `MutBorrow`: emit `&mut T`
- `Own`: emit `T` (current behavior)

Update call-site emission for `HirExpr::Call` to prepend `&` or `&mut` for Move-type arguments based on the callee's parameter conventions (looked up from `func_signatures`).

Update call-site emission for `HirExpr::MethodCall` (`obj.method(arg)`) to use convention-aware argument emission. The current codegen uses a hardcoded heuristic (`if matches!(arg.ty(), Type::Class { .. }) { write("&") }`). Replace this with convention lookup: resolve the method's `HirParam` conventions from the object's class type, then emit `&arg`/`&mut arg`/`arg` per convention. This applies to the `Type::Class` match arm and the fallback arm in the `MethodCall` handler.

**Key files:** `crates/sifr_codegen/src/lib.rs` (func_signatures type, collect_union_types, HirExpr::Call emission, HirExpr::MethodCall emission)

### 6. Codegen: Handle Borrowed Parameter Usage in Function Bodies

When a parameter is borrowed (`&T`), code inside the function body needs adjustment:

- Read access: works naturally via Rust auto-deref
- Passing to another function that also borrows: re-borrow via Rust deref coercion (automatic)
- Passing to a function that takes `own`: compiler error -- "cannot move borrowed parameter -- use `own` or `.clone()`"
- Returning the parameter: compiler error -- "cannot return borrowed parameter -- use `own` or `.clone()`"
- Storing into a struct field or collection: compiler error -- same diagnostic as returning

**Important:** The compiler does NOT silently emit `.clone()`. Per the Borrow and Lifetime Strategy contract, the compiler emits a diagnostic rather than silently cloning. The programmer must choose: add `own` to the parameter, call `.clone()` explicitly, or restructure to avoid the escape.

**Key files:** `crates/sifr_codegen/src/lib.rs`, `crates/sifr_lowering/src/lower/` (escape detection)

### Definition of Done (milestone_borrow_default)

- `ParamConvention` enum exists in the type system
- `FunctionType.params` carries conventions; `Callable` type variant carries conventions
- `mut`/`own` keywords parse correctly on function parameters
- Convention propagates from AST through HIR to codegen
- User-defined functions emit `&T` by default for Move-type params
- `borrows_args` hardcoded list is deleted
- All call paths (regular, Callable, method) use convention-aware move tracking
- Call sites emit `&arg`/`&mut arg`/`arg` based on callee conventions
- Borrowed parameter escape (return/store) produces a compiler error, not silent `.clone()`
- Existing E2E tests pass (with necessary adjustments for new semantics)
- Basic borrow-by-default programs compile and run correctly

---

## milestone_borrow_hardening: Borrow Exclusivity and Diagnostics

status: superseded by Phase 10 (historical reference only)

**Goal:** Harden the borrow-by-default model with exclusivity enforcement, clear error messages, comprehensive tests, and stdlib updates. This milestone ensures the ownership model is production-ready and documented before async/concurrency features are built on top.

### 1. Mutable Borrow Exclusivity Tracking

Add `is_mut_borrowed` tracking to `VarInfo` in scope. Implement:

- `mark_mut_borrowed(name)` -- marks a variable as mutably borrowed
- `is_mut_borrowed(name)` -- checks if mutably borrowed
- `clear_mut_borrow(name)` -- clears after the borrowing call returns

Enforce exclusivity rules:

- Cannot pass the same variable as `mut` twice in the same call
- Cannot pass a variable as both `mut` and immutable borrow in the same call
- Error: "cannot borrow `x` as mutable because it is already borrowed"

**Key files:** `crates/sifr_lowering/src/scope.rs` (VarInfo), `crates/sifr_lowering/src/lower/` (function call lowering)

### 2. Error Messages

Add clear, actionable diagnostic messages:

- "use of moved value: 'x'" -- only for `own` parameters now
- "cannot mutate borrowed parameter 'x' -- add `mut` to the parameter"
- "cannot return borrowed parameter 'x' -- use `own` or `.clone()`"
- "cannot borrow 'x' as mutable because it is already borrowed"

**Key files:** `crates/sifr_lowering/src/lower/`, `crates/sifr_driver/src/lib.rs`

### 3. Update Borrowing Audit Tests

Update the 50 tests in `audits/borrowing/` to reflect borrow-by-default semantics:

- Tests 08, 23 (function move): change to borrow-by-default behavior (pass succeeds, variable still usable)
- Tests 09, 24, 31 (use-after-move via function): update to use `own` keyword, verify they still fail correctly
- Tests 16 (move-in-loop): update to use `own` keyword
- Tests 01-07, 11-15, 17-22, 25-30, 32-50: verify unchanged behavior

Update `audits/borrowing/POST_HARDENING_REPORT.md` to document the new ownership model.

**Key files:** `audits/borrowing/*.sifr`, `audits/borrowing/POST_HARDENING_REPORT.md`

### 4. New E2E Tests

Create pass tests in `crates/sifr/tests/e2e/pass/`:

- `borrowed_parameters.sifr` -- function args borrowed by default, usable after call
- `mut_param.sifr` -- `mut` parameter allows in-place mutation
- `own_param.sifr` -- `own` parameter moves, caller loses access
- `borrow_in_loop.sifr` -- borrowed args in loops work without issues
- `mut_exclusivity.sifr` -- valid uses of `mut` with different variables

Create fail tests in `crates/sifr/tests/e2e/fail/`:

- `mutate_borrowed_param.sifr` -- cannot mutate a default-borrowed param
- `return_borrowed_param.sifr` -- cannot return a borrowed param without clone
- `double_mut_borrow.sifr` -- cannot mut-borrow same variable twice

**Key files:** `crates/sifr/tests/e2e/pass/`, `crates/sifr/tests/e2e/fail/`

### 4b. Parser Snapshot Tests

Add parser snapshot tests for `mut`/`own` soft keyword edge cases:

- `mut` and `own` used as parameter names (not keywords) -- `def f(mut: int)` parses as parameter named `mut`
- `mut`/`own` before typed parameters -- `def f(mut x: int)` parses as convention + name
- `mut`/`own` before untyped parameters -- `def f(mut x)` parses correctly
- Nested function parameters with conventions -- `def f(mut x: list[int], own y: str)`

**Key files:** `third_party/ruff/crates/ruff_python_parser/tests/`

### 4c. Multi-Module Convention Tests

Add tests that verify conventions survive across module boundaries:

- Import a function with `mut`/`own` params from another module, call it, verify correct borrow/move behavior
- Verify that `FunctionType` carries conventions through the import/export pipeline
- Test `Callable`-typed variables with conventions passed across function boundaries

**Key files:** `crates/sifr/tests/e2e/pass/`, `crates/sifr_driver/`

### 5. Stdlib Updates

- `sifr.collections` mutating functions (`set_add`, `set_remove`, `defaultdict_set`) get `mut` on their first parameter
- `str.join(items)` codegen adjusted to borrow the list parameter instead of moving it

**Key files:** `crates/sifr_stdlib`, `crates/sifr_codegen/src/lib.rs`

### Concurrency Enablement

This milestone completes the foundation for fearless concurrency in `milestone_async_core`:

- **Spawning tasks requires `own`**: `asyncio.spawn(process(own data))` -- ownership transfer is explicit and visible at the call site
- **Borrowed values cannot cross task boundaries**: the compiler rejects `&T` in spawned closures because borrows are not `'static`
- **`mut` borrows enforce exclusivity**: prevents data races at compile time (same as Rust's `&mut` aliasing rule)
- **Channel ownership**: `sifr.sync.Channel.send(own value)` makes it clear that sending through a channel transfers ownership

### Impact on Standard Library

- **95% of stdlib functions already borrow** in the codegen (using `.iter()`, `&expr`, etc.) -- borrow-by-default matches existing behavior
- The hardcoded `borrows_args` list of 25 built-in names is eliminated
- Future stdlib additions (functools, itertools, heapq, etc.) naturally use borrow-by-default with explicit `mut`/`own` where needed

### Definition of Done (milestone_borrow_hardening)

- Exclusivity errors caught by `sifr check` with clear error messages
- All 50 borrowing audit tests updated and passing/failing correctly
- New E2E pass/fail tests for borrowed_parameters, mut_param, own_param, exclusivity
- Parser snapshot tests cover `mut`/`own` soft keyword edge cases
- Multi-module convention tests verify `FunctionType`/`Callable` convention propagation across imports
- Stdlib works correctly with borrow-by-default
- Architecture documentation updated (Borrow and Lifetime Strategy, Ownership Model)
- `audits/borrowing/POST_HARDENING_REPORT.md` reflects new model

---

## Milestone ordering

The milestones within this phase are ordered as follows:

- **milestone_phase_fixes before milestone_borrow_default:** The language must be fully hardened before changing the default parameter passing convention. Borrow-by-default is a semantic change that affects every user-defined function -- it must build on a stable foundation.
- **milestone_borrow_default before milestone_async_core:** Borrow-by-default is a prerequisite for fearless concurrency. The `own` keyword makes ownership transfer explicit at task spawn boundaries. Without it, milestone_async_core would need to re-implement parameter convention logic.
- **milestone_borrow_hardening after milestone_borrow_default:** Exclusivity checking and error messages build on the working borrow-by-default codegen. Tests validate the complete model.
- **milestone_borrow_hardening before milestone_intrinsics:** The ownership model must be fully hardened (with exclusivity enforcement) before rewriting the stdlib architecture. Stdlib `.sifr` files must be written against the final borrow-by-default semantics -- retrofitting convention annotations after the fact would be error-prone and wasteful.
