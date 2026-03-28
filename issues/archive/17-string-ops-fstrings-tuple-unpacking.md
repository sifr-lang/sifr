## String Operations, F-strings, and Tuple Unpacking

#### **Current Situation**

- Strings in Sifr only support concatenation (`+` operator) and basic print.
- There are no string methods (len, upper, lower, split, strip).
- There are no f-strings for string formatting.
- There is no tuple unpacking / multiple assignment syntax.
- These are essential for writing readable, practical programs.

#### **Desired Situation**

- String methods work: `.len()`, `.upper()`, `.lower()`, `.split()`, `.strip()`, `.startswith()`, `.endswith()`.
- F-string formatting works: `f"Hello {name}, you are {age} years old"`.
- Tuple unpacking works: `a, b = 1, 2` and `a, b = get_pair()`.
- All features have proper type checking and produce correct Rust code.
- E2E tests verify all string operations and tuple unpacking.

#### **Suggested Solution**

1. **sifr_type_system** changes:
   - Add method resolution for `Str` type: `.len() -> int`, `.upper() -> str`, `.lower() -> str`, `.split(sep: str) -> list[str]`, `.strip() -> str`, `.startswith(prefix: str) -> bool`, `.endswith(suffix: str) -> bool`
   - Add method resolution for `List` type: `.len() -> int`, `.append(T) -> None`
   - Add method resolution for `Dict` type: `.len() -> int`, `.keys() -> list[K]`, `.values() -> list[V]`
   - Create a method resolution system that maps (Type, method_name) -> (param_types, return_type)

2. **sifr_hir** changes:
   - Add `HirExpr::FString { parts: Vec<FStringPart>, ty: Type }` where `FStringPart` is either `Literal(String)` or `Expr(HirExpr)`
   - Add `HirStmt::TupleUnpack { targets: Vec<(String, Type)>, value: HirExpr }` for multiple assignment
   - Implement `lower_fstring`: parse f-string parts, type-check interpolated expressions
   - Implement `lower_tuple_unpack`: validate target count matches tuple length, define variables in scope
   - Extend method call lowering with the method resolution system

3. **sifr_codegen** changes:
   - Emit `format!("...", expr1, expr2)` for f-strings
   - Emit `.len()`, `.to_uppercase()`, `.to_lowercase()`, `.split(sep).collect::<Vec<_>>()`, `.trim()`, `.starts_with(prefix)`, `.ends_with(suffix)` for string methods
   - Emit `let (a, b) = expr;` for tuple unpacking
   - Emit `.len()`, `.push(val)` for list methods
   - Emit `.len()`, `.keys().cloned().collect()`, `.values().cloned().collect()` for dict methods

4. **Tests:**
   - Unit tests for method resolution and f-string type checking
   - E2E pass tests: string_methods.sifr, fstring.sifr, tuple_unpacking.sifr
   - E2E fail tests: wrong_method_args.sifr, tuple_unpack_mismatch.sifr
