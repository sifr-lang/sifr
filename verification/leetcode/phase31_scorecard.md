# Phase 31 Compatibility Scorecard

- Review status: `external_review_approved`
- Seed corpus size: `50`
- Baseline status counts: `PASS=2`, `CHECK_ERROR=46`, `RUN_ERROR=2`
- Wave 1 status counts: `PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`
- Delta: `PASS +3`, `CHECK_ERROR -1`, `RUN_ERROR -2`

## Fixed Cases

- `0069_sqrtx`
- `0151_reverse_words_in_a_string`
- `2235_add_two_integers`

## Remaining Buckets

- `type_system.optional_narrowing_and_union_ops`: `16` remaining
- `lowering.destructuring_target_support`: `7` remaining
- `frontend.nested_function_annotation_support`: `6` remaining
- `stdlib.python_module_surface`: `6` remaining
- `type_system.recursive_node_forward_reference`: `4` remaining
- `frontend.generic_check_failure`: `3` remaining
- `lowering.attribute_expression_support`: `1` remaining
- `lowering.unsupported_ast_shape`: `1` remaining
- `ownership.borrowed_return_surface`: `1` remaining

## Unresolved Handoff

- `phase31-remediation-001` / `type_system.optional_narrowing_and_union_ops`: owner=`compiler/type_system`, priority=`P1`, handoff_target=`phase32`, remaining=`16`
- `phase31-remediation-002` / `lowering.destructuring_target_support`: owner=`compiler/lowering`, priority=`P1`, handoff_target=`phase32`, remaining=`7`
- `phase31-remediation-003` / `frontend.nested_function_annotation_support`: owner=`compiler/frontend`, priority=`P1`, handoff_target=`phase32`, remaining=`6`
- `phase31-remediation-004` / `stdlib.python_module_surface`: owner=`compiler/stdlib`, priority=`P1`, handoff_target=`phase32`, remaining=`6`
- `phase31-remediation-005` / `type_system.recursive_node_forward_reference`: owner=`compiler/type_system`, priority=`P1`, handoff_target=`phase32`, remaining=`4`
- `phase31-remediation-006` / `frontend.generic_check_failure`: owner=`compiler/frontend`, priority=`P2`, handoff_target=`phase32`, remaining=`3`
- `phase31-remediation-009` / `lowering.attribute_expression_support`: owner=`compiler/lowering`, priority=`P2`, handoff_target=`phase32`, remaining=`1`
- `phase31-remediation-010` / `lowering.unsupported_ast_shape`: owner=`compiler/lowering`, priority=`P2`, handoff_target=`phase32`, remaining=`1`
- `phase31-remediation-011` / `ownership.borrowed_return_surface`: owner=`language/ownership`, priority=`P3`, handoff_target=`deferred`, remaining=`1`

## Closure Note

- Phase 31 now has a reproducible baseline, taxonomy, remediation backlog, first remediation wave, and stable scorecard artifacts.
- External review/sign-off is complete, and the phase is approved for closure.

