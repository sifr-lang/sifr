## List, Dict, and Tuple Types with Collection Operations

#### **Current Situation**

- The Sifr type system only supports primitive types (int, float, bool, str, None) and functions.
- There are no compound data types for modeling collections of data.
- Programs cannot create lists, dictionaries, or tuples, making it impossible to work with structured data.
- The ruff-forked parser already supports list/dict/tuple literal syntax and subscript expressions from Python, but the HIR, type system, and codegen do not handle them.

#### **Desired Situation**

- `list[T]` type works with literal syntax (`[1, 2, 3]`), type annotations, `append()`, `len()`, and indexing.
- `dict[K, V]` type works with literal syntax (`{"a": 1}`), type annotations, key access, `len()`.
- `tuple[A, B, ...]` type works with literal syntax (`(1, "hello")`), type annotations, and positional indexing.
- All collection types are Move types (consistent with ownership model).
- `len()` built-in works on all collections and strings.
- Type inference works for collection literals (e.g., `[1, 2, 3]` infers `list[int]`).
- For loops can iterate over lists (in addition to ranges from Task 1).
- `in` operator works for membership testing on lists, dicts, and strings.
- Indexing (`x[0]`) and slicing (`x[1:3]`) work on lists and strings.
- E2E tests verify all collection operations.

#### **Suggested Solution**

1. **sifr_type_system** changes:
   - Add `Type::List(Box<Type>)` — maps to `Vec<T>`
   - Add `Type::Dict(Box<Type>, Box<Type>)` — maps to `HashMap<K, V>`
   - Add `Type::Tuple(Vec<Type>)` — maps to `(A, B, C)`
   - Update `ownership()`: List, Dict, Tuple are Move types
   - Update `rust_type()`: generate `Vec<T>`, `HashMap<K, V>`, `(A, B, C)`
   - Update `display_name()`: show `list[int]`, `dict[str, int]`, `tuple[int, str]`
   - Update `is_assignable_to()`: structural subtyping for collections
   - Add type checking for indexing (list by int, dict by key type, tuple by literal int)
   - Add type checking for `in` operator (returns bool)
   - Add type checking for method calls (append, len, etc.)

2. **sifr_hir** changes:
   - Add `HirExpr::ListLiteral { elements: Vec<HirExpr>, ty: Type }`
   - Add `HirExpr::DictLiteral { keys: Vec<HirExpr>, values: Vec<HirExpr>, ty: Type }`
   - Add `HirExpr::TupleLiteral { elements: Vec<HirExpr>, ty: Type }`
   - Add `HirExpr::Index { object: Box<HirExpr>, index: Box<HirExpr>, ty: Type }`
   - Add `HirExpr::Slice { object: Box<HirExpr>, lower: Option<Box<HirExpr>>, upper: Option<Box<HirExpr>>, ty: Type }`
   - Add `HirExpr::MethodCall { object: Box<HirExpr>, method: String, args: Vec<HirExpr>, ty: Type }`
   - Add `HirExpr::ContainsOp { element: Box<HirExpr>, collection: Box<HirExpr>, ty: Type }`
   - Implement lowering for list/dict/tuple literals with element type inference
   - Implement lowering for subscript (indexing + slicing)
   - Implement lowering for attribute access (method calls)
   - Implement lowering for `in` operator (Compare node with `In` op)
   - Resolve `list[T]`, `dict[K,V]`, `tuple[A,B]` type annotations
   - Update for-loop lowering to support iterating over lists (infer element type)
   - Register `len` as built-in function

3. **sifr_codegen** changes:
   - Emit `vec![elem1, elem2, ...]` for list literals
   - Emit `std::collections::HashMap::from([(k1, v1), ...])` for dict literals
   - Emit `(elem1, elem2, ...)` for tuple literals
   - Emit `x[index]` for list/tuple indexing, `x[key].clone()` or `x.get(&key)` for dict
   - Emit `x[lower..upper].to_vec()` for slicing
   - Emit `x.push(val)` for append, `x.len()` for len
   - Emit `x.contains(&val)` for list/string `in`, `x.contains_key(&key)` for dict `in`
   - Emit `for target in collection.iter() { body }` for list iteration
   - Add `use std::collections::HashMap;` when dicts are used

4. **Tests:**
   - Unit tests for collection type checking and inference
   - E2E pass tests: list_ops.sifr, dict_ops.sifr, tuple_ops.sifr, collection_iteration.sifr, indexing.sifr, in_operator.sifr
   - E2E fail tests: list_type_mismatch.sifr, wrong_index_type.sifr, dict_wrong_key.sifr
