# PRD: milestone_pattern_matching

## Goal

Add Python 3.10-style structural pattern matching (`match`/`case`) to Sifr. The parser already supports the syntax (inherited from ruff). This milestone implements HIR lowering, type checking, and Rust codegen.

## Scope

### Supported Patterns (Phase 1 - this milestone)

1. **Literal patterns**: `case 42:`, `case "hello":`, `case True:`
2. **Capture patterns**: `case x:` (binds value to `x`)
3. **Wildcard pattern**: `case _:` (matches anything)
4. **None pattern**: `case None:` (matches `None` in `T | None`)
5. **OR patterns**: `case "GET" | "POST":` (matches either)
6. **Guard patterns**: `case x if x > 0:` (pattern + condition)
7. **Class patterns**: `case Circle(radius=r):` (destructures class instances)
8. **Value patterns**: `case Color.RED:` (matches enum-like attribute access)

### Exhaustiveness Checking

- For union types (`int | str`): every variant must be covered or wildcard present
- For `T | None`: both `T` and `None` must be covered
- Missing coverage → compile error listing uncovered cases
- Non-exhaustive match on non-union type → require `case _:`

### Type Narrowing

- Each `case` arm narrows the subject type in the arm body
- `case None:` narrows `T | None` to `None`
- `case int() as n:` narrows `int | str` to `int`
- `case Circle(radius=r):` narrows `Circle | Square` to `Circle`

## Architecture

### HIR Changes

Add `HirStmt::Match` and `HirPattern` enum to `hir_nodes.rs`:

```rust
Match {
    subject: HirExpr,
    subject_ty: Type,
    arms: Vec<HirMatchArm>,
}

struct HirMatchArm {
    pattern: HirPattern,
    guard: Option<HirExpr>,
    body: Vec<HirStmt>,
    narrowed_bindings: Vec<(String, Type)>,
}

enum HirPattern {
    Wildcard,
    Capture { name: String, ty: Type },
    Literal { value: HirExpr },
    None,
    Or { patterns: Vec<HirPattern> },
    Class { class_name: String, fields: Vec<(String, HirPattern)> },
    Value { path: Vec<String> },  // for Color.RED style
}
```

### HIR Lowering

- Lower `StmtMatch` from AST to `HirStmt::Match`
- Resolve subject type, lower each case arm
- For each pattern, determine narrowed type and bindings
- Exhaustiveness check: verify all variants covered

### Codegen

Map to Rust `match` expressions:

- `T | None` → `match value { Some(inner) => ..., None => ... }`
- `int | str` → `match value { IntOrStr::Int(n) => ..., IntOrStr::Str(s) => ... }`
- Literal patterns → `match value { 42 => ..., _ => ... }`
- Wildcard → `_ =>`
- Class patterns → struct destructuring
- OR patterns → `42 | 43 =>`
- Guards → `x if x > 0 =>`

## Test Plan

### E2E Pass Tests

- `match_literal.sifr` - match on integer/string literals
- `match_union.sifr` - match on `int | str` union type
- `match_optional.sifr` - match on `T | None`
- `match_wildcard.sifr` - wildcard `case _:`
- `match_or_pattern.sifr` - OR patterns `case "a" | "b":`
- `match_guard.sifr` - guard patterns `case x if x > 0:`
- `match_class_destructure.sifr` - class pattern destructuring

### E2E Fail Tests

- `match_non_exhaustive_union.sifr` - missing union variant
- `match_non_exhaustive_optional.sifr` - missing None case

## Definition of Done

- `match`/`case` compiles and runs correctly for all supported patterns
- Exhaustiveness checking works for union and optional types
- Type narrowing works in case bodies
- All existing E2E tests still pass
- New E2E pass/fail tests added
- Demo: `demos/milestone_pattern_matching_demo.sifr`
