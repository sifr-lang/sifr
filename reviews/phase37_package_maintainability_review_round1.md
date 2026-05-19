

# Phase 37 Package Manager Maintainability Architecture Review

## Executive Summary

The proposed architecture is **sound and well-grounded** in existing Sifr conventions. One-crate with anti-corruption modules is the correct choice over many crates or direct embedding. The guardrails and module decomposition map well to proven patterns from `sifr_diagnostics` and `sifr_hir`. However, there are several concrete gaps and risks that need to be addressed before Phase 37 is ready for implementation. The most critical are: missing integration points for `sifr_frontend`, ambiguous ownership of `PackageSourceMap` between Phase 37 and Phase 36 continuation, and an incomplete plan-execute enforcement mechanism.

---

## 1. One Crate vs. Many Crates: Verdict

**Verdict: One crate is correct for Phase 37.**

The anti-corruption layer strategy works best inside a single crate because:
- All adapters (`version_semver`, `solver_pubgrub`, `graph_petgraph`, etc.) need to be reachable from the pure core without cross-crate public API leakage
- The plan-execute pattern requires that pure core types can be instantiated without depending on IO adapters
- Later splitting is achievable if the internal module boundaries are enforced at the source level (not just public API)

**However**: The "decompose into modules for later crate-split" part of the proposal is underspecified. Without explicit boundaries enforced by a guardrail script (like the HIR maintainability script), "public facade types" becomes aspirational rather than enforced. The architecture doc should explicitly reference an enforcement mechanism.

---

## 2. Anti-Corruption Layer: What Works

### 2.1 Correct approach

The proposal correctly identifies that **no external package-manager type crosses the public boundary** except stable generic primitives (Path, Url, String). This is the correct anti-corruption pattern. The adapter modules (`version_semver`, `solver_pubgrub`, etc.) isolate the external dependencies so the pure core never imports them directly.

The `sifr_diagnostics` crate validates this pattern: `codes.rs`, `model/mod.rs`, `render/mod.rs` are all internal modules; the public API in `lib.rs` exposes only Sifr-owned types. External crates (`schemars`, `serde_json`) are imported only inside their respective adapters and never leak to consumers.

### 2.2 Adapter naming is appropriate

The proposed adapter names (`version_semver`, `solver_pubgrub`, `manifest_edit`, `graph_petgraph`, `git_gix`, `registry_http`, `archive_tar_zstd`, `checksum_sha2`) follow a clear naming convention: `<domain>_<adapter_lib>`. This mirrors the pattern used in HIR lowering where a specific module is responsible for a specific semantic domain.

### 2.3 Public API guardrail requirement

The "Public API guardrail: no `pub use` or public fields exposing external crate types" is the most important enforcement point. This must be automated — it cannot be a code review convention. I recommend a guardrail script (see section 5).

---

## 3. Gaps and Risks

### 3.1 `sifr_frontend` integration point is missing

The proposed module map lists `imports::source_map` and `backend::{cargo_plan, ...}` but does not specify how `PackageSourceMap` and `CargoBackendPlan` integrate with the existing `sifr_frontend` query API. The phase doc states that `sifr_driver` consumes package plans and frontend/HIR consume `PackageSourceMap`, but:

- **Who owns `ModuleResolver` changes?** The phase 37 doc specifies `ModuleOrigin::PackageSource` but doesn't map this to `sifr_frontend`'s existing query infrastructure. Does `sifr_frontend` gain a new query entrypoint? Does `sifr_hir` gain new lowering paths?
- **What happens to `sifr_driver::project/assembly.rs`?** The existing project assembly logic discovers source roots and builds the import closure. Phase 37 must specify how this interacts with `PackageSourceMap` — does it replace the current source discovery, or does it augment it?

**Recommendation**: Add a module `frontend::{query, assembly}` to the module map that explicitly documents the integration contract. The `sifr_frontend` crate needs a new query entrypoint `package_graph()` that returns the resolved `PackageSourceMap`, and existing project lowering must call this before constructing the import closure.

### 3.2 `OperationPlan` dry-run enforcement is incomplete

