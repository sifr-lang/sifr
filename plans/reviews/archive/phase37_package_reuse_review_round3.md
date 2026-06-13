

## Final Review: Phase 37 Package Management (Post-Polish)

**Ready. The model is implementation-ready.**

---

### What's Solid

**Reuse model (uv/Cargo/Turborepo) — precise and well-justified**

The direct/reference split is clean and the reasoning for each boundary is documented:

- **Direct deps** (astral-pubgrub, semver, toml_edit, petgraph, globset/ignore/walkdir, gix, sha2/hex, tar/zstd) each have explicit selection criteria and wrapping rationale. The no-mixing `gix`/`git2` rule is correct. The HTTP client selection deadline before `milestone_37_6` is appropriate — it defers a real tradeoff without leaving it unspecified.
- **Reference-only deps** (uv-resolver, uv-workspace, uv-cache, uv-git, Cargo crates-io/resolver, Turborepo JS crates) have explicit non-reuse rationales rooted in Python/Rust/JS-specific concepts.
- The "reference implementation, not direct semantic dependency" framing is consistent and prevents implementation drift.

One minor note: the doc states "The crate is available from crates.io" for astral-pubgrub. This is accurate at the time of writing — `astral-pubgrub` publishes to crates.io. The migration clause ("any future move to a git or vendored source must pin an immutable revision and record the reason in architecture docs") is the right safeguard. Verify this before `milestone_37_1` dependency audit.

**Sifr's own semantic package manager boundary — clear and complete**

The three canonical roles (Sifr Package Manager / Cargo Backend / uv Compatibility) are mutually exclusive and collectively exhaustive:

- The Sifr Package Manager owns the full stack from `sifr.toml` parsing through module origins to CLI commands. This is not delegated to Cargo or uv.
- Cargo Backend is explicitly a generated-Rust build backend. It never becomes the user-facing import resolver.
- uv is scoped as an optional future compatibility frontend only; it cannot fork resolver behavior.
- The design principle "Package source roots are package-aware, not flattened into `[source].roots`" correctly prevents the shadowing/ambiguity problem.
- `[backend.cargo.dependencies]` is explicitly distinct from Sifr source dependencies — the boundary is unambiguous.

**Solver/lock/workspace/generated-Cargo contracts — internally consistent**

The contracts are well-specified and cross-validate correctly:

- `SolverInput` → `PubGrubDependencyProvider` → `ResolvedPackageGraph` → `CargoBackendPlan` is a coherent pipeline with defined transformation points.
- `ResolvedPackageGraph` contains `backend_cargo: CargoBackendPlan` — the graph carries the backend plan from the start.
- The lockfile model has parallel `[[package]]` (Sifr source) and `[[backend.cargo-package]]` (Cargo) sections; the required semantics explicitly require verifying generated `Cargo.lock` against the backend section by package name/version/source/checksum/features (not digest-only).
- `--locked` / `--frozen` validation is defined as "exactly satisfies active manifest graph, features, target, backend Cargo, checksums" — this covers the verification requirement.
- `ConflictPath` / `ConflictStep` structures are explicitly defined and shared across human/compact/JSON renderers.
- Workspace lockfile ownership (single `sifr.lock` at root), member participation rules (explicit opt-in), and cycle rejection are consistent.
- The resolver architecture correctly specifies pre-expanded features (not PubGrub virtual packages) as the Phase 37 approach.

---

### Three Polish Notes (Not Blockers)

These are implementation-level details, appropriately deferred, but worth flagging:

1. **Generated Cargo project location** — The doc says generated projects "Live under the generated artifact cache root, separate from the package source cache" but doesn't name the canonical path. Suggest `.sifr/cargo-gen/` alongside `.sifr/cache/` and `.sifr/artifacts/`. Implement before `milestone_37_4`.

2. **Sparse index authentication token lifecycle** — The trust model covers credential storage and redaction, but the bearer token inclusion mechanism and expiry handling in HTTP requests needs concrete description before `milestone_37_6`. The credential redaction for manifests/lockfiles/diagnostics is solid.

3. **Git dependency lockfile id format** — The lockfile example shows `id = "git+https://github.com/sifr-lang/math.git?rev=abc123#math@0.4.0"` with a clean structured form. The `rev` field is shown separately. This is sufficient for implementation.

---

### No Remaining Blockers

The round 1 review identified 5 critical blockers and 10 high/medium issues. The revised doc has resolved all of them:

| Round 1 Blocker | Status |
|---|---|
| PubGrub crate origin | ✅ Crates.io with migration clause |
| Feature modeling decision | ✅ Pre-expanded edges (not virtual packages) |
| Solver input/output types | ✅ Fully defined in the doc |
| Upgrade policy | ✅ Full spec for all forms |
| Locked/frozen/offline behavior | ✅ All modes defined with validation scope |

The phase doc is a complete implementation contract. It is ready for implementation planning.

---

### Recommendation

**Open the Phase 37 checklist issue and begin `milestone_37_1`**. The dependency audit (milestone scope item) should verify the astral-pubgrub crates.io availability and the exact version pinning strategy before writing the first line of resolver code.
