# fixture: bridge_type_matrix
# scenario-example: bridge_type_roundtrip

This hermetic package exercises the generated Sifr-to-Rust call glue over the
five crates owned by `bridge_type_matrix`. The local bridge parses and emits a
nested Serde value with `serde_json`, reports its rejected shape through a
`thiserror` error, roundtrips a `bytes::Bytes` value, and roundtrips nested
list, dictionary, and list-of-dictionary values through
`indexmap::IndexMap`. Sifr's internal `HashMap` conversion does not preserve
key iteration order.

The generated binary is built and executed from a copied fixture tree under
locked, offline Cargo policy. No external services or network access are used.
The package manifest explicitly trusts the build scripts shipped by the direct
dependencies `serde`, `serde_json`, and `thiserror`.
