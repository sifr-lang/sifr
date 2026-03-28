## Strip AST to M1-Relevant Nodes Only

#### **Current Situation**

- The forked `sifr_python_ast` crate contains the full Python AST with all node types (async, match, with, try, import, IPython, etc.).
- For M1, we only need a small subset: function definitions, if/elif/else, assignments, annotated assignments, return, expression statements, and basic expressions/literals.
- The extra nodes add complexity and confusion for agents working on the type system and codegen.

#### **Desired Situation**

- The `sifr_python_ast` crate contains only M1-relevant AST nodes.
- The `sifr_python_parser` still parses valid M1 syntax correctly.
- Unsupported syntax produces clear "not yet supported" errors.
- Parser snapshot tests are updated for the stripped AST.

#### **Suggested Solution**

1. In `sifr_python_ast`, keep only these statement nodes:
   - `StmtFunctionDef` (function definitions with type annotations)
   - `StmtIf` (if/elif/else)
   - `StmtAssign` (plain assignment)
   - `StmtAnnAssign` (annotated assignment like `x: int = 5`)
   - `StmtReturn`
   - `StmtExpr` (expression statements)
   - `StmtPass`
2. Keep only these expression nodes:
   - `ExprBoolOp`, `ExprBinOp`, `ExprUnaryOp`, `ExprCompare` (operators)
   - `ExprCall` (function calls)
   - `ExprName` (variable references)
   - `ExprNumberLiteral`, `ExprStringLiteral`, `ExprBooleanLiteral`, `ExprNoneLiteral` (literals)
   - `ExprIf` (ternary/conditional expression)
3. Remove or stub out: async, match, with, try/except, import, class (for now), yield, lambda, comprehensions, f-strings, starred, subscript, attribute, slice, etc.
4. Update the parser to emit "unsupported syntax" diagnostics for removed nodes.
5. Add parser snapshot tests for valid M1 syntax in `resources/valid/`.
6. Add parser snapshot tests for unsupported syntax in `resources/invalid/`.
