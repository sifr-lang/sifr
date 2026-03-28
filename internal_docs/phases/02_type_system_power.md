# Type System Power

This phase makes Sifr's type system truly expressive by adding protocols (traits), operator overloading, discriminated unions, single inheritance, generics, closures, iterators, generators, context managers, and function decorators. By the end of this phase, Sifr supports the full range of OOP and generic programming patterns needed for real-world applications, with all constructs compiling to clean, idiomatic Rust.

## milestone_protocols: Protocols, Operators, and Discriminated Unions

status: completed

**Goal:** Add the advanced OOP features that make the type system expressive: protocols (traits), operator overloading, discriminated unions, and pattern matching on classes. Builds on milestone_classes's basic class support and milestone_type_system's narrowing engine.

> **Note:** Protocols before generics are primarily for **operator overloading**, **discriminated union narrowing**, and **dynamic dispatch** (`&dyn Trait`). Protocol-as-generic-bound (e.g., `def sort[T: Comparable](items: list[T])`) is a milestone_generics feature -- protocols defined here will be usable as bounds once generics land.

### Design Decision: Nominal vs Structural Typing

Sifr uses **nominal typing by default** (like Rust) with **structural matching via protocols** (like TypeScript's interfaces):

- Two classes with identical fields are NOT automatically assignable to each other (nominal)
- A `Protocol` defines a structural contract -- any class that has the required fields/methods satisfies it (structural)
- This matches Rust's trait system: types are distinct, but traits provide shared interfaces

This is a deliberate middle ground between TypeScript (fully structural) and Rust (fully nominal). Protocols give the flexibility of structural typing where needed, while nominal classes prevent accidental type confusion.

### Language Features

- **Protocols/Interfaces:** `Protocol` classes map to Rust traits (structural matching -- any class with the right shape satisfies the protocol)
- **Operator overloading:** `__add__`, `__eq__`, `__lt__`, `__str__`, etc. map to Rust trait impls (`Add`, `PartialEq`, `PartialOrd`, `Display`)
- **Discriminated unions:** classes with a shared literal-typed tag field, narrowed via attribute equality (leverages milestone_type_system's narrowing engine):

```python
class Circle:
    tag: "circle" = "circle"
    radius: float

class Square:
    tag: "square" = "square"
    side: float

type Shape = Circle | Square

def area(shape: Shape) -> float:
    if shape.tag == "circle":
        return 3.14159 * shape.radius * shape.radius  # narrowed to Circle
    else:
        return shape.side * shape.side                  # narrowed to Square
```

- **Property existence narrowing (`in`):** `if "name" in obj:` narrows the type to one that has a `name` field (extends milestone_type_system's narrowing to object properties)
- **Pattern matching on classes (extends milestone_error_handling):**
  - **Field destructuring:** `case Point(x=x, y=y)` or `case Point(x, y)` in match arms
  - **Nested patterns:** `case Line(start=Point(x=0, y=0), end=end_point)`
  - **`@` bindings:** `case p @ Point(x=0, y=_)` -- bind the whole value while matching fields

### Runtime Type Representation for Classes

- **Protocol/trait objects:** when a protocol is used as a parameter type, generate `&dyn Trait` or `Box<dyn Trait>`. This is the only dynamic dispatch for class types.
- **Discriminated union of classes:** generate Rust `enum` with one variant per class. Tag-based narrowing generates `match` on the tag field.

### Algebraic Data Types (ADTs)

Class unions already provide ADT-like modeling: `Circle | Square` compiles to a Rust enum with one variant per class, and `isinstance` narrowing generates exhaustive `match`. This means Sifr already has algebraic data types via its existing union + class system.

Explicit `enum` syntax with data-carrying variants (e.g., `enum Shape: Circle(radius: float) | Rectangle(w: float, h: float)`) is an **optional ergonomic enhancement**, not a conceptual gap. It may be evaluated after milestone_protocols stabilizes as syntax sugar over class unions.

### Newtype Pattern

Newtypes -- thin wrappers around primitives that add validation and type safety:

```python
class Port(int):
    pass

def make_port(value: int) -> Result[Port, ValueError]:
    if value < 0 or value > 65535:
        raise ValueError("port must be 0-65535")
    return Port(value)
```

Construction is fallible -- callers must handle the `Result`:

```python
port = make_port(8080)?          # propagate error
port = make_port(99999)?         # returns Err(ValueError)
```

> **Note:** this example uses a module-level factory function because `@staticmethod` is not available until milestone_inheritance. Once milestone_inheritance lands, the idiomatic pattern becomes `Port.new(value)` via `@staticmethod`.

> **Note:** `class Port(int)` is a **special-cased newtype declaration** -- the compiler recognizes primitive type parents (`int`, `float`, `str`, `bool`) and generates a Rust newtype struct (e.g., `struct Port(i64)`). This is NOT general inheritance syntax; full single inheritance (`class Child(Parent)` for arbitrary classes) comes in milestone_inheritance. This follows the same pattern as `class Foo(Error)` in milestone_error_handling, which is also a special-cased declaration.

This maps to Rust's newtype pattern (`struct Port(i64)`) with zero-cost runtime representation. The compiler enforces that `Port` and `int` are distinct types -- you cannot pass an `int` where a `Port` is expected without explicit construction. Validation uses `Result`, not `assert`, because invalid input is a runtime condition (not a programmer bug).

### Struct Update / Spread Semantics

When copying a class instance with field overrides (similar to Python's `dataclasses.replace` or JS spread):

```python
new_user = User(email="new@example.com", **old_user)
```

**Contract:** spread/update **clones** non-overridden fields (implicit `.clone()`). This matches Python semantics and avoids partial-move complexity. The compiler emits `.clone()` for each non-overridden field. If a field type does not implement `Clone`, this is a compile-time error.

### Definition of Done (milestone_protocols)

- `Protocol` compiles to Rust `trait`
- Discriminated unions with tag fields narrow correctly via `match`
- Operator overloading (`__add__`, `__eq__`, `__lt__`) maps to Rust trait impls
- Pattern matching with field destructuring works on class types
- Nested patterns and `@` bindings work
- Property existence narrowing (`in`) works
- Newtype pattern works with fallible construction
- Struct update/spread clones non-overridden fields
- E2E pass tests: protocol_dispatch, discriminated_union, operator_overload, pattern_destructure, nested_pattern, at_binding, property_narrowing, newtype_basic, struct_update
- E2E fail tests: protocol_not_satisfied, non_exhaustive_match, newtype_validation_error
- Milestone demo in `./demos/protocols/main.sifr`

---

## milestone_inheritance: Inheritance and Class Utilities

status: completed

**Goal:** Add single inheritance, `super()`, class-level methods, and properties. These are important for OOP but not blocking for error handling or protocols.

### Language Features

- **Single inheritance:** via trait delegation (not Rust inheritance, which doesn't exist). A child class inherits all fields and methods from its parent. Codegen: the child struct embeds the parent struct and delegates method calls.
- **`super()`:** calls parent class method in inheritance chains. Codegen: direct call to the parent struct's impl method (e.g., `ParentType::method(self, ...)`). Works with single inheritance only.
- **`@classmethod`:** class-level methods that receive the class type rather than an instance. Codegen: associated functions (no `self` parameter) on the struct impl. Called as `MyClass.method()` rather than `instance.method()`.
- **`@staticmethod`:** methods that belong to the class namespace but receive neither `self` nor `cls`. Codegen: free functions in the struct's impl block with no receiver.
- **Properties:** `@property` maps to getter methods, `@property.setter` maps to setter methods.

### Example

```python
class Animal:
    name: str
    sound: str

    def __init__(self, name: str, sound: str):
        self.name = name
        self.sound = sound

    def speak(self) -> str:
        return f"{self.name} says {self.sound}"

class Dog(Animal):
    breed: str

    def __init__(self, name: str, breed: str):
        super().__init__(name, "Woof")
        self.breed = breed

    @classmethod
    def from_shelter(cls, name: str) -> Dog:
        return Dog(name, "Unknown")

    @staticmethod
    def species() -> str:
        return "Canis familiaris"
```

### Definition of Done (milestone_inheritance)

- Single inheritance works (child inherits parent fields and methods)
- `super()` calls parent methods correctly
- `@classmethod` compiles to associated functions
- `@staticmethod` compiles to free functions in impl block
- `@property` getter/setter works
- E2E pass tests: inheritance_basic, super_call, classmethod_basic, staticmethod_basic, property_getter_setter
- E2E fail tests: multiple_inheritance_rejected, super_no_parent
- Milestone demo in `./demos/inheritance/main.sifr`

---

## milestone_generics: Generics and Advanced Types

status: completed

**Goal:** Support generic programming, closures, and higher-order functions. Union types and type aliases already exist from milestone_type_system, so this focuses on parameterized types.

### Language Features

- **Generic functions:** `def first[T](items: list[T]) -> T` (Python 3.12 syntax)
- **Generic classes:** `class Stack[T]:` (Python 3.12 syntax)
- **Type bounds:** `def sort[T: Comparable](items: list[T])`
- **Closures / lambdas:** `lambda x: x + 1` maps to Rust closures
- **Contextual typing for lambdas:** lambda parameter types inferred from call-site context (e.g., `map_list(numbers, lambda x: x * 2)` infers `x: int` from `list[int]`)
- **Higher-order functions:** `map`, `filter`, `reduce` on collections (lazy iterators)
- **Iterators:** `__iter__` / `__next__` protocol maps to Rust `Iterator` trait
- **Generic built-in functions:** `min`, `max`, `sum`, `sorted`, `reversed`, `zip`, `enumerate`, `any`, `all` (see below)
- **Sorting:** `list.sort()`, `sorted()` with key functions and reverse option
- **Utility types (TypeScript-inspired):** built-in type aliases for common transformations:
  - `Partial[T]` -- all fields optional (maps to `Option<field>` for each field)
  - `Readonly[T]` -- all fields immutable (maps to non-`mut` references)
  - `Pick[T, "field1", "field2"]` -- subset of fields
  - `Omit[T, "field1"]` -- all fields except specified
  - `Record[K, V]` -- sugar for `dict[K, V]`
- **Mapped/conditional types (stretch):** type-level programming
- **List/dict/set comprehensions:** syntactic sugar over iterator chains (naturally belongs with iterators):
  - `[x * 2 for x in items]` -> `.iter().map(|x| x * 2).collect::<Vec<_>>()`
  - `[x for x in items if x > 0]` -> `.iter().filter(|x| x > 0).map(|x| x).collect()`
  - `{k: v for k, v in pairs}` -> `.iter().map(|(k, v)| (k, v)).collect::<HashMap<_, _>>()`
  - `{x for x in items}` -> `.iter().map(|x| x).collect::<HashSet<_>>()`
  - Nested `for` -> `.flat_map()`

### Example Program

```python
def map_list[T, U](items: list[T], f: (T) -> U) -> list[U]:
    result: list[U] = []
    for item in items:
        result.append(f(item))
    return result

def main():
    numbers: list[int] = [1, 2, 3, 4, 5]
    doubled = map_list(numbers, lambda x: x * 2)
    print(doubled)
```

### Closure Capture Rules

Closure captures are inferred from usage inside the closure body (see Cross-cutting Contracts: Borrow and Lifetime Strategy):

- Read-only access to outer variable: capture by `&T`
- Mutation of outer variable: capture by `&mut T`
- Variable consumed or closure outlives scope: capture by value (move)
- Explicit `move` keyword forces capture by value: `move lambda x: x + captured_var`

### Closure Kind Inference

Rust has three closure traits: `Fn` (immutable borrow), `FnMut` (mutable borrow), and `FnOnce` (consumes captured values). Sifr **hides these from the user** and infers the closure kind automatically:

- The compiler analyzes the closure body to determine the most permissive kind.
- Functions accepting closures declare their requirement implicitly via usage (how many times the closure is called, whether it's stored, etc.).
- If a closure moves a captured value but is called multiple times, the compiler emits a clear error: "this closure moves `x` but is called multiple times -- consider using `.clone()` or restructuring."
- The user never sees `FnOnce`, `FnMut`, or `Fn` -- these are internal codegen details.

**Codegen:** the compiler emits the correct Rust closure trait bound based on inference. `sort_by_key` gets `FnMut`, `unwrap_or_else` gets `FnOnce`, etc.

### Iterator Borrowing Semantics

Sifr's `for` loop follows Python semantics for ergonomics:

- **`for item in collection`:** borrows the collection (does not consume it). The collection remains usable after the loop. Codegen: `for item in &collection`.
- **`for item in collection.consume()`:** takes ownership of the collection. The collection is moved and cannot be used after the loop. Codegen: `for item in collection`.
- **Iterator protocol:** `__iter__` returns an iterator; `__next__` returns `Option[T]`. Maps to Rust's `Iterator` trait with `next(&mut self) -> Option<Self::Item>`.
- **Three iterator modes (internal):** the compiler generates `iter()` (borrow), `iter_mut()` (mutable borrow), or `into_iter()` (consume) based on usage context. The user only sees `for item in collection`.
- **Lazy evaluation:** `map`, `filter`, and other iterator adapters are lazy -- they produce new iterators without allocating intermediate collections. Only consuming operations (`collect`, `sum`, `for` loop) trigger evaluation.

### Generic Built-in Functions

These built-in functions require generics and the iterator protocol. Available without `import`:

- `min(iterable)` -> `Option[T]` where `T: Comparable` -- smallest element, or `None` if empty. Codegen: `.iter().min().cloned()`
- `max(iterable)` -> `Option[T]` where `T: Comparable` -- largest element, or `None` if empty. Codegen: `.iter().max().cloned()`
- `sum(iterable)` -> `T` where `T: Addable` -- sum of elements (with zero default). Codegen: `.iter().sum()`
- `sum(iterable, start)` -> `T` -- sum with custom start value. Codegen: `.iter().fold(start, |a, b| a + b)`
- `sorted(iterable)` -> `list[T]` where `T: Comparable` -- return new sorted list. Codegen: `{ let mut v = ...; v.sort(); v }`
- `sorted(iterable, key=f)` -> `list[T]` -- sort by key function. Codegen: `.sort_by_key(f)`
- `sorted(iterable, reverse=True)` -> `list[T]` -- sort descending. Codegen: `.sort(); .reverse()`
- `reversed(iterable)` -> iterator -- reverse iterator. Codegen: `.iter().rev()`
- `zip(a, b)` -> iterator of `tuple[A, B]` -- pair elements. Codegen: `a.iter().zip(b.iter())`
- `zip(a, b, c)` -> iterator of `tuple[A, B, C]` -- variadic zip (up to reasonable arity)
- `enumerate(iterable)` -> iterator of `tuple[int, T]` -- index-value pairs. Codegen: `.iter().enumerate()`
- `enumerate(iterable, start=n)` -> iterator of `tuple[int, T]` -- with custom start index
- `any(iterable)` -> `bool` -- `True` if any element is truthy. Codegen: `.iter().any(|x| x.into())`
- `all(iterable)` -> `bool` -- `True` if all elements are truthy. Codegen: `.iter().all(|x| x.into())`
- `map(f, iterable)` -> lazy iterator -- apply function to each element (already mentioned above)
- `filter(f, iterable)` -> lazy iterator -- keep elements where function returns `True`
- `reduce(f, iterable)` -> `Option[T]` -- reduce to single value, or `None` if empty. Codegen: `.iter().reduce(f)`

### Sorting Contract

Sorting requires a `Comparable` protocol (maps to Rust's `Ord` trait):

- `list.sort()` -> in-place sort. Requires `T: Comparable`. Codegen: `vec.sort()`
- `list.sort(key=f)` -> in-place sort by key. Codegen: `vec.sort_by_key(f)`
- `list.sort(reverse=True)` -> in-place sort descending. Codegen: `vec.sort(); vec.reverse()`
- `sorted(iterable)` -> new sorted list (see Generic Built-in Functions above)
- **Stability:** all sorts are stable (matching Python and Rust's default sort behavior)
- **Float sorting:** `list[float].sort()` is a compile-time error because `float` is not `Comparable` (due to `NaN`). Use `list.sort(key=lambda x: x)` with an explicit total-ordering wrapper, or filter `NaN` values first. This matches Rust's `f64` not implementing `Ord`.

### Definition of Done (milestone_generics)

- Generic functions with type parameters compile correctly (monomorphized)
- Generic classes with type parameters compile correctly
- Type bounds (`T: Protocol`) enforce constraints
- Lambda expressions compile to Rust closures
- Contextual typing infers lambda parameter types from call-site
- Closure capture inference works correctly (borrow vs move)
- Closure kind inference (Fn/FnMut/FnOnce) works automatically without user annotation
- Higher-order functions (`map`, `filter`) work with lambdas
- Iterator protocol (`__iter__` / `__next__`) maps to Rust `Iterator`
- `for item in collection` borrows by default; `collection.consume()` for ownership transfer
- Lazy iterator adapters (`map`, `filter`) work without intermediate allocations
- Generic built-ins: `min`, `max`, `sum`, `sorted`, `reversed`, `zip`, `enumerate`, `any`, `all`, `reduce`
- `list.sort()` and `sorted()` work with key functions and reverse option
- Float sorting rejected at compile time (not `Comparable`)
- List comprehensions compile to `.iter().map().collect()`
- Filtered comprehensions compile to `.iter().filter().map().collect()`
- Nested comprehensions compile to `.flat_map()`
- Dict comprehensions compile to `.collect::<HashMap>()`
- Set comprehensions compile to `.collect::<HashSet>()`
- E2E pass tests: generic_function, generic_class, lambda_basic, higher_order, iterator, for_loop_borrow, lazy_iterator, builtin_min_max_sum, sorted_basic, sorted_key_reverse, zip_enumerate, any_all, reduce_basic, list_comp, dict_comp, set_comp, filtered_comp, nested_comp
- E2E fail tests: type_bound_violation, generic_mismatch, closure_move_called_twice, float_sort_rejected, comp_type_mismatch
- CPython parity tests pass with safe error handling (no panics, `Result`/`Option` where CPython raises). Reference: `Python/bltinmodule.c` (min, max, sum, sorted, zip, enumerate, any, all), `Objects/listobject.c` (list.sort), `Lib/test/test_builtin.py`
- Milestone demo in `./demos/iterators_and_comprehensions/main.sifr`

---

## milestone_generators: Generators and Context Managers

status: completed

**Goal:** Add generators (`yield`) and context managers (`with` statement). These are complex features that deserve focused attention: generators require state machine transformation, and context managers require the `ContextManager` protocol. Comprehensions have been moved to milestone_generics since they are simple iterator sugar.

### Generator Expressions and `yield`

```python
# Generator expression (lazy)
squares = (x * x for x in range(1000000))

# Generator function
def fibonacci() -> Generator[int]:
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b
```

**Codegen:** generators compile to Rust iterators via state machine transformation:

- Generator expressions -> lazy iterator (no `.collect()`)
- `yield` functions -> a struct implementing `Iterator` with a state enum tracking the current yield point
- Each `yield` becomes a state transition; local variables are stored in the struct
- `next()` resumes from the last yield point

**`yield from`:** delegates to a sub-generator, forwarding all values:

```python
def chain(a, b):
    yield from a
    yield from b
```

Codegen: `yield from sub` desugars to `for item in sub: yield item` -- the sub-generator is iterated and each value is yielded. This compiles to chaining the sub-iterator's state machine into the parent's state machine.

**Scope:** this milestone covers sync generators only. Async generators (`async for`, `yield` in `async def`) are deferred to milestone_async_advanced.

### `with` Statement (Context Managers)

```python
with open("file.txt") as f:
    data = f.read()
# f is automatically closed here
```

**Codegen:** `with` maps to Rust's scoped resource pattern:

```rust
{
    let f = File::open("file.txt")?;
    let data = read_to_string(&f)?;
    // f is dropped (closed) at end of scope
}
```

**Protocol:** types used in `with` must implement a `ContextManager` protocol with `__enter__` and `__exit__` methods. `__exit__` maps to `Drop` in the generated Rust.

### Definition of Done (milestone_generators)

- Generator expressions produce lazy iterators (no allocation until consumed)
- `yield` functions compile to state machine iterators
- `yield from` delegates to sub-generators correctly
- `with` statement works for resource management (files, etc.)
- `ContextManager` protocol enforced at compile time
- E2E pass tests: generator_expr, yield_basic, yield_infinite, yield_from_basic, yield_from_chain, with_file, with_multiple
- E2E fail tests: yield_outside_function, with_non_context_manager
- Milestone demo in `./demos/generators/main.sifr`

---

## milestone_decorators: Basic Function Decorators

status: completed

**Goal:** Add function decorator support and variadic arguments (`*args`/`**kwargs`) -- the two features needed for milestone_web_framework's web routing (`@app.get("/")`, `@app.post("/users")`). Generic decorators require `*args`/`**kwargs` to wrap functions with arbitrary signatures. Full metaprogramming decorators (`@dataclass`, custom compile-time transforms) remain in milestone_metaprogramming.

### Language Features

- **Function decorators:** `@decorator` syntax that wraps a function with another function
- **Decorator with arguments:** `@app.get("/path")` -- decorator factories that return a decorator
- **Multiple decorators:** stacked decorators applied bottom-up (same as Python)
- **`*args`:** variadic positional arguments captured as a tuple. Codegen: tuple of trait objects or monomorphized dispatch.
- **`**kwargs`:** variadic keyword arguments captured as a dict. Codegen: `HashMap<String, T>` with trait objects or monomorphized dispatch. **Note:** basic keyword arguments (named params, defaults, keyword-only params) are in milestone_ergonomics. This milestone adds the *variadic* forms needed for generic function wrapping.

### Semantics

A decorator is simply a function that takes a function and returns a function:

```python
def my_decorator(func):
    def wrapper(*args, **kwargs):
        print("Before")
        result = func(*args, **kwargs)
        print("After")
        return result
    return wrapper

@my_decorator
def hello():
    print("Hello!")
```

**Codegen:** `@decorator` desugars to `func = decorator(func)` at compile time. The compiler verifies that the decorator's return type is compatible with the decorated function's type.

**Note:** this milestone provides runtime function wrapping and variadic arguments. Compile-time AST transformations (`@dataclass`, custom class decorators) are in milestone_metaprogramming.

### Definition of Done (milestone_decorators)

- `@decorator` syntax wraps functions correctly
- `@decorator_factory(args)` works (decorator with arguments)
- Multiple stacked decorators apply in correct order
- Type checking verifies decorator input/output compatibility
- `*args` captures extra positional arguments as a tuple
- `**kwargs` captures extra keyword arguments as a dict
- A generic decorator can wrap functions with different signatures using `*args`/`**kwargs`
- E2E pass tests: basic_decorator, decorator_with_args, stacked_decorators, args_kwargs_basic, generic_decorator_wrapping
- E2E fail tests: decorator_type_mismatch
- Milestone demo in `./demos/decorators/main.sifr`

---

## milestone_codegen_quality_v2: Phase 2 Codegen Polish

status: completed

**Goal:** Improve the quality and idiomaticity of Rust code generated by the Phase 2 milestones (protocols, inheritance, generics, generators, decorators). Phase 2 introduced new codegen patterns -- lambdas, iterator chains, inheritance field access, protocol impls, generators, and variadics -- that produce correct but non-idiomatic output. This milestone cleans up systematic quality issues before Phase 3 begins.

**Rationale:** Phase 1 had `milestone_codegen_quality` which fixed issues in the original codegen (unnecessary `mut`, redundant `format!` nesting, verbose string handling). The five Phase 2 milestones introduced new patterns with their own quality regressions. Fixing these now prevents the issues from compounding as Phase 3 adds stdlib and async features.

### Task 1: Remove redundant `.clone()` on Copy types and inside `format!`

`nums.iter().min().unwrap().clone()` -- `.clone()` on `i64` is a no-op since `i64` is `Copy`. Similarly, `self.shape.name.clone()` inside `format!("{}", ...)` is unnecessary because `format!` only borrows.

**Approach:** For built-in functions (`min`, `max`), omit `.clone()` when the element type is `Copy`. For field access inside `format!` arguments, detect when the expression is consumed by a formatting macro and skip the `.clone()`.

**Where to fix:** `needs_clone_for_type()` and the `min`/`max` built-in emission in `crates/sifr_codegen/src/lib.rs`.

### Task 2: Inline lambda body in `filter()` instead of closure-within-closure

`filter(lambda x: x > 1, nums)` emits `.filter(|x| { let x = *x; (|x| x > 1)(x) })` -- an immediately-invoked inner closure wrapping the actual lambda. Should emit `.filter(|x| x > 1)` directly by inlining the lambda body into the filter closure.

**Approach:** In the `"filter"` codegen handler, when the function argument is a `HirExpr::Lambda`, emit the lambda body directly inside the filter closure instead of emitting the lambda as a separate closure and invoking it.

**Where to fix:** `"filter"` handler in `crates/sifr_codegen/src/lib.rs`.

### Task 3: Clean up filtered list comprehension deref pattern

`[x for x in nums if x > 2]` emits `.filter(|x| { let x = **x; ... }).map(|x| { let x = *x; ... })`. The double-deref rebinding pattern is correct but verbose.

**Approach:** Use `.iter().copied()` (for Copy types) or `.iter().cloned()` (for non-Copy types) on the iterator before `.filter()` and `.map()`, eliminating the need for manual deref rebinding inside closures.

**Where to fix:** `HirExpr::ListComp` handler in `crates/sifr_codegen/src/lib.rs`.

### Task 4: Fold string literals into `format!` format string

`format!("{}{}{}", "Hello, ".to_string(), name, "!".to_string())` -- literal string parts get `.to_string()` and separate `{}` placeholders. Should emit `format!("Hello, {}!", name)` by detecting `StringLiteral` parts and folding them directly into the format string.

**Approach:** In `collect_string_concat_parts` and the `BinOp` string concat handler, when a part is a `HirExpr::StringLiteral`, embed its value directly in the format string instead of emitting it as a separate argument with `{}`.

**Where to fix:** `collect_string_concat_parts` and the `BinOp` string concat emission in `crates/sifr_codegen/src/lib.rs`.

### Task 5: Prefix unused `with` variable with underscore

`with Timer("work") as t:` emits `let t = Timer::new(...)` which triggers a Rust unused-variable warning if `t` is not referenced in the body.

**Approach:** In the `HirStmt::With` codegen handler, scan the body statements for references to the variable name. If the variable is not used, emit `let _name` instead of `let name`.

**Where to fix:** `HirStmt::With` handler in `crates/sifr_codegen/src/lib.rs`.

### Task 6: Deduplicate protocol impl methods

When a class has `describe()` and implements `Printable`, the method body is emitted twice: once in the inherent `impl` and once in the `impl Printable for`. The trait impl should delegate to the inherent method instead of duplicating the body.

**Approach:** In `emit_protocol_impls`, instead of re-emitting the full method body, emit a delegation call: `fn describe(&self) -> String { ClassName::describe(self) }` (calling the inherent method).

**Where to fix:** `emit_protocol_impls` in `crates/sifr_codegen/src/lib.rs`.

### Task 7: Inline string literals in `println!`

`println!("{}", "doing work")` passes a string literal through format machinery unnecessarily. Should emit `println!("doing work")` directly.

**Approach:** In the `"print"` handler, when the single argument is a `HirExpr::StringLiteral`, emit `println!("literal")` directly instead of `println!("{}", "literal")`.

**Where to fix:** `"print"` handler in `crates/sifr_codegen/src/lib.rs`.

### Definition of Done (milestone_codegen_quality_v2)

- Redundant `.clone()` on Copy types (`i64`, `f64`, `bool`) is eliminated from `min`/`max`/field access
- `.clone()` inside `format!` arguments on `&self` fields is removed where `format!` only borrows
- `filter(lambda, list)` emits a single closure with inlined lambda body, no closure-within-closure
- Filtered list comprehensions use `.copied()`/`.cloned()` instead of manual deref rebinding
- String literal parts in concatenation are folded into the `format!` string: `format!("Hello, {}!", name)`
- `with` variables unused in the body are prefixed with `_`
- Protocol trait impls delegate to inherent methods instead of duplicating the body
- `println!("literal")` is emitted for string literal print arguments
- All existing 94 E2E pass tests still pass
- All 12 milestone demos produce correct output
- `cargo test` passes with no regressions
- Milestone demo in `./demos/codegen_output/main.sifr`

---

## Milestone ordering

Why the milestones within this phase are in this order:

- **milestone_protocols before milestone_inheritance:** Protocols define the trait contracts; inheritance extends them. Having protocols first means inherited classes can implement protocols immediately.
- **milestone_inheritance before milestone_generics:** Generics benefit from having the full class hierarchy (including inheritance) available, enabling generic constraints over class hierarchies.
- **milestone_generics includes comprehensions:** List/dict/set comprehensions are trivial iterator sugar, naturally belonging with iterators and closures.
- **milestone_generators after milestone_generics:** Generators need closures and iterators from generics; context managers need the full type system.
- **milestone_decorators after milestone_generators, before milestone_core_stdlib:** Decorators need closures (from generics) and are useful for stdlib design patterns. They don't need async. Moving them earlier enables `@decorator` patterns in stdlib.
- **milestone_codegen_quality_v2 before milestone_core_stdlib:** Codegen refinement is a natural Phase 2 cleanup step. Fixing codegen quality now means every future milestone builds on clean, idiomatic Rust output, preventing quality issues from compounding as Phase 3 adds stdlib and async features.
