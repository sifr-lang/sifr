# opaque_resource_core

This fixture row is covered by focused runtime handle-state tests for the
stdlib-owned opaque resource core used by native stdlib resource migrations.
The `.sifr` files are declarative fixture headers for the Rust interop matrix;
the executable evidence is the named runtime test filter below.

- Positive evidence: `cargo test -p sifr_runtime interop` covers
  `sifr_runtime::interop::Handle<T>` open access, stable close transitions,
  poisoned access reporting, and successful disarming of the panic poison guard.
- Negative evidence: the same suite covers repeated close attempts remaining in
  the closed state, poisoned access winning over a previous closed state, and
  panic payload redaction at the Rust interop boundary. These are observed
  runtime `HandleStateError` states rather than compiler diagnostics.
- Scope note: this row certifies only the shared stdlib resource lifecycle core.
  Resource-shaped package ecosystems such as `reqwest`, `rusqlite`,
  `tokio-postgres`, and `redis` remain tracked by `opaque_resource_matrix`.

## Canonical validation provenance

The structured `fixture.json` record is authoritative. These names repeat its
exact executable Rust-test bindings for readers:

- Positive `stdlib_handle_close_poison_lifecycle` runs `handle_poison_guard_can_be_disarmed_after_successful_call` in `crates/sifr_runtime/src/interop.rs` through the blocking `sifr_runtime` suite at the `create-pr` profile.
- Negative `stdlib_handle_double_close_poisoned_access` runs `poisoned_state_wins_over_closed_state` in `crates/sifr_runtime/src/interop.rs` through the blocking `sifr_runtime` suite at the `create-pr` profile.
