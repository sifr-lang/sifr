

Now I have a comprehensive view of the codebase. Let me write the complete Phase 37 package management review.

---

## Phase 37 Package Management Review: Not Ready for Implementation

### 1. Verdict: **NOT READY**

The phase doc at `internal_docs/phases/37_package_management.md` contains a single milestone with three bullet points ("Dependency declaration, lockfile semantics, resolution workflow") and an exit criterion ("Package management workflows are deterministic and reproducible"). This is a sketch, not a specification. Executing against it would produce an incoherent partial implementation.

The phase doc itself acknowledges this: *"Needs more planning before execution (scope boundaries, dependency model, and acceptance gates are still draft-level)."*

---

### 2. Concrete Gaps That Block Implementation

| Gap | Severity | Impact |
|-----|----------|--------|
| **No manifest dependency model** | Critical | `[dependencies]` is reserved but undefined. No version specifiers, no source selection, no optional dependencies, no features. |
| **No source-resolution model** | Critical | No decision on git deps vs. registry vs. path deps. No sparse index protocol support. No source-preference ordering. |
| **No lockfile model** | Critical | No `sifr.lock` schema. No content-addressed source hashing. No lockfile staleness detection. |
| **No package-fetching mechanism** | Critical | No fetching of remote packages. No offline/locked/frozen workflows. No vendor/plumber support. |
| **No package directory semantics** | High | `__init__.sifr`, re-exports, wildcard imports, and package member discovery are deferred. The current resolver explicitly rejects namespace-file collisions but doesn't implement package directory resolution. |
| **No workspace inheritance model** | High | No `[workspace.dependencies]` for workspace-wide version pinning. No `workspace.package` inheritance. No `members`/`exclude` activation for package-mode. |
| **No registry publish/trust model** | High | No publish workflow. No provenance attestation. No crate ownership verification. No yanked/immutable package handling. |
| **No CLI command surface** | High | No `sifr add`, `sifr remove`, `sifr update`, `sifr fetch`, `sifr outdated`. No `--locked`/`--frozen`/`--no-sync` flags modeled on uv/Cargo. |
| **No feature toggle model** | Medium | No `[features]` table. No feature union/fallback semantics. No conditional dependency activation. |
| **No Rust/native dependency bridge** | Medium | No `sifr.toml` mapping to `Cargo.toml` for native deps. No `links`/`build` script handling. No `sifr export cargo-dependencies` for FFI bridge. |
| **No workspace/package boundary validation** | Medium | No detection of cross-package imports without explicit dependency declaration. No undeclared-dependency diagnostics. No internal package boundary enforcement. |
| **No version conflict diagnostics** | Medium | No SemVer incompatible version error model. No multi-version disambiguation UX. No workspace-wide version unification diagnostics. |
| **No negative-path validation matrix** | Medium | No documented error cases for: ambiguous resolution, missing transitive deps, cyclic dependencies, malformed lockfile, stale lockfile with changed sources, yanked version error, unsupported registry protocol, missing Cargo.lock for locked mode. |

---

### 3. Recommended Canonical Architecture

#### Cargo/uv Role Allocation

```
┌─────────────────────────────────────────────────────────────┐
│  User-facing UX (sifr CLI)                                   │
│  sifr add | sifr build | sifr run | sifr test | sifr publish  │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│  Sifr Package Manager (native, source-first)                  │
│  - sifr.toml parsing + validation                              │
│  - Dependency graph construction                              │
│  - Source resolution (git/path/registry)                      │
│  - Lockfile generation + staleness detection                  │
│  - Workspace/monorepo topology                                │
│  - Package directory resolution                               │
│  - sifr.lock generation + Cargo.lock co-existence             │
│  - --locked / --frozen / --no-sync semantics                  │
└──────────────────────────┬──────────────────────────────────┘
                           │ sifr-generated Cargo.toml
┌──────────────────────────▼──────────────────────────────────┐
│  Cargo (native build backend)                                  │
│  - Rust compilation via rustc                                  │
│  - crates.io / alternative registry deps                      │
│  - Native build scripts / FFI                                 │
│  - lockfile versioning (shared with sifr.lock)                │
│  - workspace members/exclude                                   │
└───────────────────────────────────────────────────────────────┘
```

