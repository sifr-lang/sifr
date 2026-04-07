# Full Corpus Failure Taxonomy Delta

Baseline: `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun1.json`
Current: `verification/leetcode/full_corpus_failure_taxonomy_20260408_live_rerun3.json`

## Failure Count Delta

- failing fixtures: `18 -> 0` (`-18`)

## Category Delta

- `codegen_runtime_build_gap`: `6 -> 0` (`-6`)
- `ownership_and_mutability_boundary`: `4 -> 0` (`-4`)
- `nonlocal_mutable_capture_not_supported`: `2 -> 0` (`-2`)
- `recursive_node_and_field_expression_surface`: `2 -> 0` (`-2`)
- `signature_invalid_fixture_surface`: `2 -> 0` (`-2`)
- `optional_none_flow_and_narrowing_gap`: `1 -> 0` (`-1`)
- `other_type_surface_and_api_mismatch`: `1 -> 0` (`-1`)

## Result-Status Delta

- `CHECK_ERROR`: `11 -> 0` (`-11`)
- `RUN_ERROR`: `7 -> 0` (`-7`)

Current corpus status (`rerun3`) has no failing fixtures.
