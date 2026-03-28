# Ad Hoc `own mut` Parameter Convention: Part 3 Execution

Status: complete locally
Started: 2026-03-14
Completed: 2026-03-14
Part: `codegen_and_runtime_semantics_for_owned_mutable_parameters`
PR: `#1133`

## Goal

Close the remaining runtime/codegen gap for `own mut` by making emitted Rust distinguish all four parameter modes canonically, proving a direct `1299`-style consume-mutate-return workflow, and removing the old mutable-shadow fallback that had been masking a real semantic bug.

This slice is intentionally limited to:

- emitting canonical Rust `mut x: T` for owned mutable parameters,
- keeping borrowed mutable parameters on the existing `&mut T` path,
- removing redundant owned-parameter mutable shadows from generated Rust,
- adding direct `1299`-style runnable coverage,
- and fixing the missing mutating-method semantic check for immutable parameters that the old codegen shadowing had hidden.

It does not attempt to finish:

- the final phase-closure review/documentation loop.

## Root Cause

After part 2, the compiler could parse and type-check `own mut`, but the codegen/runtime surface still had two coupled problems:

- owned mutable parameters were lowered as immutable Rust params plus `let mut x = x;` shadows instead of canonical `mut x: T`,
- and mutating method calls on immutable parameters were not rejected during HIR lowering, so the old shadowing path accidentally let some invalid sources compile through to Rust.

The fix for this slice was to make mutable named parameters first-class in Rust IR/rendering, restrict shadowing back to the borrowed/reassigned cases that still need it, and add a dedicated HIR mutating-method check so immutable parameters are rejected before codegen.

## Implementation

- Added `RustParam::NamedMut` and taught the renderer, IR import collector, IR validator, and preamble accounting to treat it as a first-class parameter kind.
- Updated function and class-method emitters so `own mut` lowers directly to Rust `mut name: T`, while borrowed mutable params keep lowering to `&mut T`.
- Removed redundant owned-parameter mutable shadows from function lowering; shadowing now only remains for borrowed/reassigned cases that still require local rebinding.
- Added a codegen regression proving that `own mut` emits `mut arr: Vec<i64>` and does not emit `let mut arr = arr;`.
- Added a dedicated `1299`-style pass fixture and the part 3 demo to prove consume-mutate-return behavior end to end.
- Fixed an unrelated but real semantic hole by rejecting mutating method calls such as `append()` on immutable parameters during HIR lowering.
- Updated the default-argument pass fixture to the canonical `own mut` spelling because it mutates and returns an owned list parameter.

## Validation

Targeted validation:

- `cargo test -p sifr_codegen own_mut_param_emits_mut_binding_without_shadow -- --nocapture`
- `cargo test -p sifr_hir own_mut_semantics_tests`
- `cargo run -q -p sifr -- emit demos/ad_hoc_own_mut_parameter_convention_part3_demo.sifr`
- `cargo run -q -p sifr -- run demos/ad_hoc_own_mut_parameter_convention_part3_demo.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/own_mut_replace_elements_1299.sifr`
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/own_parameter_method_mutation_requires_mut.sifr`

Authoritative local gates:

- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`

Demo output:

- `cargo run -q -p sifr -- run demos/ad_hoc_own_mut_parameter_convention_part3_demo.sifr` -> prints `[18, 6, 6, 6, 1, -1]`

## Coverage Added

- `crates/sifr_codegen/src/lib_codegen_tests.rs`
- `crates/sifr_hir/src/lower/own_mut_semantics_tests.rs`
- `crates/sifr_hir/src/lower/mutating_methods.rs`
- `crates/sifr/tests/e2e/pass/own_mut_replace_elements_1299.sifr`
- `crates/sifr/tests/e2e/fail/own_parameter_method_mutation_requires_mut.sifr`
- `demos/ad_hoc_own_mut_parameter_convention_part3_demo.sifr`

## Closure Decision

Part 3 is locally complete because `own mut` now lowers to canonical Rust `mut x: T`, the direct `1299`-style consume-mutate-return path checks, emits, and runs successfully, and immutable parameters no longer rely on backend shadowing to accidentally permit mutating method calls.

The remaining work is intentionally deferred:

- part 4: phase closure docs and external review cycles.
