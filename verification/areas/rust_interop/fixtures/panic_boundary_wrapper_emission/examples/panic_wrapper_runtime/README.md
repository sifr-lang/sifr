# Panic Wrapper Runtime Scenario

This locked package executes generated synchronous Rust panic boundaries. It
proves mapped panics receive the redacted `RustPanicErrorBridge`, ordinary Rust
`Result` errors still map to the declared Sifr error, and a panic inside the
mapper falls back to the original redacted `RustPanicError`.

The negative evidence replaces `src/main.sifr` and must be rejected by the
signature probe before any final generated binary is built.

Validation resolves this checked-in lock graph with `--locked`, `--offline`,
and `--frozen` Cargo modes before executing the generated package.

Validation resolves only the checked-in lock graph and runs the package with
Cargo `--locked`, `--offline`, and `--frozen`.
