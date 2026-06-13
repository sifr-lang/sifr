

Looking at this revised phase document against your eight focus areas:

## Findings

**Ready. No blockers remain.** The architecture is coherent, the reuse boundaries are explicit, and every core semantic decision is documented with sufficient precision to begin implementation.

---

### What is well-defined

**Reuse model (uv/Cargo/Turborepo)** — The direct/reference split is clear and the reasoning for each boundary is sound. Direct deps (astral-pubgrub, semver, toml_edit, petgraph, globset/ignore/walkdir, gix, sha2/hex, tar/zstd) have documented selection criteria; reference-only deps (uv-resolver, uv-workspace, Cargo crates-io/resolver/crates-io, Turborepo JS crates) have explicit non-reuse rationales.

**Version solving** — Pre-expanded feature edges before PubGrub (not virtual feature packages) is explicitly specified. Deterministic candidate ordering, locked version preferences, and structured conflict paths are documented. Upgrade policies are explicit.

**Features** — Additive union semantics, feature conflict as a Sifr diagnostic (not PubGrub's indirect detection), and feature-expanded optional dependencies becoming ordinary solver requirements are all clearly specified.

**Target-specific dependencies** — Phase 37 evaluates for one active target; the active target is CLI/default/host in priority order; target predicates are evaluated before solver input; universal multi-target locks are deferred as an additive future extension that does not fork the semantic model.

**Workspace members** — Explicit isExplicit: workspace membership alone does not make packages importable; `{ workspace = true }` opts into catalog entries; catalogs are never implicit imports. Workspace member cycles are rejected before PubGrub.

**Yanks** — New resolution ignores yanked versions; existing lockfiles may continue using yanked versions if checksum matches; resolution fails with `SIFR-PACKAGE-0104` and reports closest non-yanked alternatives.

**Lock modes** — Online/Offline/Locked/Frozen are distinct and mutually exclusive. Locked versions are preferences, not extra manifest constraints. Locked/frozen validation is explicit about what constitutes a failure.

**Generated Cargo integration** — Deterministic generated projects, source-to-generated mapping for diagnostics, verification against backend lock section (not just digest), and fail-closed behavior under `--locked`/`--frozen` are all specified.

---

### Polish suggestions (implementation detail, not blockers)

1. **Target predicate syntax** — The doc says "Sifr-owned target expressions that map cleanly to Rust target triples" but does not define the concrete syntax (e.g., `cfg(unix)` mirroring Cargo, or a Sifr-specific grammar). Implementation will need to pick one; suggest Cargo-compatible `cfg()` syntax with Sifr-specific additions behind a feature flag.

2. **Sparse index authentication** — The protocol endpoints are defined but the bearer token mechanism from `sifr login` needs more concrete description: how tokens are stored, how they are included in requests, how token refresh or expiry is handled. The credential redaction section covers manifests/lockfiles but not registry authentication state.

3. **Git dependency lockfile representation** — The lockfile example shows `source = "registry+sifr"` for registry packages but not Git URL representation. The spec says "immutable version/checksum or commit id" but the exact `[[package]]` representation for Git deps is implied rather than shown.

4. **PackageSourceMap → CargoBackendPlan wiring** — The doc shows both types but does not explicitly describe how `ResolvedPackageGraph` produces `CargoBackendPlan` from `backend.cargo.dependencies`. The integration point between package resolution and Cargo generation is architecture-level clear but the concrete transformation is deferred to milestone_37_4.

5. **Generated Cargo project location** — The doc says generated Cargo must not expose package cache paths as normal source roots, but does not specify the canonical location for generated projects (e.g., `.sifr/gen/`, `target/sifr/gen/`, workspace-relative `gen/`). Suggest `.sifr/cargo-gen/` alongside `.sifr/cache/` and `.sifr/artifacts/`.

6. **Dependency audit deadline** — The doc says "short dependency audit before milestone_37_2" but does not specify the audit criteria or what happens if a dependency fails audit (replace, defer, document exception). Consider adding a one-paragraph audit checklist.

---

### No blockers identified

The six areas you asked about are each covered with explicit semantics. The remaining gaps are syntax decisions, API shape decisions, and concrete representation choices that are appropriately deferred to implementation-level design within each milestone. The phase document provides a solid contract for implementation.
