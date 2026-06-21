# Bridge Type Matrix Evidence

This fixture row tracks bridge type generation and conversion coverage. Positive
roundtrip fixtures remain planned until generated adapters exercise values
through the full Rust bridge boundary.

- Supporting compiler coverage: `cargo test -p sifr_codegen rust_interop`
  records `RustBridgeSignatureContract` entries for bytes parameters, `Result`
  returns, generated record bridge types, generated error bridge types, and
  cross-module generated bridge field paths.
- Negative coverage: `cargo test -p sifr_driver rust_interop` rejects an
  incompatible borrowed-bytes Rust signature and rejects unsupported container
  contracts before final binary build.
