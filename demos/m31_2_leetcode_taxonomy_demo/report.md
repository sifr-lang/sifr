<!-- Reference: m31_2 -->
<!-- Source issue: phase31-algorithmic-compatibility-execution.md -->
# Phase 31 Failure Taxonomy Report

- Classified failing seed cases: `48`
- Buckets: `12`
- Spot-audit accuracy: `100%`

## Buckets

### type_system.optional_narrowing_and_union_ops
- Layer: `type_system`
- Title: Optional narrowing and union-operator gap
- Case count: `16`
- Statuses: `{'CHECK_ERROR': 16}`
- Topics: `{'strings': 3, 'two_pointers_sliding_window': 5, 'arrays': 2, 'dp': 4, 'heap_priority_queue': 1, 'hash_map': 1}`
- Smallest known repro: `0746` -> `audit/leetcode/0746_min_cost_climbing_stairs.sifr` (12 lines)
- Repro stderr excerpt: `int | None`

### lowering.destructuring_target_support
- Layer: `lowering`
- Title: Destructuring target lowering gap
- Case count: `7`
- Statuses: `{'CHECK_ERROR': 7}`
- Topics: `{'graphs': 4, 'heap_priority_queue': 2, 'strings': 1}`
- Smallest known repro: `0703` -> `audit/leetcode/0703_kth_largest_element_in_a_stream.sifr` (17 lines)
- Repro stderr excerpt: `tuple unpacking target must be a simple name`

### frontend.nested_function_annotation_support
- Layer: `frontend`
- Title: Nested function annotation/inference gap
- Case count: `6`
- Statuses: `{'CHECK_ERROR': 6}`
- Topics: `{'backtracking': 4, 'math': 1, 'arrays': 1}`
- Smallest known repro: `0050` -> `audit/leetcode/0050_powx_n.sifr` (20 lines)
- Repro stderr excerpt: `is missing a type annotation`

### stdlib.python_module_surface
- Layer: `stdlib_runtime`
- Title: Python stdlib/module surface gap
- Case count: `6`
- Statuses: `{'CHECK_ERROR': 6}`
- Topics: `{'hash_map': 2, 'math': 1, 'graphs': 1, 'heap_priority_queue': 2}`
- Smallest known repro: `0217` -> `audit/leetcode/0217_contains_duplicate.sifr` (16 lines)
- Repro stderr excerpt: `undefined function: 'set'`

### type_system.recursive_node_forward_reference
- Layer: `type_system`
- Title: Recursive node forward-reference resolution gap
- Case count: `4`
- Statuses: `{'CHECK_ERROR': 4}`
- Topics: `{'trees': 4}`
- Smallest known repro: `0100` -> `audit/leetcode/0100_same_tree.sifr` (13 lines)
- Repro stderr excerpt: `unknown type: 'TreeNode'`

### frontend.generic_check_failure
- Layer: `frontend`
- Title: Generic frontend check failure
- Case count: `3`
- Statuses: `{'CHECK_ERROR': 3}`
- Topics: `{'arrays': 1, 'hash_map': 2}`
- Smallest known repro: `0001` -> `audit/leetcode/0001_two_sum.sifr` (17 lines)
- Repro stderr excerpt: `type error: cannot index type 'dict[Any, Any]' with 'int'`

### codegen.generic_run_failure
- Layer: `codegen`
- Title: Generic codegen/runtime build failure
- Case count: `1`
- Statuses: `{'RUN_ERROR': 1}`
- Topics: `{'strings': 1}`
- Smallest known repro: `0151` -> `audit/leetcode/0151_reverse_words_in_a_string.sifr` (21 lines)
- Repro stderr excerpt: `build error: cargo build failed:`

### codegen.mutable_binding_emission
- Layer: `codegen`
- Title: Codegen mutable binding emission gap
- Case count: `1`
- Statuses: `{'RUN_ERROR': 1}`
- Topics: `{'math': 1}`
- Smallest known repro: `0069` -> `audit/leetcode/0069_sqrtx.sifr` (20 lines)
- Repro stderr excerpt: `cannot assign twice to immutable variable`

### lowering.attribute_expression_support
- Layer: `lowering`
- Title: Attribute-expression lowering gap
- Case count: `1`
- Statuses: `{'CHECK_ERROR': 1}`
- Topics: `{'trees': 1}`
- Smallest known repro: `0235` -> `audit/leetcode/0235_lowest_common_ancestor_of_a_binary_search_tree.sifr` (16 lines)
- Repro stderr excerpt: `attribute access '.`

### lowering.unsupported_ast_shape
- Layer: `lowering`
- Title: Unsupported AST lowering shape
- Case count: `1`
- Statuses: `{'CHECK_ERROR': 1}`
- Topics: `{'backtracking': 1}`
- Smallest known repro: `0052` -> `audit/leetcode/0052_n_queens_ii.sifr` (37 lines)
- Repro stderr excerpt: `unsupported statement type`

### ownership.borrowed_return_surface
- Layer: `ownership`
- Title: Borrowed return ownership gap
- Case count: `1`
- Statuses: `{'CHECK_ERROR': 1}`
- Topics: `{'arrays': 1}`
- Smallest known repro: `1299` -> `audit/leetcode/1299_replace_elements_with_greatest_element_on_right_side.sifr` (14 lines)
- Repro stderr excerpt: `cannot return borrowed parameter`

### stdlib.python_builtin_signature_surface
- Layer: `stdlib_runtime`
- Title: Python builtin signature/parity gap
- Case count: `1`
- Statuses: `{'CHECK_ERROR': 1}`
- Topics: `{'math': 1}`
- Smallest known repro: `2235` -> `audit/leetcode/2235_add_two_integers.sifr` (10 lines)
- Repro stderr excerpt: `sum() takes exactly 1 argument`

## Spot Audit

- Passed: `True`
- Accuracy: `100%`
- Threshold: `90%`

- `0003` expected `stdlib.python_module_surface` got `stdlib.python_module_surface` matched=`True`
- `0017` expected `frontend.nested_function_annotation_support` got `frontend.nested_function_annotation_support` matched=`True`
- `0052` expected `lowering.unsupported_ast_shape` got `lowering.unsupported_ast_shape` matched=`True`
- `0069` expected `codegen.mutable_binding_emission` got `codegen.mutable_binding_emission` matched=`True`
- `0100` expected `type_system.recursive_node_forward_reference` got `type_system.recursive_node_forward_reference` matched=`True`
- `0207` expected `lowering.destructuring_target_support` got `lowering.destructuring_target_support` matched=`True`
- `0238` expected `type_system.optional_narrowing_and_union_ops` got `type_system.optional_narrowing_and_union_ops` matched=`True`
- `0295` expected `lowering.destructuring_target_support` got `lowering.destructuring_target_support` matched=`True`
- `0502` expected `stdlib.python_module_surface` got `stdlib.python_module_surface` matched=`True`
- `0746` expected `type_system.optional_narrowing_and_union_ops` got `type_system.optional_narrowing_and_union_ops` matched=`True`
- `0912` expected `frontend.nested_function_annotation_support` got `frontend.nested_function_annotation_support` matched=`True`
- `1299` expected `ownership.borrowed_return_surface` got `ownership.borrowed_return_surface` matched=`True`
- `1456` expected `type_system.optional_narrowing_and_union_ops` got `type_system.optional_narrowing_and_union_ops` matched=`True`
- `2235` expected `stdlib.python_builtin_signature_surface` got `stdlib.python_builtin_signature_surface` matched=`True`
