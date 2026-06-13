# Wave 3 Batch 95 Review Pass 1

- `demos/compiled_expressions/negative_cases/idiomatic.rs`
  - OK: scaffold correctly records the direct `expected 'int', got 'str'` return mismatch contract.
- `demos/generics_impl/negative_cases/return_type_mismatch/idiomatic.rs`
  - OK: scaffold correctly documents the generic safe-indexing mismatch `expected 'T', got 'T | None'`.
- `demos/constrained_typevars/negative_cases/typevar_constraint_violation/idiomatic.rs`
  - OK: scaffold correctly documents the constrained typevar rejection for `float` against `(int, str)`.

Result: `OK` for all three files. No blockers.
