## Update HIR Lowering to Use CFG and Narrowing

#### **Current Situation**

- HIR lowering (`crates/sifr_hir/src/lower.rs`) processes if/else branches but does not narrow variable types within branches.
- The parser already handles `isinstance()` calls and `is None` comparisons as regular expressions, but the lowering pass does not extract narrowing conditions from them.
- There are no HIR nodes for type alias statements or `reveal_type()` calls.
- Union type annotations (e.g., `int | str`) in function parameters and return types are not parsed/resolved.

#### **Desired Situation**

- When lowering an `if isinstance(x, int):` statement, the lowering pass:
  1. Detects the isinstance pattern and creates a `NarrowingCondition::IsInstance`
  2. Narrows `x` to `int` in the then-branch scope
  3. Narrows `x` to the complement type in the else-branch scope
  4. Restores the original type after the if/else merge point
- Similarly for `is None`, `is not None`, equality checks, and truthiness checks.
- `type X = Y | Z` statements create type aliases in the scope.
- `reveal_type(expr)` emits a compiler diagnostic showing the inferred type.
- Union type annotations in function signatures are resolved correctly.
- The HIR nodes carry narrowed types so codegen can emit correct match expressions.

#### **Suggested Solution**

**Modified files:**
- `crates/sifr_hir/src/hir_nodes.rs`:
  - Add `HirStmt::TypeAlias { name, ty }` for `type X = ...` statements
  - Add `HirExpr::IsInstance { value, target_type }` for isinstance checks
  - Add `HirExpr::IsNone { value }` and `HirExpr::IsNotNone { value }` for None checks
  - Add narrowing metadata to `HirStmt::If` (narrowing condition, narrowed types per branch)
- `crates/sifr_hir/src/lower.rs`:
  - Add `detect_narrowing_condition()`: analyze if-condition to extract NarrowingCondition
  - Update `lower_if()`: save scope narrowing state, apply narrowing in then/else branches, restore at merge
  - Add `lower_type_alias()`: register type alias in scope
  - Add `lower_reveal_type()`: emit diagnostic with current type
  - Update `resolve_type_from_annotation()`: handle `BinOp::BitOr` as union type syntax, handle literal values in type position
  - Update function signature lowering to handle union return types and parameters
- `crates/sifr_hir/src/scope.rs`:
  - Add type alias registry (`HashMap<String, Type>`)
  - Add narrowing state save/restore methods
  - Add `lookup_type_alias()` method
- `crates/sifr_python_parser` / `sifr_python_ast`: May need minor updates if `type` statement or `isinstance` are not already parsed (verify first).

**Integration tests:**
- Lower a function with `isinstance` narrowing and verify HIR has narrowed types
- Lower a function with `is None` check and verify narrowing
- Lower a `type` alias statement and verify it resolves in subsequent usage
