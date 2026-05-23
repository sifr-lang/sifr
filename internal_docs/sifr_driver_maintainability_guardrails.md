# sifr_driver Maintainability Guardrails

This document defines the anti-regrowth guardrails for the Phase 31 `sifr_driver` decomposition work.

## File Boundaries

The canonical driver layout is rooted under `crates/sifr_driver/src/`:

- `lib.rs` stays a crate entrypoint and re-export surface only
- `diagnostics.rs`
- `stdlib/`
- `frontend/`
- `project/`
- `build/`
- `test_runner/`
- `tests/`

Monolithic files are explicitly banned:

- `crates/sifr_driver/src/stdlib.rs`
- `crates/sifr_driver/src/frontend.rs`
- `crates/sifr_driver/src/project.rs`
- `crates/sifr_driver/src/build.rs`
- `crates/sifr_driver/src/test_runner.rs`

Guardrail enforcement commands:

- `python3 scripts/check_sifr_driver_maintainability_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py`

`run_all_tests.sh` runs this check before the broader validation suite.

## Module Placement Guide

Use these placement rules when changing the driver:

- `diagnostics.rs`: compile errors, diagnostic shaping, panic-boundary conversion
- `stdlib/`: embedded stdlib registry, intrinsic mapping, stdlib cache/bootstrap
- `frontend/`: single-file parse/lower/check/compile entrypoints and frontend lowering helpers
- `project/`: multi-module export collection, dependency ordering, discovery, project frontend analysis
- `build/`: rooted entrypoint planning, project code generation assembly, workspace allocation, output materialization
- `test_runner/`: test-runner-specific orchestration, generated test lib composition, test-runner Cargo manifest generation, test execution
- `tests/`: focused crate-level regression suites grouped by concern

## Review Checklist

- [ ] New driver logic is placed in the correct module subtree.
- [ ] `crates/sifr_driver/src/lib.rs` stays crate wiring plus re-exports only.
- [ ] Test coverage lives in focused `crates/sifr_driver/src/tests/` modules or beside the extracted concern.
- [ ] Shared helpers were extracted only when they represent a real boundary, not a dumping ground.
- [ ] Unified file-size guardrail passes locally (`python3 scripts/check_file_size_guardrails.py`).
- [ ] Guardrail script still passes locally (`python3 scripts/check_sifr_driver_maintainability_guardrails.py`).
