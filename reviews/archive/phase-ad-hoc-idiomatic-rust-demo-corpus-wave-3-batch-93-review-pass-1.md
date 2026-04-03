# Wave 3 Batch 93 Review Pass 1

- `demos/generator_break_else/negative_cases/idiomatic.rs`
  - OK: scaffold correctly records both observed contracts, namely the invalid generator return annotation and the undefined `missing_value` in the reachable `except` yield path.
- `demos/recursive_calls/negative_cases/recursive_call_typo/idiomatic.rs`
  - OK: scaffold correctly documents the reachable recursive typo `reccurse(...)` under the `if n > 0` branch.
- `demos/recursive_for_else/negative_cases/idiomatic.rs`
  - OK: scaffold correctly documents that the analyzer must traverse the reachable `for`-`else` branch and diagnose `recc(...)` as undefined.

Result: `OK` for all three files. No blockers.