**uv role: None as a direct dependency.** `uv` is a Python package manager. Sifr packages are source-first (`.sifr` files), not Python wheels. The only legitimate uv use case is as an optional installer/bootstrap for the `sifr` CLI itself (i.e., `curl -LsSf https://sifr.sh/install.sh | uv run sifr@latest`). This is an installation concern, not a package management concern.

If there is a future requirement to consume Python wheels from `pyproject.toml`-style dependencies inside Sifr packages (e.g., a `sifr.python` module that wraps Python packages), that is a **separate Phase 40+ concern**, not Phase 37. It must not contaminate the Sifr-first package model.

**Cargo role: Final build backend.** The existing codegen pipeline already generates Cargo.toml. Phase 37 extends this to include external Sifr package sources as local path dependencies inside the generated Cargo workspace, and native Rust crates as regular Cargo dependencies.

#### Source Resolution Model

The current `ModuleResolver` searches entry parent then configured workspace source roots. This model must extend to package-aware origin tracking:

```
Resolution priority order:
1. Embedded stdlib (sifr.*, _sifr.*) — highest, always wins
2. Entry parent directory (unconditional local winner)
3. Workspace package sources, in [source].roots declaration order:
   a. Workspace local packages (<workspace_root>/packages/<name>)
   b. Workspace source root packages (<source_root>/...)
   c. External registry packages (<cache>/registry/<source>/<name>-<version>/)
   d. Git packages (<cache>/git/<short-hash>/)
```

Package sources are **not** flattened into `[source].roots`. The distinction is:
- `[source].roots` controls which local directories are searched for local source modules
- External packages live in a separate managed cache with explicit version pinning

**Critical rule:** A package named `utils` resolved from registry source must **not** shadow a local `utils.sifr` in a workspace root. The resolver must track package origin and reject shadowing across origin types.

#### Source Types

```toml
# sifr.toml — source type variants
[dependencies]
# Registry source (default) — resolves from sifr.sh registry
http-client = "1.2"           # semver range, resolves to 1.2.3
http-client = ">=1.0,<2.0"    # version range with comparator

# Git source
color = { git = "https://github.com/sifr-lang/color.git" }
color = { git = "https://github.com/sifr-lang/color.git", tag = "v1.0" }
color = { git = "https://github.com/sifr-lang/color.git", rev = "abc123" }

# Path source (workspace-local)
local-util = { path = "packages/local-util" }

# Registry source with optional features
toml-parser = { version = "2.0", features = ["stream"] }
```

---

### 4. Workspaces/Monorepo Model

The existing workspace model in `sifr_workspace_design.md` must extend to:

#### Workspace Types

**Root workspace:**
```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2026"

[workspace]
members = ["packages/core", "packages/cli", "tools/linter"]
exclude = ["tmp", "target", "packages/deprecated-*"]
resolver = "1"

[workspace.dependencies]    # shared version pins
http-client = ">=1.0,<2.0"
```

**Virtual workspace (no [package]):**
```toml
[workspace]
members = ["apps/*", "packages/*"]
exclude = ["packages/legacy-*"]
resolver = "1"
```

**Member package:**
```toml
[package]
name = "@myorg/core"
version = "0.1.0"
edition = "2026"

# Inherits from [workspace.dependencies] unless overridden
# http-client = ">=1.0,<2.0"  <-- inherited
```

#### Monorepo Rules (from Turborepo + Cargo experience)

1. **Dependencies belong where used.** If `packages/core` uses `http-client`, only `packages/core/sifr.toml` declares it. `apps/web` depends on `packages/core` and transitively gets `http-client` via `packages/core`.
2. **Root only has repo-level tools.** The workspace root has no application source. It has tooling (linters, formatters, release scripts) and `sifr.toml` with `[workspace]` configuration.
3. **Internal packages are explicit workspace deps.** Do not rely on implicit path resolution for internal packages. Write `depends = ["@myorg/core"]` explicitly, even for packages under `packages/*`.
4. **Package boundaries matter.** Phase 37 must detect imports outside a package's declared dependencies. If `packages/core` imports `packages/utils` without declaring it as a dependency, emit `SIFR-PACKAGE-0201: undeclared dependency`.
5. **Circular dependency rejection.** Package dependency graph must be acyclic. Cycle detection with a clear diagnostic showing the cycle path.

