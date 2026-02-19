## End-to-End Tests for M1

#### **Current Situation**

- Individual crates have their own unit and snapshot tests, but there are no tests that verify the entire pipeline from `.sifr` source to running binary.
- AI agents need a reliable way to verify the full compiler works correctly after any change.

#### **Desired Situation**

- A comprehensive E2E test suite exists that compiles `.sifr` programs, runs them, and checks output.
- Pass tests verify correct programs produce expected stdout.
- Fail tests verify invalid programs produce expected error diagnostics.
- Ownership tests verify move/borrow errors are caught.
- All tests run via `cargo test` and complete in under 30 seconds.

#### **Suggested Solution**

1. Create `tests/e2e/` directory structure:
   - `pass/` -- programs that must compile and produce expected output
   - `fail/` -- programs that must fail with expected errors
   - `ownership/` -- ownership-specific compile failures
2. Create `sifr_test_utils` crate in `crates/` with shared test helpers:
   - `compile_to_rust(source) -> Result<String, Vec<Diagnostic>>`
   - `compile_and_run(path) -> Result<Output, Error>`
   - `extract_expect_stdout(source) -> &str` (parse `# expect-stdout:` header)
   - `extract_expect_errors(source) -> Vec<&str>` (parse `# expect-error:` comments)
3. Write pass test programs:
   - `hello_world.sifr` -- prints "Hello, World!"
   - `factorial.sifr` -- recursive factorial, prints 120
   - `fibonacci.sifr` -- recursive fibonacci
   - `arithmetic.sifr` -- basic math operations
   - `string_concat.sifr` -- string concatenation
   - `if_else.sifr` -- conditional branching
   - `type_inference.sifr` -- inferred variable types
   - `multiple_functions.sifr` -- calling between functions
4. Write fail test programs:
   - `type_mismatch.sifr` -- `x: int = "hello"`
   - `undefined_var.sifr` -- use of undefined variable
   - `wrong_arg_type.sifr` -- function called with wrong type
   - `missing_annotation.sifr` -- function param without type
5. Write ownership test programs:
   - `use_after_move.sifr` -- use string after moving it
   - `copy_type_ok.sifr` -- int reuse after assignment (should pass)
6. Create E2E test runner in `tests/e2e.rs`.
