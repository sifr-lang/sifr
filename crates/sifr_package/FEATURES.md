# Sifr Package Feature Notes

package-management requirements record Sifr feature metadata and carries Cargo feature data through normalized metadata and command plans. Full source-level feature activation is reserved for the package-aware CLI integration rules, but the package-management rules model fixes the ownership boundary.

## Ownership Boundary

| Feature surface | Owner | package-management rules behavior |
| --- | --- | --- |
| Cargo feature definitions and optional dependencies | Cargo | read from `cargo metadata`; passed through command plans when selected by callers |
| Sifr source features in `sifr.toml` | Sifr | parsed into `SifrManifest::source_features` and included in package metadata |
| Backend/native feature trust impact | Sifr package validation | backend crates selected by Cargo must be named in `[trust]` before Sifr accepts them |
| Generated Rust cache identity | Sifr package validation | package build cache digests include feature selectors and Cargo/Sifr metadata digests |

## package-management rules Decisions

- Sifr feature names and Cargo feature names are not assumed to be identical.
- Cargo feature activation is represented in `CargoCommandPlan::build` through sorted `--features`, `--all-features`, and `--no-default-features` arguments.
- Backend-only Cargo features may remain Cargo-only unless they affect generated Rust or Sifr import visibility.
- If a future rules maps Sifr feature names to Cargo package features, it must validate missing Cargo packages/features with stable package diagnostics before invoking Cargo build commands.
- Feature-dependent package graphs must continue to derive from Cargo metadata rather than from Sifr's own resolver.

## Current Coverage

| Feature concern | Coverage |
| --- | --- |
| Stable Cargo feature argument ordering | `cargo_backend_integration_tests::cargo_command_plans_preserve_lock_mode_and_feature_semantics` |
| Feature selectors in cache inputs | `cargo_backend_integration_tests::package_build_cache_digest_changes_with_lock_source_and_target_inputs` |
| Backend trust selected by Cargo graph | `cargo_backend_integration_tests::backend_trust_reports_untrusted_direct_backend_crate` |
| Package filter feature-mode placeholder | `package_workspace_query_tests::filters_select_dependency_and_dependent_closures_with_negation` |

## Future Updates Required

Update this file when:

- a Sifr feature maps to a Cargo package feature;
- a backend-only Cargo feature can affect generated Rust;
- a feature influences trust validation, graph digests, package source selection, or package archive contents;
- CLI feature flags begin driving `cargo metadata` and package graph derivation directly.
