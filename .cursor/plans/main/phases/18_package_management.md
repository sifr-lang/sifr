# Package Management

**Why now:** Package management is foundational infrastructure for code sharing and project structure. It enables the ecosystem to grow beyond single-project use. With the language feature-complete and FFI available, package management can properly handle both Sifr packages and Rust crate dependencies. This is a prerequisite for the developer tools phase (LSP needs to understand project structure from `sifr.toml`) and the ecosystem phase (registry needs packages to publish).

---

## milestone_package_mgmt: Package Management

status: pending

**Goal:** Add package management infrastructure so Sifr projects can declare dependencies, resolve versions, and share code.

**Depends on:** milestone_ffi (FFI unlocks the Rust crate ecosystem; package management must handle both Sifr packages and Rust crate dependencies)

### Work Items

- `sifr.toml`: project manifest with `[project]` metadata and `[dependencies]` section specifying version ranges (semver)
- `sifr.lock`: lockfile with exact resolved versions, content hashes (SHA-256), and source URLs; committed to version control
- Version solver: PubGrub-based solver (same algorithm as Cargo and uv); resolves dependency graph with conflict detection and clear diagnostics
- CLI commands: `sifr add <package>`, `sifr remove <package>`, `sifr install` (install from lockfile), `sifr update` (re-resolve and update lockfile)
- Before the package registry exists (Phase 20), dependencies are git-only or path-only

### Definition of Done (milestone_package_mgmt)

- `sifr.toml` parsed and used for project configuration and dependencies
- `sifr.lock` generated with exact versions and content hashes
- `sifr add` / `sifr remove` work
- PubGrub solver handles version conflicts with clear diagnostics
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: add_dependency, remove_dependency, lockfile_generation, path_dependency, git_dependency
- Milestone demo in `./demos/milestone_package_mgmt_demo.sifr`
