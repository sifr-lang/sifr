# Phase 28: Decimal Types and Exact Numeric Semantics

status: completed

## Objective
Add first-class `decimal` and `bigdecimal` types with deterministic, exact base-10 arithmetic semantics for financial and precision-critical workloads.

## Depends on
- Phase 27 (`diagnostics_error_recovery_and_stability_contract`)
- Phase 27 (`runtime_safe_codegen_semantics`)

## Non-goals
- Replacing `float` semantics.
- Implicitly migrating existing numeric code to `decimal` or `bigdecimal`.
- Any lossy `float -> decimal` or `float -> bigdecimal` conversion path.

## Canonical Syntax and Numeric Policy

### Decimal/bigdecimal syntax (final)
- Unsuffixed fractional literals remain `float`:
  - `1.5` -> `float`
- No decimal literal suffix syntax in this phase.
- Canonical construction is explicit and exact:
  - `Decimal("1.5")` (string input)
  - `BigDecimal("1.5")` (string input)
- Type-annotated assignment must still use exact construction:
  - `x: decimal = Decimal("1.5")` valid
  - `y: bigdecimal = BigDecimal("1.5")` valid
  - `x: decimal = 1.5` invalid
  - `y: bigdecimal = 1.5` invalid

### Type/constructor naming contract (final)
- Type annotations use lowercase keywords: `decimal`, `bigdecimal`.
- Canonical constructors are built-in callables: `Decimal(...)`, `BigDecimal(...)`.
- Parser behavior is unchanged: both constructors parse as ordinary call expressions.
- Constructor argument validation and invalid-decimal diagnostics are semantic checks in HIR/type checking, not parser syntax errors.

### Python Decimal compatibility profiles (final)
- Finance profile: Python `Decimal` semantics map to Sifr `decimal`, backed by Rust `rust_decimal::Decimal`.
- Arbitrary-precision profile: Python `Decimal` semantics map to Sifr `bigdecimal`, backed by Rust `bigdecimal::BigDecimal`.

### Constructor and conversion rules (final)
- Allowed `Decimal` constructors:
  - `Decimal("...")` (validated decimal string)
  - `Decimal(int_value)` (exact)
  - `Decimal(int_value)` (exact, including values beyond fixed-width ranges)
- Allowed `BigDecimal` constructors:
  - `BigDecimal("...")` (validated decimal string)
  - `BigDecimal(int_value)` (exact)
  - `BigDecimal(int_value)` (exact, including values beyond fixed-width ranges)
- Cross-decimal conversion:
  - `BigDecimal(decimal_value)` is explicit and exact, returning `bigdecimal`.
  - `Decimal(bigdecimal_value)` is explicit and fallible, returning `Result[decimal, DecimalConversionError]` (must be checked; no implicit narrowing).
- Integer-target conversion semantics (Python parity):
  - `int(decimal_value)` and `int(bigdecimal_value)` are explicit and truncate toward zero (Python-compatible), returning `Result[int, DecimalConversionError]` for out-of-range or invalid values.
  - `int(decimal_value)` and `int(bigdecimal_value)` are explicit and truncate toward zero (Python-compatible), with no implicit rounding.
  - Truncating conversions are intentionally explicit; there are no implicit lossy integer conversions.
- Disallowed constructors/conversions:
  - `Decimal(float_value)` (disallowed)
  - `BigDecimal(float_value)` (disallowed)
  - `Decimal.from_float(...)` (disallowed)
  - `BigDecimal.from_float(...)` (disallowed)
  - Any implicit `float -> decimal` conversion (disallowed)
  - Any implicit `float -> bigdecimal` conversion (disallowed)

### DecimalConversionError contract (final)
- `DecimalConversionError` is a required built-in error type for fallible decimal conversions.
- Required failure categories:
  - out-of-range target representation (for example, conversion result cannot fit target `int`/`decimal`)
  - inexact narrowing (for example, `bigdecimal -> decimal` cannot be represented exactly)
  - invalid value conversion (for example, unsupported non-finite values if encountered at boundaries)
- Diagnostics must preserve stable codes and messages for each category.

### Numeric mixing policy (final)
- `int + decimal` -> `decimal` (allowed, exact)
- `int + bigdecimal` -> `bigdecimal` (allowed, exact)
- Any arithmetic/comparison mixing `decimal` with `bigdecimal` is rejected unless explicitly converted.
- Any arithmetic/comparison mixing `float` with `decimal` is rejected unless explicitly redesigned in a future phase.
- Any arithmetic/comparison mixing `float` with `bigdecimal` is rejected unless explicitly redesigned in a future phase.
- No fallback conversion paths.

