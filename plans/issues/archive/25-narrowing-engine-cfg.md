## Build Narrowing Engine and Control Flow Graph

#### **Current Situation**

- The sifr compiler has no control flow graph (CFG). HIR lowering processes statements sequentially without tracking how control flows through branches.
- There is no type narrowing. Inside an `if isinstance(x, int):` branch, `x` still has its declared union type.
- The scope (`sifr_hir/src/scope.rs`) tracks `VarInfo` with only `declared_type` and `is_moved` -- no narrowed type.

#### **Desired Situation**

- A control flow graph is built during HIR lowering, tracking how variables' types change through branches.
- A narrowing engine can narrow a type based on conditions: truthiness, isinstance, equality, is None, type predicates, and negation.
- The scope tracks `narrowed_type` per variable, which starts equal to `declared_type` and is updated by the narrowing engine at branch points.
- After an if/else, types are restored (or joined) at the merge point.

#### **Suggested Solution**

**New files:**
- `crates/sifr_type_system/src/narrow.rs`: The narrowing engine.
  - `NarrowingCondition` enum: Truthiness, IsNone, IsNotNone, IsInstance, Equality, TypePredicate, Not, And, Or.
  - `narrow_type(ty: &Type, condition: &NarrowingCondition, is_true: bool) -> Type`: Core narrowing function.
  - `subtract_type(from: &Type, to_remove: &Type) -> Type`: Remove a type from a union (for else branches).
  - `intersect_type(ty: &Type, target: &Type) -> Type`: Narrow to intersection (for then branches).
- `crates/sifr_hir/src/cfg.rs`: Control flow graph.
  - `FlowNode` enum: Start, Assignment, Condition, Label (join point), Unreachable.
  - `ControlFlowGraph` struct: Arena of FlowNodes, current node tracking.
  - Methods to create nodes, link antecedents, and walk the graph.

**Modified files:**
- `crates/sifr_hir/src/scope.rs`: Add `narrowed_type: Option<Type>` to `VarInfo`. Add methods `narrow_var()`, `restore_narrowing()`, `save_narrowing_state()`, `merge_narrowing()`.
- `crates/sifr_type_system/src/lib.rs`: Export `narrow` module.
- `crates/sifr_hir/src/lib.rs`: Export `cfg` module.

**Unit tests:**
- Narrow `int | str` with `IsInstance(_, Int)` true -> `Int`
- Narrow `int | str` with `IsInstance(_, Int)` false -> `Str`
- Narrow `str | None` with `IsNotNone` true -> `Str`
- Narrow `str | None` with `Truthiness` true -> `Str`
- Narrow `"GET" | "POST"` with `Equality(_, "GET")` true -> `LiteralStr("GET")`
- Subtract: `subtract_type(Union(Int, Str), Int)` -> `Str`
