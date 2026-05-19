# Phase 37 Workspace/Monorepo Implementation-Readiness Review

**Scope**: Workspace and monorepo concerns only. All other phase 37 areas are out of scope.
**Primary doc**: `internal_docs/phases/37_package_management.md` (1150 lines)
**Status of prior review**: `reviews/phase37_workspace_gap_review_round1.md` (empty — prior review covered general gaps, not workspaces specifically)

---

## Summary

The document correctly makes Cargo workspace membership the package-selection source of truth (line 513), delegates workspace root discovery to Cargo (line 452), and includes Turborepo-style selectors (lines 523–531). However, the workspace section ("Cargo Workspace And Sifr Workspace Semantics", lines 511–533) is severely under-specified. Several critical monorepo constructs are referenced but never defined: virtual workspaces, `default-members`, `exclude`, `[workspace.dependencies]`, path dependencies between members, mixed Sifr/Rust workspaces, shared `Cargo.lock` behavior from subdirectories, and workspace-level vs per-package `sifr.toml` semantics. LSP/editor integration in monorepos and diagnostics for ambiguous package selection are entirely absent.

---

## Critical Gaps (blockers)

### G-1: Virtual workspace handling is undefined

**Where**: Lines 511–533 ("Cargo Workspace And Sifr Workspace Semantics"), Package Discovery Flow step 1 (line 452), milestone_37_5 scope (line 1084).

**What**: Cargo supports virtual workspaces — a root `Cargo.toml` that has no `[package]` section and only `[workspace]` members. The document never specifies how Sifr handles:
- A virtual workspace root that has no package of its own.
- The Sifr package selection when there is no root package to anchor the graph.
- Whether `[package.metadata.sifr]` is required on every member or optional, and what the diagnostic is when it is missing on a workspace member.
- How `cargo metadata --workspace-features` interacts with virtual workspaces and Sifr feature mappings.

**Impact**: Sifr will fail or misbehave on any real-world Cargo monorepo that uses the virtual-workspace pattern (extremely common in practice).

**Fix**: Add a subsection under "Cargo Workspace And Sifr Workspace Semantics" covering virtual workspaces:

> **Virtual workspaces**: A Cargo workspace whose root manifest has no `[package]` section is a virtual workspace. Sifr treats every workspace member that exposes `[package.metadata.sifr]` as a Sifr package. A virtual workspace root has no Sifr package identity of its own — it is a pure workspace aggregator. Sifr must not require a root package for graph derivation when `cargo metadata` reports a virtual workspace. If `--workspace` is used with a virtual workspace, Sifr selects all Sifr-capable members. If a workspace member lacks `[package.metadata.sifr]`, Sifr classifies it as a backend Rust crate only (not an error at discovery time; see SIFR-PACKAGE-0102 scope rules).

---

### G-2: `default-members` and `exclude` are referenced but never defined behavior

**Where**: Line 961 ("workspace tests: default members, virtual workspaces, package selection"), milestone_37_5 DoD (line 1093).

**What**: Cargo supports `members` (glob patterns), `default-members`, and `exclude` in `[workspace]`. The document references "default members" in test coverage but never specifies:
- Whether Sifr respects Cargo's `default-members` when `--workspace` is used without explicit filters.
- Whether Sifr respects Cargo's `exclude` patterns, or whether it has its own exclude mechanism.
- Whether Cargo's glob-based `members = ["packages/*"]` pattern is expanded by Sifr or consumed as-is from `cargo metadata`.
- What happens when a user passes `--workspace` but only wants the default members (i.e., wants `cargo build --workspace --exclude <x>` behavior ported).

**Impact**: Inconsistent behavior between `cargo build --workspace` and `sifr build --workspace`, or users unable to selectively exclude members.

**Fix**: Add the following paragraph after the workspace-semantics paragraph (after line 522):

