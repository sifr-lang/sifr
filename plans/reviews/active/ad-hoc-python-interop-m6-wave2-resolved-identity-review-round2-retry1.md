I've completed a rigorous review of the local M6 wave-2 implementation. Here's my analysis.

## Reviewed surface
- `crates/sifr_package/src/python/bridge_resolution.rs` (new, 217 lines)
- `crates/sifr_package/src/python/bridge_resolution_tests.rs` (new, 237 lines)
- `crates/sifr_driver/src/build/python_bridges.rs` (new, 127 lines)
- Modifications to `sifr_codegen/src/python_interop_plan.rs`, driver `entrypoint.rs`/`mod.rs`, CLI `check_and_package_commands.rs`, `sifr_package/src/lib.rs` and `python/mod.rs`
- Interaction with `bridge_inventory`, `requirements`, `trust_policy`, `environment`, `direct_dependency_scopes`, and lowering gates in `sifr_lowering/src/lower/python_interop.rs`

## Verified invariants against wave criteria

**Criterion 1 — deterministic, valid-identifier, collision-resistant identity**
- `resolved_python_bridge_package_key` uses SHA-256 with domain separator `sifr-python-bridge-package-v1\0` over the stable `SifrPackageId` (`{name}@{version}#{source|"path"}`) — not filesystem-dependent.
- 64 hex chars (256 bits) provides collision resistance; the `p_` prefix guarantees a valid Python identifier segment regardless of leading hex digit.
- Runtime package is `__sifr_bridge__.p_<hex>` — both components are valid identifiers, and the wave test asserts stability, distinctness across differing IDs, and full ASCII-hex composition.

**Criterion 2 — same-package rewrite, third-party requirements, root-authorized trust**
- `resolve_package` classifies `PythonBridgeImport::SamePackage { module }` → `ResolvedPythonBridgeImport::SamePackage { module, runtime_module: "{runtime_package}.{module}" }` after verifying against `known_module_names` (which materializes namespace ancestors via prefix expansion).
- Missing same-package targets produce `PYIMP_INVALID_BRIDGE_SOURCE`; test `unresolved_same_package_bridge_import_is_rejected` covers this.
- `PythonBridgeImport::ThirdParty { root }` becomes `PythonRequirementContribution { kind: PythonRequirementKind::BridgeImport, package_id, source: "{pkg}:{module} imports {root}" }`.
- CLI wires `bridge_graph.requirements` into `derived_python_requirements`, which flows through `canonical_python_requirements` and `validate_python_trust_policy` — dependency-owned bridge roots must still be listed in root `[trust].python`, verified by `dependency_bridge_requirements_remain_root_authorized`.

**Criterion 3 — selected-graph propagation to driver/codegen/cache, public gating preserved**
- `selected_runtime_packages` performs a DFS from the root through `graph.direct_dependency_scopes`, which by construction (`scopes.rs:74`) only holds `dependency.dependency_kind.is_none()` edges — dev-only deps are excluded. The `unselected` package in the wave test confirms unreachable packages contribute neither identities nor requirements.
- Driver `apply_package_python_bridge_metadata` populates `InteropBuildPlan.python.{bridge_packages, required_import_roots}`, and `push_python_plan_cache_key` emits `python.bridge_package=`, `python.bridge_module=`, and `python.bridge_import=` lines including `resolved_package_key`, `runtime_package`, `inventory_digest`, `source_digest`, and `runtime_module`. `materialize.rs:423` folds `interop.cache_key_fragment()` into artifact cache identity — bridge structural changes therefore invalidate the cache.
- Public `bridge.*` decorators remain gated at lowering: `python_interop.rs:417,582` still emit `PYRES_UNIMPLEMENTED_DECLARATION`, and the CLI decorator scan (`check_and_package_commands.rs:306`) still skips `bridge`/`Self` roots so gated targets don't leak into declaration requirements.

## Other checks
- Error accumulation across packages: `resolve_python_bridge_graph` continues iterating on `Err` and returns the aggregated diagnostic vector; requirements built for successful packages up to a failure are correctly discarded because the function returns `Err` before yielding them.
- Determinism: `selected` is a `BTreeSet`; inventories are pre-sorted; requirements are sorted+deduped; classifier uses `BTreeSet<PythonBridgeImport>`.
- File-size guardrail: `entrypoint.rs` is 897 lines (right below 900), `check_and_package_commands.rs` 881, `bridge_resolution.rs` 217, `python_bridges.rs` 127 — all within the cap.
- Duplicate resolution (CLI resolves for requirements, driver resolves for planning) is inefficient but deterministic on identical inputs and does not double-report diagnostics because CLI failures short-circuit before the driver is invoked.
- Root-missing branch returns `PACKAGE_METADATA_PARSE`, a slightly loose code choice for an internal state check, but this is a defensive path unreachable from the CLI in practice and does not affect correctness.

No actionable defect surfaced against the wave-2 requirements, so I do not have grounds to request changes.

VERDICT: SATISFIED
