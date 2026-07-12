I have gathered enough evidence to complete the review.

## Review — PR #2945 (M6 wave 1: package Python bridge inventory)

Scope of what I verified in the actual diff and current tree:

- Discovery is path-locked to `src/python_bridges`; misplaced-root scan composes candidates from manifest source roots and filters lexically against the canonical root (`filesystem.rs:15-33`). Correct.
- `module_name` distinguishes `RootPackageReserved` and `InvalidPath` reasons, then round-trips the module through `sifr_syntax::parse_module_suite` so keywords/non-identifiers/dunders are rejected via a real Python parser (`filesystem.rs:113-172`). Correct.
- `ImportCollector` follows `Import`/`ImportFrom`/`Assign`/`AnnAssign`/walrus targets (including tuple/list destructuring) to grow `importlib_aliases`/`builtins_aliases`/`dynamic_function_aliases`. `DynamicImportVisitor` only fires on `Expr::Call` (fixing the earlier "reference is a call" false positive) and additionally handles `getattr(importlib, 'import_module')` dispatch (`imports.rs`). Correct.
- Reserved-namespace check fires on both `Import` and `ImportFrom`, on the original module name (asname does not launder it) (`imports.rs:786-799`, `imports.rs:806-810`).
- Same-package classification: `from bridge import X`, `from bridge.<pkg> import <names>`, `import bridge.<pkg>...`, `from .<subpkg> import <names>`, and `from . import <names>` all funnel through `insert_same_package` (which materializes every ancestor prefix) plus `insert_known_children` (filtered on `known_modules`) — resolving the round-2 concern about explicit-module submodule edges being dropped. Verified by `same_package_imports_include_package_and_intermediate_ancestors`.
- Third-party imports are recorded by root only (`classify_absolute`); the `bridge` root itself is silently dropped. Correct.
- Digests use SHA-256 (`sha2` crate); source digest hashes UTF-8 bytes; inventory digest hashes `serde_json::to_vec(&(PYTHON_BRIDGE_ROOT, &modules))`. `PythonBridgeImport` derives `Ord`, `imports` is built from a `BTreeSet`, and modules are sorted by name → serialization is deterministic. Determinism preserved between projection and packaging (both go through `discover_python_bridge_inventory_at`).
- Writer/repair/validate/projection integration: `repair_python_bridge_inventory` is wired into `projection.rs::repair_projection` (round-1 #1 fix), `default_cargo_include_entries` and `projection_bridge::cargo_include_entries` both add `src/**/*.py` and `src/python_bridges/__sifr_inventory__.json`, `projection_diagnostics` flags the include drift only when bridge sources exist, and `validate_package_archive` requires the fresh inventory manifest. End-to-end coverage via `projection_repair_generates_canonical_python_bridge_inventory` and `package_validation_rejects_missing_and_stale_generated_inventory`.
- `PackageDiagnosticOrigin::PythonBridgeSource` is threaded to the driver renderer with `origin_kind`, `cargo_package_id`, `bridge_path` args (`sifr_driver/src/diagnostics.rs:117-133`).
- Registry, catalog, baseline coverage, docs page, and check-compact baseline all reflect the new `SIFR-PYIMP-0002` (18 → 18 errors bump).
- Transfer guardrails updated with the new bridge-inventory reads and the projection line-number shift.
- No user-triggerable panic paths: `unwrap`/`expect` only appear in tests. Filesystem/UTF-8/serde/parser errors are all funneled into `PackageDiagnostic::invalid_python_bridge_source`.
- Module layout is decomposed (`filesystem.rs`, `imports.rs`, `import_classification.rs`, `mod.rs`); the largest is `mod.rs` at 357 lines — well under the 900-line guardrail.

## Actionable findings

### MEDIUM — Canonical bridge root symlink escapes hermeticity

`filesystem::collect_python_paths` (`crates/sifr_package/src/python/bridge_inventory/filesystem.rs:52`) enters the canonical root with `fs::read_dir(directory)`, which transparently follows a symlink. `is_symlink()` is only checked on child `DirEntry`s (line 97), and `misplaced_root_diagnostics` builds candidates from `package_root.join("python_bridges")` and `package_root.join(source_root).join("python_bridges")` — the canonical `src/python_bridges` is explicitly filtered out of that candidate set, so it is never examined. `discover_python_bridge_inventory_at` (`.../mod.rs:63`) also does not stat the root.

Failure scenario: `rm -rf src/python_bridges && ln -s /somewhere/outside/repo src/python_bridges`. `discover_python_bridge_inventory` returns `Ok(...)` with the external contents inventoried as `src/python_bridges/*.py`, source digests computed from the external bytes, `__sifr_inventory__.json` generated at the symlinked location, and packaging succeeding. The wave objective requires the "fixed package-owned root `src/python_bridges/**/*.py`" and "reject … symlinks"; the child-only check leaves the root as a hermeticity bypass. The existing `symbolic_link_bridge_source_is_rejected` test only covers a symlinked file inside the (real) canonical root.

Fix: stat `root` with `fs::symlink_metadata` in `discover_python_bridge_inventory_at` (or at the entry point of `collect_python_paths`); if `is_symlink()`, emit `PYIMP_INVALID_BRIDGE_SOURCE` with the existing "symbolic links are not allowed in bridge source roots" message and add a `#[cfg(unix)]` test that symlinks the whole `src/python_bridges` directory.

Everything else — misplaced-root diagnostics, dynamic-import alias propagation, callable-reference false-positive avoidance, explicit-module same-package edges, ancestor closure, digest stability, projection/packaging integration, diagnostic/catalog/baseline/docs synchronization, transfer-guardrail update, and layout — checks out against the wave requirements and the resolutions claimed in rounds 1–3.

VERDICT: CHANGES_REQUESTED