### Operator coverage (final)
- Arithmetic operators supported for `decimal` and `bigdecimal`: `+`, `-`, `*`, `/`, `//`, `%`, `**`.
- Unary operators supported for both types: unary `+`, unary `-`.
- Comparison operators supported for both types: `==`, `!=`, `<`, `<=`, `>`, `>=`.
- Operator semantics must enforce the explicit conversion/mixing policy above (no hidden coercions).
- `//` follows Python floor-division semantics (toward negative infinity), not truncation toward zero.
  - Example parity rule: `Decimal("-1.9") // Decimal("1") == Decimal("-2")`.
- Operator protocol hooks for implementation alignment:
  - Binary arithmetic dunder mapping: `__add__`, `__sub__`, `__mul__`, `__truediv__`, `__floordiv__`, `__mod__`, `__pow__`.
  - Comparison dunder mapping: `__eq__` and ordering via `__lt__` contract.

### Stdlib/API surface (final)
- Phase 28 scope explicitly includes user-facing decimal APIs beyond core operators.
- Required `decimal` and `bigdecimal` methods (or equivalent built-ins) in this phase:
  - `quantize(...)`
  - `sqrt(...)`
  - `round(...)`
  - `abs(...)`
  - `is_zero()`
  - `is_finite()`
  - stable string formatting for deterministic serialization/display
- API behavior must be deterministic and must honor explicit context/rounding policy.
- Any deferred API for Python `decimal` parity must be listed explicitly in the phase checklist issue with owner and follow-up phase/issue link.

### Decimal context defaults (final)
- `decimal` (`rust_decimal`) is fixed-precision by representation for financial workloads; no hidden global context mutation.
- `bigdecimal` (`bigdecimal`) is arbitrary precision with explicit context APIs.
- `bigdecimal` default context:
  - Default precision: `28` significant digits
  - Default rounding mode: `HALF_EVEN`
- Context overrides are explicit (no hidden global mutation semantics in user code).

### Ownership and parameter conventions (final)
- `decimal` is `Copy` in the Sifr ownership model.
- `bigdecimal` is `Move` in the Sifr ownership model.
- Default parameter conventions follow ownership rules:
  - `decimal` defaults to by-value (`own`) parameters.
  - `bigdecimal` defaults to borrowed parameters unless explicitly marked `own`.

### Runtime implementation (final)
- `decimal` maps to `rust_decimal::Decimal` (finance-oriented fixed precision).
- `bigdecimal` maps to `bigdecimal::BigDecimal` (arbitrary precision).
- Codegen must emit required imports/usages for both runtime types when referenced.
- Codegen must surface `rust_decimal` and `bigdecimal` through explicit `required_crates` metadata so Cargo dependency generation stays deterministic.
- Any crate change requires ADR + benchmark + compatibility sign-off.

## Milestones

### milestone_28_1: Type-System, Parser, and HIR Integration
- Scope:
  - Keep parser grammar unchanged; use existing generic call-expression parsing for `Decimal(...)` and `BigDecimal(...)`.
  - Add `decimal` and `bigdecimal` to core type enum and type rendering.
  - Define rendering contract:
    - Source/display names: `decimal`, `bigdecimal`
    - Rust type names: `Decimal`, `BigDecimal`
  - Add parsing/lowering for `Decimal("...")` and `Decimal(int)`.
  - Add parsing/lowering for `BigDecimal("...")` and `BigDecimal(int)`.
  - Add built-in call lowering/type-check paths for `Decimal(...)` and `BigDecimal(...)` (constructor arity/type validation + diagnostics).
  - Enforce constructor validity rules and mixed-numeric policy in type checking.
- Definition of done:
  - `decimal` and `bigdecimal` are first-class through parser -> HIR -> type checker -> codegen.
  - Ownership behavior is defined and enforced (`decimal`=`Copy`, `bigdecimal`=`Move`).
  - Invalid construction or forbidden mixed usage fails with stable diagnostics.

### milestone_28_2: Deterministic Arithmetic and Context Semantics
- Scope:
  - Implement `decimal` arithmetic/comparison using `rust_decimal` with deterministic behavior.
  - Implement `bigdecimal` arithmetic/comparison using `bigdecimal`.
  - Implement the full operator coverage contract (`+`, `-`, `*`, `/`, `//`, `%`, `**`, unary ops, comparisons) for both types.
  - Implement required decimal API surface in this phase (`quantize`, `sqrt`, `round`, `abs`, `is_zero`, `is_finite`, deterministic formatting).
  - Implement default `bigdecimal` context (precision=28, rounding=HALF_EVEN).
  - Enforce panic-free invalid-operation handling in user paths.
- Definition of done:
  - `decimal` and `bigdecimal` arithmetic are deterministic across repeated runs.
  - Required decimal API surface is available and contract-tested for both types.
  - No user-path data-dependent `unwrap`/`expect`/`panic!`.

