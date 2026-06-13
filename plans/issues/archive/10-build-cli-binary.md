## Build sifr CLI Binary

#### **Current Situation**

- The driver crate provides the compilation API but there is no user-facing command-line interface.
- Developers need a `sifr` command to compile, run, check, and inspect their programs.

#### **Desired Situation**

- A `sifr` CLI binary exists with four commands: `build`, `run`, `check`, `emit`.
- `sifr build <file.sifr>` compiles to a native binary.
- `sifr run <file.sifr>` compiles and immediately runs the program.
- `sifr check <file.sifr>` type-checks without compiling (fast feedback).
- `sifr emit <file.sifr>` shows the generated Rust source code.
- Error output is colored and user-friendly.

#### **Suggested Solution**

1. Create `crates/sifr/` as a binary crate with `clap` for argument parsing.
2. Implement four subcommands:
   - `build`: call `driver::compile()`, write Rust project, invoke `cargo build`, copy binary to output
   - `run`: same as build + execute the binary
   - `check`: call `driver::check()`, print diagnostics, exit with code 0 (clean) or 1 (errors)
   - `emit`: call `driver::compile()`, print generated Rust to stdout
3. Handle CLI options:
   - `--output` / `-o` for build output path
   - `--verbose` / `-v` for detailed output
   - Input file path (positional argument)
4. Implement colored output:
   - Use `colored` or `termcolor` crate
   - Errors in red, warnings in yellow, success in green
5. Add integration tests for the CLI:
   - Test that `sifr build` produces a binary
   - Test that `sifr check` reports errors correctly
   - Test that `sifr emit` outputs valid Rust
