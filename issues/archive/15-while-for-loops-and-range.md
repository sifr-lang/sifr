## While/For Loops, Break/Continue, and range()

#### **Current Situation**

- The Sifr compiler (M1) supports functions, if/else, primitives, and basic expressions but has no loop constructs.
- Programs cannot iterate over data or repeat operations, severely limiting what can be expressed.
- The ruff-forked parser already supports `while`, `for`, `break`, `continue` AST nodes from Python syntax, but the HIR, type system, and codegen do not handle them.

#### **Desired Situation**

- `while` loops compile and run correctly, including nested loops.
- `for` loops over `range()` compile and run correctly.
- `break` and `continue` statements work inside loops.
- `range(n)` and `range(start, end)` are supported as built-in functions.
- Loop variables in `for` loops are automatically typed as `int` when iterating over a range.
- Errors are reported for `break`/`continue` used outside of loops.
- E2E tests verify all loop constructs produce correct output.

#### **Suggested Solution**

1. **sifr_type_system** changes:
   - Add `Type::Range` variant (maps to Rust `std::ops::Range<i64>`)
   - Add `range` to built-in functions in type checking

2. **sifr_hir** changes:
   - Add `HirStmt::While { condition: HirExpr, body: Vec<HirStmt> }`
   - Add `HirStmt::For { target: String, target_ty: Type, iter: HirExpr, body: Vec<HirStmt> }`
   - Add `HirStmt::Break` and `HirStmt::Continue`
   - Add `HirExpr::RangeLiteral { start: Option<Box<HirExpr>>, end: Box<HirExpr>, ty: Type }`
   - Implement `lower_while`: lower condition + body, push/pop scope for body
   - Implement `lower_for`: lower iterable, infer target type from iterable element type, define target in scope, lower body
   - Implement `lower_break` / `lower_continue`: validate inside loop context
   - Track loop nesting depth in `LowerCtx` for break/continue validation
   - Register `range` as built-in function

3. **sifr_codegen** changes:
   - Emit `while condition { body }` for while loops
   - Emit `for target in start..end { body }` for for-range loops
   - Emit `break;` and `continue;`
   - Emit `start..end` for range literals

4. **Tests:**
   - Unit tests for loop lowering and type inference
   - E2E pass tests: while_loop.sifr, for_loop.sifr, for_range.sifr, nested_loops.sifr, break_continue.sifr
   - E2E fail tests: break_outside_loop.sifr, continue_outside_loop.sifr