### milestone_28_3: Conversion and Boundary Contracts
- Scope:
  - Implement explicit conversions:
    - `int <-> decimal`
    - `int <-> decimal`
    - `str <-> decimal`
    - `int <-> bigdecimal`
    - `int <-> bigdecimal`
    - `str <-> bigdecimal`
  - Enforce Python-compatible integer-target conversion behavior:
    - `int(decimal|bigdecimal)` truncates toward zero and is fallible on range/invalid target values.
    - `int(decimal|bigdecimal)` truncates toward zero and is explicit.
  - Implement explicit cross-decimal conversions:
    - `decimal -> bigdecimal` via `BigDecimal(decimal_value)` (exact)
    - `bigdecimal -> decimal` via `Decimal(bigdecimal_value)` (fallible `Result`, checked)
  - Explicitly ban all `float -> decimal` and `float -> bigdecimal` paths in compiler and stdlib.
  - Define JSON/model boundary contract:
    - `decimal` and `bigdecimal` serialize as string by default to preserve precision.
- Definition of done:
  - Conversions are explicit, deterministic, and test-covered, with Python-compatible truncation/floor semantics where specified.
  - Any float-to-decimal attempt fails with clear diagnostics.
  - Any float-to-bigdecimal attempt fails with clear diagnostics.

### milestone_28_4: Decimal Diagnostics Contract
- Scope:
  - Add decimal-specific diagnostics with stable codes and precise spans for both types.
  - Reserve decimal diagnostic range `SIFR-DECIMAL-0001` through `SIFR-DECIMAL-0099`.
  - Initial required codes:
    - `SIFR-DECIMAL-0001` invalid `Decimal` input literal/string
    - `SIFR-DECIMAL-0002` invalid `BigDecimal` input literal/string
    - `SIFR-DECIMAL-0003` invalid mixed numeric arithmetic (`float` with `decimal`/`bigdecimal`)
    - `SIFR-DECIMAL-0004` invalid mixed numeric arithmetic (`decimal` with `bigdecimal` without explicit conversion)
    - `SIFR-DECIMAL-0005` invalid decimal conversion/construction
    - `SIFR-DECIMAL-0006` invalid bigdecimal conversion/construction
    - `SIFR-DECIMAL-0007` decimal precision/overflow condition
    - `SIFR-DECIMAL-0008` bigdecimal context error
- Definition of done:
  - Decimal diagnostics are stable and regression-locked.

### milestone_28_5: Verification Corpus and Determinism Gates
- Scope:
  - Add decimal corpus in:
    - `crates/sifr/tests/e2e/pass`
    - `crates/sifr/tests/e2e/fail`
  - Corpus coverage:
    - exact `Decimal` string construction
    - exact `BigDecimal` string construction
    - exact `int` construction for both types
    - `int(decimal|bigdecimal)` truncation-toward-zero behavior, including negative values
    - rounding boundaries
    - `//` floor-division behavior boundaries (including negative operands)
    - `quantize` and `sqrt` behavior boundaries (success/failure/rounding context)
    - API determinism for `round`, `abs`, `is_zero`, `is_finite`, and formatting
    - `decimal <-> bigdecimal` cross-conversion pass/fail boundaries
    - conversion failures
    - repeated-run determinism
  - Add/update milestone demo:
    - `demos/<milestone_demo>.sifr` exercises both `decimal` and `bigdecimal` end-to-end.
  - Add negative seeded cases for nondeterminism, forbidden mixed-type arithmetic, and forbidden float-conversion paths.
- Definition of done:
  - Decimal corpus is reproducible and version-controlled.
  - Determinism and safety gates are enforced locally and in CI.

## Quality Contract

### Entry criteria
- Phase 27 exit gate is satisfied and recorded.
- Decimal policy (`decimal` + `bigdecimal` construction, mixing, conversions, defaults) is approved.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.

### Milestone quality checks
- Local validation gates pass before merge.
- Full local suite passes:
  - `/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`
- Generated Rust for decimal corpus compiles with `-D warnings`.
- No emitted `todo!`/`unimplemented!` in production `decimal`/`bigdecimal` paths.
- No data-dependent emitted `.unwrap()`/`.expect()`/`panic!` in user runtime `decimal`/`bigdecimal` paths.
- Determinism checks pass across repeated runs.
- Milestone demo command runs successfully:
  - `cargo run -q -p sifr -- run demos/<milestone_demo>.sifr`
- Validation evidence is recorded in the phase checklist issue.

### Exit criteria
- All milestone DoDs are satisfied.
- `decimal` and `bigdecimal` semantics are deterministic, panic-safe, and fully contract-tested.
- Construction/mixing/conversion/context behavior is documented and enforced by tests.
- Required decimal API surface is implemented and regression-locked.
- Decimal diagnostics are stable and regression-locked.
- Any waiver is explicit, time-bounded, owner-assigned, and issue-linked.

## Exit Gate
`decimal` and `bigdecimal` are first-class, deterministic, panic-safe, and production-ready across compiler, generated code, stdlib, and model/serialization boundaries with no float-to-decimal or float-to-bigdecimal path.
Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
