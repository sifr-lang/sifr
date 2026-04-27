# Phase 31 Failure Taxonomy Report

- Classified failing seed cases: `129`
- Buckets: `9`
- Spot-audit accuracy: `0%`

## Buckets

### frontend.generic_check_failure
- Layer: `frontend`
- Title: Generic frontend check failure
- Case count: `53`
- Statuses: `{'CHECK_ERROR': 53}`
- Topics: `{'math': 1, 'arrays': 52}`
- Smallest known repro: `1930` -> `audits/leetcode/src/1930_unique_length_3_palindromic_subsequences.sifr` (15 lines)
- Repro stderr excerpt: `type error: str has no method 'rfind'`

### type_system.optional_narrowing_and_union_ops
- Layer: `type_system`
- Title: Optional narrowing and union-operator gap
- Case count: `29`
- Statuses: `{'CHECK_ERROR': 29}`
- Topics: `{'arrays': 29}`
- Smallest known repro: `0739` -> `audits/leetcode/src/0739_daily_temperatures.sifr` (17 lines)
- Repro stderr excerpt: `Any | None`

### lowering.destructuring_target_support
- Layer: `lowering`
- Title: Destructuring target lowering gap
- Case count: `28`
- Statuses: `{'CHECK_ERROR': 28}`
- Topics: `{'arrays': 28}`
- Smallest known repro: `0280` -> `audits/leetcode/src/0280_wiggle_sort.sifr` (15 lines)
- Repro stderr excerpt: `tuple unpacking target must be a simple name`

### stdlib.python_module_surface
- Layer: `stdlib_runtime`
- Title: Python stdlib/module surface gap
- Case count: `6`
- Statuses: `{'CHECK_ERROR': 6}`
- Topics: `{'arrays': 6}`
- Smallest known repro: `0383` -> `audits/leetcode/src/0383_ransom_note.sifr` (16 lines)
- Repro stderr excerpt: `undefined function: 'Counter'`

### codegen.generic_run_failure
- Layer: `codegen`
- Title: Generic codegen/runtime build failure
- Case count: `4`
- Statuses: `{'RUN_ERROR': 4}`
- Topics: `{'arrays': 4}`
- Smallest known repro: `1498` -> `audits/leetcode/src/1498_number_of_subsequences_that_satisfy_the_given_sum_condition.sifr` (25 lines)
- Repro stderr excerpt: `warning: int left shift (<<) with non-constant shift amount may overflow i64 at runtime; consider using bigint`

### frontend.untyped_any_propagation
- Layer: `frontend`
- Title: Untyped Any propagation gap
- Case count: `4`
- Statuses: `{'CHECK_ERROR': 4}`
- Topics: `{'arrays': 4}`
- Smallest known repro: `1851` -> `audits/leetcode/src/1851_minimum_interval_to_include_each_query.sifr` (22 lines)
- Repro stderr excerpt: `cannot index type 'Any'`

### frontend.nested_function_annotation_support
- Layer: `frontend`
- Title: Nested function annotation/inference gap
- Case count: `3`
- Statuses: `{'CHECK_ERROR': 3}`
- Topics: `{'arrays': 3}`
- Smallest known repro: `0118` -> `audits/leetcode/src/0118_pascals_triangle.sifr` (24 lines)
- Repro stderr excerpt: `is missing a type annotation`

### lowering.attribute_expression_support
- Layer: `lowering`
- Title: Attribute-expression lowering gap
- Case count: `1`
- Statuses: `{'CHECK_ERROR': 1}`
- Topics: `{'arrays': 1}`
- Smallest known repro: `0230` -> `audits/leetcode/src/0230_kth_smallest_element_in_a_bst.sifr` (68 lines)
- Repro stderr excerpt: `attribute access '.`

### stdlib.python_builtin_signature_surface
- Layer: `stdlib_runtime`
- Title: Python builtin signature/parity gap
- Case count: `1`
- Statuses: `{'CHECK_ERROR': 1}`
- Topics: `{'arrays': 1}`
- Smallest known repro: `0079` -> `audits/leetcode/src/0079_word_search.sifr` (45 lines)
- Repro stderr excerpt: `sum() takes exactly 1 argument`

## Spot Audit

- Passed: `False`
- Accuracy: `0%`
- Threshold: `90%`

- `0003` expected `stdlib.python_module_surface` got `None` matched=`False`
- `0017` expected `frontend.nested_function_annotation_support` got `None` matched=`False`
- `0052` expected `lowering.unsupported_ast_shape` got `None` matched=`False`
- `0069` expected `codegen.mutable_binding_emission` got `None` matched=`False`
- `0100` expected `type_system.recursive_node_forward_reference` got `None` matched=`False`
- `0207` expected `lowering.destructuring_target_support` got `None` matched=`False`
- `0238` expected `type_system.optional_narrowing_and_union_ops` got `None` matched=`False`
- `0295` expected `lowering.destructuring_target_support` got `None` matched=`False`
- `0502` expected `stdlib.python_module_surface` got `None` matched=`False`
- `0746` expected `type_system.optional_narrowing_and_union_ops` got `None` matched=`False`
- `0912` expected `frontend.nested_function_annotation_support` got `None` matched=`False`
- `1299` expected `ownership.borrowed_return_surface` got `None` matched=`False`
- `1456` expected `type_system.optional_narrowing_and_union_ops` got `None` matched=`False`
- `2235` expected `stdlib.python_builtin_signature_surface` got `None` matched=`False`
