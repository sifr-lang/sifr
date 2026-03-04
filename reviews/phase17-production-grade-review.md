# Phase 17 Production-Grade Review: Import and Externals Correctness

**Review Date**: 2026-03-04
**Reviewer**: Claude Code
**Phase**: 17 - Import and Externals Correctness
**Status**: INCOMPLETE - Multiple Critical Bugs Remain

---

## Executive Summary

Phase 17 addresses import and external resolution correctness across all compiler pipelines (`check`, `run`, `build`, `test`). The phase has three milestones:

| Milestone | Status | Key Issue |
|-----------|--------|-----------|
| 17_1: Frontend-Only Check Path | **NOT IMPLEMENTED** | `check` triggers full codegen |
| 17_2: Non-Main Externals Resolution | **PARTIALLY IMPLEMENTED** | Constants not exported |
| 17_3: Test and Constant Import Parity | **NOT IMPLEMENTED** | Missing stdlib in tests |

**Overall Assessment**: The phase is not ready for production. Critical correctness bugs remain that violate the fundamental contract of the compiler pipelines.

---

## Correctness Risks

### CRITICAL: Check Pipeline Performs Unnecessary Codegen

**Severity**: Critical (violates fundamental compiler contract)

**Location**: `crates/sifr_driver/src/lib.rs:633-638`

```rust
pub fn check(source: &str) -> Vec<CompileError> {
    match compile(source) {
        CompileResult::Success { .. } => vec![],
        CompileResult::Errors { errors } => errors,
    }
}
```

**Problem**: The `check` function calls `compile(source)` which executes ALL phases:
- Phase 0: Compile embedded stdlib (lines 562-566)
- Phase 1: Parse (lines 568-592)
- Phase 2: Lower to HIR with type checking (lines 594-619)
- **Phase 3: Generate Rust code** (lines 621-622) ← SHOULD NOT RUN

**Expected Behavior**: Per the milestone definition, "check no longer triggers full code generation."

**Actual Behavior**: Full code generation runs on every `check` invocation.

**Impact**:
1. Performance degradation - unnecessary codegen for type-checking
2. User expectation mismatch - check should be fast type-only operation
3. Runtime coupling - check pulls in codegen dependencies unnecessarily

**Required Fix**: Create a dedicated type-checking path that stops after HIR lowering, or add a parameter to `compile_with_metadata` to skip codegen.

---

### CRITICAL: Test Runner Missing Stdlib Externals

**Severity**: Critical (breaks test infrastructure)

**Location**: `crates/sifr_driver/src/lib.rs:1073-1089`

```rust
// Lower to HIR
let lowering_result = match lower_module(parsed.suite()) {
    Ok(result) => result,
    // ...
};

// Generate Rust code in test mode
let codegen_result = generate_rust_test(&lowering_result.module);
```

**Problem 1**: `lower_module` uses empty externals
- `lower_module` calls `lower_module_with_externals(stmts, &ExternalDefs::default())` (see `crates/sifr_hir/src/lower.rs:287-288`)
- No stdlib definitions are available during type-checking

**Problem 2**: `run_tests` never calls `compile_stdlib()`
- Compare to `compile_with_metadata` at line 563 which calls `compile_stdlib()`

**Problem 3**: `generate_rust_test` doesn't register stdlib metadata
- Unlike `generate_rust_with_stdlib` which pre-registers constants and function signatures (lines 177-217 in `lib.rs`)
- `generate_rust_test` only handles basic import needs

**Impact**:
- Test files cannot import from stdlib (e.g., `from sifr.test import assert_eq`)
- Internal testing of stdlib functionality is broken
- User tests that rely on stdlib testing utilities fail

**Required Fix**:
1. Call `compile_stdlib()` in `run_tests` to get stdlib definitions
2. Use `lower_module_with_externals` with stdlib definitions
3. Use `generate_rust_with_stdlib` with test mode flag instead of `generate_rust_test`

---

### HIGH: Constants Not Exported from Non-Main Modules

**Severity**: High (breaks module contract)

**Location**: `crates/sifr_driver/src/lib.rs:725-799`

```rust
// Collect exports for this module
let mut fn_exports = HashMap::new();
let mut class_exports = HashMap::new();

// ... collects functions and classes ...

external_defs
    .functions
    .insert(module_name.clone(), fn_exports);
external_defs
    .classes
    .insert(module_name.clone(), class_exports);
// MISSING: constants not exported!
hir_modules.insert(module_name.clone(), result.module);
```

**Problem**: The code exports functions and classes from non-main modules, but **constants are not collected or exported**.

**Expected Behavior**: Per milestone 17_3: "Support local-module constant imports in externals model"

**Actual Behavior**: Constants defined in non-main modules cannot be imported by other modules.

**Impact**:
- Local constants cannot be shared across modules
- Forces code duplication or workaround patterns
- Inconsistent with function/class export behavior

**Required Fix**: Collect and export constants from non-main modules:
```rust
let mut const_exports = HashMap::new();
for constant in &result.module.constants {
    const_exports.insert(constant.name.clone(), constant.ty.clone());
}
external_defs.constants.insert(module_name.clone(), const_exports);
```

---

## Robustness Gaps

