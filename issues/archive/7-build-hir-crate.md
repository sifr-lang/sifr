## Build sifr_hir Crate

#### **Current Situation**

- The parser produces an untyped AST with no name resolution or type information.
- There is no intermediate representation that carries type information and resolved references.
- Ownership tracking (move vs copy) is not implemented.

#### **Desired Situation**

- A `sifr_hir` crate exists that lowers the untyped AST into a typed HIR.
- Every expression in the HIR carries its resolved `Type`.
- Every name reference is resolved to a definition (DefId).
- Scope-based name resolution works for function-level and block-level scopes.
- Ownership is tracked: move on assignment for `str`, copy for `int`/`float`/`bool`.
- Use-after-move errors are detected and reported.
- Mdtest and unit tests verify name resolution, type assignment, and ownership errors.

#### **Suggested Solution**

1. Create `crates/sifr_hir/` with HIR node types that mirror the AST but include:
   - `resolved_type: Type` on every expression
   - `def_id: DefId` on every name reference
   - `ownership: OwnershipState` on variable bindings (Owned, Moved, Borrowed)
2. Implement name resolution:
   - Scope stack (module scope, function scope, block scope)
   - Variable declarations add to current scope
   - Name lookups search from innermost to outermost scope
   - Undefined name produces an error
   - Function definitions are visible in module scope
3. Implement AST-to-HIR lowering:
   - Walk the AST and produce HIR nodes
   - Run type inference (using `sifr_type_system`) during lowering
   - Attach resolved types to all expression nodes
4. Implement ownership tracking:
   - Track which variables are in "owned" vs "moved" state
   - Assignment of a move-type variable marks it as "moved"
   - Using a moved variable produces a "use-after-move" error
   - Copy-type variables are never marked as moved
5. Add tests:
   - Unit tests for scope resolution
   - Mdtest for type resolution on expressions
   - Mdtest for use-after-move detection
   - Mdtest for undefined variable errors
