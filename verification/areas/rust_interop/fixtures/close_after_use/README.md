# close_after_use

This fixture row is covered by focused runtime handle-state tests for the
shared Rust interop handle primitive.

- Positive evidence: `cargo test -p sifr_runtime interop` covers stable
  `HandleStateError::Closed` and `HandleStateError::Poisoned` access surfaces
  without panicking.
- Negative evidence: the same suite covers repeated close attempts remaining in
  the closed state and poisoned access winning over closed state after a caught
  Rust panic.
- Scope note: this row covers the runtime state primitive. Generated wrapper
  close/aclose behavior over resource-shaped crates remains tracked by
  `opaque_resource_matrix`.
