# milestone_error_safety: Error Class Enforcement & Exhaustiveness Checking

**Goal:** Make the sifr compiler enforce the error handling model defined in architecture.md contract #3. After this milestone, all `Result` error types must be classes extending `Error`, `str` is no longer a valid error type, and the compiler performs exhaustiveness checking on `except` arms — ensuring every error type from every fallible call in a `try` block is either handled by a specific `except` arm or covered by `except Error as e`.

This milestone is the **enabler** for all subsequent stdlib safety work. Once the compiler enforces these rules, stdlib functions can be migrated from panicking intrinsics to `Result`-returning functions with proper error classes, and user code is guaranteed to handle every failure path.

---

## Background

### Current State

The compiler currently supports `try`/`except` with auto-unwrap and `raise` as sugar for `return Err(...)`. However:

1. **`str` is used as the error type everywhere.** All existing E2E tests and demos use `Result[T, str]` with `except str as e`. There is no enforcement that error types must be classes.
2. **No exhaustiveness checking.** The compiler does not verify that all error types from fallible calls in a `try` block are covered by `except` arms. A developer can write `except IOError as e` and silently miss a `TOMLDecodeError` from another call in the same `try` block.
3. **The `Error` base class exists** (`is_error_class()` in `lower.rs` checks for `(Error)` base), but it's only used for codegen — not for enforcing that all error types extend it.
4. **No built-in error classes.** The stdlib intrinsics either panic (`.unwrap()`) or return `Result[T, str]`. There are no standard `IOError`, `ParseError`, `ValueError`, etc.

### Target State (architecture.md contract #3)

1. **`E` in `Result[T, E]` must be a class extending `Error`.** Using `str`, `int`, or any non-Error type is a compile-time error.
2. **`except Error as e`** is the catch-all — covers all error types.
3. **`except SpecificError as e`** triggers exhaustiveness checking — the compiler verifies every error type from every fallible call is covered.
4. **Mixing is allowed** — specific arms first, `except Error as e` covers the rest.
5. **Uncovered error types are a compile error** with a diagnostic pointing at the uncovered call.

---

## Work Items

### 1. Define Built-in Error Classes

Create a set of built-in error classes that the compiler recognizes. These are not defined in `.sifr` files — they are compiler built-ins, like `int`, `str`, `bool`.

```
Error                  # root — all errors extend this
├── IOError            # file I/O, network, filesystem
├── ParseError         # string-to-type conversions (int(s), float(s))
├── ValueError         # invalid argument values (out of range, invalid format)
├── DivisionError      # division by zero
├── KeyError           # dict operations (deferred — dict currently returns Option)
├── JSONDecodeError    # JSON parsing
├── TOMLDecodeError    # TOML parsing
└── RegexError         # invalid regex pattern
```

**Compiler changes:**
- Register these in the type system as built-in class types (similar to how `int`, `str`, `bool` are registered)
- Each built-in error class has a `message: str` field
- User-defined error classes (`class MyError(Error)`) continue to work as they do today

### 2. Enforce Error Type Constraint on Result

The type checker must verify that the `E` in `Result[T, E]` is a class extending `Error`.

**Compiler changes:**
- In the type checker / HIR lowering, when a `Result[T, E]` type annotation is encountered, verify that `E` resolves to a class type that extends `Error` (either a built-in error class or a user-defined `class Foo(Error)`)
- If `E` is `str`, `int`, or any non-Error type, emit a compile error:
  ```
  error[S0050]: invalid error type in Result
    --> file.sifr:4:30
     |
  4  | def foo() -> Result[int, str]:
     |                           ^^^ `str` is not a valid error type
     |
     = help: use a class extending Error, e.g. `Result[int, ValueError]`
  ```

### 3. Implement Exhaustiveness Checking on `except` Arms

The compiler must track which error types can arise from fallible calls inside a `try` block and verify that the `except` arms cover all of them.

**Compiler changes:**

**Step 3a: Collect error types from `try` body.**
- When lowering a `try` block, walk the body and collect the set of error types from every `Result`-returning call. For each call expression where the return type is `Result[T, E]`, add `E` to the set of possible error types for this `try` block.

**Step 3b: Check `except` arm coverage.**
- For each `except` arm, determine which error types it covers:
  - `except Error as e` — covers ALL error types (catch-all)
  - `except SpecificError as e` — covers exactly `SpecificError` and its subclasses
- After processing all `except` arms, check if any error types from Step 3a are uncovered.
- If uncovered types remain, emit a compile error listing each uncovered type and the call that produces it.

