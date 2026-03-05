# Phase 25: Decimal Type and Exact Numeric Semantics

status: planned

## Objective
Add a first-class `Decimal` type with deterministic, exact base-10 arithmetic semantics for financial and precision-critical workloads.

## Depends on
- Phase 24 (`diagnostics_error_recovery_and_stability_contract`)

## Renumbering impact
- Inserting this phase after Phase 24 shifts existing phases `25..38` to `26..39`.

## Non-goals
- Replacing `float` semantics.
- Implicitly migrating existing numeric code to `Decimal`.
- Any lossy `float -> decimal` conversion path.

## Canonical Syntax and Numeric Policy

### Decimal syntax (final)
- Unsuffixed fractional literals remain `float`:
  - `1.5` -> `float`
- No decimal literal suffix syntax in this phase.
- Canonical decimal construction is explicit and exact:
  - `Decimal("1.5")` (string input)
- Type-annotated assignment must still use exact decimal construction:
  - `x: decimal = Decimal("1.5")` valid
  - `x: decimal = 1.5` invalid

### Constructor and conversion rules (final)
- Allowed constructors:
  - `Decimal("...")` (validated decimal string)
  - `Decimal(int_value)` (exact)
  - `Decimal(bigint_value)` (exact)
- Disallowed constructors/conversions:
  - `Decimal(float_value)` (disallowed)
  - `Decimal.from_float(...)` (disallowed)
  - Any implicit `float -> decimal` conversion (disallowed)

### Numeric mixing policy (final)
- `int + decimal` -> `decimal` (allowed, exact)
- `bigint + decimal` -> `decimal` (allowed, exact)
- Any arithmetic/comparison mixing `float` with `decimal` is rejected unless explicitly redesigned in a future phase.
- No fallback conversion paths.

### Decimal context defaults (final)
- Default precision: `28` significant digits
- Default rounding mode: `HALF_EVEN`
- Context overrides are explicit via decimal context APIs (no hidden global mutation semantics in user code).

### Runtime implementation (final)
- Backing implementation: `bigdecimal` crate.
- Any crate change requires ADR + benchmark + compatibility sign-off.

## Milestones

### milestone_25_1: Type-System, Parser, and HIR Integration
- Scope:
  - Add `Decimal` to core type enum and type rendering.
  - Add parsing/lowering for `Decimal("...")`, `Decimal(int)`, `Decimal(bigint)`.
  - Enforce constructor validity rules and mixed-numeric policy in type checking.
- Definition of done:
  - Decimal is first-class through parser -> HIR -> type checker -> codegen.
  - Invalid decimal construction or mixed float/decimal usage fails with stable diagnostics.

### milestone_25_2: Deterministic Arithmetic and Context Semantics
- Scope:
  - Implement decimal arithmetic/comparison using `bigdecimal`.
  - Implement default context (precision=28, rounding=HALF_EVEN).
  - Enforce panic-free invalid-operation handling in user paths.
- Definition of done:
  - Decimal arithmetic is deterministic across repeated runs.
  - No user-path data-dependent `unwrap`/`expect`/`panic!`.

### milestone_25_3: Conversion and Boundary Contracts
- Scope:
  - Implement explicit exact conversions:
    - `int <-> decimal`
    - `bigint <-> decimal`
    - `str <-> decimal`
  - Explicitly ban all `float -> decimal` paths in compiler and stdlib.
  - Define JSON/model boundary contract:
    - Decimal serializes as string by default to preserve precision.
- Definition of done:
  - Conversions are explicit, exact, deterministic, and test-covered.
  - Any float-to-decimal attempt fails with clear diagnostics.

### milestone_25_4: Decimal Diagnostics Contract
- Scope:
  - Add decimal-specific diagnostics with stable codes and precise spans.
  - Reserve decimal diagnostic range `E2501-E2599`.
  - Initial required codes:
    - `E2501` invalid decimal input literal/string
    - `E2502` invalid mixed numeric arithmetic (`float` with `decimal`)
    - `E2503` decimal context error
    - `E2504` invalid decimal conversion/construction
    - `E2505` decimal precision/rounding overflow condition
- Definition of done:
  - Decimal diagnostics are stable and regression-locked.

### milestone_25_5: Verification Corpus and Determinism Gates
- Scope:
  - Add decimal corpus covering:
    - exact string construction
    - int/bigint construction
    - rounding boundaries
    - conversion failures
    - repeated-run determinism
  - Add negative seeded cases for nondeterminism and forbidden float-conversion paths.
- Definition of done:
  - Decimal corpus is reproducible and version-controlled.
  - Determinism and safety gates are enforced locally and in CI.

## Quality Contract

### Entry criteria
- Phase 24 exit gate is satisfied and recorded.
- Decimal policy (construction, mixing, conversions, defaults) is approved.

### Milestone quality checks
- Local validation gates pass before merge.
- Generated Rust for decimal corpus compiles with `-D warnings`.
- No emitted `todo!`/`unimplemented!` in production decimal paths.
- No data-dependent emitted `.unwrap()`/`.expect()`/`panic!` in user runtime decimal paths.
- Determinism checks pass across repeated runs.
- Validation evidence is recorded in the phase checklist issue.

### Exit criteria
- All milestone DoDs are satisfied.
- Decimal semantics are deterministic, panic-safe, and fully contract-tested.
- Construction/mixing/conversion/context behavior is documented and enforced by tests.
- Decimal diagnostics are stable and regression-locked.
- Any waiver is explicit, time-bounded, owner-assigned, and issue-linked.

## Exit Gate
`Decimal` is first-class, deterministic, panic-safe, and production-ready across compiler, generated code, stdlib, and model/serialization boundaries with no float-to-decimal path.
