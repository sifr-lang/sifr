# Signature Adaptation Review Fixes Targeted Validation (2026-04-02)

| Fixture | Check | Run | Notes |
|---|---|---|---|
| `0018_4sum` | FAIL | SKIP | type error: undefined variable: 's' type error: unsupported operand type(s) for *: 'int | None' and 'int' type error: unsupported operand type(s) for : 'int | None' and 'int | None' type error: unsupported operand type(s) for -: 'int' and 'int | None'  |
| `0025_reverse_nodes_in_k_group` | FAIL | SKIP | type error: attribute access '.next' is not supported as an expression; use as a method call type error: attribute access '.next' is not supported as an expression; use as a method call type error: cannot return borrowed parameter `curr`: borrowed parameters c |
| `0131_palindrome_partitioning` | PASS | PASS | ok |
| `0202_happy_number` | PASS | PASS | ok |
| `0286_walls_and_gates` | FAIL | SKIP | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' type error: cannot unpack non-tuple type 'T' type error: undefined variable: 'r' type error: undefin |
| `0647_palindromic_substrings` | PASS | PASS | ok |
| `0665_non_decreasing_array` | FAIL | SKIP | type error: list subscript assignment index must be 'int', got '0'  |
| `0680_valid_palindrome_ii` | PASS | PASS | ok |
| `0698_partition_to_k_equal_sum_subsets` | PASS | PASS | ok |
| `0909_snakes_and_ladders` | FAIL | SKIP | type error: argument 1 ('val') of deque.append(): expected 'T', got 'list[int]' type error: cannot unpack non-tuple type 'T' type error: if condition must be bool or collection/string truthiness, got 'int' type error: undefined variable: 'nextSquare' type erro |
| `1448_count_good_nodes_in_binary_tree` | PASS | PASS | ok |
| `2017_grid_game` | PASS | PASS | ok |
