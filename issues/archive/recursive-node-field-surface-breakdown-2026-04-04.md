# Recursive Node/Field Surface Breakdown (2026-04-04)

- Source taxonomy: `verification/leetcode/full_corpus_failure_taxonomy_20260404_live_rerun1.json`
- Source diagnostics: `tmp/recursive_node_field_34_diagnostics_20260404.txt`
- Inventory CSV: `verification/leetcode/recursive_node_field_surface_20260404_inventory.csv`
- Cases in bucket: `34`

## Subcategory Breakdown
- `field_expression_access_unsupported`: `24`
- `nullable_function_boundary_signature`: `6`
- `optional_node_in_container_elements`: `2`
- `optional_return_variance_mismatch`: `1`
- `quoted_forward_ref_boundary_mismatch`: `1`

## Resolution Ownership
- `both`: `27`
- `sifr_adaptation`: `5`
- `compiler_fix`: `2`

## Root-Cause Conclusions
- Dominant compiler root cause is missing class-node field expression support (`.next/.left/.right/.val/.children/.prev/.end`) in expression positions.
- Secondary compiler root cause is optional-node container element typing/refinement around BFS tuple/list/deque flows.
- Dominant adaptation root cause is non-canonical fixture signatures/contracts (nullable boundaries, duplicate definitions, quoted forward refs, return optionality mismatch).
- Several fixtures require both: compiler closure to unblock node field reads, then Sifr-canonical rewrites for mutability/ownership/signature strictness.

## Execution Recommendation
- Lane A (compiler): node field-expression surface + nullable container refinement.
- Lane B (adaptation): canonicalize signatures and helpers for the adaptation-only fixtures (`0021`, `0203`, `0606`, `0617`, `0894`).
- Lane C (mixed): revisit the `both` set after Lane A lands; apply targeted fixture rewrites only for residuals.
