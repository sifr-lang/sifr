# Error Safety (Compiler Infrastructure)

**Why first:** Every subsequent phase depends on proper error types. You cannot make stdlib safe without error classes. You cannot do exhaustiveness checking without the compiler infrastructure. This is the [milestone_error_safety](../issues/archive/milestone_error_safety.md) issue, which is the enabler for everything.

---

## milestone_error_safety: Error Class Enforcement and Exhaustiveness Checking

status: done

**Goal:** Make the sifr compiler enforce the error handling model defined in architecture.md contract #3. After this milestone, all `Result` error types must be classes extending `Error`, `str` is no longer a valid error type, and the compiler performs exhaustiveness checking on `except` arms.

As defined in [plans/issues/archive/milestone_error_safety.md](../issues/archive/milestone_error_safety.md):

### 1. Define Built-in Error Classes

Create a set of built-in error classes that the compiler recognizes. These are not defined in `.sifr` files — they are compiler built-ins, like `int`, `str`, `bool`:

- `Error` (base class)
- `IOError`
- `ParseError`
- `ValueError`
- `DivisionError`
- `KeyError`
- `JSONDecodeError`
- `TOMLDecodeError`
- `RegexError`

Each built-in error class has a `message: str` field. User-defined error classes (`class MyError(Error)`) continue to work as they do today.

> **Note:** In `milestone_error_subclasses` (Phase 09), error types are expanded into a subclass hierarchy (e.g., `FileNotFoundError` extends `IOError`) with compile-time exhaustiveness checking. All errors keep `message: str`; some gain additional structured fields (`line`, `column`, `detail`). See [09_stdlib_safety_remediation.md](09_stdlib_safety_remediation.md).

### 2. Enforce `E` in `Result[T, E]` Must Extend `Error`

The type checker must verify that the `E` in `Result[T, E]` is a class extending `Error`. `Result[T, str]` becomes a compile error.

### 3. Implement Exhaustiveness Checking on `except` Arms

The compiler must track which error types can arise from fallible calls inside a `try` block and verify that the `except` arms cover all of them.

- Collect error types from `try` body (walk all calls, collect `E` from each `Result[T, E]`)
- Verify `except` arms cover all collected error types
- `except Error as e` is the catch-all — covers all error types
- Uncovered error types are a compile error with diagnostic listing uncovered types

### 4. Update `raise` to Require Error Class Instances

`raise "message"` becomes a compile error. `raise` must take an instance of a class extending `Error`.

### 5. Migrate All Existing E2E Tests

Migrate all existing tests from `Result[T, str]` to proper error classes.

### 6. Update Codegen for Multi-Error-Type `try` Blocks

Generate a local error enum when a `try` block contains calls that can fail with different error types. This enum wraps each possible error type and enables the `match` arms to dispatch correctly.

### Definition of Done (milestone_error_safety)

- Built-in error classes (`Error`, `IOError`, `ParseError`, `ValueError`, `DivisionError`, `KeyError`, `JSONDecodeError`, `TOMLDecodeError`, `RegexError`) are registered in the type system as compiler built-ins, available without imports
- `E` in `Result[T, E]` must extend `Error` — `Result[T, str]` is a compile error
- `except SpecificError as e` without covering all error types from the `try` block is a compile-time error
- Mixed `except` arms (specific + `Error` catch-all) work correctly
- `raise "message"` is a compile error — `raise` requires Error class instances
- All existing E2E tests migrated from `Result[T, str]` to proper error classes
- Multi-error-type `try` blocks generate local error enums in codegen
- New E2E pass/fail tests for exhaustiveness checking
- Milestone demo in `./demos/error_safety/main.sifr`

---

## milestone_error_safety_stdlib_types: Module-Specific Error Types

status: done

**Goal:** Define and export module-specific error types from stdlib `.sifr` files. These are distinct from the core compiler built-in error types — they require an import to use.

**Depends on:** milestone_error_safety (compiler infrastructure must enforce error classes and exhaustiveness)

### Module-Specific Error Types

- `StatisticsError` for `sifr.statistics` — defined and exported from `statistics.sifr`
- `CycleError` for `sifr.graphlib` — defined and exported from `graphlib.sifr`

### Error Type Export Pipeline

Validate the error type export pipeline from stdlib `.sifr` files. This was explicitly deferred in Phase 07. The pipeline must support:

- Defining a class extending `Error` in a `.sifr` file
- Exporting it so user code can `from sifr.statistics import StatisticsError`
- Using it in `except` arms with exhaustiveness checking

### Definition of Done (milestone_error_safety_stdlib_types)

- `StatisticsError` defined in `statistics.sifr` and importable by user code
- `CycleError` defined in `graphlib.sifr` and importable by user code
- Error type export pipeline validated end-to-end
- E2E tests proving error types can be imported and caught by user code
- E2E test: `try` block with both a built-in error type and a module-specific error type, exhaustiveness checking works correctly

---

## Error Taxonomy Policy

Phase 08 defines error types at two levels:

- **Core compiler built-in error types** (`Error`, `IOError`, `ParseError`, `ValueError`, `DivisionError`, `KeyError`, `JSONDecodeError`, `TOMLDecodeError`, `RegexError`) are registered in the compiler's type system like `int`, `str`, `bool`. Available without imports.
- **Module-specific error types** (`StatisticsError`, `CycleError`, and future module-level errors) are defined and exported from their respective stdlib `.sifr` files (e.g., `StatisticsError` from `sifr.statistics`). These require an import to use.
- **Generic error types** (`ParseError`, `ValueError`, `IOError`) are used for operations where the failure mode is common across domains (e.g., base64 decoding, hex parsing, string-to-number conversion) and fine-grained distinction adds no value.
- **Rule of thumb:** if two different fallible calls in the same `try` block could fail with the same error type but the user would want to handle them differently, they need distinct error types.

---

## Milestone Ordering

- **milestone_error_safety before milestone_error_safety_stdlib_types:** The compiler infrastructure (error class enforcement, exhaustiveness checking, codegen for multi-error-type `try` blocks) must exist before module-specific error types can be defined and tested.
- **milestone_error_safety before Phase 09 (Stdlib Safety Remediation):** You cannot make intrinsics return `Result[T, IOError]` if the compiler doesn't enforce that `IOError` extends `Error` and all existing tests use `Result[T, str]`.
