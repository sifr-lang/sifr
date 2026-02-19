# PRD: milestone_integer_safety

## Goal

Resolve the integer overflow contradiction with Sifr's "if it compiles, it works" guarantee. `int` maps to `i64` — overflow panics in debug mode and wraps silently in release mode. Both behaviors violate the safety promise. This milestone introduces a `bigint` type for arbitrary-precision arithmetic (matching Python's `int` behavior) and adds compiler diagnostics for potential overflow in `int` operations.

## Scope

### `bigint` type — arbitrary-precision integers

```python
x: bigint = 10 ** 100
y: bigint = factorial(1000)
```

- `bigint` maps to Rust's `num_bigint::BigInt` crate
- Supports all arithmetic operators: `+`, `-`, `*`, `/`, `//`, `%`, `**`
- Supports comparison operators: `==`, `!=`, `<`, `>`, `<=`, `>=`
- `bigint` is `Eq`, `Hash`, `Clone`, `Debug`, `Comparable` — usable as dict keys, in sets, sorted collections
- `bigint` literals: any integer literal assigned to `bigint` emits `BigInt::from(...)`
- Conversion: `int(b)` converts `bigint` to `int`, returns `Result[int, OverflowError]`
- Conversion: `bigint(n)` converts `int` to `bigint` (always succeeds)
- `bigint` is NOT `Copy` — heap-allocated, follows move semantics

### `int` stays as `i64` with compiler diagnostics

- `int` remains `i64` for performance
- The compiler emits a warning when `int` arithmetic could overflow at runtime
- `int` overflow behavior is unchanged: panic in debug, wrap in release

### Type system integration

- `bigint` is a new `Type::BigInt` variant
- `int` and `bigint` are NOT implicitly convertible
- Mixed arithmetic (`int + bigint`) is a compile error
- Mixed comparison is also a compile error

## Architecture

### Type System
- Add `Type::BigInt` to `sifr_type_system/src/types.rs`
- `BigInt` is `Move` (heap-allocated)
- `BigInt` is hashable, comparable

### HIR
- Recognize `bigint` as a type annotation
- `bigint(n)` call → `HirExpr::Call { func: "bigint", ... }`
- `int(b)` call with bigint arg → returns `Result[int, OverflowError]`

### Codegen
- `bigint` type → `num_bigint::BigInt`
- `bigint` literal → `num_bigint::BigInt::from(value_i64)`
- `bigint(n)` → `num_bigint::BigInt::from(n)`
- `int(b)` with bigint → `i64::try_from(&b).map_err(|_| OverflowError { message: "bigint value out of range for int".to_string() })`
- Arithmetic on bigint → standard Rust operators (num_bigint implements them)
- Add `num-bigint = "0.4"` to Cargo.toml when bigint is used

## Test Plan

- `bigint_basic.sifr` — basic bigint creation and print
- `bigint_arithmetic.sifr` — all arithmetic operators
- `bigint_comparison.sifr` — comparison operators
- `bigint_to_int.sifr` — int(bigint) conversion with Result
- `int_to_bigint.sifr` — bigint(int) conversion
- `bigint_as_dict_key.sifr` — bigint as dict key
- `bigint_factorial.sifr` — computing large factorial

## Definition of Done

- `bigint` type works end-to-end
- `bigint` literals compile correctly
- `int(bigint_val)` returns `Result[int, OverflowError]`
- `bigint(int_val)` always succeeds
- Mixed `int`/`bigint` arithmetic is a compile error
- All existing E2E tests still pass
- Demo: `demos/milestone_integer_safety_demo.sifr`
