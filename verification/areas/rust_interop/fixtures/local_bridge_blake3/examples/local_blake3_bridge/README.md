# fixture: local_bridge_blake3
# scenario-example: local_blake3_bridge

This scenario models a package-local Rust bridge. `sifr.toml` declares
`[rust] bridges = ["src/bridges"]` and trusts the user-owned
`src/bridges/blake3.rs` file. The Sifr source resolves
`bridge.blake3.hash_bytes` through that local bridge namespace, while
`Cargo.toml` keeps the backend crate dependency explicit.
