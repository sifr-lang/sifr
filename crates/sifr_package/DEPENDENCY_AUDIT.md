# sifr_package Dependency Audit

## Cargo CLI metadata JSON

- Surface: `cargo metadata --format-version 1`.
- Used by: `crates/sifr_package::cargo::metadata`.
- Reason: Phase 37 consumes Cargo's stable JSON command output without linking to Cargo internals.
- Stability risk: Cargo may add fields, and package/dependency ordering is not a semantic contract.
- Mitigation: Sifr deserializes only consumed fields, preserves unknown JSON outside the public facade, sorts selected records before graph derivation, and computes graph digests from the normalized representation.
- Fallback: if a future Cargo CLI changes required fields incompatibly, `SIFR-PACKAGE-0103` reports a metadata normalization error with the Cargo action and required field.

No `cargo_metadata` crate is linked in milestone 37.1. If it is introduced later, this file must record the exact crate version, Cargo CLI version range, consumed fields, ordering risks, and fallback behavior.