#### Package Naming Convention

Sifr uses scoped package names: `@org/name`. The registry enforces namespace ownership. This matches Cargo's crate naming and provides a clear ownership model for the ecosystem.

---

### 5. CLI UX Contract

```bash
# Package management
sifr add <spec>                    # Add dependency (updates sifr.toml + sifr.lock)
sifr add <spec> --dev              # Add as dev dependency
sifr add <spec> --optional         # Add as optional dependency (feature-gated)
sifr remove <name>                 # Remove dependency (updates sifr.toml + sifr.lock)
sifr update                        # Update all dependencies to latest semver
sifr update <name>                 # Update specific dependency
sifr update <name> --dry-run       # Show what would change without modifying lockfile
sifr fetch                         # Fetch all dependencies to local cache
sifr outdated                      # Show which dependencies have newer versions
sifr tree                          # Show dependency tree with version conflict visualization

# Build / run with package awareness
sifr build [file.sifr]            # Compiles entry + dependencies
sifr run [file.sifr]              # Builds and runs
sifr check [file.sifr]            # Type-checks with dependency type info
sifr test [dir]                    # Runs tests with dependency graph

# Lockfile workflows (Cargo/uv inspired)
sifr build --locked               # Fail if sifr.lock is stale
sifr build --frozen               # Fail if sifr.lock is stale OR missing
sifr build --offline              # Build using only cached packages, no network

# Workspace selection
sifr build --workspace            # Build all workspace members
sifr build -p <package>           # Build specific package
sifr build --exclude <pattern>     # Exclude matching packages

# Registry / publish
sifr login                         # Authenticate with registry (sifr.sh)
sifr publish                       # Publish package to registry
sifr yank <name>@<version>         # Yank a published version
sifr owner <name> --add <user>     # Manage package ownership
```

**Flag semantics aligned with Cargo/uv:**
- `--locked`: require lockfile matches `sifr.lock` exactly; error on any change
- `--frozen`: require lockfile exists and matches; error if missing (fails fast)
- `--offline`: use only locally cached packages; fail on cache miss (like `CARGO_NET_OFFLINE`)

---

### 6. Manifest + Lockfile Model

#### sifr.toml Schema Extension

```toml
[package]
name = "@org/project"             # Required for published packages
version = "1.2.3"                 # Semver
edition = "2026"                  # Language edition
description = "..."               # For registry metadata
license = "MIT"                   # SPDX identifier
repository = "https://github.com/..."  # For provenance
authors = ["..."]                 # For registry metadata
readme = "README.md"              # For registry metadata

[workspace]
members = ["packages/*"]
exclude = ["packages/legacy-*"]
resolver = "1"

[workspace.dependencies]          # Shared version pins across workspace members
shared-dep = ">=1.0,<2.0"

[source]                          # Workspace-local source roots (existing)
roots = ["src", "packages/*/src"]

[dependencies]                   # Regular dependencies
http-client = ">=1.0,<2.0"
http-client = { version = ">=1.0,<2.0", features = ["stream"] }
local-util = { path = "packages/local-util" }
color = { git = "https://github.com/...", tag = "v1.0" }

[dev-dependencies]               # Development-only dependencies
pytest-compat = ">=0.1"

[optional-dependencies]           # Feature-gated dependencies
stream = ["http-client/stream"]

[features]                        # Feature definitions
stream = ["optional-deps/http-stream"]

[profile.release]                # Build profile overrides (future)
opt-level = 3
lto = true
```

#### sifr.lock Schema

```toml
# sifr.lock — content-addressed, committed to version control
version = 1

[[package]]
name = "@org/core"
version = "1.2.3"
source = "registry+sifr.sh"
source-id = "sifr.sh/@org/core@1.2.3"
chksum = "sha256:abc123..."     # SHA-256 of all source files in the package
dependencies = [
    { name = "@org/http-client", version = ">=1.0,<2.0", resolved = "1.1.0" },
]

[[package]]
name = "@org/http-client"
version = "1.1.0"
source = "registry+sifr.sh"
source-id = "sifr.sh/@org/http-client@1.1.0"
chksum = "sha256:def456..."
dependencies = []

[metadata]
created-at = "2026-05-17T12:00:00Z"
resolver-version = "1"
```