### Gap 1: No Test for Check Performance/Behavior

**Issue**: No test verifies that `check` stops after type-checking.

**Current Tests**: Only tests that check returns errors for invalid programs exist (lines 1217-1237).

**Missing Coverage**:
- Test that `check` doesn't produce Rust output
- Test that `check` is faster than full compilation
- Test that check mode doesn't require runtime dependencies

---

### Gap 2: No E2E Test for Multi-File Imports

**Issue**: No end-to-end test validates non-main module imports work correctly.

**Current Tests**: Only unit tests for `compile` function exist.

**Missing Coverage**:
- Multi-file project with local imports
- Importing functions from non-main modules
- Importing classes from non-main modules

---

### Gap 3: No Test for Test Runner with Stdlib

**Issue**: No test validates test files can import from stdlib.

**Current Tests**: No tests for `run_tests` function with actual test files.

**Missing Coverage**:
- Test runner with stdlib imports
- Test runner with sifr.test.assert_eq
- Test runner with sifr.io functionality

---

### Gap 4: No Test for Constant Exports

**Issue**: No test validates local constants can be imported across modules.

**Current Tests**: No tests for multi-file constant sharing.

**Missing Coverage**:
- Define constant in non-main module
- Import constant in main module
- Verify constant has correct type

---

## Missing Regression Coverage

### Regression Tests Required

1. **Check Path Regression**
   - Verify check doesn't produce Rust output
   - Verify check completes in reasonable time

2. **Test Runner Regression**
   - Verify test files can use stdlib imports
   - Verify test runner compiles with stdlib dependencies

3. **Module Export Regression**
   - Verify function exports work
   - Verify class exports work
   - Verify constant exports work

---

## Test Coverage Analysis

### Current Test Coverage in `crates/sifr_driver/src/lib.rs`

| Test | Coverage Area | Phase 17 Coverage |
|------|---------------|------------------|
| `test_compile_hello_world` | Basic compile | ❌ |
| `test_compile_factorial` | Function compile | ❌ |
| `test_type_mismatch_error` | Error detection | ❌ |
| `test_check_valid_program` | Check success path | ❌ |
| `test_generate_test_runner_cargo_toml_*` | Cargo.toml generation | ❌ |

**Phase 17 Specific Tests**: None

---

## Behavioral Summary

| Command | Expected | Actual | Status |
|---------|----------|--------|--------|
| `sifr check` | Type-check only | Full compile + codegen | ❌ REGRESSION |
| `sifr build` | Full compile | Works | ✅ |
| `sifr run` | Full compile + run | Works | ✅ |
| `sifr test` | Test with stdlib | No stdlib | ❌ REGRESSION |
| Multi-file import (functions) | Works | Works | ✅ |
| Multi-file import (classes) | Works | Works | ✅ |
| Multi-file import (constants) | Works | **BROKEN** | ❌ |

---

## Recommendations

### Priority 1: Fix Check Pipeline

1. Create `type_check_only(source: &str) -> Vec<CompileError>` function
2. Should only execute: compile_stdlib → parse → lower_module_with_externals
3. Should NOT execute: generate_rust_with_stdlib
4. Update `check` to use new function

### Priority 2: Fix Test Runner

1. Call `compile_stdlib()` at start of `run_tests`
2. Change `lower_module` to `lower_module_with_externals(parsed.suite(), &stdlib_compiled.defs)`
3. Change `generate_rust_test` to use `generate_rust_with_stdlib` with test mode flag

### Priority 3: Fix Constant Exports

1. Add constant collection loop in `build_project` (around line 724)
2. Insert constants into `external_defs.constants`
3. Test constant imports across modules

### Priority 4: Add Regression Tests

1. Test for check command behavior
2. Test for multi-file imports
3. Test for test runner with stdlib
4. Test for constant exports

---

## Exit Gate Assessment

**Current Status**: Phase 17 does NOT meet exit criteria.

**Required Evidence**:
- [ ] `check` stops after frontend/type phases (not implemented)
- [ ] `check` no longer triggers full code generation (not implemented)
- [ ] Non-main modules can import stdlib correctly (partially works)
- [ ] Multi-file projects type-check consistently (works)
- [ ] Test runner imports behave like compile pipeline (not implemented)
- [ ] Local constants import successfully across modules (not implemented)

---

## Files Modified in Phase 17

| File | Changes |
|------|---------|
| `crates/sifr_driver/src/lib.rs` | Check path, build_project, run_tests |
| `crates/sifr_codegen/src/lib.rs` | generate_rust_with_stdlib enhancements |
| `crates/sifr_codegen/src/entrypoints.rs` | generate_rust_test function |
| `crates/sifr_codegen/src/module_constants.rs` | Module constant handling |
| `crates/sifr_hir/src/lower.rs` | lower_module implementation |

---

## Conclusion

Phase 17 has **NOT** achieved production readiness. Critical correctness bugs remain:

1. **Check performs unnecessary codegen** - violates user contract and causes performance regression
2. **Test runner lacks stdlib access** - breaks test infrastructure
3. **Constants not exported** - breaks module semantics

These issues must be resolved before the phase can be considered complete. The implementation requires significant additional work to meet the stated definition of done for all three milestones.