The proposal states "mutating CLI code must expose a dry-run `OperationPlan` before executing." This is a good architectural goal but the implementation contract is underspecified. Specifically:

- **What does an `OperationPlan` look like?** It must cover at minimum: manifest edits, lockfile writes, cache operations, Git fetches, and generated Cargo materialization. But the proposal doesn't define the struct.
- **How is dry-run enforced?** `sifr add --dry-run` must produce identical plan output to `sifr add` (without `--dry-run`) except no writes occur. This means the operation layer needs a mode where the plan is produced but not applied. In `sifr_diagnostics`, this pattern is used for `--dry-run=json` but the exact boundary between "plan" and "execute" is not specified.
- **What about non-CLI paths?** IDE/editor integration uses the same resolver/cache APIs. Does the plan-execute pattern apply there too? If so, the `operations` module needs to expose plan types that the LSP adapter can consume.

**Recommendation**: Define `OperationPlan` as a struct with typed fields for each class of mutation (manifest edits, lockfile writes, cache actions, fetch actions, generated Cargo changes). The `operations` module should have an `OperationPlanner` trait that the CLI and LSP both implement — the CLI planner logs to stdout, the LSP planner emits diagnostics via `sifr_diagnostics`.

### 3.3 Module boundary for `diagnostics` is ambiguous

The proposal lists `diagnostics::{codes, origins, redaction}` and says "maybe integrate with existing sifr_diagnostics." This ambiguity is a problem:

- `sifr_diagnostics` already owns `SifrDiagnostic`, `DiagnosticCode`, `Severity`, `DiagnosticSink`, and the render schema.
- `SIFR-PACKAGE-*` codes are reserved in the phase doc but not yet defined in `sifr_diagnostics`.
- If `sifr_package::diagnostics` is a separate module, it risks creating a parallel diagnostic infrastructure that doesn't integrate with the existing `sifr_diagnostics::render` pipeline.

**Recommendation**: The `SIFR-PACKAGE-*` codes should be added to `sifr_diagnostics::codes` during `milestone_37_1`, not created in a new module. The `sifr_package::diagnostics` module should be `sifr_package::diag` and should construct `SifrDiagnostic` values using `sifr_diagnostics` types. The phase doc should specify that `SIFR-PACKAGE-*` codes are additive to the existing `SIFR-*` namespace, not a separate diagnostic family.

### 3.4 `test_support` behind `cfg(test)` is correct but needs clarity

The proposal says `test_support` is behind `cfg(test)` for fake registry/git/cache and resolver DSL. This matches existing Sifr patterns (`#[cfg(test)]` in individual crate files). However, the proposal doesn't address the test infrastructure problem: the phase doc mentions a "fake sparse registry server" and "temporary Git repository helpers" but these are high-level tests, not unit tests.

**Recommendation**: Add a `test_assets/` directory at `crates/sifr_package/test_assets/` for:
- Registry server mock (an in-process HTTP server using `tiny_http` or `axum` for tests)
- Git repo fixtures (on-disk temp repos created per test)
- Fixture packages (minimal `.sifr` packages for integration testing)

This matches the pattern of `tests/e2e/` fixtures in the main workspace.

### 3.5 The `operations` module is too large as a single module

The proposed module map lists `operations::{add, remove, update, sync, fetch, tree, outdated, publish, plan}`. These are 9 separate operations, each of which is significant. In `sifr_driver`, the equivalent decomposition uses `build/mod.rs`, `test_runner/mod.rs`, etc. The `operations` module as a container for 9 operations is the most likely monolith to emerge.

**Recommendation**: Split `operations` into:
- `ops::{add, remove, update}` — manifest-mutating operations (each a separate module)
- `ops::{sync, fetch}` — resolution-and-cache operations
- `ops::{tree, outdated}` — read-only graph operations
- `ops::{publish}` — registry operations
- `ops::plan` — the `OperationPlan` type and dry-run logic

Or alternatively: `ops/mutate.rs`, `ops/resolve.rs`, `ops/read.rs`, `ops/publish.rs` as four focused modules. The guardrail script should set explicit line limits on each.

