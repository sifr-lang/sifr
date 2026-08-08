# bridge_version_field_removal

Passing evidence for the one unversioned Rust bridge manifest contract.

The removed versioned-schema evidence was replaced atomically:

- an unversioned `[rust]` manifest is accepted by the normal compiler manifest
  diagnostic path; and
- any `bridge-version` field is rejected as removed, with no rewrite,
  compatibility mode, versioned glue, or fallback.

The positive driver contract resolves an unversioned package bridge. The
negative driver contract parses a manifest containing the removed field and
requires `SIFR-RUST-CARGO-0001` before interop resolution.
