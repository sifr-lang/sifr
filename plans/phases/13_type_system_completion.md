# Type System Completion

**Why now:** The stdlib is complete and remediated (Phase 12), the ownership model is proven (Phase 10), and safety is enforced (Phases 8-9). But the type system has critical gaps that block every subsequent phase: user-facing generics are incomplete (generic class field/method substitution doesn't work), the stdlib is monomorphic (functions duplicated per type), there is no pattern matching, no enum type, no auto-generated constructors, integer overflow contradicts the "if it compiles, it works" guarantee, and generic container classes like `Counter` and `deque` are hardcoded to specific types. Every one of these gaps would infect the async runtime, typed serde, web extractors, and package ecosystem if left unfixed. Fixing them now means all subsequent phases build on a complete, expressive type system from day one.

**Why this ordering within the phase:** Each milestone builds on the previous one. Auto-init must come first because it eliminates boilerplate that would otherwise be duplicated in every subsequent milestone's new classes. User-facing generics come second because pattern matching, enums, and the stdlib rewrite all depend on them. Pattern matching comes third because enum types use it for exhaustive handling. Enums come fourth because they depend on both generics and pattern matching. Integer safety comes fifth to resolve the overflow contradiction before the stdlib rewrite. The stdlib generic rewrite comes last because it exercises everything built in milestones 1-5 and serves as the integration test for the entire phase.

---

## milestone_auto_init: Auto-Generated Constructors

status: done

**Goal:** Eliminate the most common boilerplate in Sifr code. When a class declares typed fields but does not define an explicit `__init__`, the compiler auto-generates a constructor that accepts one positional argument per field (in declaration order) and assigns each to `self`. This is the single highest-impact ergonomic improvement — every class in the demos, stdlib, and user code currently repeats this pattern manually.

**Depends on:** milestone_stdlib_remediation (Phase 12 must be complete; the remediated stdlib classes provide the test surface)

### Compiler Changes

#### 1. Auto-init detection and generation (sifr_lowering)

During HIR lowering, when a class has typed field declarations but no explicit `__init__` method:

- Collect all field declarations in declaration order: `[(name, type), ...]`
- Generate a synthetic `__init__` method in the HIR with:
  - Parameters: `self` + one parameter per field, same name and type as the field
  - Body: one `self.field = param` assignment per field
- The synthetic `__init__` must be indistinguishable from a hand-written one in all downstream phases (type checking, codegen, borrow checking)
- If the class has an explicit `__init__`, do nothing — the user's definition takes precedence

#### 2. Inheritance interaction

- If a child class extends a parent class and the child has no `__init__`:
  - The auto-generated `__init__` includes the child's own fields only
  - The compiler does NOT auto-call `super().__init__()` — the child must define an explicit `__init__` if it needs to initialize parent fields
  - This matches Python's behavior: if you inherit and don't define `__init__`, you get the parent's constructor (which only knows about parent fields)
- If a child class has new fields AND inherits from a parent:
  - The compiler emits a diagnostic: "class X has fields but no `__init__`; parent fields will not be initialized. Define an explicit `__init__` with `super().__init__(...)`"
  - This prevents silent bugs where parent fields are uninitialized

#### 3. Default field values

- Fields with default values (`x: int = 0`) generate parameters with defaults in the auto-init: `def __init__(self, x: int = 0)`
- Fields without defaults are required parameters
- Required parameters must come before defaulted parameters (same rule as Python). If the field declaration order violates this, emit a compile error: "required field 'x' declared after field 'y' which has a default value"

#### 4. Auto-generated `__eq__` and `__str__`

When a class has auto-init (no explicit `__init__`), also auto-generate:

- `__eq__`: field-by-field equality comparison (only if all fields implement `PartialEq`). If any field is `float`, derive `PartialEq` but not `Eq` (matching contract #10 in architecture.md)
- `__str__`: format as `ClassName(field1=value1, field2=value2)` using `Debug`-style formatting

These are only generated if the class does not already define them explicitly.

### Codegen Changes

- The generated Rust struct already has fields from the class declaration
- The auto-generated `__init__` produces the same `impl ClassName { fn new(...) -> Self { ... } }` as a hand-written one
- Auto-generated `__eq__` maps to the existing `#[derive(PartialEq)]` (already in contract #10)
- Auto-generated `__str__` maps to the existing `#[derive(Debug)]` with a `Display` impl that formats as `ClassName(field=value, ...)`

### Migration

- Audit all existing stdlib classes in `stdlib/sifr/` and demos. For each class where `__init__` is a simple field-assignment constructor, remove the explicit `__init__` and verify the auto-generated one produces identical behavior
- Do NOT remove `__init__` from classes that have logic in the constructor (validation, computed fields, `super().__init__()` calls)
- Update all demos to use the shorter form where applicable

### Definition of Done (milestone_auto_init)

- Classes without `__init__` get auto-generated constructors from field declarations
- Default field values work as default parameters
- Inheritance diagnostic fires when child has fields but no `__init__` and extends a parent
- Auto-generated `__eq__` and `__str__` work for eligible classes
- Explicit `__init__`, `__eq__`, `__str__` always take precedence over auto-generated versions
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- Stdlib classes migrated where applicable (boilerplate `__init__` removed)
- New E2E pass tests: `auto_init_basic`, `auto_init_defaults`, `auto_init_eq`, `auto_init_str`, `auto_init_explicit_override`, `auto_init_inheritance_warning`
- New E2E fail tests: `auto_init_required_after_default` (field ordering error), `auto_init_inheritance_missing_super` (diagnostic)
- Milestone demo in `./demos/auto_init/main.sifr`

---

## milestone_generics_v2: User-Facing Generics Completion

status: done

**Goal:** Complete the generics implementation so that users can define generic functions, generic classes, and use type parameters in all positions. The current implementation supports generic functions (PEP 695 syntax and `TypeVar` declarations) but generic classes have incomplete type parameter substitution — type parameters in fields and methods are not substituted at instantiation sites. This milestone completes the story.

**Depends on:** milestone_auto_init (auto-init must work before generic classes can use it; generic classes with auto-init is a key use case)

### Compiler Changes

#### 1. Generic class field and method substitution (sifr_lowering + sifr_type_system)

Currently, `class Stack[T]` parses and the type parameter is recorded, but when `Stack[int]` is instantiated, the `T` in field types and method signatures is not substituted with `int`. Fix this:

- When a generic class is instantiated with concrete type arguments (e.g., `Stack[int]`), create a substitution map `{T: int}`
- Apply the substitution to all field types, method parameter types, and method return types
- The substituted class instance type is `GenericInstance(ClassId, vec![Int])` (already in the `Type` enum)
- Field access on a `GenericInstance` must resolve through the substitution: `stack.items` where `items: list[T]` resolves to `list[int]`
- Method calls on a `GenericInstance` must substitute type parameters in the method signature before type-checking arguments

#### 2. Generic class auto-init interaction

- When a generic class has auto-init, the generated `__init__` parameters use the type variables: `def __init__(self, items: list[T])` for `class Stack[T]: items: list[T]`
- At the call site `Stack[int]([1, 2, 3])`, the compiler substitutes `T = int` and type-checks the argument as `list[int]`

#### 3. Type parameter inference at class instantiation

- When a generic class constructor is called without explicit type arguments, infer from the arguments:
  - `Stack([1, 2, 3])` infers `T = int` from the `list[int]` argument
  - `Pair("hello", 42)` infers `T = str, U = int`
- If inference fails (e.g., no arguments, or ambiguous), require explicit type arguments: `Stack[int]()`
- Empty generic collections require explicit type arguments: `Stack[int]()` (matches the existing empty collection inference rule from architecture.md)

#### 4. Generic type bounds (protocol constraints)

- Support `def f[T: Comparable](x: T)` syntax — `T` must implement the `Comparable` protocol
- Support multiple bounds: `def f[T: Comparable & Display](x: T)`
- At call sites, verify the concrete type satisfies all bounds
- Codegen: `fn f<T: Clone + std::fmt::Display + Ord>(x: &T)` (map protocol names to Rust trait bounds)
- Bounds checking produces clear diagnostics: "type 'MyClass' does not implement protocol 'Comparable' required by type parameter 'T'"

#### 5. Generic type aliases

- `type Pair[T] = tuple[T, T]` — generic type aliases
- Substitution applies when the alias is used: `Pair[int]` becomes `tuple[int, int]`

#### 6. Codegen for generic classes

- Generic classes emit Rust generics: `struct Stack<T: Clone + std::fmt::Display> { items: Vec<T> }`
- Rust handles monomorphization — no explicit monomorphization pass in Sifr
- Trait bounds on the Rust struct match the protocol constraints from the Sifr source
- Auto-derived traits (`Debug`, `Clone`, `PartialEq`) are conditionally derived based on whether the type parameter's bounds include the necessary protocols

### None as a standalone type and value

As part of completing the type system, ensure `None` works fully as a standalone value and type:

- `x: None = None` is valid (type is `Type::None`, value is `()` in Rust)
- `def f() -> None` is valid and equivalent to returning nothing (returns `()`)
- `None` can be used in any position where a value is expected if the type allows it
- `x: int | None = None` works (already implemented, verify no regressions)
- `if x is None` and `if x is not None` narrowing works for all `T | None` types (already implemented, verify no regressions)
- `None` is `Eq`, `Hash`, `Clone`, `Debug` — it can be used as a dict key, set member, etc.
- Codegen: standalone `None` maps to `()` in Rust. `T | None` still maps to `Option<T>`.

### Definition of Done (milestone_generics_v2)

- Generic classes with type parameter substitution in fields and methods work end-to-end
- Generic class auto-init works (type parameters in generated `__init__`)
- Type parameter inference at class instantiation works
- Protocol bounds on type parameters work with clear diagnostics
- Generic type aliases work
- `None` works as a standalone value and type in all positions
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- New E2E pass tests: `generic_class_basic`, `generic_class_field_access`, `generic_class_method`, `generic_class_auto_init`, `generic_class_inference`, `generic_bounds_comparable`, `generic_bounds_multiple`, `generic_type_alias`, `none_standalone_value`, `none_standalone_type`, `none_as_dict_key`
- New E2E fail tests: `generic_bounds_not_satisfied`, `generic_class_missing_type_arg`, `generic_wrong_type_arg`
- Milestone demo in `./demos/generic_classes/main.sifr`

---

## milestone_pattern_matching: Match/Case Syntax

status: done

**Goal:** Add Python 3.10-style structural pattern matching (`match`/`case`) to Sifr. This completes the safety story: union types, literal types, and type narrowing already exist, but users must use `isinstance` chains and `if`/`elif` to handle them. `match`/`case` provides a declarative, exhaustiveness-checked syntax that maps directly to Rust's `match` expression.

**Depends on:** milestone_generics_v2 (generic types must be complete so that pattern matching works on generic union types like `Option[T]`, `Result[T, E]`)

### Language Design

#### Syntax

Follow Python 3.10 PEP 634 syntax with Sifr adaptations:

```python
match value:
    case pattern1:
        body1
    case pattern2:
        body2
    case _:
        default_body
```

#### Supported patterns

1. **Literal patterns**: `case 42:`, `case "hello":`, `case True:`
2. **Capture patterns**: `case x:` (binds the value to `x`)
3. **Wildcard pattern**: `case _:` (matches anything, discards)
4. **Class patterns**: `case Circle(radius=r):` (destructures class instances, binds fields)
5. **Union variant patterns**: `case int() as n:` (narrows union type to specific variant)
6. **None pattern**: `case None:` (matches `None` in `T | None`)
7. **OR patterns**: `case "GET" | "POST":` (matches either)
8. **Guard patterns**: `case x if x > 0:` (pattern + condition)
9. **Nested patterns**: `case Circle(radius=r) if r > 0:` (class pattern + guard)
10. **Tuple patterns**: `case (x, y):` (destructures tuples)

#### Exhaustiveness checking

The compiler checks that all possible values are covered:

- For union types (`int | str`): every variant must have a matching `case` or a wildcard `case _:`
- For literal union types (`"GET" | "POST" | "PUT"`): every literal must be covered or a wildcard present
- For `T | None`: both the `T` case and the `None` case must be covered
- For class unions (`Circle | Square`): every class must have a matching `case`
- Missing coverage is a compile error with a diagnostic listing uncovered cases

If exhaustiveness cannot be statically verified (e.g., matching on `int` without a wildcard), the compiler emits: "non-exhaustive match: add `case _:` to handle remaining values"

#### Type narrowing in case bodies

Each `case` arm narrows the matched variable's type within the body:

- `case Circle(radius=r):` narrows `shape: Circle | Square` to `shape: Circle` and binds `r: float`
- `case None:` narrows `x: int | None` to `x: None` (and the variable is known to be `None`)
- `case int() as n:` narrows `x: int | str` to `n: int`

### Compiler Changes

#### Parser (sifr_python_parser) — ALREADY DONE

The parser already supports `match`/`case` syntax. The `pattern.rs` module exists in `sifr_python_parser`, and AST nodes for all pattern types (`MatchCase`, `PatternMatchValue`, `PatternMatchClass`, `PatternMatchOr`, etc.) are already defined in `sifr_python_ast`. No parser changes are needed for this milestone. The work is in HIR lowering, type checking, and codegen.

For reference, the existing parser infrastructure includes:
- `match` and `case` are parsed as part of the Python 3.10 syntax support inherited from ruff
- `StmtMatch { subject: Expr, cases: Vec<MatchCase> }` AST node exists
- `MatchCase { pattern: Pattern, guard: Option<Expr>, body: Vec<Stmt> }` exists
- Pattern types (`Literal`, `Capture`, `Wildcard`, `Class`, `Or`, etc.) are defined

#### Type checker (sifr_type_system)

- Implement exhaustiveness checking algorithm (based on Rust's exhaustiveness checker or the algorithm from "Warnings for pattern matching" by Luc Maranget)
- For each `match` statement, collect the subject type and verify all cases cover the type space
- Narrow the subject type in each case body based on the pattern
- Verify that guard expressions are `bool`-typed
- Verify that class patterns reference valid fields
- Verify that capture variables don't shadow existing variables (or emit a warning)

#### HIR Data and Lowering (`sifr_ir` / `sifr_lowering`)

- Lower `StmtMatch` to HIR match node
- Each case arm becomes an HIR branch with narrowed type environment
- Class patterns desugar to field extraction + narrowing
- OR patterns desugar to multiple arms with shared body

#### Codegen (sifr_codegen)

- `match` on union types maps to Rust `match` on the generated enum:
  ```rust
  match value {
      IntOrStr::Int(n) => { ... }
      IntOrStr::Str(s) => { ... }
  }
  ```
- `match` on `T | None` maps to `match` on `Option<T>`:
  ```rust
  match value {
      Some(inner) => { ... }
      None => { ... }
  }
  ```
- Literal patterns map to value comparisons in match guards
- Class patterns map to enum variant destructuring with field bindings
- Wildcard `_` maps to Rust `_`
- Guard patterns map to Rust `if` guards in match arms

### Definition of Done (milestone_pattern_matching)

- `match`/`case` syntax parses correctly for all pattern types
- Exhaustiveness checking works for union types, literal unions, optional types, and class unions
- Non-exhaustive matches produce clear compile errors listing uncovered cases
- Type narrowing works in each case body
- Class patterns destructure fields correctly
- OR patterns, guards, tuple patterns, and nested patterns work
- Codegen produces correct Rust `match` expressions
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- New E2E pass tests: `match_literal`, `match_union`, `match_optional`, `match_class_destructure`, `match_or_pattern`, `match_guard`, `match_tuple`, `match_nested`, `match_wildcard`, `match_exhaustive_literal_union`
- New E2E fail tests: `match_non_exhaustive_union`, `match_non_exhaustive_optional`, `match_non_exhaustive_literal`, `match_invalid_field_name`, `match_type_mismatch_guard`
- Milestone demo in `./demos/pattern_matching/main.sifr`

---

## milestone_enums: Simple Enum Types

status: done

**Goal:** Add a dedicated `enum` type to Sifr for simple value enumerations (no associated data). While literal union types (`"GET" | "POST"`) partially fill this role, they are stringly-typed and don't provide namespacing. A proper enum gives namespaced constants, type safety, exhaustive matching, and direct mapping to Rust enums. Data-carrying variants are NOT included — Sifr's existing union types + classes + pattern matching cover that use case (see design rationale below).

**Depends on:** milestone_pattern_matching (enum types are most useful with `match`/`case` for exhaustive handling)

### Language Design

#### Simple enums

```python
enum Color:
    RED
    GREEN
    BLUE
```

- Each variant is a distinct value of type `Color`
- Access via `Color.RED`, `Color.GREEN`, `Color.BLUE`
- Enums are `Eq`, `Hash`, `Clone`, `Debug` by default
- Can be used as dict keys, set members, match subjects
- Codegen: `#[derive(Debug, Clone, PartialEq, Eq, Hash)] enum Color { Red, Green, Blue }`

#### Enum with integer values

```python
enum HttpStatus:
    OK = 200
    NOT_FOUND = 404
    INTERNAL_ERROR = 500
```

- Variants can have explicit integer values
- `HttpStatus.OK.value` returns `200` (via a `.value` property)
- Codegen: `enum HttpStatus { Ok = 200, NotFound = 404, InternalError = 500 }` with `#[repr(i64)]`

#### Enum methods

```python
enum Direction:
    NORTH
    SOUTH
    EAST
    WEST

    def is_vertical(self) -> bool:
        match self:
            case Direction.NORTH:
                return True
            case Direction.SOUTH:
                return True
            case _:
                return False
```

- Methods defined inside the enum body apply to all variants
- `self` is the enum type; use `match self` to dispatch on variant
- Codegen: `impl Direction { fn is_vertical(&self) -> bool { match self { ... } } }`

#### Pattern matching integration

Enums integrate with `match`/`case` from milestone_pattern_matching:

```python
def describe(color: Color) -> str:
    match color:
        case Color.RED:
            return "red"
        case Color.GREEN:
            return "green"
        case Color.BLUE:
            return "blue"
```

- The compiler checks exhaustiveness: all enum variants must be covered
- Each case arm narrows to the specific variant

### Design Rationale: No Associated Data

Sifr intentionally does NOT support enums with associated data (algebraic data types). The reasoning:

1. **Union types + classes already cover this.** `type Shape = Circle | Square` with separate class definitions is functionally equivalent to Rust's `enum Shape { Circle { radius: f64 }, Square { side: f64 } }`. The compiler already generates Rust enums from union types and checks exhaustiveness.
2. **One obvious way.** Adding a second mechanism for the same concept violates the Pythonic principle. TypeScript made the same choice — discriminated unions, no algebraic enums — and developers don't miss them.
3. **Smaller language surface.** Every new syntax form increases learning cost, compiler complexity, and interaction surface with generics, protocols, and pattern matching.
4. **More Pythonic.** Python developers think in classes, not enum variants with data. Keeping classes + union types as the primary pattern is more natural for the target audience.

### Compiler Changes

#### Parser

- Add `enum` as a keyword
- Parse `enum Name:` block with variant declarations
- Variants: `NAME` (unit) or `NAME = integer_literal` (valued)
- Methods inside enum body parsed as regular method definitions

#### Type system

- Add `Type::Enum(EnumId)` to the type enum
- Enum variants are values of the enum type (not subtypes)
- Exhaustiveness checking: extend the pattern matching exhaustiveness checker to handle enum variants
- Enum types implement `Eq`, `Hash`, `Clone`, `Debug` unconditionally

#### HIR

- Lower enum declarations to HIR enum nodes
- Lower variant access (`Color.RED`) to HIR enum variant reference
- Lower `match` on enums to HIR match with variant patterns

#### Codegen

- Emit Rust `enum` with unit variants (or `#[repr(i64)]` for valued enums)
- Emit `impl` block for methods
- Auto-derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`
- Variant access maps to Rust enum variant reference
- Pattern matching maps to Rust `match` with variant patterns

### Relationship to existing union types

- Union types (`int | str`, `Circle | Square`) remain unchanged — they are structural unions of existing types
- Enum types are nominal — `Color` is a distinct type, not a union of separate classes
- Enums are preferred for finite sets of named constants; unions are preferred for combining independent types with data
- The compiler does NOT auto-convert between enums and unions

### Definition of Done (milestone_enums)

- Simple enums (no data) work end-to-end
- Enums with integer values work
- Enum methods work
- Pattern matching on enums is exhaustiveness-checked
- Enums are `Eq`, `Hash`, `Clone`, `Debug` by default
- Enums can be used as dict keys and set members
- `.value` property works for valued enums
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- New E2E pass tests: `enum_simple`, `enum_valued`, `enum_methods`, `enum_match_exhaustive`, `enum_as_dict_key`, `enum_in_set`, `enum_value_property`
- New E2E fail tests: `enum_match_non_exhaustive`, `enum_invalid_variant`, `enum_duplicate_value`
- Milestone demo in `./demos/enums/main.sifr`

---

## milestone_integer_safety: Integer Overflow and Exact Int

status: done

**Goal:** Resolve the integer overflow contradiction with Sifr's "if it compiles, it works" guarantee. This milestone originally experimented with a second public arbitrary-precision integer surface, but the canonical integer model supersedes that bootstrap design: source-level `int` is exact arbitrary precision and fixed-width integer families are explicit representation choices. See `internal_docs/integer_model.md` for the authoritative current contract.

**Depends on:** milestone_enums (the full type system feature set should be in place before changing arithmetic semantics)

### Language Design

#### Exact `int` and explicit fixed-width integers

```python
x: int = 10 ** 100
port: uint16 = 443
```

- `int` is the Python-simple exact arbitrary-precision scalar.
- Fixed-width types (`int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `isize`, `usize`) are explicit for storage, schemas, dtypes, binary formats, and FFI.
- Ordinary fixed-width scalar arithmetic promotes to exact `int`; representation-preserving behavior is exposed through explicit checked/wrapping/saturating/overflowing APIs.
- Narrowing from exact `int` to fixed-width types is explicit and fallible unless the compiler proves a constant fits.
- The temporary second public integer alias and its migration diagnostic have been removed.

#### Type system integration

- `Type::Int` represents exact source-level integers.
- Fixed-width families are first-class type variants.
- Bool/integer comparison mistakes use `SIFR-INT-0007`.
- Exact integer failure boundaries use `Result` or active integer diagnostics instead of panics.

### Compiler Changes

- Parser and HIR preserve large integer literal text without truncation.
- Type checking performs fixed-width const fitting and rejects implicit narrowing.
- Codegen uses the shared `sifr_runtime::SifrInt` runtime for exact integer values and keeps source-level `int` value-semantic.
- Integer diagnostics live in the `SIFR-INT-*` family documented in `internal_docs/integer_model.md`.

### Definition of Done (milestone_integer_safety)

- `int` literals and ordinary arithmetic use exact integer semantics.
- Fixed-width integer annotations and constructors require fitting constants or explicit fallible narrowing.
- Fixed-width scalar arithmetic widens to `int` unless an explicit representation-preserving API is used.
- Public docs and demos use `int` for arbitrary precision and fixed-width types for representation-sensitive values.
- Exact-integer fixtures use canonical `int`, including values beyond fixed-width ranges.
- All existing E2E tests still pass
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- Milestone demo in `./demos/integer_safety/main.sifr`

---

## milestone_stdlib_generic_rewrite: Stdlib Generification

status: done

**Goal:** Rewrite the monomorphic stdlib to use generics. This is the integration test for the entire phase — every compiler feature from milestones 1-4 is exercised. After this milestone, the stdlib is type-safe, generic, and free of duplicated type-specific functions.

**Depends on:** milestone_integer_safety (exact integer and fixed-width type-system features must be complete before rewriting the stdlib)

### Scope

#### itertools.sifr — full generic rewrite

Current state: every function is `list[int]`-specific, with `_str` variants for strings.

Target state:

| Current | Generic replacement |
|---|---|
| `chain(a: list[int], b: list[int]) -> list[int]` | `def chain[T](a: list[T], b: list[T]) -> list[T]` |
| `chain_str(a: list[str], b: list[str]) -> list[str]` | **deleted** (covered by generic `chain`) |
| `repeat(value: int, times: int) -> list[int]` | `def repeat[T](value: T, times: int) -> list[T]` |
| `take(n: int, data: list[int]) -> list[int]` | `def take[T](n: int, data: list[T]) -> list[T]` |
| `flatten(lists: list[list[int]]) -> list[int]` | `def flatten[T](lists: list[list[T]]) -> list[T]` |
| `pairwise(data: list[int]) -> list[list[int]]` | `def pairwise[T](data: list[T]) -> list[list[T]]` |
| `accumulate(data: list[int]) -> list[int]` | `def accumulate[T: Addable](data: list[T]) -> list[T]` |
| `accumulate_float(...)` | **deleted** (covered by generic `accumulate`) |
| `compress(data: list[int], selectors: list[bool]) -> list[int]` | `def compress[T](data: list[T], selectors: list[bool]) -> list[T]` |
| `dropwhile(threshold: int, data: list[int]) -> list[int]` | `def dropwhile[T](pred: Callable[[T], bool], data: list[T]) -> list[T]` |
| `takewhile(threshold: int, data: list[int]) -> list[int]` | `def takewhile[T](pred: Callable[[T], bool], data: list[T]) -> list[T]` |
| `filterfalse(threshold: int, data: list[int]) -> list[int]` | `def filterfalse[T](pred: Callable[[T], bool], data: list[T]) -> list[T]` |
| `zip_longest(a: list[int], b: list[int], fill: int) -> list[list[int]]` | `def zip_longest[T](a: list[T], b: list[T], fill: T) -> list[tuple[T, T]]` |
| `count_from(start: int, step: int, n: int) -> list[int]` | Keep as `int`-specific (CPython's `count` is also int-specific) |
| `cycle(data: list[int], n: int) -> list[int]` | `def cycle[T](data: list[T], n: int) -> list[T]` |
| `batched(data: list[int], n: int) -> Result[...]` | `def batched[T](data: list[T], n: int) -> Result[list[list[T]], ValueError]` |
| `islice(data: list[int], stop: int) -> list[int]` | `def islice[T](data: list[T], stop: int) -> list[T]` |

Note: `dropwhile`, `takewhile`, and `filterfalse` change from threshold-based to predicate-based APIs to match CPython's actual signatures. The old threshold-based versions are non-CPython and are removed as part of this rewrite.

#### functools.sifr — generic rewrite

| Current | Generic replacement |
|---|---|
| `reduce(func: Callable[[int, int], int], data: list[int], initial: int) -> int` | `def reduce[T, U](func: Callable[[U, T], U], data: list[T], initial: U) -> U` |

#### collections.sifr — generic Counter and deque

**Counter:**

| Current | Generic replacement |
|---|---|
| `class Counter` with `counts: dict[str, int]` | `class Counter[T]` with `counts: dict[T, int]` |
| `from_list(items: list[str]) -> Counter` | `def from_list[T](items: list[T]) -> Counter[T]` |
| All methods operate on `str` keys | All methods operate on `T` keys (requires `T: Hashable`) |
| Standalone addition/subtraction helpers | `__add__` / `__sub__` source methods on `Counter[T]` (completed) |

The `Counter[T]` class requires `T` to be `Hashable` (since it's used as a dict key). The compiler enforces this via the generic bounds system from milestone_generics_v2.

**deque:**

| Current | Generic replacement |
|---|---|
| `class deque` with `_data: list[int]` | `class deque[T]` with Rust `VecDeque<T>` backing |
| All methods operate on `int` | All methods operate on `T` |
| O(n) `appendleft`/`popleft` (rebuilds list) | O(1) via `VecDeque` intrinsics |

The `deque` implementation must be backed by Rust intrinsics for O(1) front operations. Add `_sifr.collections.deque_*` intrinsic set:

- `deque_new(maxlen: int)` — create new deque
- `deque_append(mut d, val)` — append to back
- `deque_appendleft(mut d, val)` — append to front
- `deque_pop(mut d) -> Option` — pop from back
- `deque_popleft(mut d) -> Option` — pop from front
- `deque_len(d) -> int` — length

The Sifr `deque[T]` class wraps these intrinsics, providing the same API but with O(1) performance for all front/back operations.

#### heapq.sifr — generic rewrite

| Current | Generic replacement |
|---|---|
| All functions operate on `list[int]` | All functions operate on `list[T]` where `T: Comparable` |
| `heapify(mut data: list[int])` | `def heapify[T: Comparable](mut data: list[T])` |
| `heappush(mut heap: list[int], item: int)` | `def heappush[T: Comparable](mut heap: list[T], item: T)` |
| `heappop(mut heap: list[int]) -> int \| None` | `def heappop[T: Comparable](mut heap: list[T]) -> T \| None` |
| `nsmallest(n: int, data: list[int]) -> list[int]` | `def nsmallest[T: Comparable](n: int, data: list[T]) -> list[T]` |
| `nlargest(n: int, data: list[int]) -> list[int]` | `def nlargest[T: Comparable](n: int, data: list[T]) -> list[T]` |

Internal helpers (`_sift_down`, `_sift_up`) also become generic.

#### bisect.sifr — verify and harden

`bisect.sifr` already uses `TypeVar("T")` and is generic. Verify it works correctly with the new generics infrastructure. Migrate from `T = TypeVar("T")` to PEP 695 `def bisect_left[T: Comparable](...)` syntax for consistency.

#### Other stdlib modules to audit

For each module in `stdlib/sifr/`, audit whether any functions are type-specific where they should be generic:

- `statistics.sifr`: functions operate on `list[float]` — correct (statistics are inherently float-based), no change needed
- `math.sifr`: functions operate on `int` or `float` — correct, no change needed
- `random.sifr`: `shuffle` and `sample` operate on `list[int]` — make generic: `def shuffle[T](data: list[T]) -> list[T]`, `def sample[T](data: list[T], k: int) -> list[T]`
- `test.sifr`: `assert_eq` should be generic: `def assert_eq[T](actual: T, expected: T)`

### Test migration

All existing E2E tests that use the monomorphic stdlib functions must be updated to use the generic versions. Since the generic versions accept the same concrete types, most tests should work without changes — the compiler infers the type parameter from the arguments. Tests that use `chain_str`, `accumulate_float`, or other deleted type-specific functions must be migrated to the generic equivalents.

### Definition of Done (milestone_stdlib_generic_rewrite)

- All listed stdlib modules rewritten with generic type parameters
- `chain_str`, `accumulate_float`, and other type-specific duplicates deleted
- `Counter[T]` works for any `Hashable` type (not just `str`)
- `Counter` has `__add__` / `__sub__` operator overloads (not standalone functions)
- `deque[T]` is backed by `VecDeque<T>` intrinsics with O(1) front operations
- `heapq` functions work for any `Comparable` type
- `itertools` functions work for any type (with `Addable` bound on `accumulate`)
- `functools.reduce` is fully generic
- `dropwhile`/`takewhile`/`filterfalse` use predicate-based APIs matching CPython
- `bisect.sifr` migrated to PEP 695 syntax with `Comparable` bound
- `random.shuffle`/`random.sample` are generic
- `test.assert_eq` is generic
- All existing E2E tests still pass (with migration where needed)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- New E2E pass tests: `generic_chain_str`, `generic_chain_float`, `generic_counter_int`, `generic_counter_custom_class`, `generic_deque_str`, `generic_deque_float`, `generic_heapq_float`, `generic_reduce_str`, `generic_accumulate_float`, `generic_dropwhile_predicate`, `generic_shuffle_str`, plus canonical exact-`int` coverage.
- New E2E fail tests: `generic_counter_unhashable` (float as Counter key), `generic_heapq_uncomparable` (type without Comparable)
- API naming divergences table in `architecture.md` updated: remove `chain_str`, `accumulate_float`, and any other deleted type-specific entries; update `itertools.count_from` if its rationale changes
- Milestone demo in `./demos/generic_stdlib/main.sifr`

---

## Milestone Ordering

- **milestone_auto_init first:** Eliminates boilerplate that would otherwise be duplicated in every subsequent milestone's new classes and demos. Also provides the auto-generated `__eq__` and `__str__` that generic classes and enums will rely on.
- **milestone_generics_v2 second:** User-facing generics are the foundation for everything else in this phase. Pattern matching needs to work on generic types. Enums benefit from pattern matching which benefits from generics. The stdlib rewrite needs generic functions and classes.
- **milestone_pattern_matching third:** Pattern matching depends on generics (matching on `Option[T]`, `Result[T, E]`). Enums depend on pattern matching for exhaustive variant handling.
- **milestone_enums fourth:** Enums depend on pattern matching for exhaustive `match` on variants. They are the capstone language feature of this phase.
- **milestone_integer_safety fifth:** Resolves the integer overflow contradiction with the safety guarantee. The current canonical model uses exact `int`, explicit fixed-width integer families, typed failure boundaries, and stable `SIFR-INT-*` diagnostics. Must come after enums so the full type system is in place.
- **milestone_stdlib_generic_rewrite last:** The stdlib rewrite exercises every feature from milestones 1-5 including exact `int` and fixed-width support where appropriate. It is the integration test for the entire phase and must come after all language features are stable.
