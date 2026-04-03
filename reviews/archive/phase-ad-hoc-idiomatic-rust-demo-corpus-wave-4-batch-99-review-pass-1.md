# Wave 4 Batch 99 Review Pass 1

- `demos/normalized_fixtures/idiomatic.rs`
  - OK: direct `to_string()` conversion and direct string assertions remove redundant formatting without changing the demo behavior.
- `demos/error_subclasses/idiomatic.rs`
  - OK: collapsing duplicate `.to_string()` chains in the JSON helper and subclass-kind checks preserves the same output and error-family behavior.
- `demos/python_regressions/idiomatic.rs`
  - OK: the string-cleanup changes are behavior-preserving, and switching `repeat` to `result.into_iter()` fixes a real standalone iterator-lifetime bug without changing the observed regression-demo output.

Result: `OK` for all three files. No blockers.
