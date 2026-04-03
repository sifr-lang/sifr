# Wave 3 Batch 92 Review Pass 1

- `demos/optional_arithmetic/negative_cases/optional_arithmetic_without_narrowing/idiomatic.rs`
  - OK: scaffold accurately records the two observed contracts: arithmetic on `int | None` without narrowing and the resulting declared `-> int` return-path failure.
- `demos/optional_indexing/negative_cases/option_method_without_narrowing/idiomatic.rs`
  - OK: scaffold correctly states that `.len()` is invalid on `list[int] | None` before narrowing.
- `demos/indexing_rules/negative_cases/invalid_index_type/idiomatic.rs`
  - OK: scaffold correctly documents the deterministic rejection of indexing `list[int]` with `str`.

Result: `OK` for all three files. No blockers.
