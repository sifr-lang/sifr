# Full Corpus Failure Taxonomy (2026-04-02 live)

- Total cases: `411`
- Failing cases: `241`
- Categories: `13`

## Category Counts

- `ownership_and_mutability_boundary`: `47`
- `optional_none_flow_and_narrowing_gap`: `39`
- `recursive_node_and_field_expression_surface`: `38`
- `destructuring_and_assignment_target_surface_gap`: `22`
- `any_unknown_typing_and_container_specialization_gap`: `18`
- `operator_and_truthiness_typing_gap`: `18`
- `return_path_and_function_contract_gap`: `13`
- `signature_invalid_fixture_surface`: `13`
- `callable_argument_contract_mismatch`: `10`
- `python_stdlib_and_builtin_parity_gap`: `9`
- `codegen_runtime_build_gap`: `7`
- `container_iteration_and_indexing_gap`: `6`
- `nonlocal_mutable_capture_not_supported`: `1`

## Ownership Subcategories

- `immutable_parameter_mutation`: `30`
- `immutable_parameter_reassignment`: `11`
- `borrowed_parameter_escape_store`: `4`
- `borrowed_parameter_escape_return`: `2`
