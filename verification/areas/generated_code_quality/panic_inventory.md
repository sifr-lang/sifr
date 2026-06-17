# Generated-Code Panic Inventory

The historical diagnostic-panic inventory could not be located under
`verification/` at generated-code quality rules start, so this file is the refreshed generated-code quality source
of truth for generated user-path scans.

## Generated User-Path Policy

Generated Rust user runtime paths must not contain:

- `.unwrap(`
- `.expect(`
- `panic!`
- `todo!`
- `unimplemented!`
- `unsafe`
- `#[allow(...)]` gate suppression

The generated-code quality scanner enforces these patterns on generated `.rs` files for every
positive manifest entry. Data-dependent user paths have no allowlist.

## Internal-Invariant Allowlist

No generated-code internal-invariant allowlist entries are active.

Any future entry must include an owner, rationale, removal criteria, and linked
issue. Data-dependent user paths may not be allowlisted.