**Key design decisions:**
1. `sifr.lock` is the single source of truth for resolved versions. It **coexists** with `Cargo.lock` — they are different lockfiles for different dependency types.
2. `Cargo.lock` is generated from Sifr's lockfile for native deps, and Sifr's lockfile tracks only Sifr packages.
3. `source-id` is the canonical identifier (registry + name + version). Two packages with the same `source-id` must have identical content.
4. Content hashing uses all source files in the package, not just the manifest. This prevents dependency substitution attacks.

---

### 7. Resolver/Import/Source Package Semantics

#### Resolution Algorithm

1. **Phase 1: Manifest parsing.** Parse `sifr.toml` and `[workspace.dependencies]` inheritance. Build the declared dependency graph.
2. **Phase 2: Version solving.** Use PubGrub (same as Cargo and uv) to resolve version ranges. Handle: semver constraints, git revision pinning, path sources, feature union, optional dependencies.
3. **Phase 3: Source fetching.** For each resolved package, fetch source to the local cache. Verify content hashes against `sifr.lock`.
4. **Phase 4: Module resolution.** For each source file in fetched packages, register its modules into the import namespace with **package-origin tracking**.
5. **Phase 5: Conflict detection.** Check for namespace collisions across package origins. Emit `SIFR-PACKAGE-0101` (ambiguous import) with package-origin disambiguation hints.
6. **Phase 6: Import resolution.** During HIR lowering, resolve imports through the package-origin-aware module resolver.

#### Package Directory Semantics

A Sifr package may have a package directory structure:

```
@org/http-client/
  __init__.sifr          # Package public API — defines exports
  client.sifr             # Internal module
  auth/
    __init__.sifr         # Sub-package exports
    bearer.sifr           # Internal module
    basic.sifr            # Internal module
  sifr.toml               # Package manifest
  LICENSE
  README.md
```

