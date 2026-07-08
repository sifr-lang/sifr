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
  panic payload redaction at the Rust interop boundary.
- Scope note: this row certifies only the shared stdlib resource lifecycle core.
  Resource-shaped package ecosystems such as `reqwest`, `rusqlite`,
  `tokio-postgres`, and `redis` remain tracked by `opaque_resource_matrix`.