> Sifr respects Cargo's `[workspace]` filtering for `--workspace` selection:
>
> - `default-members` is honored when `--workspace` is used without explicit package filters.
> - `exclude` patterns are honored and Sifr never selects an excluded member as a build target.
> - Glob patterns in `members` are expanded by Cargo before `cargo metadata`; Sifr consumes the flattened member list from `cargo metadata`.
> - If the user wants a subset of default members, they must use explicit `--filter` or `-p` selectors; there is no separate Sifr exclude mechanism.

---

### G-3: `[workspace.dependencies]` is unaddressed

**Where**: Package Discovery Flow (lines 448–464), `SifrPackageGraph` struct (lines 401–409), Derived Inputs (lines 387–398).

**What**: Cargo workspaces support `[workspace.dependencies]` as a mechanism for workspace-wide dependency version unification. The document never addresses:
- Whether Sifr imports and packages can reference workspace-level dependencies defined only in the root `Cargo.toml`.
- How workspace dependencies interact with scoped imports — is the workspace root a "pseudo-dependency" of every member?
- Whether `[workspace.dependencies]` should be exposed as Sifr dependency scope, importable by all members.
- What happens if a member has a direct dependency on a crate that is also in `[workspace.dependencies]` but at a different version.

**Impact**: Real-world monorepos commonly use `[workspace.dependencies]` for shared deps (e.g., all members depend on `tokio = "1.x"` declared once). If Sifr ignores this, Sifr packages will fail to resolve common dependencies.

**Fix**: Add a subsection under "Cargo Workspace And Sifr Workspace Semantics":

> **`[workspace.dependencies]`**: Cargo's `[workspace.dependencies]` table defines shared dependency specifications that workspace members inherit. For Sifr package resolution, workspace-level dependencies are treated as implicit dependencies of every workspace member. They are resolved through normal Cargo dependency resolution and participate in scoped import semantics like any direct Cargo dependency. A member's explicit `[dependencies]` table takes precedence over `[workspace.dependencies]` for version selection in that member's Cargo dependency scope. Sifr does not need to model workspace dependencies as a separate entity — `cargo metadata` already flattens them into each package's resolved dependencies.

---

### G-4: Path dependencies between workspace members are under-specified

**Where**: Package Discovery Flow step 3 (line 454), `SifrPackageGraph` struct (lines 401–409), CLI contract `--workspace` (line 517).

**What**: In Cargo monorepos, workspace members often depend on each other via path dependencies: `my-lib = { path = "../my-lib" }`. The document covers `sifr build -p app` but never specifies:
- Whether a workspace path dependency on a package without `[package.metadata.sifr]` is an error or warning.
- How the generated Rust namespace isolation works when two workspace members each contain `.sifr` source that transitively depend on each other via path dependencies.
- Whether `sifr build --workspace` compiles workspace members in dependency order or attempts parallel compilation.
- Whether path dependency cycles between Sifr workspace members are detected and reported as an error (SIFR-PACKARE-XXXX or SIFR-PACKAGE-0201-adjacent).
- Whether path dependency workspace members are always included when `--workspace` is used, regardless of their `default-members` status.

**Impact**: Monorepo users with inter-member Sifr dependencies via path will have undefined behavior around compilation order, namespace isolation, and cycle detection.

**Fix**: Add the following paragraph under "Cargo Workspace And Sifr Workspace Semantics":

> **Path dependencies between workspace members**: When package A has a path dependency on package B and both are Sifr packages in the same workspace, they participate in normal scoped import resolution. Package A's import scope includes B's exports. Path dependency cycles between Sifr packages are an error — SIFR-PACKAGE-0205 reports the cycle with the full dependency path. `sifr build --workspace` compiles workspace members in Cargo topological order, with Sifr packages compiled before their Rust-only or non-Sifr dependents. A path dependency on a workspace member that lacks `[package.metadata.sifr]` produces `SIFR-PACKAGE-0102` only if that member is expected to be a Sifr-capable package (e.g., imported by a Sifr package that uses its `.sifr` source).

Add diagnostic `SIFR-PACKAGE-0205` to the diagnostics table (after SIFR-PACKAGE-0204):
| `SIFR-PACKAGE-0205` | circular path dependency between workspace Sifr packages |

---

