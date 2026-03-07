# Phase 28: Decimal Types and Exact Numeric Semantics

status: planned

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

### Python Decimal compatibility profiles (final)
- Finance profile: Python `Decimal` semantics map to Sifr `decimal`, backed by Rust `rust_decimal::Decimal`.
- Arbitrary-precision profile: Python `Decimal` semantics map to Sifr `bigdecimal`, backed by Rust `bigdecimal::BigDecimal`.

### Constructor and conversion rules (final)
- Allowed `Decimal` constructors:
  - `Decimal("...")` (validated decimal string)
  - `Decimal(int_value)` (exact)
  - `Decimal(bigint_value)` (exact)
- Allowed `BigDecimal` constructors:
  - `BigDecimal("...")` (validated decimal string)
  - `BigDecimal(int_value)` (exact)
  - `BigDecimal(bigint_value)` (exact)
- Cross-decimal conversion:
  - `BigDecimal(decimal_value)` is explicit and exact.
  - `Decimal(bigdecimal_value)` is explicit and fallible (must be checked; no implicit narrowing).
- Disallowed constructors/conversions:
  - `Decimal(float_value)` (disallowed)
  - `BigDecimal(float_value)` (disallowed)
  - `Decimal.from_float(...)` (disallowed)
  - `BigDecimal.from_float(...)` (disallowed)
  - Any implicit `float -> decimal` conversion (disallowed)
  - Any implicit `float -> bigdecimal` conversion (disallowed)

### Numeric mixing policy (final)
- `int + decimal` -> `decimal` (allowed, exact)
- `bigint + decimal` -> `decimal` (allowed, exact)
- `int + bigdecimal` -> `bigdecimal` (allowed, exact)
- `bigint + bigdecimal` -> `bigdecimal` (allowed, exact)
- Any arithmetic/comparison mixing `decimal` with `bigdecimal` is rejected unless explicitly converted.
- Any arithmetic/comparison mixing `float` with `decimal` is rejected unless explicitly redesigned in a future phase.
- Any arithmetic/comparison mixing `float` with `bigdecimal` is rejected unless explicitly redesigned in a future phase.
- No fallback conversion paths.

### Decimal context defaults (final)
- `decimal` (`rust_decimal`) is fixed-precision by representation for financial workloads; no hidden global context mutation.
- `bigdecimal` (`bigdecimal`) is arbitrary precision with explicit context APIs.
- `bigdecimal` default context:
  - Default precision: `28` significant digits
  - Default rounding mode: `HALF_EVEN`
- Context overrides are explicit (no hidden global mutation semantics in user code).

### Runtime implementation (final)
- `decimal` maps to `rust_decimal::Decimal` (finance-oriented fixed precision).
- `bigdecimal` maps to `bigdecimal::BigDecimal` (arbitrary precision).
- Any crate change requires ADR + benchmark + compatibility sign-off.

## Milestones

### milestone_28_1: Type-System, Parser, and HIR Integration
- Scope:
  - Add `decimal` and `bigdecimal` to core type enum and type rendering.
  - Add parsing/lowering for `Decimal("...")`, `Decimal(int)`, `Decimal(bigint)`.
  - Add parsing/lowering for `BigDecimal("...")`, `BigDecimal(int)`, `BigDecimal(bigint)`.
  - Enforce constructor validity rules and mixed-numeric policy in type checking.
- Definition of done:
  - `decimal` and `bigdecimal` are first-class through parser -> HIR -> type checker -> codegen.
  - Invalid construction or forbidden mixed usage fails with stable diagnostics.

### milestone_28_2: Deterministic Arithmetic and Context Semantics
- Scope:
  - Implement `decimal` arithmetic/comparison using `rust_decimal` with deterministic behavior.
  - Implement `bigdecimal` arithmetic/comparison using `bigdecimal`.
  - Implement default `bigdecimal` context (precision=28, rounding=HALF_EVEN).
  - Enforce panic-free invalid-operation handling in user paths.
- Definition of done:
  - `decimal` and `bigdecimal` arithmetic are deterministic across repeated runs.
  - No user-path data-dependent `unwrap`/`expect`/`panic!`.

### milestone_28_3: Conversion and Boundary Contracts
- Scope:
  - Implement explicit exact conversions:
    - `int <-> decimal`
    - `bigint <-> decimal`
    - `str <-> decimal`
    - `int <-> bigdecimal`
    - `bigint <-> bigdecimal`
    - `str <-> bigdecimal`
  - Implement explicit cross-decimal conversions:
    - `decimal -> bigdecimal` (exact)
    - `bigdecimal -> decimal` (fallible, checked)
  - Explicitly ban all `float -> decimal` and `float -> bigdecimal` paths in compiler and stdlib.
  - Define JSON/model boundary contract:
    - `decimal` and `bigdecimal` serialize as string by default to preserve precision.
- Definition of done:
  - Conversions are explicit, exact, deterministic, and test-covered.
  - Any float-to-decimal attempt fails with clear diagnostics.
  - Any float-to-bigdecimal attempt fails with clear diagnostics.

### milestone_28_4: Decimal Diagnostics Contract
- Scope:
  - Add decimal-specific diagnostics with stable codes and precise spans for both types.
  - Reserve decimal diagnostic range `E2501-E2599`.
  - Initial required codes:
    - `E2501` invalid `Decimal` input literal/string
    - `E2502` invalid `BigDecimal` input literal/string
    - `E2503` invalid mixed numeric arithmetic (`float` with `decimal`/`bigdecimal`)
    - `E2504` invalid mixed numeric arithmetic (`decimal` with `bigdecimal` without explicit conversion)
    - `E2505` invalid decimal conversion/construction
    - `E2506` invalid bigdecimal conversion/construction
    - `E2507` decimal precision/overflow condition
    - `E2508` bigdecimal context error
- Definition of done:
  - Decimal diagnostics are stable and regression-locked.

### milestone_28_5: Verification Corpus and Determinism Gates
- Scope:
  - Add decimal corpus covering:
    - exact `Decimal` string construction
    - exact `BigDecimal` string construction
    - int/bigint construction for both types
    - rounding boundaries
    - `decimal <-> bigdecimal` cross-conversion pass/fail boundaries
    - conversion failures
    - repeated-run determinism
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
- Generated Rust for decimal corpus compiles with `-D warnings`.
- No emitted `todo!`/`unimplemented!` in production `decimal`/`bigdecimal` paths.
- No data-dependent emitted `.unwrap()`/`.expect()`/`panic!` in user runtime `decimal`/`bigdecimal` paths.
- Determinism checks pass across repeated runs.
- Validation evidence is recorded in the phase checklist issue.

### Exit criteria
- All milestone DoDs are satisfied.
- `decimal` and `bigdecimal` semantics are deterministic, panic-safe, and fully contract-tested.
- Construction/mixing/conversion/context behavior is documented and enforced by tests.
- Decimal diagnostics are stable and regression-locked.
- Any waiver is explicit, time-bounded, owner-assigned, and issue-linked.

## Exit Gate
`decimal` and `bigdecimal` are first-class, deterministic, panic-safe, and production-ready across compiler, generated code, stdlib, and model/serialization boundaries with no float-to-decimal or float-to-bigdecimal path.
Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