### 3.6 Upgrade/maintenance process is underspecified for adapter contracts

The proposal says "External dependency upgrades require adapter contract tests, SAT/metamorphic suite, traceability review against upstream Cargo/uv tests, and architecture note if behavior changes." This is good but the specific trigger for "when to run the upgrade process" is missing.

**Recommendation**: The guardrail script should enforce a dependency audit file at `crates/sifr_package/DEPENDENCY_AUDIT.md` that records, for each external crate:
- Current pinned version and source
- Last upgrade date
- Test coverage for adapter contract
- Any intentional behavioral differences from upstream

When `Cargo.lock` changes (indicating an upstream upgrade), the guardrail script should fail unless the audit file is updated. This matches the pattern used in `check_hir_maintainability_guardrails.py` for checklist documentation.

---

## 4. Guardrail Recommendations

### 4.1 Create `check_package_manager_guardrails.py`

Analogous to `check_hir_maintainability_guardrails.py`, this script should enforce:

1. **Module/file size limits** (key ones):
   - `solver/mod.rs` — 400 lines
   - `manifest/mod.rs` — 400 lines
   - `lockfile/mod.rs` — 400 lines
   - `resolver/` adapters — 200 lines each
   - No single file > 600 lines

2. **Dependency-boundary guardrail**: Automated check that each external crate only appears in its designated adapter module. This can be enforced by a `pubgrub` search that reports any `.rs` file outside the adapter directory importing `pubgrub`.

3. **Public API guardrail**: Check that no `pub use` in the public API surface exposes an external crate type. This can be done with a script that reads the module tree and verifies the re-export pattern.

4. **Plan/execute guardrail**: Verify that each CLI command path in `sifr` (the binary) goes through an `OperationPlan` before executing mutations.

5. **Traceability guardrail**: Verify that `crates/sifr_package/TRACEABILITY.md` exists and maps each borrowed Cargo/uv correctness category to a Sifr test file.

### 4.2 Specific guardrail for the adapter pattern

The script should verify that for each adapter module (e.g., `version_semver`), only the designated external crate is imported in that module's subtree. A simple approach: each adapter module gets a `#[cfg(test)]` module that verifies no external types escape the adapter boundary.

```rust
#[cfg(test)]
mod adapter_boundary_tests {
    // Verify that semver::Version is not pub anywhere in version/
    // This test fails if someone accidentally adds pub semver::Version
    // or pub use of an external type in the version module subtree
}
```

### 4.3 Module map enforcement

The guardrail script should verify that the actual module tree matches the documented module map. If a new module is added without updating the phase doc, the script should warn.

---

## 5. Concrete Phase Doc Changes

### 5.1 Add integration point section

After "Module Map" (section 5 of the proposed model), add:

> **Integration with `sifr_frontend`**
> 
> `PackageSourceMap` is produced by the `imports::source_map` module and consumed by `sifr_frontend::project` for import-closure construction. `sifr_frontend` exposes a `package_graph()` query entrypoint that returns the active `ResolvedPackageGraph`. `sifr_driver` uses this to construct `RootedEntrypointPlan` with package-aware origins. The `ModuleOrigin` enum in `sifr_hir` gains a `PackageSource` variant with the fields specified in the phase 37 doc.
> 
> `CargoBackendPlan` is produced by `backend::cargo_plan` and consumed by `sifr_driver::build::cargo_manifest` for generated Cargo materialization. `sifr_driver` calls `CargoBackendPlan::verify()` against generated `Cargo.lock` as part of the build lock validation.

### 5.2 Define `OperationPlan` structure

Add to the module map section:

