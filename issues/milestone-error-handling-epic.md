# milestone_error_handling: Error Handling

## Product Requirements

### Objective

Provide safe error handling that maps to Rust's `Result`/`Option` types rather than Python's exception model. Errors are values, not exceptions. All fallible operations return `Result` or `Option`, and the compiler enforces handling.

### Scope

#### Features In

1. `Result[T, E]` type -> `Result<T, E>` in Rust
2. `Option[T]` type -> sugar for `T | None`, maps to `Option<T>`
3. `?` operator for early return on error
4. `try`/`except` syntax -> pattern matching on `Result`
5. `raise` -> `Err()` in Result-returning functions
6. `assert` statement -> `assert!()` / `panic!()`
7. Custom error types: `class AppError(Error)` (special-cased, not general inheritance)
8. Fallible built-ins: `int(str)`, `float(str)` -> `Result[T, ParseError]`
9. Infallible conversions: `int(float)`, `float(int)`, `str(x)`, `bool(x)`
10. `#[must_use]` enforcement for Result values
11. `let _ = expr` for explicit discard

#### Features Out

| Feature | Reason |
|---------|--------|
| Division by zero Result | Deferred -- complex, needs static analysis |
| `input()` built-in | Deferred to milestone_io |
| `try`/`except`/`finally` | Deferred -- needs Drop trait |
| Match guards | Deferred to milestone_protocols |
| Struct destructuring in match | Deferred to milestone_protocols |

## Solution Design

### Architecture

```
sifr_type_system  (Result[T,E], Option[T] types, must_use checking)
       ↓
sifr_hir          (new HIR nodes for try/except, raise, assert, ?)
       ↓
sifr_codegen      (Result/Option codegen, match on errors, assert!)
       ↓
sifr (tests)      (E2E pass/fail tests)
```

### Task Breakdown

**Task 1: Result/Option Types & Assert**
- Add `Type::Result(T, E)` and recognize `Option[T]` as `T | None`
- Add `HirStmt::Assert { test, msg }` and codegen
- Add `HirExpr::QuestionMark { expr, ty }` for `?` operator
- Add `HirStmt::Raise { value }` for `raise` -> `Err()`

**Task 2: Error Types & Try/Except**
- `class AppError(Error)` special-cased error type declarations
- `try`/`except` lowering to match on Result variants
- Exhaustiveness checking for except arms
- Variable binding in except arms (`except ValueError as e`)

**Task 3: Fallible/Infallible Built-ins**
- `int(str)` -> `Result[int, str]`, `float(str)` -> `Result[float, str]`
- `int(float)` -> `int` (truncate), `float(int)` -> `float` (widen)
- `str(x)` for any type, `bool(x)` for any type
- `#[must_use]` enforcement for Result values
- `let _ = expr` explicit discard

**Task 4: E2E Tests & Demo**
- Pass tests: result_basic, try_except, error_propagation, infallible_conversions, assert_basic
- Fail tests: unhandled_error, unused_result_error
- Regression tests
- Milestone demo

### Testing Strategy

| Test | Layer | Check |
|------|-------|-------|
| result_basic | E2E pass | Result type, ? operator, raise |
| try_except | E2E pass | try/except pattern matching |
| error_propagation | E2E pass | ? operator propagation |
| infallible_conversions | E2E pass | int(float), float(int), str(x) |
| assert_basic | E2E pass | assert statement |
| unhandled_error | E2E fail | Missing error handling |
| unused_result_error | E2E fail | Unused Result value |
