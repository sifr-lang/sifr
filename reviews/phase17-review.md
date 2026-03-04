# Phase 17 Review: Import and Externals Correctness

## Summary

This review analyzes the implementation of Phase 17 (Import and Externals Correctness) against its three milestones: 17_1 (Frontend-Only Check Path), 17_2 (Non-Main Externals Resolution), and 17_3 (Test and Constant Import Parity).

**Overall Assessment: INCOMPLETE - Multiple critical bugs found**

---

## Milestone 17_1: Frontend-Only Check Path

**Status: NOT IMPLEMENTED - Regression Bug**

### Definition of Done
- Ensure `check` stops after frontend/type phases
- Remove codegen/runtime coupling from check flow
- `check` no longer triggers full code generation

### Finding

**BUG: `check` still triggers full code generation**

Location: `crates/sifr_driver/src/lib.rs:633-638`

```rust
pub fn check(source: &str) -> Vec<CompileError> {
    match compile(source) {
        CompileResult::Success { .. } => vec![],
        CompileResult::Errors { errors } => errors,
    }
}
```

The `check` function calls `compile(source)` which goes through ALL phases:
1. Phase 0: Compile embedded stdlib .sifr files (line 562-566)
2. Phase 1: Parse (line 568-592)
3. Phase 2: Lower to HIR with stdlib externals (line 594-619)
4. **Phase 3: Generate Rust code** (line 621-622)

This violates the milestone requirement that "check no longer triggers full code generation."

### Correct Implementation Required

The check function should only execute:
- Phase 0: Compile stdlib (needed for type resolution)
- Phase 1: Parse
- Phase 2: Lower to HIR (type checking)

It should NOT execute Phase 3 (codegen). The fix would require creating a `type_check_only` function or adding a parameter to control whether codegen runs.

---

## Milestone 17_2: Non-Main Externals Resolution

**Status: PARTIALLY IMPLEMENTED**

### Definition of Done
- Resolve stdlib/local externals in non-main modules
- Ensure multi-file projects type-check consistently
- Non-main modules can import stdlib/local modules correctly

### Findings

**Positive: Multi-file project compilation is implemented**

Location: `crates/sifr_driver/src/lib.rs:643-825`

The `build_project` function correctly:
1. Discovers all .sifr files in the project directory (lines 646-656)
2. Parses all modules (lines 671-697)
3. Compiles stdlib (line 700)
4. Lowers non-main modules first to collect exports (lines 706-800)
5. Lowers main module with external definitions (lines 802-818)

This implementation correctly exports functions and classes from non-main modules.

---

## Milestone 17_3: Test and Constant Import Parity

**Status: NOT IMPLEMENTED - Multiple critical bugs**

### Definition of Done
- Align `sifr test` import behavior with regular compilation
- Support local-module constant imports in externals model
- Test runner imports behave like compile pipeline

### Finding 1: Test runner missing stdlib externals

Location: `crates/sifr_driver/src/lib.rs:1073-1089`

```rust
// Lower to HIR
let lowering_result = match lower_module(parsed.suite()) {
    Ok(result) => result,
    Err(errors) => {
        // ...
    }
};

// Generate Rust code in test mode
let codegen_result = generate_rust_test(&lowering_result.module);
```

**Problems:**
1. `lower_module(parsed.suite())` at line 1074 uses empty externals (see `crates/sifr_hir/src/lower.rs:287-288`):
   ```rust
   pub fn lower_module(stmts: &[Stmt]) -> Result<LoweringResult, Vec<LoweringError>> {
       lower_module_with_externals(stmts, &ExternalDefs::default())
   }
   ```
2. The test runner never calls `compile_stdlib()` to get stdlib definitions
3. `generate_rust_test` at line 1089 doesn't register stdlib constants and function signatures (unlike `generate_rust_with_stdlib`)

**Impact:** Tests cannot import from stdlib modules (e.g., `from sifr.test import assert_eq`). This breaks the milestone requirement that "Test runner imports behave like compile pipeline."

### Finding 2: Constants not exported from non-main modules

Location: `crates/sifr_driver/src/lib.rs:725-800`

When building the external_defs for non-main modules, the code exports:
- Functions (lines 729-743)
- Classes (lines 746-790)

But constants are **NOT** exported:

```rust
external_defs
    .functions
    .insert(module_name.clone(), fn_exports);
external_defs
    .classes
    .insert(module_name.clone(), class_exports);
// MISSING: constants are not exported!
```

**Impact:** Local module constants cannot be imported by other modules. This violates the milestone requirement: "Support local-module constant imports in externals model."

---

## Behavioral Regressions

### Regression 1: Performance regression
The `check` command performs unnecessary codegen, making it slower than necessary. For large projects, this adds significant overhead.

### Regression 2: Feature regression
- Test runner cannot use stdlib imports
- Local constants cannot be imported across modules

---

## Missing Tests

1. **Test for check command stopping after type-checking**
   - No test verifies that `check` doesn't trigger codegen

2. **Test for multi-file project with local imports**
   - No e2e test validates non-main module imports work correctly

3. **Test for test runner with stdlib imports**
   - No test validates that test files can import from stdlib

4. **Test for constant exports from non-main modules**
   - No test validates local constants can be imported across modules

---

## Production Risks

### Risk 1: Incorrect error detection
Users using `sifr check` may experience different behavior than expected. The command is meant to be a fast type-check, but it performs full compilation.

### Risk 2: Broken test infrastructure
The test runner (`sifr test`) cannot properly run tests that depend on stdlib imports. This affects:
- Internal testing of stdlib functionality
- User tests that rely on stdlib testing utilities

### Risk 3: Inconsistent import semantics
Local constants cannot be shared across modules, leading to code duplication or workaround patterns.

---

## Recommendations

1. **For milestone 17_1**: Create a `type_check_only` function that stops after lowering phase, or add a parameter to `compile_with_metadata` to skip codegen.

2. **For milestone 17_3 (Test runner)**: Update `run_tests` to:
   - Call `compile_stdlib()` to get stdlib definitions
   - Use `lower_module_with_externals` with stdlib definitions
   - Use `generate_rust_with_stdlib` instead of `generate_rust_test`

3. **For milestone 17_3 (Constants)**: Update `build_project` to also collect and export constants from non-main modules to `external_defs.constants`.

4. **Add tests** for:
   - Check command performance/behavior
   - Multi-file project imports
   - Test runner with stdlib
   - Local constant imports