### G-5: Shared `Cargo.lock` behavior from subdirectories is undefined

**Where**: "Cargo.lock" section (lines 155–166), lock mode semantics (lines 784–794), `--workspace` flag (lines 517–519).

**What**: Cargo uses a single shared `Cargo.lock` at the workspace root. A user running `sifr build` from a subdirectory expects the shared lock to be used. The document never specifies:
- Whether `sifr build --workspace` discovers the workspace root automatically (implied by line 452) or requires the user to be at the workspace root.
- What happens when `sifr build` is run from a member subdirectory without `--workspace`.
- Whether `Cargo.lock` lock/network mode enforcement (`--locked`, `--offline`, `--frozen`) works correctly when invoked from a subdirectory.
- Whether `sifr fetch` from a subdirectory fetches for the entire workspace or only the selected subpackage.

**Impact**: A common ergonomics failure: users expect `cd packages/my-lib && sifr build` to build just that package using the workspace lock, but without clear semantics the behavior is unpredictable.

**Fix**: Add a paragraph under "Package Discovery Flow" after step 1 (after line 452):

> **Subdirectory invocation**: Sifr always discovers the nearest Cargo workspace root for lock/network mode enforcement and package graph derivation. Running `sifr build` from a member subdirectory without `--workspace` builds only that package but uses the shared workspace `Cargo.lock` and lock mode semantics. Running `sifr build --workspace` from a subdirectory compiles the full workspace. `sifr fetch` from a subdirectory operates on the full workspace, not just the subdirectory member, to ensure all selected package sources are materialized.

---

### G-6: Workspace-level `sifr.toml` vs per-package `sifr.toml` is unresolved

**Where**: "sifr.toml" section (lines 167–187), module map (line 907), milestone_37_5 (lines 1080–1097).

**What**: The document defines per-package `sifr.toml` but never addresses whether a workspace-level `sifr.toml` at the root (distinct from per-package manifests) exists or is desirable. Related unresolved questions:

- Should there be a root `sifr.toml` that defines workspace-wide Sifr settings (e.g., shared source roots, workspace-level trust policy, shared edition)?
- Does the existing `internal_docs/sifr_workspace_design.md`'s `[workspace]` table (which supports `members`, `exclude`, `resolver`) map to a Cargo workspace concept or a Sifr workspace concept? The two documents appear to use "workspace" to mean different things.
- If both a root `sifr.toml` and per-package `sifr.toml` exist, what is the precedence?
- If there is no root `sifr.toml`, what is the source of truth for workspace-wide Sifr settings?

**Impact**: Architectural contradiction between `sifr_workspace_design.md` (defining a Sifr-native workspace concept) and Phase 37 (delegating workspace to Cargo). Implementing milestone_37_5 without resolving this will require choosing one interpretation, risking a breaking change later.

**Fix**: Add a subsection under "Cargo Workspace And Sifr Workspace Semantics":

> **Sifr workspace manifest vs Cargo workspace**: Phase 37 delegates *package* workspace membership to Cargo. The `[workspace]` table in `sifr_workspace_design.md` is a *source resolution* workspace concept (module search path), not a package management concept. They are orthogonal. Phase 37 does not introduce a root-level Sifr workspace manifest. Workspace-wide Sifr settings (edition migration policy, shared trust policy, monorepo-specific diagnostics) are deferred to a future Phase 40+ workspace-policy document. Per-package `sifr.toml` remains the sole Sifr compiler metadata file in Phase 37.

> Remove the reference to `internal_docs/sifr_workspace_design.md` in line 184, or clarify it applies only to the Phase-36-era `[workspace]` table for module resolution, not to Phase-37 package workspaces.

---

## Moderate Gaps

### G-7: Package selectors are missing `--no-default-features` and `--all-features`

**Where**: CLI Contract (lines 755–769), milestone_37_5 (lines 1080–1097).

