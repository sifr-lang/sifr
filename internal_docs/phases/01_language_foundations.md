# Language Foundations

This phase builds the core language from first principles: variables, functions, types, control flow, classes, error handling, safe indexing, multi-file imports, and codegen quality. By the end, Sifr can compile single-file and multi-file programs with a complete safety story (no panics from data access) and clean, idiomatic Rust output.

---

## milestone_core_language: Core Language (First Working Compiler)

`status: completed`

**Goal:** Compile a simple program with variables, functions, basic types, and branching to a native binary.

### Language Features

- **Types:** `int`, `float`, `bool`, `str`, `None`
- **Literals:** integer, float, string, boolean, None
- **Variables:** typed declarations (`x: int = 5`), inferred declarations (`x = 5`)
- **Functions:** typed parameters and return types, recursion
- **Expressions:** arithmetic (`+`, `-`, `*`, `/`, `//`, `%`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`), boolean (`and`, `or`, `not`), string concatenation
- **Statements:** assignment, return, expression statements, `if`/`elif`/`else`
- **Built-in:** `print()` function
- **Entry point:** `main()` function as program entry
- **Move semantics:** move on assignment for `str`, copy for primitives (`int`, `float`, `bool`)
- **CLI:** `sifr build`, `sifr run`, `sifr check`, `sifr emit`

### Implementation Steps

1. Fork ruff parser/AST crates into `crates/` with `sifr_` prefix; use git deps for infrastructure crates
2. Strip the AST to milestone_core_language-relevant nodes only
3. Build `sifr_type_system` -- Type enum, inference from initializers, checking binary ops / function calls
4. Build `sifr_lowering` -- Typed IR with name resolution and ownership tracking
5. Build `sifr_codegen` -- Emit Rust source code, generate Cargo.toml + main.rs
6. Build `sifr_driver` -- Orchestrate the pipeline with nice error diagnostics
7. Build `sifr` CLI binary with clap
8. End-to-end tests (hello world, factorial, fibonacci)

### Example Program

```python
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    x: int = factorial(5)
    print(x)
```

### Type Mapping (milestone_core_language)

- `int` -> `i64`
- `float` -> `f64`
- `bool` -> `bool`
- `str` -> `String`
- `None` -> `()`

---

## milestone_control_flow: Control Flow and Data Structures

`status: completed`

**Goal:** Support loops and compound data types so programs can process collections of data.

### Language Features

- **Loops:** `while` loop, `for` loop over ranges and iterables
- `**break` / `continue`:** loop control flow (exit loop early or skip to next iteration)
- **Data types:** `list[T]`, `dict[K, V]`, `tuple[T, ...]`
- **Indexing:** `my_list[0]`, `my_dict["key"]`
- **Slicing:** `my_list[1:3]`
- **String operations:** `.len()`, `.upper()`, `.lower()`, `.split()`, `.strip()`, f-strings
- **Type inference:** infer collection element types from usage
- `**in` operator:** membership testing (`item in collection`)
- `**not in` operator:** negated membership testing (`item not in collection`) -- compiles to `!collection.contains(&item)`
- `**range()` built-in**
- **Multiple assignment:** `a, b = 1, 2` (tuple unpacking)

### Example Program

```python
def sum_list(numbers: list[int]) -> int:
    total: int = 0
    for n in numbers:
        total = total + n
    return total

def main():
    nums: list[int] = [1, 2, 3, 4, 5]
    result: int = sum_list(nums)
    print(f"Sum: {result}")
```

### Type Mapping (New)

- `list[T]` -> `Vec<T>`
- `dict[K, V]` -> `std::collections::HashMap<K, V>`
- `tuple[A, B, C]` -> `(A, B, C)`
- `range(n)` -> `0..n`

### Deferred Built-ins and Methods

milestone_control_flow established the core data types but deferred comprehensive method suites and built-in functions to later milestones:

- **Collection methods (concrete returns)** (list `.append()`, `.clear()`, dict `.keys()`, `.values()`, etc.) -> milestone_ergonomics
- **Collection methods (Option/Result returns)** (list `.pop()`, `.index()`, dict `.get()`, `.pop()`, etc.) -> milestone_safe_indexing
- **Extended string methods (concrete)** (`.replace()`, `.startswith()`, `.join()`, etc.) -> milestone_ergonomics
- **Extended string methods (Option)** (`.find()`, `.rfind()`) -> milestone_safe_indexing
- **Non-generic built-in functions** (`len()`, `abs()`, `round()`, `repr()`) -> milestone_ergonomics; `hash()` -> milestone_classes
- **Fallible conversions** (`int(s)`, `float(s)`, `input()`) -> milestone_error_handling (return `Result`)
- **Generic built-in functions** (`min()`, `max()`, `sorted()`, `zip()`, `enumerate()`) -> milestone_generics (require generics)
- **Extended collection types** (`frozenset`, `Counter`, `defaultdict`, `bytes`) -> milestone_ext_collections

---

## milestone_type_system: Advanced Type System

`status: completed`

**Goal:** Add union types, intersection types, literal types, and full control-flow-based type narrowing to the sifr compiler. This makes sifr's type system as expressive as TypeScript's while compiling to Rust.

### Why milestone_type_system (before Error Handling)

Union types, literal types, and type narrowing are **prerequisites** for clean error handling and later milestones:

- milestone_error_handling's `Result[T, E]` and `Option[T]` are union-based types
- milestone_protocols's discriminated unions (e.g., `Shape` with a `.tag` field) need narrowing
- milestone_generics's generics need type bounds with unions
- Every milestone after milestone_type_system benefits from the advanced type system

### Syntax Design Principles

Sifr reuses familiar syntax from Python, TypeScript, and Rust rather than inventing new constructs:

- **Python-first:** if Python has syntax for it, use that (`isinstance`, `is None`, `type` statement)
- **TypeScript for types:** where Python's typing module is verbose, borrow TypeScript's cleaner syntax (values as types: `"GET" | "POST"` instead of `Literal["GET"] | Literal["POST"]`)
- **No redundant sugar:** one way to do things. `str | None` for optionals, no `T?` shorthand
- **No user-facing syntax for internal features:** intersection types are internal to the narrowing engine, not exposed as `A & B` syntax

### Language Features

- **Union types:** `int | str`, `A | B | C` -- a value can be one of several types (Python 3.10+ syntax)
- **Literal types:** values used directly as types in type position (TypeScript style):

```python
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE"
type StatusCode = 200 | 404 | 500
type Toggle = True | False
```

- **Type aliases:** `type UserId = int`, `type HttpMethod = "GET" | "POST"` (Python 3.12 `type` statement)
- **Optional types:** `str | None` -- no shorthand, just Python's union-with-None (Python 3.10+ syntax)
- `**Unknown` type:** safe top type -- accepts any value but must be narrowed (via `isinstance`, equality, etc.) before use. Unlike `Any` which opts out of type checking, `Unknown` forces the programmer to prove the type before operating on it
- **Type narrowing via control flow analysis:**
  - Truthiness checks: `if x:` narrows `x: str | None` to `x: str`
  - `isinstance()` checks: `if isinstance(x, int):` narrows union (Python built-in)
  - Equality checks: `if x == "GET":` narrows `x: str` to `x: "GET"` in the then-branch
  - `is None` / `is not None` checks (Python idiom)
  - `== None` diagnostic: the compiler emits a warning suggesting `is None` instead of `== None` (identity check is more correct and idiomatic for None comparisons, matching Python best practice and linter rules)
  - `not` negation: else branches get the complement type
- **Type predicates:** user-defined narrowing via return type annotation (Python typing style):

```python
def is_string(x: int | str) -> TypeGuard[str]:
    return isinstance(x, str)

# Usage: if is_string(val): ... val is str here
```

- `**reveal_type()` built-in:** prints inferred type at compile time (same as mypy/pyright)
- `**never` exhaustiveness:** matching all union variants leaves `never` -- compiler error if not exhaustive
- **Intersection types:** internal to the narrowing engine only. No user-facing `A & B` syntax in milestone_type_system. Exposed later when protocols land in milestone_classes

Note: **Discriminated unions** (union of structs with a shared tag field) are deferred to milestone_protocols when protocols and pattern matching exist. milestone_type_system focuses on unions of primitive/literal types with narrowing via isinstance and equality.

### Compiler Architecture Changes

#### Type System Changes

Extend the `Type` enum in `crates/sifr_type_system/src/types.rs`:

```rust
enum Type {
    // ... existing types ...

    // Union: value is one of these types
    Union(Vec<Type>),

    // Intersection: value satisfies all of these (internal, for narrowing)
    Intersection(Vec<Type>),

    // Literal types: specific values as types
    LiteralInt(i64),
    LiteralStr(String),
    LiteralBool(bool),

    // Optional sugar: T | None
    Optional(Box<Type>),

    // Type alias reference (resolved during checking)
    Alias(String, Box<Type>),

    // Safe top type: must be narrowed before use (unlike Any which opts out)
    Unknown,
}
```

Key design decisions:

- `Optional(T)` is sugar that normalizes to `Union(vec![T, None])` internally
- Union types are **flattened** and **deduplicated** (no nested unions)
- Literal types **widen** to their base type at mutable assignment (like TypeScript's fresh literal behavior)
- `Union` maps to Rust `enum` in codegen (auto-generated discriminated enum)
- `Unknown` vs `Any`: `Any` disables type checking (escape hatch). `Unknown` accepts any value but requires narrowing before any operation -- it is the safe alternative. `Unknown` maps to `Box<dyn Any>` in Rust codegen but the compiler enforces narrowing at every use site.

#### Control Flow Graph (new module: `sifr_lowering/src/cfg.rs`)

**Inspired by TypeScript's binder** (see `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/binder.md`):

Build a control flow graph during HIR lowering. Each statement/expression gets a `FlowNode` that points to its antecedents:

```rust
enum FlowNode {
    Start,
    Assignment { var: String, ty: Type, antecedent: FlowNodeId },
    Condition { expr: HirExprId, true_branch: FlowNodeId, false_branch: FlowNodeId },
    Label { antecedents: Vec<FlowNodeId> },  // join point
    Unreachable,
}
```

#### Narrowing Engine (new module: `sifr_type_system/src/narrow.rs`)

**Inspired by TypeScript's checker narrowing** (see `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-widening-narrowing.md`) and **ty's intersection-based narrowing**:

```rust
/// Narrow a type based on a condition being true/false.
fn narrow_type(ty: &Type, condition: &NarrowingCondition, is_true: bool) -> Type

enum NarrowingCondition {
    Truthiness(VarId),                          // if x:
    IsNone(VarId),                              // if x is None
    IsNotNone(VarId),                           // if x is not None
    IsInstance(VarId, Type),                     // if isinstance(x, int)
    Equality(VarId, LiteralValue),              // if x == "GET"
    TypePredicate(VarId, Type),                 // user-defined guard
    AttributeEquality(VarId, String, LiteralValue), // if x.tag == "circle"
    Not(Box<NarrowingCondition>),               // negation
    And(Vec<NarrowingCondition>),               // conjunction
    Or(Vec<NarrowingCondition>),                // disjunction
}
```

#### Scope Changes (update `sifr_lowering/src/scope.rs`)

The scope must track **narrowed types** per variable at each point in the control flow:

```rust
struct VariableInfo {
    declared_type: Type,     // the annotation or inferred type
    narrowed_type: Type,     // current type after narrowing (starts = declared_type)
    is_moved: bool,
}
```

#### Codegen Changes (update `sifr_codegen/src/lib.rs`)

Union types map to Rust enums:

```python
# Sifr
x: int | str = 42
```

```rust
// Generated Rust
enum IntOrStr {
    Int(i64),
    Str(String),
}
let x: IntOrStr = IntOrStr::Int(42);
```

Narrowing maps to `match` or `if let`:

```python
# Sifr
def process(x: int | str):
    if isinstance(x, int):
        print(x + 1)     # x is int here
    else:
        print(x.upper())  # x is str here
```

```rust
// Generated Rust
fn process(x: IntOrStr) {
    match &x {
        IntOrStr::Int(x_val) => {
            println!("{}", x_val + 1);
        }
        IntOrStr::Str(x_val) => {
            println!("{}", x_val.to_uppercase());
        }
    }
}
```

### Example Programs (milestone_type_system)

**Union types and narrowing:**

```python
type Shape = "circle" | "square"

def area(shape: Shape, size: float) -> float:
    if shape == "circle":
        return 3.14159 * size * size
    else:
        return size * size

def main():
    print(area("circle", 5.0))
    print(area("square", 4.0))
```

**Optional / None narrowing:**

```python
def find_user(name: str) -> str | None:
    if name == "alice":
        return "Alice Smith"
    return None

def main():
    user: str | None = find_user("alice")
    if user is not None:
        print(user.upper())   # narrowed to str
    else:
        print("not found")
```

**isinstance narrowing:**

```python
def describe(x: int | str) -> str:
    if isinstance(x, int):
        return f"number: {x + 1}"   # x is int here
    else:
        return f"text: {x.upper()}"  # x is str here

def main():
    print(describe(42))
    print(describe("hello"))
```

**Type predicates:**

```python
def is_nonempty(s: str | None) -> TypeGuard[str]:
    return s is not None and len(s) > 0

def main():
    name: str | None = "alice"
    if is_nonempty(name):
        print(name.upper())  # name narrowed to str
```

**Unknown type (safe top type):**

```python
def process(data: Unknown) -> str:
    if isinstance(data, str):
        return data.upper()       # narrowed to str
    if isinstance(data, int):
        return str(data)          # narrowed to int
    return "unknown"

def main():
    print(process("hello"))
    print(process(42))
```

### Files to Modify/Create for milestone_type_system

**Modify:**

- `crates/sifr_type_system/src/types.rs` -- extend `Type` enum
- `crates/sifr_type_system/src/check.rs` -- type checking for unions
- `crates/sifr_type_system/src/infer.rs` -- inference with unions/literals
- `crates/sifr_lowering/src/hir_nodes.rs` -- new HIR nodes for narrowing
- `crates/sifr_lowering/src/lower.rs` -- lowering with CFG and narrowing
- `crates/sifr_lowering/src/scope.rs` -- narrowed type tracking
- `crates/sifr_codegen/src/lib.rs` -- union -> enum codegen
- `crates/sifr_driver/src/lib.rs` -- pipeline updates

**Create:**

- `crates/sifr_type_system/src/narrow.rs` -- narrowing engine
- `crates/sifr_type_system/src/union.rs` -- union construction, normalization, simplification
- `crates/sifr_type_system/src/literal.rs` -- literal type handling, widening
- `crates/sifr_lowering/src/cfg.rs` -- control flow graph
- E2E test files in `crates/sifr/tests/e2e/pass/` and `fail/`

---

## milestone_ergonomics: Language Ergonomics

`status: completed`

**Goal:** Add essential language features that make Sifr pleasant to use for everyday programming. These features have no dependency on error handling (`Option`/`Result`) -- they work with concrete types only. Safe indexing (returning `Option`) is deferred to milestone_safe_indexing (after milestone_error_handling) so that users have `?` and `match` available when they need to handle `Option` values.

### Augmented Assignment Operators

Add compound assignment operators used in virtually every Python program:

- `+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `**=`
- Codegen: `x += 1` -> `x += 1` in Rust (direct mapping for numeric types)
- For strings: `s += "suffix"` -> `s.push_str("suffix")`
- For lists: `items += [4, 5]` -> `items.extend([4, 5])`

### Conditional Expressions (Ternary)

Add Python's conditional expression syntax:

```python
x = "positive" if n > 0 else "non-positive"
```

Codegen: `let x = if n > 0 { "positive".to_string() } else { "non-positive".to_string() };`

This is simple syntax sugar over `if`/`else` but used as an expression rather than a statement. Both branches must have the same type.

### Keyword Arguments

Add support for keyword (named) arguments in function calls. This is basic call ergonomics used in virtually every Python API:

```python
def greet(name: str, greeting: str = "Hello") -> str:
    return f"{greeting}, {name}!"

# All valid call styles:
greet("Alice")                        # positional
greet("Alice", "Hi")                  # positional
greet(name="Alice")                   # keyword
greet(name="Alice", greeting="Hi")    # keyword
greet("Alice", greeting="Hi")         # mixed
```

**Features:**

- **Default parameter values:** `def f(x: int, y: int = 0)` -- parameters with defaults can be omitted at call site
- **Keyword arguments at call site:** `f(name="Alice")` -- pass arguments by name
- **Mixed positional and keyword:** positional args must come before keyword args (same as Python)
- **Keyword-only parameters:** parameters after `*` separator must be passed by name: `def f(x: int, *, verbose: bool = False)`

**Codegen:** Rust does not have named arguments. The compiler resolves keyword arguments to positional order at compile time and emits a normal positional function call. Default values are inserted for omitted parameters.

**Note:** `*args` and `**kwargs` (variadic arguments) are in milestone_decorators, where they are needed for generic function decorators.

### For-Loop Borrow Semantics

Fix `for item in collection` to borrow the collection rather than consuming it:

- `**for item in collection`:** borrows immutably. The collection remains usable after the loop. Codegen: `for item in &collection`.
- `**for item in collection.consume()`:** takes ownership (move). Codegen: `for item in collection` (Rust's `into_iter`).
- Current behavior may already borrow in some cases; this milestone ensures it is consistent and tested.

### List Slice Copy Semantics

Verify and enforce that `list[a:b]` produces a new list (copy semantics, not a view):

- Codegen: `vec[a..b].to_vec()`
- The original list is not affected by mutations to the slice
- Views (borrowed slices mapping to `&[T]`) are deferred to a future milestone

### Negative Indexing

Add support for negative indices, a heavily used Python idiom:

```python
items = [1, 2, 3, 4, 5]
last = items[-1]        # returns last element
second_last = items[-2] # returns second-to-last
s = "hello"
s[-1]                   # returns "o"
```

**Semantics:** negative index `i` is equivalent to `len - abs(i)`. In this milestone, indexing returns the value directly (panics on out-of-bounds, like current milestone_control_flow behavior). This is a **temporary measure** -- `Option`/`Result` types don't exist yet (they arrive in milestone_error_handling). Safe indexing returning `Option[T]` is added in milestone_safe_indexing, which retroactively replaces all panic-based indexing with safe `Option` returns. No user-facing API changes are needed because the switch is transparent to callers who already handle the value.

> **Safety staging note:** milestones before milestone_safe_indexing use panic-based indexing as a bootstrap mechanism. The global no-panic guarantee (see Safety Philosophy) is fully enforced from milestone_safe_indexing onward. Tests written in earlier milestones are updated in milestone_safe_indexing to use `Option` handling.

**Codegen:** `if i < 0 { collection[((len as isize) + i) as usize] } else { collection[i] }`

### Step Slicing

Add support for step (stride) slicing:

```python
items = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
evens = items[::2]      # [0, 2, 4, 6, 8]
odds = items[1::2]      # [1, 3, 5, 7, 9]
reversed = items[::-1]  # [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
subset = items[1:7:2]   # [1, 3, 5]
```

**Full slice syntax:** `collection[start:stop:step]` where all three components are optional.

**Semantics:**

- Positive step: iterate forward from `start` to `stop` (exclusive), taking every `step`-th element
- Negative step: iterate backward (e.g., `[::-1]` reverses)
- Negative start/stop: resolved relative to length (same as negative indexing)
- Returns a new collection (copy semantics, consistent with existing slice contract)

**Codegen:**

- Positive step: `vec.iter().skip(start).take(stop - start).step_by(step).cloned().collect()`
- Negative step: `vec.iter().rev().skip(len - start - 1).take(start - stop).step_by(step.abs()).cloned().collect()`
- String step slicing: same logic over `.chars()` iterator

### Tuple Slicing

Add support for slicing tuples, a common Python idiom for parsing and ETL:

```python
t = (1, "hello", True, 3.14)
first_two = t[0:2]     # (1, "hello")
last = t[-1]            # 3.14
```

**Semantics:** tuple slicing is resolved at compile time because tuple element types can differ. `t[0:2]` on `tuple[int, str, bool, float]` returns `tuple[int, str]`. The slice indices must be compile-time constants (literals or `const` values).

**Codegen:** direct field access on the Rust tuple. `t[0:2]` -> `(t.0, t.1)`.

**Limitation:** variable-index tuple slicing is not supported (the return type cannot be determined at compile time). Use list conversion for dynamic slicing.

### String Semantics (UTF-8 Fixes)

Current codegen lowers `s[i]` to `x[i as usize]`, which is invalid for Rust `String` (byte-indexed, not character-indexed). This milestone fixes string operations to be character-based:

- `**s[i]`:** returns the i-th character (Unicode code point) as a single-character `str`. Codegen: `s.chars().nth(i).unwrap().to_string()`. In this milestone, panics on out-of-bounds (temporary -- safe `Option` return replaces this in milestone_safe_indexing).
- `**s.len()`:** returns the number of Unicode code points (not bytes). Codegen: `s.chars().count()`.
- `**s.byte_len()`:** returns the number of bytes (O(1)). Codegen: `s.len()`.
- `**s[a:b]`:** returns characters from position `a` to `b` (exclusive). Codegen: `s.chars().skip(a).take(b - a).collect::<String>()`. Returns an empty string if indices are out of range.

**Complexity note:** character-based indexing is O(n) for non-ASCII strings. The compiler should emit a diagnostic note when string indexing is used in a loop, suggesting `.chars()` iteration instead.

### List Methods (Concrete Returns)

List methods that return concrete types (no `Option`/`Result`):

- `.append(item)` -> `vec.push(item)` -- add item to end
- `.extend(other)` -> `vec.extend(other)` -- add all items from another list
- `.insert(i, item)` -> `vec.insert(i, item)` -- insert at index (clamps to bounds)
- `.clear()` -> `vec.clear()` -- remove all items
- `.copy()` -> `vec.clone()` -- shallow copy
- `.reverse()` -> `vec.reverse()` -- reverse in place
- `.count(item)` -> `int` via `vec.iter().filter(|x| x == item).count()` -- count occurrences
- `.contains(item)` -> `bool` via `vec.contains(item)` -- membership test (also via `in` operator)
- `.sort()` -> in-place sort, **primitive types only** (`list[int]`, `list[str]`, `list[bool]`). Codegen: `vec.sort()` (Rust's `Ord` trait covers these types natively -- no protocol dispatch needed). No key functions, no reverse option, no float support in this milestone. The full generic sorting API (key functions, reverse, float rejection, `sorted()` built-in) comes in milestone_generics once `Comparable` protocol and generic bounds exist.

**Deferred to milestone_safe_indexing:** `.pop()` -> `Option[T]`, `.pop(i)` -> `Option[T]`, `.index(item)` -> `Option[int]`, `.remove(item)` -> `Result[None, ValueError]`

### Dict Methods (Concrete Returns)

Dict methods that return concrete types:

- `.keys()` -> iterator over keys. Codegen: `map.keys()`
- `.values()` -> iterator over values. Codegen: `map.values()`
- `.items()` -> iterator over `tuple[K, V]` pairs. Codegen: `map.iter()`
- `.update(other)` -> `map.extend(other)` -- merge another dict (overwrites existing keys)
- `.clear()` -> `map.clear()` -- remove all entries
- `.copy()` -> `map.clone()` -- shallow copy
- `.contains(key)` -> `bool` via `map.contains_key(key)` -- key membership (also via `in` operator)
- `len(d)` -> `int` via `map.len()` -- number of entries

**Deferred to milestone_safe_indexing:** `.get(key)` -> `Option[V]`, `.pop(key)` -> `Option[V]`, `.setdefault(key, default)` -> `V`

### String Methods (Extended)

Beyond what milestone_control_flow already provides (`.len()`, `.upper()`, `.lower()`, `.split()`, `.strip()`):

- `.replace(old, new)` -> `str` via `s.replace(old, new)`
- `.startswith(prefix)` -> `bool`
- `.endswith(suffix)` -> `bool`
- `.join(iterable)` -> `str` -- join items with separator
- `.count(sub)` -> `int` -- count non-overlapping occurrences
- `.isdigit()` -> `bool`, `.isalpha()` -> `bool`, `.isalnum()` -> `bool`, `.isspace()` -> `bool`
- `.lstrip()` -> `str`, `.rstrip()` -> `str` -- strip from left/right only
- `.title()` -> `str`, `.capitalize()` -> `str`, `.swapcase()` -> `str`
- `.center(width)` -> `str`, `.ljust(width)` -> `str`, `.rjust(width)` -> `str`
- `.zfill(width)` -> `str` -- pad with zeros

**Deferred to milestone_safe_indexing:** `.find(sub)` -> `Option[int]`, `.rfind(sub)` -> `Option[int]`

### Tuple Methods

Tuples are immutable (enforced at compile time -- no mutation methods):

- `len(t)` -> `int` -- number of elements (compile-time known)
- Unpacking: `a, b, c = my_tuple` (already in milestone_control_flow)
- `.count(item)` -> `int` -- count occurrences

**Deferred to milestone_safe_indexing:** `.index(item)` -> `Option[int]`

### Built-in Functions (Non-Generic)

Built-in functions that do not require generics (available without `import`):

- `len(x)` -> `int` -- works on `list`, `dict`, `str`, `tuple`. Codegen: `.len()` or `.chars().count()` for strings
- `abs(x)` -> `int` or `float` -- absolute value. Codegen: `.abs()`
- `round(x)` -> `int` -- round float to nearest integer. Codegen: `.round() as i64`
- `round(x, n)` -> `float` -- round to n decimal places
- `isinstance(x, T)` -> `bool` -- already in milestone_type_system for type narrowing
- `repr(x)` -> `str` -- debug representation. Codegen: `format!("{:?}", x)` (requires auto-derived `Debug`)

**Deferred to milestone_classes:** `hash(x)` -> `int` (needs `Hash + Eq` traits from class system)

### Chained Comparisons

Add Python's chained comparison syntax:

```python
if 1 < x < 10:
    print("in range")

if a <= b <= c:
    print("sorted")
```

**Codegen:** `1 < x < 10` desugars to `1 < x && x < 10`, with `x` evaluated only once (use a temporary if `x` is a complex expression).

### String Multiplication

Add string repetition via the `*` operator:

```python
line = "-" * 40     # "----------------------------------------"
header = "abc" * 3  # "abcabcabc"
```

**Codegen:** `"-".repeat(40)` in Rust.

### `pass` Statement

Add the `pass` statement for empty function/class bodies:

```python
def placeholder():
    pass

class EmptyBase:
    pass
```

**Codegen:** no-op (empty block `{}` in Rust).

### Star Unpacking

Add star unpacking for capturing remaining elements:

```python
first, *rest = [1, 2, 3, 4, 5]
# first = 1, rest = [2, 3, 4, 5]

first, *middle, last = [1, 2, 3, 4, 5]
# first = 1, middle = [2, 3, 4], last = 5
```

**Codegen:** slice operations on the underlying `Vec`. `first, *rest = items` -> `let first = items[0]; let rest = items[1..].to_vec();`

### Walrus Operator (`:=`)

Add assignment expressions for concise assign-and-test patterns:

```python
if (n := len(items)) > 10:
    print(f"Too many items: {n}")

while (line := read_line()) != "":
    process(line)
```

**Codegen:** `let n = items.len(); if n > 10 { ... }` -- the compiler hoists the assignment and uses the bound variable in the condition.

### Power Operator Codegen

Specify the codegen for the `**` exponentiation operator (syntax already parsed in milestone_core_language):

- `int ** int` -> `i64::pow(base, exp as u32)` (panics on negative exponent; safe version in milestone_safe_indexing)
- `float ** float` -> `f64::powf(base, exp)`
- `float ** int` -> `f64::powi(base, exp as i32)`

### Multiple Return Values

Explicitly support returning multiple values as tuples (syntax already works via milestone_control_flow tuples, but should be tested):

```python
def divmod(a: int, b: int) -> tuple[int, int]:
    return a // b, a % b

q, r = divmod(17, 5)  # q = 3, r = 2
```

### `for`/`while` ... `else` Clauses

Add Python's loop `else` clause:

```python
for item in items:
    if item == target:
        print("Found!")
        break
else:
    print("Not found")  # runs only if loop completes without break
```

**Codegen:** use a boolean flag to track whether `break` was executed:

```rust
let mut _broke = false;
for item in &items {
    if item == &target {
        println!("Found!");
        _broke = true;
        break;
    }
}
if !_broke {
    println!("Not found");
}
```

### Definition of Done (milestone_ergonomics)

- Augmented assignment (`+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `**=`) works for numeric types, strings, and lists
- Conditional expressions (`a if cond else b`) work as expressions
- Keyword arguments resolve correctly at call site (positional, keyword, mixed)
- Default parameter values are inserted for omitted arguments
- Keyword-only parameters (after `*`) enforced at compile time
- `for item in list` borrows the list; list is usable after the loop
- `list[a:b]` produces a new list (copy, not view)
- Negative indexing: `a[-1]` returns last element
- Step slicing: `a[::2]`, `a[::-1]`, `a[1:7:2]` all produce new collections
- Tuple slicing with compile-time constant indices works
- String indexing is character-based (UTF-8 safe), `s.len()` returns character count
- List methods (concrete): `append`, `extend`, `insert`, `clear`, `copy`, `reverse`, `count`, `contains`, `sort` (primitive types only -- `list[int]`, `list[str]`, `list[bool]`)
- Dict methods (concrete): `keys`, `values`, `items`, `update`, `clear`, `copy`, `contains`
- String methods: `replace`, `startswith`, `endswith`, `join`, `count`, `isdigit`, `isalpha`, `isalnum`, `isspace`, `lstrip`, `rstrip`, `title`, `capitalize`
- Tuple methods: `count` (immutability enforced)
- Built-in functions: `len`, `abs`, `round`, `repr`
- Chained comparisons: `1 < x < 10` works
- String multiplication: `"abc" * 3` works
- `pass` statement works in empty bodies
- Star unpacking: `first, *rest = items` works
- Walrus operator: `if (n := len(x)) > 0` works
- Power operator: `x ** y` has correct codegen for int and float
- Multiple return values: `return a, b` works as tuple packing
- `for`/`while` ... `else` clauses work correctly
- E2E pass tests: augmented_assign, ternary_expr, keyword_args_basic, keyword_args_default, keyword_only_params, for_loop_borrow, list_slice_copy, negative_index_list, negative_index_string, step_slice_basic, step_slice_reverse, step_slice_string, tuple_slice, string_char_index, string_char_len, string_slice, list_methods_concrete, dict_methods_concrete, string_replace, chained_comparison, string_multiply, pass_statement, star_unpacking, walrus_operator, power_operator, multiple_return, loop_else
- E2E fail tests: ternary_type_mismatch, keyword_after_positional_error, missing_keyword_only_arg
- Existing milestone_core_language/milestone_control_flow/milestone_type_system E2E tests still pass (no regressions)
- Milestone demo in `./demos/ergonomics/main.sifr`

---

## milestone_classes: Basic Classes

`status: completed`

**Goal:** Provide minimal class support -- enough to define data types and error types. This must land before milestone_error_handling because typed error hierarchies (`class ValueError(Error)`) require classes. milestone_classes is structurally simpler than error handling: a basic `class Point: x: float; y: float` with `__init__` and methods is straightforward struct codegen.

### Language Features

- `**class` -> `struct` + `impl`:** class definitions become Rust structs with named fields
- `**__init__` -> `new()`:** constructor mapping
- **Methods:** `self` parameter maps to `&self` or `&mut self`
- **Field access:** `obj.field` maps to Rust field access
- **Method receiver inference:** compiler determines `&self` vs `&mut self` vs `self` from body analysis (see Cross-cutting Contracts: Borrow and Lifetime Strategy)
- **Auto-derived traits:** `Debug`, `Clone`, `PartialEq` auto-derived on all classes (conditional `Eq`/`Hash` when all fields support it)
- `**isinstance` narrowing for class types:** extends milestone_type_system's narrowing engine to class instances
- **Class instances as union members:** `Circle | Square` -> Rust enum with one variant per class

### Example Program

```python
class Point:
    x: float
    y: float

    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y

    def distance(self, other: Point) -> float:
        dx: float = self.x - other.x
        dy: float = self.y - other.y
        return (dx * dx + dy * dy) ** 0.5

def main():
    p1 = Point(0.0, 0.0)
    p2 = Point(3.0, 4.0)
    print(p1.distance(p2))  # 5.0
```

### Generated Rust

```rust
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx: f64 = self.x - other.x;
        let dy: f64 = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}
```

### `hash()` Built-in

Now that classes exist with auto-derived `Hash + Eq`:

- `hash(x)` -> `int` -- hash value (only for types where all fields are `Hash + Eq`, compile-time enforced). Codegen: uses `std::hash::Hash` trait.

### Definition of Done (milestone_classes)

- `class` compiles to Rust `struct` + `impl`
- `__init__` maps to `new()` constructor
- Method receiver inference (`&self` / `&mut self` / `self`) works correctly
- Field access compiles to Rust field access
- Auto-derived traits (`Debug`, `Clone`, `PartialEq`, conditional `Eq`/`Hash`) on all classes
- `isinstance` narrowing works for class types
- Class instances work as union type members
- `hash(x)` works for hashable types
- E2E pass tests: class_basic, class_methods, class_field_access, class_isinstance, class_union, hash_builtin
- E2E fail tests: missing_field, use_after_move_self, unhashable_dict_key
- Existing milestone_core_language/milestone_control_flow/milestone_type_system/milestone_ergonomics E2E tests still pass (no regressions)
- Milestone demo in `./demos/classes/main.sifr`

---

## milestone_error_handling: Error Handling

`status: completed`

**Goal:** Provide safe error handling that maps to Rust's `Result`/`Option` types rather than Python's exception model. Benefits from milestone_type_system's union types -- `Result[T, E]` and `Option[T]` are union-based. Benefits from milestone_classes's classes -- typed error hierarchies (`class ValueError(Error)`) are available immediately.

### Language Features

- `**Result[T, E]` type:** explicit error return type (replaces exceptions)
- `**Option[T]` type:** sugar for `T | None`, maps to Rust `Option<T>` (leverages milestone_type_system's union types)
- `**try`/`except` syntax:** reinterpreted as pattern matching on `Result`
- `**try`/`except`/`finally`:** the `finally` block maps to Rust's scope-based cleanup (`Drop` trait). Code in `finally` always executes when the scope exits, regardless of whether an error occurred. Codegen: the `finally` body is placed after the `match` on `Result`, or uses a scope guard pattern. For resource cleanup, prefer `with` statement (milestone_generators) which provides the same guarantee more idiomatically.
- `**?` operator:** early return on error (borrowed from Rust, new syntax for Sifr)
- `**raise` -> `Err()`:** raising maps to returning an error
- **Custom error types:** classes that implement an `Error` protocol
- `**assert` statement**

> **Note:** `class Foo(Error)` in this milestone is a **special-cased error declaration** -- the compiler recognizes the `(Error)` marker and generates the appropriate Rust error type. This is NOT general inheritance syntax. Full single inheritance (arbitrary `class Child(Parent)`) comes in milestone_inheritance.

### Fallible Built-in Functions

Built-in functions that can fail return `Result` (following the Safety Philosophy):

- `int(s)` where `s: str` -> `Result[int, ParseError]` -- parse string to integer. Codegen: `s.parse::<i64>()`
- `float(s)` where `s: str` -> `Result[float, ParseError]` -- parse string to float. Codegen: `s.parse::<f64>()`
- `bool(s)` where `s: str` -> `Result[bool, ParseError]` -- parse "true"/"false" to bool
- `input()` -> `Result[str, IOError]` -- read a line from stdin. Codegen: `std::io::stdin().read_line()`
- `input(prompt)` -> `Result[str, IOError]` -- print prompt, then read from stdin

**Infallible conversions** (no `Result` wrapping needed):

- `int(x)` where `x: float` -> `int` -- truncate float to integer. Codegen: `x as i64`
- `float(x)` where `x: int` -> `float` -- widen integer to float. Codegen: `x as f64`
- `str(x)` for any type -> `str` -- string representation. Codegen: `format!("{:?}", x)` using `Debug` (auto-derived for all classes from milestone_classes). Once milestone_protocols provides `Display` via user-defined `__str__`, `str(x)` upgrades to `format!("{}", x)` for types that implement `Display`, falling back to `Debug` for types that don't.
- `bool(x)` for any type -> `bool` -- truthiness. Codegen: type-specific (0/empty = false, else true)

### Design Decision

Sifr does NOT use Python's exception model (stack unwinding). Instead, errors are values:

```python
def parse_int(s: str) -> Result[int, str]:
    # ...implementation...
    raise "not a number"   # becomes Err("not a number".to_string())

def main():
    result = parse_int("42")?   # early return on error
    print(result)
```

This maps cleanly to Rust's `Result<T, E>` and `?` operator.

### `except` Arm Matching Semantics

The `try`/`except` syntax is reinterpreted as pattern matching on `Result`. Each `except` arm matches a specific error type:

```python
try:
    data = read_file("config.json")?
    config = parse_json(data)?
except IOError as e:
    print(f"File error: {e}")
except ParseError as e:
    print(f"Parse error: {e}")
```

**Rules:**

- `except` arms are matched in order (like `match` arms in Rust)
- Each arm must specify a concrete error type (no bare `except:`)
- The compiler checks exhaustiveness: if the `Result`'s error type is a union `IOError | ParseError`, all variants must be handled (or a catch-all `except Error` must be present)
- `except` arms generate Rust `match` on the error enum variants (see Cross-cutting Contracts: Error Semantics Matrix)

### Typed Error Hierarchies

Error types are classes (milestone_classes provides basic class support, which is now a prerequisite for milestone_error_handling):

```python
class AppError(Error):
    message: str

class ValueError(AppError):
    pass

class IOError(AppError):
    path: str
```

> **Note:** In `milestone_error_subclasses` (Phase 09), built-in error types are expanded into a subclass hierarchy (e.g., `FileNotFoundError` extends `IOError`) with compile-time exhaustiveness checking. All errors keep `message: str`; some gain additional structured fields. See [09_stdlib_safety_remediation.md](09_stdlib_safety_remediation.md).

**Codegen:** Error types generate Rust enums (not structs with inheritance). `AppError` becomes `enum AppError { ValueError(ValueError), IOError(IOError) }`. The `Error` protocol maps to Rust's `std::error::Error` trait.

### Safety Boundary (No-Panic Guarantee)

Sifr's safety philosophy: **all fallible operations return `Result` or `Option`; the compiler enforces handling.** Panic is reserved for programmer invariant violations only.

**Operations that return `Result`:**

- **Division:** `a / b` returns `Result[int, DivisionError]` (or `Result[float, DivisionError]`) when the divisor cannot be statically proven non-zero. If the compiler can prove `b != 0` (e.g., literal divisor `a / 2`), it returns the value directly with no wrapping. Codegen: checked division with zero-check.
- **Integer overflow:** arithmetic on `int` panics on overflow in debug mode (like Rust) and wraps in release mode. This matches Rust's default behavior and avoids making every arithmetic expression require error handling. **Future enhancement:** an opt-in `checked` mode where `a + b` returns `Result[int, OverflowError]` using `checked_add()` etc. This is deferred to avoid making basic programs excessively verbose.
- **Type conversions:** `int(s)` where `s: str` returns `Result[int, ParseError]`. `float(s)` returns `Result[float, ParseError]`. Conversions between numeric types that cannot lose precision (e.g., `int` to `float`) are implicit and infallible.
- **Rust library panics (milestone_ffi FFI):** caught at FFI boundaries via `catch_unwind` where possible and converted to `Result::Err`. C library crashes are non-recoverable (see milestone_ffi FFI contract).

**Operations that return `Option`:**

- **Indexing:** `x[i]` returns `Option[T]` for all indexable types (`str`, `list`, `dict`). Never panics.
- **Dict lookup:** `d[key]` returns `Option[V]`. Never panics on missing key.

**The only panic -- `assert`:**

- `**assert` statements:** generate `assert!()` or `panic!()` in Rust. These are programmer invariant checks -- they catch bugs in logic, not runtime errors. They are intentionally unrecoverable and not catchable by `try`/`except`. `assert` is the ONE place where Sifr intentionally panics.

**Must-Use Contract:**

- `Result` values are `#[must_use]`. Ignoring a `Result` returned by a function is a **compile-time error**.
- `Option` values returned by functions are also `#[must_use]`.
- To explicitly discard an error: `let _ = fallible_operation()` -- this acknowledges the error is intentionally ignored.
- This is the key "if it compiles, it works" guarantee: every error path is either handled or explicitly acknowledged.

### Pattern Matching (milestone_error_handling Foundation)

milestone_error_handling introduces pattern matching as the mechanism for `try`/`except` and `Result`/`Option` handling. This establishes the foundation that milestone_protocols extends with struct destructuring.

**milestone_error_handling pattern matching scope:**

- **Exhaustiveness checking:** `match` on `Result` and `Option` must cover all variants. Missing arms are compile-time errors.
- **Variable binding in arms:** `except ValueError as e` binds the error value.
- **Catch-all arms:** `except Error as e` matches any error type (like `_` in Rust `match`).
- **Match guards:** `case x if x > 0` -- extra conditions on match arms. Codegen: Rust match guards (`pattern if condition => ...`).

**Deferred to milestone_protocols:**

- Struct/class field destructuring in match arms
- Nested pattern matching
- `@` bindings (bind and match simultaneously)

**Deferred to milestone_generics:**

- Pattern matching on generic types

### Definition of Done (milestone_error_handling)

- `Result[T, E]` type compiles to `Result<T, E>` in Rust
- `?` operator works in functions returning `Result`
- `try`/`except` generates correct `match` on error variants
- `raise` inside a `Result`-returning function generates `Err(...)`
- `assert` generates `assert!()` / `panic!()` -- the only panic source in user code
- Division by zero returns `Result[T, DivisionError]`, not a panic
- Integer overflow panics in debug mode, wraps in release mode (matches Rust behavior)
- `int(s)` / `float(s)` / `bool(s)` string conversions return `Result[T, ParseError]`
- `input()` returns `Result[str, IOError]`
- Infallible conversions: `int(f)`, `float(i)`, `str(x)`, `bool(x)` work without `Result`
- Unused `Result` is a compile-time error (`#[must_use]` enforcement)
- Explicit discard via `let _ = expr` compiles without error
- Exhaustiveness checking for `except` arms
- E2E pass tests: result_basic, option_chaining, error_propagation, try_except, division_by_zero_result, int_parse_result, float_parse_result, input_basic, infallible_conversions
- E2E fail tests: unhandled_error, non_exhaustive_except, unused_result_error
- CPython parity tests pass with safe error handling (no panics, `Result`/`Option` where CPython raises). Reference: `Python/bltinmodule.c` (int/float/bool conversions, input), `Lib/test/test_builtin.py`
- Unit tests for Result/Option type checking and inference
- Milestone demo in `./demos/error_handling/main.sifr`

---

## milestone_safe_indexing: Safe Indexing and Option Returns

`status: completed`

**Goal:** Now that `Option[T]`, `Result[T, E]`, the `?` operator, and `try`/`except` exist (from milestone_error_handling), make all indexing and fallible collection operations safe. This eliminates the last remaining panic sources from collection access.

### Safe Indexing

All indexing operations now return `Option[T]` instead of panicking on out-of-bounds:

- `**list[i]`** -> `Option[T]` via `vec.get(i).cloned()` -- returns `None` if out-of-bounds
- `**dict[key]`** -> `Option[V]` via `map.get(key).cloned()` -- returns `None` if key missing
- `**str[i]`** -> `Option[str]` via `s.chars().nth(i).map(|c| c.to_string())` -- returns `None` if out-of-bounds
- **Negative indexing:** `list[-1]` -> `Option[T]` -- negative index resolved relative to length, then safe lookup

This is the core of Sifr's "no panic" guarantee for data access. Users handle the `Option` with `?`, `match`, `.unwrap_or(default)`, or `.expect("msg")`.

### List Methods (Option/Result Returns)

Methods deferred from milestone_ergonomics that return `Option` or `Result`:

- `.pop()` -> `Option[T]` via `vec.pop()` -- remove and return last item, or `None` if empty
- `.pop(i)` -> `Option[T]` -- remove and return item at index, or `None` if out-of-bounds
- `.index(item)` -> `Option[int]` via `vec.iter().position(|x| x == item)` -- find index, or `None`
- `.remove(item)` -> `Result[None, ValueError]` -- remove first occurrence, or error if not found

### Dict Methods (Option Returns)

- `.get(key)` -> `Option[V]` -- safe lookup (same as `d[key]` under safe indexing)
- `.pop(key)` -> `Option[V]` via `map.remove(key)` -- remove and return value
- `.setdefault(key, default)` -> `V` -- return value if key exists, otherwise insert default and return it

### String Methods (Option Returns)

- `.find(sub)` -> `Option[int]` -- find first occurrence index, or `None`
- `.rfind(sub)` -> `Option[int]` -- find last occurrence index, or `None`

### Tuple Methods (Option Returns)

- `.index(item)` -> `Option[int]` -- find index of item

### Safe Power Operator

- `int ** negative_int` -> `Result[int, ValueError]` (negative exponents produce fractions, not representable as `int`)

### `del` Statement (Item/Key Deletion)

Add `del` for collection item removal as syntax sugar:

```python
items = [1, 2, 3, 4, 5]
del items[2]          # removes element at index 2 -> items = [1, 2, 4, 5]

config = {"a": 1, "b": 2}
del config["a"]       # removes key "a" -> config = {"b": 2}
```

**Semantics:**

- `del d[key]` -> `d.pop(key)` (discards the returned `Option`)
- `del a[i]` -> `a.pop(i)` (discards the returned `Option`)
- `del a[i:j]` -> removes a slice of elements
- `del x` (name unbinding) -> **not supported** in Sifr. Variables are dropped at scope end (Rust's RAII). This is an intentional divergence from Python.

**Codegen:** `del d[key]` -> `let _ = d.remove(&key);`

### Definition of Done (milestone_safe_indexing)

- `list[i]` returns `Option[T]` -- no panic on out-of-bounds
- `dict[key]` returns `Option[V]` -- no panic on missing key
- `str[i]` returns `Option[str]` -- no panic on out-of-bounds
- Negative indexing returns `Option[T]` consistently
- List methods: `pop`, `index`, `remove` return `Option`/`Result`
- Dict methods: `get`, `pop`, `setdefault` return `Option`
- String methods: `find`, `rfind` return `Option`
- Tuple methods: `index` returns `Option`
- `del d[key]` and `del a[i]` work as syntax sugar
- `int ** negative_int` returns `Result`
- Users can ergonomically handle `Option` with `?`, `match`, `.unwrap_or()`, `.expect()`
- E2E pass tests: safe_list_index, safe_dict_key, safe_string_index, list_pop_option, list_index_option, list_remove_result, dict_get_option, dict_pop_option, string_find_option, del_dict_key, del_list_item, safe_negative_index, safe_power_negative
- E2E fail tests: unused_option_error, unused_result_error
- CPython parity tests pass with safe error handling (no panics, `Result`/`Option` where CPython raises). Reference: `Objects/listobject.c`, `Objects/dictobject.c`, `Objects/unicodeobject.c`, `Lib/test/test_list.py`, `Lib/test/test_dict.py`, `Lib/test/test_str.py`
- Existing E2E tests still pass (no regressions)
- Milestone demo in `./demos/safe_indexing/main.sifr`

---

## milestone_imports: Multi-file Compilation and Imports

`status: completed`

**Goal:** Support multi-file projects with imports, enabling real application structure. This milestone focuses on the compilation model only -- package management (`sifr.toml`, `sifr.lock`, dependency resolution) is deferred to milestone_package_mgmt (just before milestone_ecosystem) since it's only useful once there's an ecosystem to manage.

### Language Features

- `**import` / `from ... import`:** maps to Rust `mod` / `use`
- **Multi-file compilation:** compile a directory of `.sifr` files into one binary
- **Package structure:** `__init__.sifr` defines a package (like `mod.rs`)
- **Visibility:** `_private` prefix convention enforced as `pub`/non-`pub`
- **Relative imports:** `from .utils import helper` works within a package

### Project Structure

```
my_app/
  src/
    main.sifr
    models/
      __init__.sifr
      user.sifr
    utils/
      __init__.sifr
      helpers.sifr
```

### Import and Module Semantics

- **Import cycle detection:** the compiler builds a module dependency graph during compilation. Circular imports are a compile-time error with a clear diagnostic showing the cycle path (e.g., `a.sifr -> b.sifr -> c.sifr -> a.sifr`).
- `**__init__.sifr` semantics:** defines the public API of a package. Only symbols explicitly defined or re-exported in `__init__.sifr` are importable from outside the package. No side effects on import (unlike Python's `__init__.py` which executes on import).
- **Module compilation order:** topological sort of the dependency graph. Each module is compiled exactly once per compilation run. The driver maintains a module cache keyed by canonical file path.
- **Relative imports:** `from .utils import helper` works within a package. Relative imports cannot escape the package root.
- **Multi-file span/diagnostic mapping:** error messages for imported modules show the correct source file and line number, not the generated Rust file.

### Example

```python
# src/models/user.sifr
class User:
    name: str
    email: str

    def __init__(self, name: str, email: str):
        self.name = name
        self.email = email

# src/main.sifr
from models.user import User

def main():
    user = User("Alice", "alice@example.com")
    print(user.name)
```

### Definition of Done (milestone_imports)

- `import` / `from ... import` compiles to Rust `mod` / `use`
- Multi-file projects compile into a single binary
- `__init__.sifr` controls package public API
- `_private` prefix enforced as non-`pub` in generated Rust
- Circular import detection with clear diagnostics showing the cycle path
- Multi-file diagnostics show correct source file and line numbers
- Relative imports work within packages
- E2E pass tests: multi_file_basic, package_import, relative_import
- E2E fail tests: circular_import, private_access, missing_module
- Milestone demo in `./demos/imports/main.sifr` (multi-file project)

---

## milestone_codegen_quality: Codegen Quality Refinement

`status: completed`

**Goal:** Improve the quality and idiomaticity of generated Rust code by eliminating systematic codegen patterns that produce correct but non-idiomatic output. This is a Phase 1 refinement step that ensures all future milestones build on clean codegen.

**Rationale:** Phase 1 is complete, so all codegen patterns are now established. Every demo generates correct Rust, but with recurring quality issues: unnecessary `mut`, redundant `format!` nesting, verbose string handling, and wasteful HashMap lookups. Fixing these now prevents the issues from compounding as Phase 2 adds more complex codegen.

> **Note:** Some codegen issues are already covered by upcoming milestones: method receiver inference (`&self` vs `&mut self`) is in `milestone_classes`, redundant `as f64` will be addressed in `milestone_protocols` with operator overloading, and `std::collections::HashMap` qualification will improve as import handling matures.

### Tasks

#### Task 1: Remove unnecessary `mut` on variables never reassigned

Every `let` binding is currently emitted as `let mut`. The codegen should track whether a variable is ever reassigned and only emit `mut` when needed.

**Approach:** Before emitting a function body, scan the HIR statements to collect which variables are assigned more than once (or assigned after their initial `let` binding). Only emit `mut` for those variables.

**Where to fix:** `crates/sifr_codegen/src/lib.rs` -- the variable declaration / `let` emission logic.

**Expected impact:** ~60 fewer unnecessary `mut` annotations across all demos.

#### Task 2: Eliminate `println!("{}", format!(...))` double-formatting

When `print(f"...")` is compiled, it generates `println!("{}", format!("...", args))` -- a redundant double-format. Should emit `println!("...", args)` directly.

**Approach:** When the `print` argument is an f-string (`HirExpr::FString`), instead of emitting `println!("{}", <fstring_expr>)`, inline the f-string format string and arguments directly into the `println!` macro call.

**Where to fix:** `crates/sifr_codegen/src/lib.rs` -- the `print` call handling in `emit_expr` and the f-string emission logic.

**Expected impact:** ~40 fewer redundant `format!` calls.

#### Task 3: Remove redundant `.to_string()` on string literals in display contexts

Patterns like `println!("{}", "hello".to_string())` and `"literal".to_string()` appear in contexts where `&str` suffices.

**Approach:** In display contexts (println, format), emit string literals as `"hello"` not `"hello".to_string()`. Only call `.to_string()` when a `String` (owned) is actually needed (variable binding, function argument expecting `String`, etc.).

**Where to fix:** `crates/sifr_codegen/src/lib.rs` -- string literal emission.

**Expected impact:** ~20 fewer redundant `.to_string()` calls.

#### Task 4: Remove `"lit".to_string().as_str()` for string method arguments

`s.starts_with("sifr".to_string().as_str())` should be `s.starts_with("sifr")`.

**Approach:** When emitting a string literal as an argument to a method that accepts `&str` (like `starts_with`, `ends_with`, `contains`, `replace`, `find`), emit the literal directly without `.to_string().as_str()`.

**Where to fix:** `crates/sifr_codegen/src/lib.rs` -- string method call emission.

**Expected impact:** ~10 fewer verbose string method calls.

#### Task 5: Simplify HashMap lookups with string literal keys

`ages.get(&"alice".to_string())` allocates a `String` unnecessarily. Should be `ages.get("alice")` since `HashMap<String, V>::get` accepts `&str` via `Borrow`.

**Approach:** When the key expression is a string literal, emit `"key"` directly instead of `&"key".to_string()`.

**Where to fix:** `crates/sifr_codegen/src/lib.rs` -- dict indexing / `.get()` emission.

**Expected impact:** ~10 fewer unnecessary String allocations.

#### Task 6: Flatten nested `format!` for string concatenation

`format!("{}{}", format!("{}{}", a, b), c)` instead of `format!("{}{}{}", a, b, c)`.

**Approach:** Flatten chained string `+` operations into a single `format!` call with all parts, by collecting all operands of a chain of `BinOp::Add` on strings before emitting.

**Where to fix:** `crates/sifr_codegen/src/lib.rs` -- string concatenation (`+` operator on strings).

**Expected impact:** Cleaner string concatenation in generated code.

### Definition of Done (milestone_codegen_quality)

- Generated Rust from all demos produces zero `cargo clippy` warnings (beyond vendored crate suppression)
- No unnecessary `mut` on variables that are never reassigned
- No `println!("{}", format!(...))` -- all print+fstring combos emit a single `println!`
- String literals are not wrapped in `.to_string()` in display/borrow contexts
- HashMap lookups with string literal keys use `"key"` not `&"key".to_string()`
- No nested `format!` for string concatenation chains
- All existing tests pass (no regressions)
- New unit tests in `sifr_codegen` for each pattern
- Re-emitted `.rs` files in `demos/` show clean, idiomatic Rust

---

## Milestone Ordering

- **milestone_ergonomics before milestone_classes:** Language ergonomics (ternary, kwargs, methods, slicing) make the language usable before adding classes
- **milestone_classes before milestone_error_handling:** Basic classes must exist before error handling so typed error hierarchies (`class ValueError(Error)`) work immediately in milestone_error_handling
- **milestone_error_handling before milestone_safe_indexing:** Error handling tools (`?`, `match`, `unwrap_or`) must exist before safe indexing returns `Option` values that users need to handle
- **milestone_safe_indexing before milestone_imports:** Safe indexing completes the safety story for single-file programs before adding multi-file compilation
- **milestone_imports before milestone_codegen_quality:** Phase 1 is complete after imports, so all codegen patterns are established. Fixing codegen quality now means every future milestone builds on clean, idiomatic Rust output.
- **milestone_codegen_quality before milestone_protocols:** Codegen refinement is a natural Phase 1 cleanup step. Protocols add significant new codegen complexity, so starting from clean codegen avoids compounding quality issues.
