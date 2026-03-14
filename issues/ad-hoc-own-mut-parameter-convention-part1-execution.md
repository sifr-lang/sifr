# Ad Hoc `own mut` Parameter Convention: Part 1 Execution

Status: complete
Started: 2026-03-14
Completed: 2026-03-14
Part: `orthogonal_parameter_convention_model_and_frontend_normalization`
PR: `#1130`

## Goal

Close the root cause that made `own mut` impossible to model cleanly by replacing the old three-state parameter convention surface with an orthogonal ownership/mutability representation and wiring that representation through the frontend.

This slice is intentionally limited to:

- parser acceptance for `own mut` and `mut own`,
- structural AST/type-system/HIR representation of both axes,
- normalization and duplicate-modifier regression coverage,
- and a runnable demo proving the new syntax survives end-to-end through the current pipeline without fallback behavior.

It does not attempt to finish:

- borrow-checking policy changes for all four parameter modes,
- mutable-owned Rust parameter emission (`mut x: T`),
- or the `1299` runtime closure work.

## Root Cause

The compiler treated parameter passing mode as a single three-state convention. That representation could express borrowed immutable, borrowed mutable, and owned immutable parameters, but it could not express an owned parameter whose local binding was also mutable.

The correct fix for this slice was to represent:

- ownership as `Borrow` vs `Own`, and
- local mutability as `Immutable` vs `Mutable`

directly in the AST and type-system model instead of adding more ad hoc branches on the old enum.

## Implementation

- Replaced the AST parameter convention enum with an orthogonal `AstParamConvention` struct plus explicit ownership and mutability axes.
- Replaced the type-system `ParamConvention` enum with an orthogonal struct plus helper constructors and predicates used by HIR/codegen.
- Extended parameter parsing to accept both `own mut x: T` and `mut own x: T`, normalize both spellings to the same structural convention, and reject duplicate modifiers deterministically.
- Updated HIR lowering to preserve explicit mutability while keeping the existing default ownership decision for unannotated parameters.
- Migrated convention-aware HIR/codegen call sites to derive their behavior from `is_owned`, `is_borrowed`, `is_shared_borrow`, and `is_mut_borrow`.
- Added parser regressions for normalization, duplicate-modifier diagnostics, and soft-keyword disambiguation.
- Added dedicated HIR lowering regressions that lock the four structural parameter modes and the `mut own` normalization path.
- Added `demos/ad_hoc_own_mut_parameter_convention_part1_demo.sifr` as the part-1 demo.

## Validation

Targeted validation:

- `cargo test -p sifr_python_parser parameter_`
- `cargo test -p sifr_hir own_mut_param_tests`
- `cargo run -q -p sifr -- run demos/ad_hoc_own_mut_parameter_convention_part1_demo.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

Demo output:

- `cargo run -q -p sifr -- run demos/ad_hoc_own_mut_parameter_convention_part1_demo.sifr` -> prints `3` then `3`

## Coverage Added

- `crates/sifr_python_parser/src/parser/tests.rs`
- `crates/sifr_hir/src/lower/own_mut_param_tests.rs`
- `demos/ad_hoc_own_mut_parameter_convention_part1_demo.sifr`

## Closure Decision

Part 1 is locally complete because the frontend now has a canonical structural model for ownership plus mutability, the parser accepts both supported spellings of `own mut`, duplicate-modifier regressions are locked, and the syntax survives through the current end-to-end pipeline with deterministic local validation.

The remaining work is intentionally deferred:

- part 2: borrow/exclusivity/escape semantics for all four parameter modes,
- part 3: emitted Rust/runtime semantics for owned mutable parameters plus `1299`,
- part 4: phase closure docs plus external review cycles.
