# Rust Interop Panic Wrapper Emission Fixtures

This fixture family tracks generated wrapper behavior that is not covered by the contract-only panic boundary evidence.

Planned coverage:

- emitted wrappers catch recoverable Rust panics and surface the declared Sifr error channel,
- `panic=map_error(path)` adapters are signature-checked,
- mapper panics fall back to the original redacted `RustPanicError`,
- invalid mapper signatures fail before final generated binary build.
