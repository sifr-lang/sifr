# Demo Cleanup Reasoning Report

## Removed Demos

| Removed demo | Replacement kept | Reason |
|---|---|---|
| `demos/m1_env_demo.sifr` | `demos/m30_1a_env_parity_demo/main.sifr` | Legacy single-module parity demo; Phase 30 env parity demo is broader, newer, and already the canonical validation target. |
| `demos/m2_bytes_demo.sifr` | `demos/m30_1a_bytes_parity_demo/main.sifr` | Legacy single-module parity demo; superseded by the newer Phase 30 bytes parity demo. |
| `demos/m3_base64_demo.sifr` | `demos/m30_1a_base64_parity_demo/main.sifr` | Legacy single-module parity demo; superseded by the newer Phase 30 base64 parity demo. |
| `demos/m4_math_demo.sifr` | `demos/m30_1b_math_parity_demo/main.sifr` | Legacy single-module parity demo; superseded by the newer Phase 30 math parity demo. |
| `demos/m5_hashlib_demo.sifr` | `demos/m30_1a_hashlib_parity_demo/main.sifr` | Legacy single-module parity demo; superseded by the newer Phase 30 hashlib parity demo. |
| `demos/milestone_codegen_quality_demo.sifr` | `demos/milestone_codegen_quality_v2_demo.sifr` | Older codegen quality demo with no active references; the v2 demo is the retained successor. |
| `demos/milestone_codegen_quality_demo.rs` | `demos/milestone_codegen_quality_v2_demo.sifr` | Rust-only companion for the older codegen quality milestone; no active references remain and the milestone has a newer retained successor. |
| `demos/milestone_narrowing_v2_demo.sifr` | `demos/milestone_narrowing_v3_demo.sifr` | Explicitly superseded by the retained v3 narrowing demo. |

## Duplicate Demo Fixtures Found But Not Removed

These are exact-content duplicates inside retained multi-file demo suites. They were left in place because removing them would require suite-specific fixture surgery rather than milestone cleanup:

- `demos/m23_3_project_test_discovery_parity_contract_demo/negative_cases/reachable_parse_error/helper.sifr`
- `demos/m23_4_invocation_scoped_temp_workspace_isolation_demo/negative_cases/reachable_parse_error/helper.sifr`
- `demos/m23_5_graph_isolation_regression_matrix_demo/negative_cases/reachable_parse_error/helper.sifr`
- `demos/m19_1_dependency_safe_module_ordering_demo/negative_cases/a.sifr`
- `demos/m23_2_deterministic_module_graph_cycle_diagnostics_demo/negative_cases/module_cycle/a.sifr`
- `demos/m23_5_graph_isolation_regression_matrix_demo/negative_cases/module_cycle/a.sifr`
- `demos/m23_2_deterministic_module_graph_cycle_diagnostics_demo/negative_cases/module_cycle/b.sifr`
- `demos/m23_5_graph_isolation_regression_matrix_demo/negative_cases/module_cycle/b.sifr`
- `demos/m23_2_deterministic_module_graph_cycle_diagnostics_demo/negative_cases/module_cycle/c.sifr`
- `demos/m23_5_graph_isolation_regression_matrix_demo/negative_cases/module_cycle/c.sifr`
- `demos/m19_1_dependency_safe_module_ordering_demo/negative_cases/main.sifr`
- `demos/m23_5_graph_isolation_regression_matrix_demo/negative_cases/module_cycle/main.sifr`
- `demos/m17_4_import_form_semantics_closure_demo/negative_cases/unsupported_multi_relative.sifr`
- `demos/m18_4_cli_resolver_trigger_matrix_closure_demo/negative_cases/main_multi_level_relative.sifr`
- `demos/m17_4_import_form_semantics_closure_demo/negative_cases/unsupported_bare_relative.sifr`
- `demos/m18_4_cli_resolver_trigger_matrix_closure_demo/negative_cases/main_bare_relative.sifr`