> **OperationPlan structure**
> 
> ```rust
> struct OperationPlan {
>     manifest_edits: Vec<ManifestEdit>,
>     lockfile_writes: Vec<LockfileWrite>,
>     cache_actions: Vec<CacheAction>,
>     fetch_actions: Vec<FetchAction>,
>     generated_cargo_changes: Vec<CargoEdit>,
>     diagnostics: Vec<SifrDiagnostic>,
> }
> 
> enum ManifestEdit {
>     AddDependency { path: PathBuf, spec: DependencySpec },
>     RemoveDependency { path: PathBuf, alias: DependencyAlias },
>     UpdateDependency { path: PathBuf, alias: DependencyAlias, new_spec: DependencySpec },
> }
> 
> enum CacheAction {
>     Extract { id: PackageId, source: SourceKind, path: PathBuf },
>     Verify { id: PackageId, expected_checksum: String },
>     Skip { id: PackageId, reason: String },
> }
> 
> // ... etc
> ```
> 
> All CLI commands use `OperationPlanner` to produce an `OperationPlan` before executing mutations. `--dry-run` emits the plan as JSON without applying it. The plan is also used for IDE/editor diagnostics where mutations are logged but not applied.

### 5.3 Clarify diagnostics ownership

Change the module map entry from:
```
diagnostics::{codes, origins, redaction} maybe integrate with existing sifr_diagnostics
```

To:
```
diag::{codes, origins, redaction}
```

And add:
> `SIFR-PACKAGE-*` codes are added to `sifr_diagnostics::codes` during `milestone_37_1`. The `diag` module constructs `SifrDiagnostic` values and uses `sifr_diagnostics::render` for output. No separate diagnostic rendering pipeline exists in `sifr_package`.

### 5.4 Add a `FEATURES.md` document

Create `crates/sifr_package/FEATURES.md` (not checked into the phase doc itself) that maps each adapter feature flag to the external crate feature it gates. This prevents "feature flag creep" where adapter modules accumulate features that aren't actually needed.

### 5.5 Add the guardrail enforcement section

Add to the phase doc's Milestone section (specifically `milestone_37_7`):

> **Maintainability guardrail enforcement**
> 
> The `check_package_manager_guardrails.py` script runs as part of local validation. It enforces:
> 
> 1. File size limits for all `sifr_package` modules (see `crates/sifr_package/MAX_LINES_BY_FILE` in the script)
> 2. Dependency-boundary enforcement: `pubgrub` only in `solver/pubgrub_adapter`, `semver` only in `version/semver_adapter`, etc.
> 3. Public API enforcement: no `pub use` exposing external crate types from `sifr_package` public modules
> 4. Plan/execute enforcement: all CLI mutation commands in `sifr` (the binary crate) call through `OperationPlan` before executing
> 5. Traceability enforcement: `crates/sifr_package/TRACEABILITY.md` exists and is updated with each correctness test addition
> 
> The guardrail script is the single source of truth for module boundary enforcement. Code review is supplementary, not primary.

### 5.6 Add upgrade process documentation

Add a section to `milestone_37_2` scope:

> **Dependency audit and upgrade process**
> 
> A `DEPENDENCY_AUDIT.md` file in `crates/sifr_package` records for each external crate:
> - Pinned version and source
> - Adapter module that owns the dependency
> - Adapter contract test coverage
> - Intentional behavioral differences from upstream
> - Last upgrade date
> 
> When `Cargo.lock` changes for a `sifr_package` dependency, the guardrail script fails until `DEPENDENCY_AUDIT.md` is updated. Upgrades require running the SAT oracle suite and property tests with recorded seeds.

---

## 6. Module Boundary Recommendations

The proposed module map is good but needs these adjustments:

| Proposed | Recommended | Rationale |
|---|---|---|
| `operations::{add, remove, update, ...}` | `ops/mutate.rs`, `ops/resolve.rs`, `ops/read.rs`, `ops/publish.rs` | Avoids a monolithic `operations` directory; groups operations by mutation class |
| `diagnostics::{codes, origins, redaction}` | `diag::{codes, origins, redaction}` | Integrates with `sifr_diagnostics`; `codes` adds to existing namespace |
| `test_support` | `test_assets/` + `#[cfg(test)] mod tests` | Keeps test assets discoverable and separate from unit test modules |

Additionally, add these modules that are missing:
- `frontend.rs` — integration with `sifr_frontend` (query API entrypoints)
- `plan.rs` — `OperationPlan` struct and `OperationPlanner` trait
- `trust.rs` — may be needed as a separate module from `backend` if trust validation is complex

