# Full Corpus Taxonomy Delta (rerun2 vs rerun1)

- baseline taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun1.json`
- current taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260407_live_rerun2.json`
- baseline results: `verification/leetcode/full_corpus_current_results_20260407_live_rerun1.json`
- current results: `verification/leetcode/full_corpus_current_results_20260407_live_rerun2.json`

## Status Count Delta

| status | rerun1 | rerun2 | delta |
|---|---:|---:|---:|
| CHECK_ERROR | 42 | 28 | -14 |
| NO_ORACLE | 0 | 179 | +179 |
| PASS | 359 | 194 | -165 |
| RUN_ERROR | 10 | 10 | 0 |

## Category Count Delta

| category | rerun1 | rerun2 | delta |
|---|---:|---:|---:|
| callable_argument_contract_mismatch | 1 | 0 | -1 |
| codegen_runtime_build_gap | 5 | 5 | 0 |
| destructuring_and_assignment_target_surface_gap | 1 | 1 | 0 |
| nonlocal_mutable_capture_not_supported | 2 | 2 | 0 |
| operator_and_truthiness_typing_gap | 11 | 0 | -11 |
| optional_none_flow_and_narrowing_gap | 1 | 1 | 0 |
| other_type_surface_and_api_mismatch | 11 | 11 | 0 |
| ownership_and_mutability_boundary | 4 | 4 | 0 |
| python_stdlib_and_builtin_parity_gap | 10 | 10 | 0 |
| recursive_node_and_field_expression_surface | 2 | 2 | 0 |
| return_path_and_function_contract_gap | 2 | 0 | -2 |
| signature_invalid_fixture_surface | 2 | 2 | 0 |