**What**: The Turborepo-style selectors (lines 523–531) cover basic selection but omit:
- `--no-default-features` for workspace packages (Cargo's equivalent exists).
- `--all-features` for exhaustive testing of all feature combinations.
- Negation syntax for filters (e.g., `--filter '!exclude-me'`).
- Interaction between multiple `--filter` flags (AND vs OR semantics).

**Fix**: Add to the selector list after line 531:

> Additional selector flags:
> - `--no-default-features` deactivates default Sifr/Cargo feature activation for selected packages.
> - `--all-features` activates all Sifr and Cargo features for selected packages.
> - `--filter pkg --filter other` selects packages matching either filter (OR semantics). `--filter pkg,other` is AND semantics.
> - `--filter '!pkg'` excludes `pkg` from the current selection set.

---

### G-8: Diagnostic for ambiguous package selection is missing

**Where**: Diagnostics table (lines 851–880), Package Discovery Flow (lines 448–464).

**What**: In a Cargo workspace with multiple packages exposing Sifr metadata and sharing the same Sifr package name or import root, the document defines ambiguity diagnostics for import scope (SIFR-PACKAGE-0201) but not for CLI-level ambiguous package selection. Example: `sifr build --workspace` when two workspace members have the same `[package.metadata.sifr.manifest]` path relative to their respective roots, or when `--filter pkg` matches multiple packages with the same display name.

**Fix**: Add to the diagnostics table:

| `SIFR-PACKAGE-0601` | package selector matches multiple packages without additional disambiguation |
| `SIFR-PACKAGE-0602` | duplicate Sifr package import root across workspace members that share a Cargo dependency scope |
| `SIFR-PACKAGE-0603` | changed-file mapping failed: changed file does not fall under any selected package's source roots |

---

### G-9: Mixed Sifr/Rust workspace behavior is under-specified

**Where**: Cargo Workspace And Sifr Workspace Semantics (lines 511–533), milestone_37_5 (line 1093).

**What**: Real monorepos mix Sifr packages and Rust-only packages. The document states (line 518) that Rust-only packages are built only when reachable as backend dependencies. But it never specifies:
- Whether Rust-only packages in the same workspace should be visible to Sifr for trust validation even when not transitively reachable.
- Whether `[workspace.dependencies]` applies to Rust-only members, Sifr members, or both.
- Whether a Rust-only workspace member can depend on a Sifr package (and if so, does Sifr need to compile that package's source).
- Whether `sifr build --workspace` skips Rust-only members that are not dependencies of any Sifr package (line 518 says this) but still needs them for trust validation (line 849 implies this).

**Impact**: Trust validation may fail silently or produce incorrect diagnostics in mixed workspaces.

**Fix**: Add a paragraph under "Cargo Workspace And Sifr Workspace Semantics":

> **Mixed Sifr/Rust workspaces**: A Cargo workspace may contain both Sifr packages and Rust-only packages. Sifr packages are always selected by `--workspace`. Rust-only packages are selected only when reachable as backend dependencies of a Sifr package. Rust-only packages that are dependencies of no selected Sifr package are not compiled by Sifr but may still be relevant for trust validation when they are transitive Cargo dependencies of a trusted backend crate. Rust-only packages that are Cargo workspace members but are not reachable from any Sifr package are ignored by Sifr in Phase 37. A Rust-only package depending on a Sifr package is not a valid Phase 37 pattern — `SIFR-PACKAGE-0106` reports this with remediation that the Rust package should either be converted to a Sifr package or the dependency should be made optional through Cargo features.

Add diagnostic `SIFR-PACKAGE-0106` to the diagnostics table.

---

### G-10: LSP/editor behavior in monorepos is absent

**Where**: milestone_37_3 DoD (line 1060: "editor analysis uses the same package source map as CLI builds"), milestone_37_5 (line 1088).

**What**: The document references editor analysis but never specifies:
- How the Sifr LSP server discovers the correct workspace root when opening files in a monorepo subdirectory.
- Whether the LSP can handle multiple open workspaces simultaneously.
- How `--workspace` vs per-file analysis is selected in the LSP.
- Whether the LSP's package graph discovery is incremental or full-reparse on each file open.
- How the LSP handles the case where the workspace `Cargo.lock` is stale (LSP should never mutate state).

**Fix**: Add a subsection under "Cargo Workspace And Sifr Workspace Semantics":

> **LSP and editor integration**: The Sifr language server uses the same workspace discovery as the CLI (`cargo metadata` at the nearest workspace root). When multiple workspace roots are present in a multi-root editor session, each root is treated as an independent Sifr workspace. The LSP always uses `--frozen`-equivalent behavior: no lock mutation, no network access, no manifest writes. Package graph derivation for LSP analysis is incremental — only changed packages trigger recomputation. The LSP does not run user-visible compilation but may use the same `PackageSourceMap` as CLI builds for hover/completion/goto-definition. Sifr diagnostics from the package graph (SIFR-PACKAGE-0101 through SIFR-PACKAGE-0603) are surfaced in the editor as inline diagnostics when the package graph is stale or unresolvable.

---

## Nits

### N-1: `sifr outdated` description is underspecified

**Where**: milestone_37_5 scope (line 1088–1089).

**What**: The description of `sifr outdated` says "use Cargo registry/source metadata where available and report unsupported sources as explicit unknowns." But the document never specifies what happens for Git dependencies, path dependencies, or private registries — the most common monorepo dependency sources. In practice, "outdated" for a Git-tagged dependency or a path dependency is undefined.

**Fix**: Tighten the description:

> `sifr outdated` reports the current locked version, newest compatible version, source, and unknown status without changing manifests or lockfiles. For registry dependencies, it uses Cargo's registry index metadata. For Git dependencies, it reports the current tag/branch/revision and whether the remote has advanced. For path dependencies, it reports the current local version as pinned. For alternate registries or private registries without index metadata, it reports the source as unsupported with `SIFR-PACKAGE-0604`.

---

### N-2: "Organization Demo Repositories" lacks a mixed workspace demo

**Where**: Organization Demo Repositories (lines 550–749).

**What**: All four demo repos are flat, single-package repos. There is no demo for an actual Cargo workspace with multiple members and inter-member path dependencies. This is the most common real-world monorepo pattern and is entirely undemonstrated.

**Fix**: Add a fifth demo repository requirement:

> **Additional demo repo required**:
> - `https://github.com/sifr-lang/sifr-demo-workspace`: A Cargo workspace with three members (`sifr-demo-core`, `sifr-demo-utils`, `sifr-demo-app`) where `sifr-demo-app` has path dependencies on `sifr-demo-core` and `sifr-demo-utils`. This demo exercises:
>   - `--workspace` selection from subdirectory.
>   - `default-members` / `exclude` semantics.
>   - Path dependency compilation order.
>   - `[workspace.dependencies]` shared across members.
>   - Mixed Sifr/Rust member (add a Rust-only `sifr-demo-backend-utils` member to the same workspace).

---

### N-3: Test coverage gap for `[workspace.dependencies]`

**Where**: Sifr-specific tests (lines 971–994), Correctness And Test Reuse (lines 950–1005).

**What**: No test case covers a workspace member importing a dependency declared in `[workspace.dependencies]` that is also explicitly declared in the member at a different version.

**Fix**: Add to the Sifr-specific tests list:

> - workspace member imports a dependency declared only in `[workspace.dependencies]` — resolves correctly.
> - workspace member redeclares a `[workspace.dependencies]` version at a different semver — member's version wins.
> - workspace member depends on a path member via `[workspace.dependencies]` path spec.

---

### N-4: Milestone_37_5 DoD line 1093 is vague

**Where**: milestone_37_5 Definition of done (lines 1091–1097).

**What**: "Cargo workspace packages with Sifr metadata work from subdirectories" is underspecified. It does not cover path dependency order, mixed workspaces, or workspace-level lock enforcement from subdirectories.

**Fix**: Replace line 1093 DoD bullet with:

> - `sifr build --workspace` from a workspace subdirectory builds the full workspace using the shared `Cargo.lock`; `sifr build` from a subdirectory builds only that member using the shared `Cargo.lock`.
> - `--workspace` selection honors `default-members` and `exclude`.
> - Path dependency workspace members are compiled in Cargo topological order.
> - `[workspace.dependencies]` are importable by all workspace members.
> - Mixed Sifr/Rust workspace members are handled correctly (Rust-only members not reachable from Sifr packages are not compiled by Sifr).
> - Changed-package selection maps Git-modified files to the correct workspace member source roots.

---

## Recommendations

### R-1: Split the workspace section into a dedicated document

The "Cargo Workspace And Sifr Workspace Semantics" section (lines 511–533) is 22 lines for the most operationally complex part of Phase 37. It should be extracted into `internal_docs/phases/37_workspace_design.md` with full detail for each construct, then referenced from the main phase doc. The main doc should then summarize and link to the detail doc.

### R-2: Add a workspace-specific module to `crates/sifr_package`

The module map at line 907 shows `graph/{derive,scopes,workspace,filters,changed,digest}.rs`. The `workspace` submodule exists but its responsibilities are not enumerated. Recommend adding an internal architecture note in the milestone_37_5 module comments that specifies what `graph::workspace` owns vs what `graph::derive` owns, since the distinction affects implementation order.

### R-3: Update sifr_workspace_design.md compatibility note

The Phase 37 doc references `internal_docs/sifr_workspace_design.md` for the `sifr.toml` schema (line 184). That doc defines a Sifr-native `[workspace]` table for module resolution, which is conceptually different from a Cargo workspace. These two "workspace" concepts should be explicitly disambiguated in `sifr_workspace_design.md` before Phase 37 implementation begins, or the reference in line 184 should be removed.

---

## Section-Level Edit Summary

| Location | Issue | Suggested action |
|---|---|---|
| Lines 511–533 | Section too sparse; missing virtual workspace, `[workspace.dependencies]`, path deps, mixed workspaces, subdirectory semantics | Expand into full subsection with G-1 through G-6 fixes |
| Lines 523–531 | Selector list missing `--no-default-features`, `--all-features`, negation, AND/OR semantics | Add G-7 fixes |
| Line 849 | Trust validation against workspace Rust-only members is ambiguous | Clarify with G-9 fix |
| Lines 851–880 | Diagnostics table missing workspace-specific diagnostics | Add G-8 and G-9 new codes |
| Lines 959–963 | "workspace tests: default members, virtual workspaces, package selection" in TRACEABILITY but no matching implementation text | Expand into explicit test contract |
| Lines 971–994 | Sifr-specific tests missing workspace scenarios | Add N-3 fix |
| Line 1084–1097 | milestone_37_5 DoD vague on subdirectory behavior, path deps, workspace deps, mixed workspaces | Replace with N-4 fix |
| Lines 550–749 | Demo repos all single-package; no workspace demo | Add N-2 fix |
| Line 184 | References `sifr_workspace_design.md` for schema but they use "workspace" differently | Add R-3 clarification |

---

## Verdict

**not-ready**

The Phase 37 document has the right foundational choices (Cargo workspace as source of truth, `cargo metadata` as integration surface, per-package `sifr.toml`), but the workspace/monorepo section is too sparse for implementation. Six critical gaps block implementation of milestone_37_5:

1. **Virtual workspaces** — undefined behavior will cause failures on standard Cargo monorepo patterns.
2. **`default-members` / `exclude`** — inconsistent `--workspace` behavior relative to Cargo.
3. **`[workspace.dependencies]`** — will break on common shared-dependency monorepo patterns.
4. **Path dependency semantics** — compilation order, cycle detection, and namespace isolation are undefined.
5. **Subdirectory lock behavior** — undefined, a common user pain point.
6. **Root `sifr.toml` vs per-package `sifr.toml`** — architectural contradiction between `sifr_workspace_design.md` and Phase 37 needs resolution before any implementation begins.

The document needs at minimum the six critical-section expansions above before implementation of milestone_37_5 (or any workspace-affecting milestone) can begin safely. The gaps are not nits — they represent real-world monorepo patterns that will break if implemented against the current spec.