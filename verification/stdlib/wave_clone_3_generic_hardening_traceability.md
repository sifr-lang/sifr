# wave_clone_3 Generic Hardening Traceability

Date: 2026-03-21
Phase: `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`
Wave: `wave_clone_3`

## Objective

Harden ownership planning and lowering for conservative generic/dynamic cases while
closing tuple-ownership gaps that caused unnecessary cloning in iterator surfaces.

## Validation Commands

- `cargo test -p sifr_type_system test_tuple_ownership_all_copy_is_copy`
- `cargo test -p sifr_type_system test_tuple_ownership_with_move_is_move`
- `cargo test -p sifr_codegen -- helpers::tests`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_cloning.sifr`
- `cargo run -q -p sifr -- run demos/generic_cloning/main.sifr`
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/generic_cloning.sifr`

## Emitted Rust Evidence

Observed from `generic_cloning.sifr`:

- copy tuple iteration now emits copy-oriented traversal:
  - `for pair in pairs.iter().copied() { ... }`
- dynamic/`Any` list iteration remains conservative and borrow-based:
  - `for _v in anys.iter() { ... }`
  - no `.cloned()` / `.copied()` is emitted for `Vec<Box<dyn Any>>`.

## Root-Cause Closure Notes

- tuple ownership now reflects element ownership (`Copy` only when all tuple members are `Copy`)
- planner no longer forces copy/clone yield behavior from element hints when source
  iteration metadata cannot prove element ownership
- tuple-literal value-category classification is hardened to only classify as
  reusable `Place` when tuple ownership is `Copy` and all tuple elements are reusable places
