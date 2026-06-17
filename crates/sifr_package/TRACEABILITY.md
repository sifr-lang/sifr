# sifr_package Traceability

package-management rules reuses Cargo behavior where Cargo already owns dependency resolution, source materialization, lockfiles, workspaces, publishing, and vendoring. Sifr adapts those facts into compiler-facing package metadata and validates Sifr-specific semantics.

Status values:

- `adapted`: Cargo-shaped behavior is exercised through Sifr-owned package models or command plans.
- `ported`: Cargo behavior shape is reimplemented directly against Sifr public behavior.
- `non-port`: Cargo behavior is intentionally delegated or excluded from local validation.

## Behavior Matrix

| Cargo behavior category | Status | Rules area | Sifr coverage |
| --- | --- | --- | --- |
| metadata package ordering | adapted | metadata adapter | `tests::shuffled_cargo_metadata_has_stable_digest` |
| package metadata discovery | adapted | metadata adapter | `tests::pure_sifr_package_graph_derives_from_cargo_metadata` |
| manifest pointer resolution | adapted | metadata adapter | `[package.metadata.sifr].manifest` resolved relative to selected package root |
| malformed/missing Sifr manifest | adapted | metadata adapter | `tests::missing_manifest_reports_package_0002` |
| unsupported Sifr compiler metadata in Cargo metadata | adapted | metadata adapter | `tests::misplaced_compiler_metadata_reports_package_0003` |
| pure marker target validation | adapted | metadata adapter | `tests::non_trivial_pure_marker_reports_package_0501` |
| Rust-backed Sifr package classification | adapted | metadata adapter/cargo backend integration | `PackageClassification::RustBackedSifr`; backend trust tests |
| resolved dependency rename identity | adapted | dependency scope | `derive_direct_dependency_scopes` uses `resolve.nodes[].deps[].name` |
| multiple selected versions | adapted | dependency scope | `same_import_root_can_resolve_to_different_versions_in_different_scopes` |
| same-scope duplicate roots | adapted | dependency scope | `duplicate_direct_import_root_in_one_scope_reports_0201` |
| dependency aliases | adapted | dependency scope/package source map | `direct_dependency_aliases_allow_same_export_root_in_one_scope`; `alias_import_root_remaps_to_dependency_export_root` |
| package instance type identity | adapted | dependency scope | `type_identity_mismatch_reports_0204_for_distinct_package_instances` |
| package-aware source map | adapted | package source map | `package_source_map_resolves_own_and_direct_dependency_modules` |
| transitive import boundary | adapted | package source map | `transitive_dependency_import_reports_0202` |
| private dependency module boundary | adapted | package source map | `private_dependency_module_reports_0203` |
| lock mode arguments | adapted | cargo backend integration | `cargo_command_plans_preserve_lock_mode_and_feature_semantics` |
| offline/frozen unavailable source | adapted | cargo backend integration | `offline_mode_reports_missing_sifr_source_package` |
| Cargo failure diagnostics | adapted | cargo backend integration | `cargo_failure_mapping_redacts_private_credentials` |
| backend native trust policy | adapted | cargo backend integration | `backend_trust_reports_untrusted_direct_backend_crate` |
| stale trust entries | adapted | cargo backend integration | `backend_trust_rejects_stale_non_direct_trust_entry` |
| reproducible package build cache inputs | adapted | cargo backend integration | `package_build_cache_digest_changes_with_lock_source_and_target_inputs` |
| workspace member selection | adapted | workspace query | `explicit_rust_only_selection_reports_0102` |
| Rust-only workspace member depending on Sifr | adapted | workspace query | `rust_only_member_depending_on_sifr_reports_0106` |
| duplicate workspace import roots | adapted | workspace query | `workspace_duplicate_import_roots_report_0602` |
| package filters and dependency closures | adapted | workspace query | `filters_select_dependency_and_dependent_closures_with_negation` |
| changed-package selection | adapted | workspace query | `changed_file_mapping_reports_0603` |
| outdated source classification | adapted | workspace query | `outdated_query_classifies_path_registry_and_git_sources_read_only` |
| unsupported outdated source | adapted | workspace query | `outdated_unknown_source_reports_0604` |
| package archive required files | adapted | publish and archive | `archive_missing_required_entry_reports_0403` |
| missing `.sifr` archive source | adapted | publish and archive | `archive_missing_sifr_source_reports_0401` |
| archive traversal rejection | adapted | publish and archive | `archive_traversal_reports_0402` |
| package dry-run delegation | adapted | publish and archive | `package_dry_run_includes_cargo_package_and_publish_dry_run_commands` |
| publish/vendor command delegation | adapted | publish and archive | `publish_and_vendor_plans_delegate_to_cargo_with_redaction_ready_commands` |
| package-management rules fixture category coverage | ported | verification matrix | `verification/areas/package_management/data/package_e2e_fixture_matrix.json`; `verification/areas/package_management/tools/check_package_manager_guardrails.py` |
| organization demo repository subrepos | ported | demo repository corpus | `verification/areas/package_management/data/package_demo_repositories.json`; `verification/areas/package_management/corpora/demo_repositories/`; `package_demo_subrepos_cover_required_org_repos` |

## Explicit Non-Port Decisions

| Cargo behavior category | Status | Reason | Sifr coverage |
| --- | --- | --- | --- |
| live registry publish/upload | non-port | Local validation must not mutate external registries or require credentials. Sifr validates package preflight and delegates upload to Cargo. | publish command plans; credential redaction tests |
| yanking live registry versions | non-port | Yank is Cargo registry behavior and requires live registry state. package-management requirements record delegation only. | Cargo command surface audit |
| login/logout credential storage | non-port | Credential storage belongs to Cargo. Sifr must avoid logging credentials and must redact Cargo failures. | `cargo_failure_mapping_redacts_private_credentials` |
| alternate/private registry protocol tests | non-port | Protocol compatibility is Cargo-owned. Sifr consumes source ids opaquely and reports unsupported/unknown metadata explicitly. | source classification and credential diagnostics |
| Cargo registry/Git cache layout walking | non-port | Sifr must not rely on Cargo cache internals. | package-manager guardrail and dependency audit |

## Readiness Evidence

The package-management readiness matrix is `verification/areas/package_management/data/package_e2e_fixture_matrix.json`. It maps pure Sifr packages, Rust-backed packages, workspaces, path/Git/registry sources, multiple-version graphs, aliases, and publishing to concrete Sifr tests or explicit non-port decisions.

The organization demo repositories are `sifr-lang/sifr-demo-*` git submodules under `verification/areas/package_management/corpora/demo_repositories/` and are indexed by `verification/areas/package_management/data/package_demo_repositories.json`. They preserve the required package shapes locally while exercising the same subrepo workflow used by the rest of Sifr.