**`__init__.sifr` semantics:**
- Defines the public API of the package
- Symbols not re-exported from `__init__.sifr` are private
- **No side effects on import** (unlike Python's `__init__.py`)
- Enables explicit public API surface control

**Import resolution rules:**
- `from @org/http-client import client` → `client.sifr` in package
- `from @org/http-client import BearerAuth` → `__init__.sifr` must re-export `BearerAuth`
- `from @org/http-client.auth import basic` → `auth/basic.sifr`
- Re-exports from `__init__.sifr` use explicit syntax: `from client import Client as Client`

#### Feature Semantics

```toml
[dependencies]
http-client = { version = "2.0", features = ["stream", "compression"] }

[features]
stream = ["http-client/stream"]
compression = ["http-client/compression"]
default = ["stream"]
```

Features activate optional dependencies. Feature union follows dependency declaration order. Circular feature dependencies must be rejected with `SIFR-PACKAGE-0203: cyclic feature dependency`.

---

### 8. Registry/Publish/Security/Trust Model

#### Registry Protocol (Sparse Index)

Based on the Cargo sparse protocol:

```
GET /index/<short-name>           # Package index (name → versions)
GET /package/<name>/<version>    # Package metadata
GET /download/<name>/<version>   # Package tarball download
```

The index is the authoritative source for package metadata. Package tarballs are content-addressed and immutable once published.

#### Publishing Flow

```bash
sifr publish
  → Validate sifr.toml (name, version, required fields)
  → Verify registry ownership (authenticated user owns namespace)
  → Run pre-publish checks (sifr check, sifr test)
  → Build package tarball (source files + manifest + metadata)
  → Compute content hash
  → Upload to registry
  → Update index
```

#### Trust Model

1. **Provenance attestation:** Published packages include authorship metadata linked to registry accounts. Registry accounts require verified ownership (email + OAuth).
2. **Immutable releases:** Once published, a version is immutable. Yanked versions are marked but not deleted (allows existing lockfiles to remain valid).
3. **Content hashing:** Every published package has a SHA-256 hash of its source content. `sifr.lock` records this hash. On fetch, the hash is verified.
4. **No unsigned packages in production.** Phase 37 targets internal ecosystem with no hostile actors. A full cryptographic signing model (Sigstore/cose) is Phase 41 material.
5. **Namespace ownership.** Only verified namespace owners can publish to `@namespace/package`. Collaborators can be added via `sifr owner`.

#### Security-Critical Constraints

- Package names must not contain path traversal characters (`../`, `..`, `/` in path component)
- Package source files must not escape the package root via path traversal
- Lockfile must be validated for source integrity before use
- Registry URLs must be validated as HTTPS (HTTP is rejected)
- Git sources must use known-good commit IDs (not just branch names in production)

---

### 9. Validation/Acceptance Gates

#### Positive-Path Validation

| Gate | Criterion | Test Pattern |
|------|-----------|-------------|
| `add` positive | `sifr add @org/pkg@">=1.0"` updates sifr.toml and sifr.lock | End-to-end fixture: add dep → build → run |
| `add` with features | Feature-activated deps compile correctly | Feature dep fixture → compile → verify feature is live |
| `remove` positive | `sifr remove pkg` removes from sifr.toml and sifr.lock | Remove dep → build → no compile errors |
| `update` positive | `sifr update pkg` bumps version in lockfile | Update dep → verify lockfile updated → build → run |
| `update --dry-run` | Shows proposed changes without modification | Dry run → verify lockfile unchanged |
| `build --locked` | Fails when lockfile is stale | Mutate dep version in sifr.toml → `sifr build --locked` → expect error |
| `build --frozen` | Fails when lockfile is missing | Delete lockfile → `sifr build --frozen` → expect error |
| `build --offline` | Uses cached packages only | Cache deps → `sifr build --offline` → expect success |
| `build --offline` negative | Fails on cache miss | No cache → `sifr build --offline` → expect error |
| `fetch` positive | All resolved packages are in local cache | `sifr fetch` → verify cache contents |
| Workspace members | Workspace packages resolve correctly | Monorepo fixture → `sifr build -p @org/core` → expect success |
| Workspace inheritance | `[workspace.dependencies]` are inherited | Workspace fixture → verify inherited versions |
| Package directory | `__init__.sifr` re-exports work | Package with re-exports → import → compile → run |
| Feature union | Multiple features compile correctly | Multi-feature dep → compile → verify all features |
| Lockfile persistence | Lockfile survives re-runs | `sifr build` → verify lockfile stable → `sifr build` → same lockfile |
| Transitive deps | Transitive deps are fetched and compiled | Deep dep graph → compile → run → expect correct transitive behavior |
| Cycle detection | Circular dependencies are rejected | Circular dep fixture → `sifr build` → expect diagnostic |
| Undeclared dep detection | Imports without declared deps fail | Import from undeclared pkg → `sifr build` → expect SIFR-PACKAGE-0201 |

#### Negative-Path Validation

| Gate | Criterion | Expected Diagnostic |
|------|-----------|-------------------|
| Ambiguous import | Two packages provide same module name | `SIFR-PACKAGE-0101` with package-origin disambiguation |
| Missing transitive dep | Transitive dependency not in lockfile | `SIFR-PACKAGE-0102` with attempted resolution paths |
| Version conflict | Semver incompatible versions required | `SIFR-PACKAGE-0103` with conflict path |
| Malformed sifr.toml | Invalid dependency spec syntax | `SIFR-PACKAGE-0104` with location and expected format |
| Missing package | Registry does not contain the package | `SIFR-PACKAGE-0105` with registry URL |
| Yanked version | Requested version is yanked | `SIFR-PACKAGE-0106` with yanked status |
| Lockfile drift | Lockfile source hash mismatch | `SIFR-PACKAGE-0107` with expected vs actual hash |
| Stale lockfile | Lockfile missing entries for sifr.toml changes | `SIFR-PACKAGE-0108` with hint to run `sifr update` |
| Namespace collision | Local `utils.sifr` shadows registry `utils` | `SIFR-PACKAGE-0109` with origin labels |
| Path traversal in dep | Dependency name contains `../` | `SIFR-PACKAGE-0110` with rejected name |
| Unsatisfied feature | Requested feature does not exist in dep | `SIFR-PACKAGE-0111` with available features |
| Missing __init__.sifr | Package directory without `__init__.sifr` | `SIFR-PACKAGE-0112` with path to missing file |
| Circular feature dep | Feature depends on itself through chain | `SIFR-PACKAGE-0203` with cycle path |
| Private API access | Import from non-exported symbol | `SIFR-PACKAGE-0204` with available exports |

#### Phase 27 Non-Regression Gates

These gates are **mandatory** and **non-negotiable** per the quality contract:

1. No user-triggerable panic paths in package manager code
2. No data-dependent `.unwrap()`/`.expect()` in user runtime paths
3. Stable diagnostic contract: codes, severity, spans, URLs, suggestions
4. Canonical JSON diagnostics schema maintained
5. Exit-code stability: `0/1/2/3` preserved (package manager adds no new exit codes)
6. Recovery limits enforced for all package resolution diagnostics

---

### 10. Concerns with User Notes

#### Concern 1: "Source-first" requires precise definition

"Source-first" is correct but must be scoped: **source-first for Sifr packages**. Native Rust dependencies (FFI, existing Rust crates) are not source-first. The design must distinguish between:

- **Sifr packages**: `.sifr` source distributed via registry, compiled by Sifr
- **Rust packages**: Crates that Sifr's codegen wraps as Cargo dependencies (they ARE already source, but via rustc, not Sifr)

The current codegen already handles Rust crates. The Phase 37 gap is only Sifr package management.

#### Concern 2: "Do not flatten dependency source roots into [source].roots"

This concern is **correct and critical**. Flattering registry packages into `[source].roots` would:
1. Pollute the workspace source root namespace
2. Make disambiguation impossible (which `utils` is intended?)
3. Break existing workspace discovery (registry dirs don't exist on disk before fetch)

The current `ModuleResolver` must be extended with a **package origin registry** that tracks which modules come from which package source, separate from the `[source].roots` local search path.

#### Concern 3: "Need complete model, not MVP"

This is the right instinct. The Phase 37 doc's single milestone is insufficient. The complete model I recommend requires **at minimum 4 sub-milestones**:

1. **m37.1: Dependency Declaration Model** — Manifest extension, source types, version constraints, workspace inheritance
2. **m37.2: Lockfile + Resolution** — PubGrub solver, sifr.lock schema, content-addressed cache, resolution algorithm
3. **m37.3: Package Directory + Import Resolution** — `__init__.sifr`, re-exports, package-origin-aware resolver, undeclared-dep detection
4. **m37.4: Workspace + CLI + Publishing** — Workspace members/exclude, full CLI surface, registry publish flow, negative-path diagnostics

#### Concern 4: Cargo/uv role is correctly specified

The user notes correctly identify that Cargo is the final native build backend and uv is an optional bridge. This is the **correct canonical architecture**. The concern is that the current codebase has no package manager at all — Phase 37 must build a **native Sifr package manager** from scratch, not attempt to adapt uv for Sifr packages (which would be architecturally wrong).

---

### Summary: What the Phase Doc Needs

The phase doc at `internal_docs/phases/37_package_management.md` must be rewritten to include:

1. **A complete manifest schema** with all dependency source types, features, optional deps, dev deps, workspace inheritance
2. **A complete lockfile schema** with content hashing, source-id tracking, and metadata envelope
3. **A detailed resolution algorithm** with package-origin tracking, PubGrub solver spec, and conflict detection
4. **A package directory spec** with `__init__.sifr`, re-export syntax, and private API enforcement
5. **A CLI surface spec** with all commands and flag semantics aligned with Cargo/uv conventions
6. **A registry protocol spec** (sparse index, publish, yank, ownership management)
7. **A monorepo enforcement spec** with undeclared-dep detection, package boundary rules, and cycle detection
8. **A negative-path diagnostic matrix** with specific error codes for every failure mode
9. **A 4-submilestone breakdown** with clear entry/exit criteria for each
10. **Validation evidence requirements** mapping each gate to positive and negative test fixtures

The architecture doc at `internal_docs/architecture.md` already references PubGrub in contract #4 and a `sifr_registry` crate in the crate structure. Phase 37 must implement those deferred items, not defer them further.

---

**Recommendation:** Rewrite `internal_docs/phases/37_package_management.md` with the complete model above before any implementation begins. The current draft-level phase doc is a planning placeholder, not an execution contract. Proceeding with implementation against it would produce the fragmented, incomplete package manager that the quality contract explicitly forbids.
