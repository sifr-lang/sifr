# fixture: shared_bridge_crate
# scenario-example: shared_hash_bridge

This scenario models a reusable shared Rust bridge crate. The Sifr package calls
`sifr_shared_hash_bridge.digest` and exchanges only stable runtime bridge types:
`bytes`, `str`, and the public `SharedDigest` record.

`Cargo.toml` owns the shared crate path dependency. `sifr.toml` owns the
bridge-version, direct-binding enablement, and no-panic trust assertions.

The Rust source intentionally mentions `crate::__sifr_bridge` only in a comment
so the positive fixture covers the allowed boundary. The paired negative
evidence imports that generated package-private module from shared crate source
and must remain rejected.
