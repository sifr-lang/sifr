# fixture: same_workspace_crate
# scenario-example: workspace_hash_crate

This scenario models a Sifr package that calls a Rust crate from the same
workspace through an explicit Cargo path dependency. `Cargo.toml` owns the
`workspace_hash` path dependency and workspace membership; `sifr.toml` owns the
Sifr rust-interop compiler semantics and trust policy.

The paired negative evidence keeps using `undeclared_workspace_hash` to prove
that sibling crates are only visible when declared.
