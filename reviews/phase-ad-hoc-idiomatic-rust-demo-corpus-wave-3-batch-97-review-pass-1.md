# Wave 3 Batch 97 Review Pass 1

- `demos/default_values/negative_cases/unsupported_default_call_expression/idiomatic.rs`
  - OK: scaffold correctly records the unsupported call-expression default and the deterministic missing-argument follow-on diagnostic.
- `demos/protocol_bounds/negative_cases/unknown_protocol_bound_forwarding/idiomatic.rs`
  - OK: scaffold correctly documents the forwarded unknown-protocol-bound failure.
- `demos/variance_rules/negative_cases/list_variance_violation/idiomatic.rs`
  - OK: scaffold correctly documents list invariance rejecting `list[int]` where `list[int | str]` is required.
- `demos/while_else/negative_cases/idiomatic.rs`
  - OK: scaffold correctly documents the runtime guard contract that `while`-`else` must skip the `else` arm after `break` and still print `ok`.

Result: `OK` for all four files. No blockers.