**Step 3c: Ordering validation.**
- If `except Error as e` appears before a more specific `except` arm, emit a warning: the specific arm is unreachable.

### 4. Update `raise` to Require Error Class Instances

Currently `raise "message"` works because `str` is a valid error type. After this milestone:

- `raise ErrorInstance` — valid (e.g., `raise ValueError("out of range")`)
- `raise "message"` — compile error:
  ```
  error[S0051]: raise requires an Error class instance
    --> file.sifr:5:11
     |
  5  |     raise "division by zero"
     |           ^^^^^^^^^^^^^^^^^^^ `str` is not an Error class
     |
     = help: use `raise DivisionError("division by zero")`
  ```

### 5. Migrate Existing E2E Tests and Demos

All existing tests use `Result[T, str]` and `except str as e`. These must be migrated to use proper error classes.

**Files to migrate:**
- `crates/sifr/tests/e2e/pass/result_basic.sifr`
- `crates/sifr/tests/e2e/pass/error_propagation.sifr`
- `crates/sifr/tests/e2e/pass/discard_result.sifr`
- `crates/sifr/tests/e2e/pass/custom_error.sifr` (already uses error class — verify)
- `crates/sifr/tests/e2e/fail/unused_result.sifr`
- `demos/milestone_error_handling_demo.sifr`
- All `audits/` files that use `Result[T, str]`

### 6. Add New E2E Tests for Error Safety

**Pass tests:**
- `error_exhaustive_catchall.sifr` — `except Error as e` catches all error types
- `error_exhaustive_specific.sifr` — all error types covered by specific `except` arms
- `error_exhaustive_mixed.sifr` — specific arms + `except Error as e` catch-all
- `error_builtin_classes.sifr` — built-in error classes (`IOError`, `ParseError`, etc.) work correctly
- `error_custom_class.sifr` — user-defined error class with fields
- `error_nested_try.sifr` — nested `try` blocks with independent exhaustiveness scopes

**Fail tests:**
- `error_str_not_allowed.sifr` — `Result[int, str]` is a compile error
- `error_unhandled_type.sifr` — missing `except` arm for an error type
- `error_raise_str.sifr` — `raise "message"` is a compile error
- `error_unreachable_except.sifr` — `except Error` before specific arm (warning)

### 7. Update Codegen for Error Class Matching

The current codegen emits a simple `match` on `Err(e)`. With multiple error types in a single `try` block, the codegen needs to emit a multi-arm match on the error enum.

**Current codegen (single error type):**
```rust
match (|| -> Result<(), String> {
    // body
    Ok(())
})() {
    Ok(()) => {}
    Err(e) => { /* handler */ }
}
```

**Target codegen (multiple error types):**
```rust
match (|| -> Result<(), AppError> {
    // body with calls returning different error types
    // compiler wraps each ? to convert specific errors into the try-block's error enum
    Ok(())
})() {
    Ok(()) => {}
    Err(AppError::IOError(e)) => { /* IOError handler */ }
    Err(AppError::TOMLDecodeError(e)) => { /* TOMLDecodeError handler */ }
    Err(e) => { /* catch-all handler */ }
}
```

The compiler needs to generate a local error enum when a `try` block contains calls that can fail with different error types. This enum wraps each possible error type and enables the `match` arms to dispatch correctly.

---

## Dependencies

- **Requires:** milestone_compiler_hardening (done) — error classes already work in user code
- **Enables:** all subsequent stdlib safety milestones — once the compiler enforces error classes and exhaustiveness, stdlib intrinsics can be migrated from `.unwrap()` panics to `Result[T, SpecificError]` returns, and the compiler guarantees user code handles every failure

## Definition of Done

- `Result[T, E]` where `E` is not a class extending `Error` is a compile-time error
- `raise "message"` (bare string) is a compile-time error
- `except Error as e` is recognized as a catch-all covering all error types
- `except SpecificError as e` without covering all error types from the `try` block is a compile-time error, with a diagnostic listing uncovered types
- Mixed `except` arms (specific + `Error` catch-all) work correctly
- Built-in error classes (`IOError`, `ParseError`, `ValueError`, `DivisionError`, etc.) are available without imports
- All existing E2E tests migrated from `Result[T, str]` to proper error classes
- New E2E pass/fail tests for exhaustiveness checking
- `cargo test` passes (zero regressions)
- Demo: `demos/milestone_error_safety_demo.sifr`
