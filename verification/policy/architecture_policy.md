# Method dispatch and unsafe ABI policy

The source method inventory binds current lexical dispatch sites, typed admission,
compiler constituents and representation adaptations to their reviewed owners.
The unsafe inventory is the complete disjoint union of `python_core.json`,
`python_resources.json` and `external.json` under `unsafe_abi_sites/`. Partition
membership derives from source modules and scopes, not mutable record owners.

Both live guards validate their inventories against the schemas under `schemas/`
and reject missing, duplicate, changed or stale sites. Unsafe records bind the
actual operation, adjacent SAFETY evidence, four effect obligations and local
source proofs. Finite sharing binds exact existing sites and fingerprints;
new operations need their own review. SQL, cache, driver and generated Rust
retain their existing acceptance owners. Generated Rust text belongs to X02.

Run `python3 scripts/check_method_dispatch_authority.py` and
`python3 scripts/check_unsafe_abi_contracts.py`; each has a scripts-only cold
`--self-test`. Create-pr, merge, nightly and release select both blocking guards
and the developer tooling `architecture-policy` suite. That suite runs the full
policy fixtures, including schema/profile drift and actual missing-segment guard
execution. Its source-bound positive and negative proof checks have a 15-minute
area budget. Coverage registration binds this suite to compiler/codegen.

These static policies admit reviewed classifications and contracts. Executable
ABI, lifetime, aliasing, callback and exact-once release assertions remain owned
by the native runtime/codegen tests. Policy success does not certify runtime
safety or qualify optional external producers, a full merge gate or a release.
