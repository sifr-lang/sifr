# Phase 34 Generated-Code Panic Inventory

Phase 27's historical panic inventory could not be located under
`verification/` at Phase 34 start, so this file is the refreshed Phase 34 source
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

The Phase 34 scanner enforces these patterns on generated `.rs` files for every
positive manifest entry. Data-dependent user paths have no allowlist.

## Internal-Invariant Allowlist

No generated-code internal-invariant allowlist entries are active.

Any future entry must include an owner, rationale, removal criteria, and linked
issue. Data-dependent user paths may not be allowlisted.