---

## 7. Risks That Remain

### 7.1 SAT oracle implementation complexity

The SAT oracle is described in detail but the implementation cost is high. The phase doc specifies 15 explicit constraints that the oracle must encode. This is a non-trivial piece of formal methods work. The risk is that `milestone_37_2` scope grows to include both the solver integration and the SAT oracle, making the milestone too large.

**Mitigation**: Implement the SAT oracle as a separate, well-tested component first. Write property-based tests before implementing the solver adapter. The oracle's correctness should not depend on the resolver being complete — it should be testable with mock `ResolvedPackageGraph` inputs.

### 7.2 `gix` complexity and async runtime integration

`gix` is a large, complex library. The phase doc says "Sifr must not mix `gix` and `git2`" but doesn't specify how `gix` integrates with Sifr's async runtime. The `gix` library has its own async model (using `maybe-async`) which may conflict with Sifr's `tokio`-based async.

**Recommendation**: Before `milestone_37_5`, add a specific design issue that addresses `gix` async runtime integration. The adapter should be concrete (not trait-based) for the initial implementation since there's only one Git implementation.

### 7.3 Feature expansion fixed-point termination

The phase doc says "Feature expansion runs to a deterministic fixed point before PubGrub input is built" and "Cyclic feature aliases are rejected; cyclic optional dependency graphs are rejected unless expansion reaches a stable finite fixed point." The termination guarantee for cyclic optional dependency expansion is non-trivial. A cyclic optional dependency graph where A activates B, B activates C, C activates A could expand forever.

**Mitigation**: The feature expansion algorithm should have an explicit iteration bound derived from the strongly connected components of the feature graph. If the expansion doesn't stabilize within N iterations (where N = number of packages × number of features), the algorithm emits `SIFR-PACKAGE-0103` and fails. This bound should be documented in the resolver architecture section.

### 7.4 Generated Cargo verification vs. cargo's own resolution

The phase doc says "If Cargo resolution differs from the backend dependency section recorded in `sifr.lock`, the build fails instead of silently updating native dependencies." This is correct but the implementation needs to handle the case where the generated `Cargo.lock` differs in non-essential ways (e.g., only feature ordering changed, or only non-Sifr-affecting fields changed).

**Recommendation**: Add a `CargoLockDiff` struct that categorizes differences into:
- **Critical** (package name, version, source, checksum, features differ) → fail
- **Non-critical** (only ordering, timestamps, optional fields differ) → warn, don't fail

The `backend/cargo_lock_verify` module should implement this categorization.

---

## 8. Summary of Recommendations

| Priority | Recommendation | Where |
|---|---|---|
| **Critical** | Define `OperationPlan` structure explicitly | Phase doc / module map |
| **Critical** | Add `sifr_frontend` integration point section | Phase doc |
| **Critical** | Clarify diagnostics ownership (SIFR-PACKAGE-* in sifr_diagnostics) | Phase doc / module map |
| **High** | Create `check_package_manager_guardrails.py` | `scripts/` |
| **High** | Split `operations` into focused sub-modules | Module map |
| **High** | Define `CargoLockDiff` categorization | Phase doc / backend module |
| **Medium** | Add dependency audit file and upgrade process | Phase doc / milestone scope |
| **Medium** | Document feature expansion termination bound | Resolver architecture |
| **Medium** | Add `gix` async runtime integration design issue | Phase doc / roadmap |
| **Low** | Add `FEATURES.md` document | `crates/sifr_package/` |

---

## Conclusion

The proposed architecture is the right approach for Phase 37. The anti-corruption layer, one-crate strategy, and guardrail pattern are all correct and well-grounded in Sifr's existing conventions. The main gaps are in the integration with existing crates (`sifr_frontend`), the enforcement mechanism for the plan-execute pattern, and the concrete definition of `OperationPlan`. These should be addressed before implementation starts.

The guardrail script (`check_package_manager_guardrails.py`) is the single most important deliverable for maintainability — without automated enforcement, the module boundaries and dependency rules will drift over time. The HIR maintainability script provides a proven template that can be adapted for the package manager.
