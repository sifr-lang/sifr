## Build sifr_type_system Crate

#### **Current Situation**

- There is no type system implementation for Sifr.
- The parser produces an untyped AST, but we need to enforce types at compile time.
- Sifr requires strict typing with opt-in `Any`, type inference from initializers, and ownership-aware type checking.

#### **Desired Situation**

- A `sifr_type_system` crate exists with a complete M1 type system.
- Types can be represented, compared, and checked.
- Type inference works for variable initializers.
- Function signatures are type-checked.
- Markdown-based tests (mdtest) verify type inference and error reporting.

#### **Suggested Solution**

1. Create `crates/sifr_type_system/` with a `Type` enum:
   - Primitives: `Int`, `Float`, `Bool`, `Str`, `None`
   - `Function(FunctionType)` with parameter types and return type
   - `Any` (escape hatch)
   - `Never` (bottom type)
2. Implement type inference:
   - Integer literals -> `Int`
   - Float literals -> `Float`
   - String literals -> `Str`
   - Boolean literals -> `Bool`
   - `None` literal -> `None`
   - Function call return type -> function's return type
3. Implement type checking:
   - Binary ops: arithmetic requires `Int`/`Float`, string concat requires `Str`
   - Comparison ops: require same type on both sides
   - Boolean ops: require `Bool`
   - Function calls: check argument types match parameter types
   - Assignment: check value type matches annotation (if present)
4. Implement subtyping rules:
   - `Never` is subtype of everything
   - `Any` is compatible with everything
   - Exact type match for primitives
5. Implement ownership classification:
   - `Int`, `Float`, `Bool` -> `Copy` (assignment copies)
   - `Str` -> `Move` (assignment moves)
6. Add mdtest markdown tests in `resources/mdtest/` covering:
   - Literal type inference
   - Variable type inference
   - Function parameter checking
   - Type mismatch errors
   - Ownership classification
