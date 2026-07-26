# close_after_use

This fixture row is covered by focused runtime handle-state tests for the
shared Rust interop handle primitive.

- Positive evidence: `cargo test -p sifr_runtime interop` covers stable
  `HandleStateError::Closed` and `HandleStateError::Poisoned` access surfaces
  without panicking.
- Negative evidence: the same suite covers repeated close attempts remaining in
  the closed state and poisoned access winning over closed state after a caught
  Rust panic. This is a runtime `HandleStateError` observation, not a compiler
  diagnostic claim.
- Scope note: this row covers the runtime state primitive. Generated wrapper
  close/aclose behavior over resource-shaped crates remains tracked by
  `opaque_resource_matrix`.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `closed_handle_error_surface` runs `handle_reports_closed_and_poisoned_states_without_panicking` in `crates/sifr_runtime/src/interop.rs` through the blocking `sifr_runtime` suite at the `create-pr` profile.
- Negative `double_close_and_poisoned_access` runs `double_close_keeps_stable_closed_state` in `crates/sifr_runtime/src/interop.rs` through the blocking `sifr_runtime` suite at the `create-pr` profile.
