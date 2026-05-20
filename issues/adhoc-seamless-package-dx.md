# Adhoc Phase: Seamless Package DX And Production Package Management

Related phase: `internal_docs/phases/37_package_management.md`

## Status

- [ ] milestone_adhoc_pkg_1: Package UX contract and source layout
- [ ] milestone_adhoc_pkg_2: Sifr-managed Cargo projection
- [ ] milestone_adhoc_pkg_3: Package session and CLI command integration
- [ ] milestone_adhoc_pkg_4: Package-aware compiler imports
- [ ] milestone_adhoc_pkg_5: Workspaces, aliases, and multiple versions
- [ ] milestone_adhoc_pkg_6: Packaging, publishing, vendoring, and release checks
- [ ] milestone_adhoc_pkg_7: Migration, docs, demos, and long-term guardrails

## Problem

Phase 37 established the Cargo-backed package-management substrate: Cargo metadata parsing, Sifr package graph derivation, package source maps, lock modes, trust policy, workspace selection, publish/archive plans, guardrails, and concrete demo repositories.

The developer experience is still not seamless:

- Users must understand Cargo commands (`cargo fetch`, `cargo metadata`, `cargo check`) to work with a Sifr package app.
- `sifr run`, `sifr check`, and `sifr test` are not package-aware end to end.
- The current demo package layout (`sifr/<package>/*.sifr`) is more verbose than necessary and does not feel like a natural source layout.
- Public package APIs are described through manifest exports today, but Rust-like long-term maintainability is better achieved by declaring API shape in source, and Python-like Sifr ergonomics are better achieved with `__init__.sifr`.
- Multiple versions, aliases, workspaces, lock modes, publishing, and Rust-backed packages need one coherent Sifr UX that does not inherit Cargo's complexity.

## Goal

Create a production-grade package-management experience where a user can clone a Sifr package app and run Sifr commands without directly operating Cargo:

```bash
git clone https://github.com/sifr-lang/sifr-demo-app
cd sifr-demo-app
sifr fetch --locked
sifr run --locked --offline
```

Local development should also support a Rust-like convenience path:

```bash
sifr run
```

When online, `sifr run` may fetch missing dependencies and create or update the lockfile if the command is not constrained by `--locked`, `--offline`, or `--frozen`.

## Non-Goals

- Do not create an independent Sifr registry, resolver, source cache, or archive format in this phase.
- Do not expose raw Cargo internals as the Sifr user model.
- Do not require users to edit Cargo source IDs, Cargo metadata aliases, or registry cache paths.
- Do not support Python package tools (`pyproject.toml`, `uv.lock`, wheels) in this phase. Future Python/uv interop must lower into the same package session model.
- Do not make every `.sifr` file in a dependency public by default.

## Changes From Phase 37

Phase 37 remains the substrate. This adhoc phase changes the user-facing model and wires the substrate into the compiler/CLI.

Changed behavior:

- `sifr init`: creates the canonical `src/` layout instead of the Phase 37 demo `sifr/<package>/` layout.
- `sifr add`: updates Sifr-facing dependency declarations first, then projects to Cargo dependencies and Cargo dependency renames.
- `sifr remove`: removes the Sifr-facing dependency declaration and its projected Cargo dependency.
- `sifr update`: updates through the Sifr-facing dependency identity, then delegates lockfile mutation to Cargo.
- `sifr run`: can select an implicit `src/main.sifr` target and can fetch missing dependencies in unconstrained online local development.
- package public API: new packages derive public APIs from `__init__.sifr` namespaces instead of manifest `[exports].modules`.

Unchanged substrate behavior:

- Cargo remains responsible for registry/Git/path resolution, lockfile format, checksums, source caches, upload authentication, vendoring, and workspace membership.
- Sifr continues to use Cargo metadata as the authoritative resolved graph input.
- Sifr continues to treat Cargo source IDs as opaque.
- `sifr fetch`, `sifr tree`, `sifr package`, `sifr publish`, and `sifr vendor` remain Cargo-delegating operations, but their CLI output and diagnostics become Sifr-owned.

## Design Principles

- Cargo remains the substrate for dependency resolution, lockfiles, registries, Git/path dependencies, publishing, vendoring, credentials, and Rust backend crates.
- Sifr owns the user-facing package semantics, commands, diagnostics, package API, and source layout.
- The package API is declared in source through `__init__.sifr`, not through a manifest export list.
- Default package layout should feel simple: `.sifr` files live under `src/` beside the Rust marker or backend file.
- Online local development should be convenient; CI and offline workflows must be reproducible and explicit.
- Sifr diagnostics must wrap Cargo failures with stable Sifr package diagnostic codes and actionable help.

## Target Package Layout

Library package:

```text
demo_json/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    __init__.sifr
    parse.sifr
    value.sifr
    errors.sifr
    internal/
      cache.sifr
    lib.rs
```

App package:

```text
demo_app/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    main.sifr
    __init__.sifr
    cli.sifr
    lib.rs
```

Rust-backed Sifr package:

```text
demo_http/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    __init__.sifr
    client.sifr
    lib.rs
```

`src/lib.rs` remains a Cargo target. For pure Sifr packages it must be a pure marker. For Rust-backed packages it may contain implementation/shims, but every direct native crate must be declared in Sifr trust policy.

## `sifr.toml` Contract

Minimal library:

```toml
[package]
name = "demo_json"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
root = "src"
```

Minimal app:

```toml
[package]
name = "demo_app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
root = "src"

[[bin]]
name = "demo-app"
path = "src/main.sifr"
```

Defaults:

- `[source].root` defaults to `src`.
- A library package exposes `src/__init__.sifr` when present.
- `sifr run` defaults to `src/main.sifr` when no `[[bin]]` target is configured.
- Apps may also include `src/__init__.sifr` if they are importable by other packages.

Compatibility:

- Existing Phase 37 `source.roots = ["sifr"]` layouts remain supported through explicit configuration.
- New package creation commands must generate the `src/` layout.
- Manifest-level `[exports] modules = [...]` becomes legacy compatibility, not the recommended public API model.

Conflict resolution:

- New packages created by `sifr init` must not generate `[exports].modules`.
- If `__init__.sifr` exists at the package source root, the package's public API is derived from `__init__.sifr`.
- `[exports].modules` is accepted for backward compatibility only when source-root `__init__.sifr` is absent.
- If both source-root `__init__.sifr` and `[exports].modules` are present and they expose different public roots, Sifr reports `SIFR-PACKAGE-0701` and requires the maintainer to remove one model or make them agree.

Initialization semantics:

```text
sifr init --lib demo_json
  creates Cargo.toml with Cargo package name sifr-demo-json
  creates sifr.toml with [source].root = "src" or omits it to use the default
  creates src/__init__.sifr
  creates src/lib.rs with the canonical pure marker
  does not create [exports].modules

sifr init --bin demo_app
  creates Cargo.toml with Cargo package name sifr-demo-app
  creates sifr.toml with default source root and one [[bin]] target
  creates src/main.sifr
  creates src/lib.rs with the canonical pure marker
  may create src/__init__.sifr only when --importable is requested
```

Binary target schema:

```toml
[[bin]]
name = "demo-app"
path = "src/main.sifr"
```

`sifr run` resolution order:

1. If `--bin <name>` is provided, use the matching `[[bin]]` target from `sifr.toml`.
2. If `src/main.sifr` exists, use it as the implicit binary target.
3. If exactly one `[[bin]]` target exists, use that target.
4. Otherwise report `SIFR-PACKAGE-0605` for missing or ambiguous binary target.

## Public API And Namespace Rules

Sifr uses Python-shaped `__init__.sifr` files with enforced package boundaries.

Within a package:

- Any module under the package source root may import any other module in the same package using local or package-relative imports.
- Internal organization is unrestricted.

Across packages:

- A package root is public if `src/__init__.sifr` exists.
- A namespace submodule is public if that namespace is a directory containing `__init__.sifr`.
- Public names are the names defined or imported by that namespace's `__init__.sifr`.
- Plain implementation files such as `src/parse.sifr` are private across package boundaries unless their symbols are re-exported by an accessible `__init__.sifr`.
- Direct cross-package imports into implementation files are rejected with a privacy diagnostic.

Example:

```text
demo_json/src/
  __init__.sifr
  parse.sifr
  value.sifr
  codecs/
    __init__.sifr
    json.sifr
    yaml.sifr
  internal/
    cache.sifr
```

`demo_json/src/__init__.sifr`:

```sifr
from .parse import parse_json
from .value import DemoJsonValue
```

`demo_json/src/codecs/__init__.sifr`:

```sifr
from .json import decode_json, encode_json
from .yaml import decode_yaml, encode_yaml
```

Allowed from another package:

```sifr
from demo_json import parse_json, DemoJsonValue
from demo_json.codecs import decode_json
```

Rejected from another package:

```sifr
from demo_json.parse import parse_json
from demo_json.codecs.json import decode_json
from demo_json.internal.cache import Cache
```

If maintainers want `from demo_json.parse import parse_json`, they should make `parse` a public namespace:

```text
demo_json/src/parse/
  __init__.sifr
  parser.sifr
```

`demo_json/src/parse/__init__.sifr`:

```sifr
from .parser import parse_json
```

Then:

```sifr
from demo_json.parse import parse_json
```

is public and stable.

Implementation requirements:

- `PackageSourceMap` must parse every public namespace `__init__.sifr`.
- Re-exported public names are extracted from supported `from .module import name` and `from .namespace import name` forms.
- Definitions written directly in `__init__.sifr` are public names of that namespace.
- A public namespace path is valid only when every namespace segment is represented by a directory with `__init__.sifr`.
- Privacy checks use the derived namespace API graph, not filesystem presence alone.
- A cross-package import into an implementation file that is not reachable through a public namespace API graph reports `SIFR-PACKAGE-0203`.
- The first implementation should reject dynamic or wildcard public API construction in `__init__.sifr` with a stable diagnostic rather than guessing.

## Dependency Model

Sifr dependencies should be declared in Sifr-facing configuration and projected to Cargo.

### Dependency Declaration Schema

New packages declare dependencies in `sifr.toml`. This replaces `[package.metadata.sifr.aliases]` for new packages.

```toml
[dependencies]
demo_json = { package = "sifr-demo-json", git = "https://github.com/sifr-lang/sifr-demo-json", tag = "v0.1.0" }
demo_http = { package = "sifr-demo-http", git = "https://github.com/sifr-lang/sifr-demo-http", tag = "v0.1.0" }
local_utils = { package = "sifr-demo-utils", path = "../utils" }
registry_json = { package = "sifr-json", version = "1.2" }

[dev-dependencies]
demo_test_support = { package = "sifr-demo-test-support", git = "https://github.com/sifr-lang/sifr-demo-test-support", tag = "v0.1.0" }
```

Multiple-version aliases:

```toml
[dependencies]
demo_json_v1 = { package = "sifr-demo-json", import = "demo_json_v1", git = "https://github.com/sifr-lang/sifr-demo-json", tag = "v0.1.0" }
demo_json_v2 = { package = "sifr-demo-json", import = "demo_json_v2", git = "https://github.com/sifr-lang/sifr-demo-json", tag = "v0.2.0" }
```

Field meanings:

- table key: local dependency alias and projected Cargo dependency name;
- `package`: upstream Cargo package name;
- `import`: optional public import root for this dependency instance; defaults to the resolved Sifr package name when unambiguous, otherwise defaults to the table key;
- `git`, `tag`, `rev`, `branch`, `path`, `version`, `registry`, `features`, `default-features`: projected to Cargo-compatible dependency fields;
- `workspace = true`: allowed only inside Cargo workspaces and projected to Cargo workspace dependency inheritance.

Backward compatibility:

- If `sifr.toml` has no `[dependencies]` and Cargo metadata contains `[package.metadata.sifr.aliases]`, Sifr parses the Cargo metadata aliases using the Phase 37 model.
- If both Sifr dependencies and Cargo metadata aliases are present for the same package, Sifr reports `SIFR-PACKAGE-0708` unless the projected alias mapping is identical.
- `sifr add --alias name` writes the table key `name` and sets `import = "name"` when the alias differs from the dependency's resolved Sifr package name.

Imports:

```sifr
from demo_json_v1 import parse_json
from demo_json_v2 import parse_json
from demo_json_v2.codecs import decode_json
```

Invariant:

```text
import root + importing package scope -> exactly one resolved package instance
```

If an unaliased import root can resolve to two direct dependency instances in the same scope, Sifr must reject the import and require an alias.

## Sifr-Managed Cargo Projection

To avoid inheriting Cargo complexity, Sifr should manage the Cargo projection for normal packages.

User-facing source of truth:

```text
sifr.toml
src/**/*.sifr
```

Generated or managed substrate:

```text
Cargo.toml
Cargo.lock
src/lib.rs
```

Rules:

- `sifr init`, `sifr add`, `sifr remove`, and `sifr update` update Sifr package metadata first.
- Sifr then updates the Cargo projection deterministically.
- Sifr-owned Cargo sections are marked and guarded.
- Advanced users may keep custom Cargo sections for Rust-backed packages, but Sifr validates they do not conflict with the package graph.
- Cargo source IDs are treated as opaque outputs of Cargo metadata.
- Sifr never shells into Cargo internals or links to Cargo private APIs.

Generated Cargo projection example:

```toml
[package]
name = "sifr-demo-json"
version = "0.1.0"
edition = "2024"
include = ["Cargo.toml", "Cargo.lock", "sifr.toml", "src/**/*.sifr", "src/lib.rs", "README.md", "LICENSE"]

[package.metadata.sifr]
manifest = "sifr.toml"
```

Pure marker:

```rust
// Pure Sifr package marker. Sifr source lives in sifr.toml source roots.
```

Drift diagnostics:

- `SIFR-PACKAGE-0702`: projected Cargo dependency or alias differs from `sifr.toml` dependency declaration.
- `SIFR-PACKAGE-0703`: `Cargo.toml` is missing `[package.metadata.sifr]` or points at the wrong manifest.
- `SIFR-PACKAGE-0704`: Cargo include/exclude omits `sifr.toml`, `src/**/*.sifr`, or the required marker/backend files.
- `SIFR-PACKAGE-0705`: `[source].root` points to a missing or non-directory path.
- `SIFR-PACKAGE-0709`: pure package marker is missing and cannot be regenerated because a user-owned Rust target already exists.

Recovery:

- `sifr fix --package <name>` may update Sifr-owned Cargo projection sections, regenerate the pure marker, and repair include patterns.
- `sifr fix` must not overwrite user-owned Rust backend code.
- `--locked`, `--offline`, and `--frozen` commands report drift before running Cargo operations.
- `--frozen` rejects any automatic projection write.

## CLI Commands

Package-aware core commands:

```bash
sifr run [--bin name] [--locked|--offline|--frozen] [--features f1,f2|--all-features|--no-default-features]
sifr check [path-or-package] [--workspace] [-p package] [--filter selector] [--locked|--offline|--frozen]
sifr build [path-or-package] [--workspace] [-p package] [--filter selector] [--locked|--offline|--frozen]
sifr test [path-or-package] [--workspace] [-p package] [--filter selector] [--locked|--offline|--frozen]
```

Dependency and package operations:

```bash
sifr init [--lib|--bin] [name]
sifr fetch [--locked|--offline|--frozen]
sifr add <package> [--git url] [--tag tag] [--path path] [--dev] [--alias name] [--features f1,f2]
sifr remove <package-or-alias> [--dev]
sifr update [package-or-alias]
sifr tree [--workspace|-p package] [--sifr-only|--all] [--depth N]
sifr vendor <dir> [--locked]
sifr package [--dry-run]
sifr publish [--dry-run]
sifr fix [--package name] [--check]
```

Command semantics:

- `sifr run` without lock/network flags may fetch dependencies and update `Cargo.lock`, matching Rust's local development convenience.
- `sifr fetch --locked` fetches exactly the locked graph and fails if the lockfile needs an update.
- `sifr check --locked --offline` never touches the network and fails if any selected dependency source is absent locally.
- `sifr --explain <diagnostic-code>` documents package diagnostics and recovery commands.

`sifr --explain` behavior:

- accepts any stable Sifr diagnostic code;
- prints the diagnostic meaning, common causes, package-manager-specific recovery commands, and links to docs;
- never performs package operations.

## Package Session

All package-aware commands must go through a single orchestration layer:

```text
PackageSession
  workspace_root
  selected_packages
  lock_mode
  network_mode
  feature_selection
  target/profile
  cargo_command_plan
  normalized_cargo_metadata
  sifr_package_graph
  package_source_map
  trust_summary
  build_cache_inputs
```

Operation plan schema:

```text
OperationPlan
  commands: Vec<CargoCommandPlan>
  writes_projection: bool
  writes_lockfile: bool
  requires_network: bool
  selected_packages: Vec<PackageSelection>
  topological_order: Vec<SifrPackageId>

CargoCommandPlan
  command: metadata | fetch | build | check | test | package | publish | vendor | add | remove | update
  current_dir: PathBuf
  targets: Vec<String>
  lock_mode: unlocked | locked | offline | frozen
  features: CargoFeatureSelection
  args: Vec<String>
```

Planning rules:

- `--frozen` rejects any operation with `writes_projection`, `writes_lockfile`, or `requires_network`.
- `--locked` rejects operations that would mutate `Cargo.lock`.
- `--offline` rejects plans whose selected package sources are absent locally and reports `SIFR-PACKAGE-0104`.
- Projection drift is checked before Cargo command execution.
- Multi-package operations compute package topological order from `SifrPackageGraph`; Cargo may build in its own internal order, but Sifr diagnostics and cache keys use the stable topological order.

Responsibilities:

- Discover package/workspace root.
- Ensure the Cargo projection is current.
- Run or plan Cargo fetch/metadata/build/package/publish operations.
- Parse and normalize Cargo metadata.
- Derive `SifrPackageGraph`.
- Build the `PackageSourceMap` using `__init__.sifr` namespace rules.
- Validate compiler compatibility, privacy, trust policy, lock modes, and package selection.
- Translate Cargo failures into Sifr diagnostics.
- Provide a stable input digest for incremental/cache behavior.

## Compiler Integration

Package-aware compilation must use `PackageSourceMap` rather than file-only import discovery.

Resolution order for a module in package `P`:

1. Local package modules under `P` source root.
2. Public APIs of direct Sifr dependencies in `P`'s direct dependency scope.
3. Configured aliases for direct dependency package instances.
4. Standard library modules.

Rejected:

- Transitive dependency imports.
- Private implementation file imports across package boundaries.
- Ambiguous import roots.
- Compiler-incompatible dependency packages.
- Rust-only packages selected as Sifr packages.

Codegen namespace rules:

- Every resolved package instance gets a generated Rust namespace that includes a stable package-instance hash.
- The hash is derived from the normalized Cargo package id, Cargo version, Cargo source, and Sifr package name.
- Example generated modules:
  - `demo_json_v1` imports lower to `sifr_gen_demo_json_7f3a2c10`.
  - `demo_json_v2` imports lower to `sifr_gen_demo_json_b81d044e`.
- HIR type identity includes the Sifr package instance id, not only the textual Sifr package name.
- Values from different package instances are not assignable unless an explicit conversion API is called.
- Cross-instance type mismatches report `SIFR-PACKAGE-0204` at compile time.
- Generated runtime code must not use data-dependent `unwrap` or `expect` for package dispatch.

## Workspace Semantics

Cargo remains authoritative for workspace membership, default members, path dependencies, lockfile location, and workspace dependency inheritance.

Sifr semantics:

- A virtual workspace root has no Sifr package identity.
- Sifr-capable members are packages with Sifr metadata.
- Rust-only members are allowed but ignored unless reachable as trusted backend dependencies.
- `sifr run` from a member selects that member.
- `sifr check --workspace` selects all Sifr-capable workspace members, filtered by `default-members` where applicable.
- `-p package` selects one Sifr package by Sifr name, Cargo package name, or unambiguous alias.
- `--filter` supports package selectors, dependency closure, dependent closure, changed paths, and negation.
- Root `Cargo.toml`, `Cargo.lock`, and workspace dependency changes invalidate the whole graph.

Virtual workspace root edge case:

- If root `Cargo.toml` is a virtual `[workspace]` with no `[package]` and `sifr.toml` exists at the workspace root, Sifr reports `SIFR-PACKAGE-0706` as a warning.
- The warning says the root `sifr.toml` has no package identity and should be moved to a package member or the workspace root should be converted into a package.
- A root `sifr.toml` in a virtual workspace is not used for package graph derivation.

## Packaging And Publishing

Sifr packages are Cargo packages with Sifr metadata and source files. The archive format remains Cargo's package archive.

Preflight before `cargo package` or `cargo publish`:

- `sifr.toml` is present and valid.
- `src/**/*.sifr` files required by public API and selected targets are included.
- `src/__init__.sifr` exists for libraries that expose a public API.
- `src/main.sifr` or configured `[[bin]]` exists for apps.
- Pure marker `src/lib.rs` has no implementation code for pure Sifr packages.
- Rust-backed packages declare direct native dependencies in trust policy.
- Archive contents do not contain path traversal entries.
- Cargo include/exclude does not omit required Sifr files.
- Credentials and private source URLs are redacted in diagnostics.

Commands:

```bash
sifr package --dry-run
sifr publish --dry-run
sifr publish
```

`sifr publish` delegates upload/authentication to Cargo but owns Sifr preflight diagnostics.

## Diagnostics

Required diagnostic behavior:

- Missing offline package source: `SIFR-PACKAGE-0104`, with help to run `sifr fetch --locked`.
- Cargo command failure: `SIFR-PACKAGE-0101`, with redacted command summary.
- Ambiguous import root: existing package import ambiguity code or a new stable code if needed.
- Private cross-package module access: `SIFR-PACKAGE-0203`.
- Transitive dependency import: `SIFR-PACKAGE-0202`.
- Type identity mismatch across package instances: `SIFR-PACKAGE-0204`.
- Untrusted Rust backend dependency: `SIFR-PACKAGE-0301`.
- Stale trust entry: `SIFR-PACKAGE-0305`.
- Package archive missing Sifr source: `SIFR-PACKAGE-0403`.
- Workspace/package selection errors: existing `SIFR-PACKAGE-06xx` family.

Diagnostics must include:

- package name and resolved package instance when available;
- dependency alias when relevant;
- source kind (`path`, `git`, `registry`) without relying on Cargo cache internals;
- lock/network mode;
- exact recovery command when possible.

The `SIFR-PACKAGE-07xx` diagnostic range is reserved for this adhoc package DX phase:

- `0701`: conflicting `__init__.sifr` and legacy `[exports].modules` public API models.
- `0702`: Cargo projection dependency or alias drift.
- `0703`: missing or incorrect `[package.metadata.sifr]`.
- `0704`: Cargo include/exclude omits required Sifr package files.
- `0705`: invalid source root.
- `0706`: ignored `sifr.toml` at virtual workspace root.
- `0707`: layout migration validation failed.
- `0708`: conflicting Sifr dependency aliases and legacy Cargo metadata aliases.
- `0709`: pure marker missing but user-owned Rust target prevents regeneration.

## Migration Plan

Existing Phase 37 demo layout:

```text
sifr/demo_json/__init__.sifr
sifr/demo_json/parse.sifr
```

New canonical layout:

```text
src/__init__.sifr
src/parse.sifr
```

Migration command:

```bash
sifr package migrate-layout --from sifr-rooted --to src-init
```

Migration rules:

- Move package source root to `src`.
- Convert package-root `__init__.sifr` into `src/__init__.sifr`.
- Rewrite `[source].root = "sifr"` to `[source].root = "src"` or remove it when `src` is the default.
- Regenerate Cargo include patterns from `sifr/**/*.sifr` to `src/**/*.sifr`.
- Regenerate or verify `src/lib.rs` pure marker for pure packages.
- Rewrite local imports as needed.
- Preserve public API names.
- Keep `source.roots = ["sifr"]` supported for existing packages until a later deprecation phase.
- Update demo repositories only after the package-aware compiler supports the new layout.

Migration validation:

1. Before migration, snapshot the public API from `__init__.sifr` or legacy `[exports].modules`.
2. After migration, derive the public API from the new `src/__init__.sifr` namespace graph.
3. Diff the public API snapshots and report `SIFR-PACKAGE-0707` if names are lost or added without an explicit flag.
4. Compile/package-check all local imports in the package to catch broken relative paths.
5. Run archive preflight to confirm required Sifr files are included.
6. Do not modify `Cargo.lock` package source revisions during layout migration.

Rollback:

- Before rewriting files, `sifr package migrate-layout` writes `<package>.sifr-migration-backup.tar`.
- `sifr package migrate-layout --rollback <backup.tar>` restores the original layout.
- Failed validation leaves the original tree in place unless `--apply-partial` is explicitly passed; `--apply-partial` is intended only for manual repair.

## Milestones

### milestone_adhoc_pkg_1: Package UX Contract And Source Layout

Scope:

- Finalize the `src/` layout, `src/__init__.sifr` public API rule, namespaced `__init__.sifr` rule, and `src/main.sifr` run target.
- Add parser/source-map tests for public root imports, public namespace imports, private implementation rejection, and local private imports.
- Implement `parse_init_sifr_reexports` and namespace API graph derivation in `PackageSourceMap`.
- Add compatibility and conflict tests for legacy `[exports].modules`.
- Document the layout in `docs/package_management.md`.

Validation:

- `cargo test -p sifr_package source_layout`
- `python3 scripts/check_package_manager_guardrails.py`
- `python3 scripts/check_diagnostic_docs_sync.py`

Acceptance:

- The source map can derive public package APIs solely from `__init__.sifr`.
- No manifest `[exports]` entry is needed for new packages.

### milestone_adhoc_pkg_2: Sifr-Managed Cargo Projection

Scope:

- Add projection code that creates/updates Cargo package metadata from `sifr.toml`.
- Generate pure marker `src/lib.rs` when absent.
- Preserve user-owned Cargo sections for Rust-backed packages.
- Add drift diagnostics when Cargo projection does not match Sifr package metadata.
- Implement `sifr init --lib`, `sifr init --bin`, and `sifr fix --check` projection paths.

Validation:

- `cargo test -p sifr_package cargo_projection`
- Guardrail that Sifr does not link Cargo private APIs.

Acceptance:

- `sifr init --lib demo_json` produces a valid Sifr/Cargo package.
- Projection is deterministic and idempotent.

### milestone_adhoc_pkg_3: Package Session And CLI Command Integration

Scope:

- Add `PackageSession`.
- Wire `sifr fetch`, `sifr tree`, and package-aware `sifr check` through the session.
- Implement lock/network mode behavior for `--locked`, `--offline`, and `--frozen`.
- Translate Cargo fetch/metadata failures into Sifr diagnostics.

Validation:

- `cargo test -p sifr_package package_session`
- `cargo test -p sifr -- package_cli`
- Demo command: `sifr fetch --locked` in `sifr-demo-app`.

Acceptance:

- A fresh cache can run `sifr fetch --locked` and then `sifr check --locked --offline`.

### milestone_adhoc_pkg_4: Package-Aware Compiler Imports

Scope:

- Connect `PackageSourceMap` to HIR/name resolution/import resolution.
- Support package root imports and public namespace imports.
- Reject private, transitive, and ambiguous imports with stable diagnostics.
- Make `sifr run` compile selected package app targets.

Validation:

- E2E pass/fail fixtures for package imports.
- Demo command: `sifr run --locked --offline` in `sifr-demo-app`.

Acceptance:

- `sifr-demo-app` can import `demo_json_v1`, `demo_json_v2`, and `demo_http` through Sifr package APIs without direct Cargo commands.

### milestone_adhoc_pkg_5: Workspaces, Aliases, And Multiple Versions

Scope:

- Support `sifr check --workspace`, `-p`, and `--filter` through `PackageSession`.
- Support aliases for multiple versions through Sifr-facing dependency config.
- Validate same package at multiple versions with distinct type identities.
- Ensure workspace dependency inheritance flows through Cargo metadata into Sifr scopes.
- Implement codegen namespace hashing for package instances.
- Add type identity tests that prove aliased package versions do not collide in HIR or generated Rust.

Validation:

- E2E workspace fixtures using `sifr-demo-workspace`.
- Multiple-version alias tests for `demo_json_v1` and `demo_json_v2`.

Acceptance:

- Workspaces and aliases behave consistently across CLI, source map, diagnostics, and cache digests.

### milestone_adhoc_pkg_6: Packaging, Publishing, Vendoring, And Release Checks

Scope:

- Wire `sifr package`, `sifr publish`, and `sifr vendor`.
- Validate archive contents for the new `src/` layout.
- Keep Cargo upload/authentication delegated.
- Add release dry-run checks for pure, Rust-backed, app, and workspace packages.

Validation:

- `sifr package --dry-run` on demo repositories.
- `cargo test -p sifr_package package_publish_vendor`
- Redaction tests for credentials/private source URLs.

Acceptance:

- Sifr packages can be published as Cargo packages without exposing Cargo complexity to normal users.

### milestone_adhoc_pkg_7: Migration, Docs, Demos, And Long-Term Guardrails

Scope:

- Add layout migration command or documented manual migration.
- Update `sifr-demo-*` repositories to canonical `src/` layout.
- Add end-to-end demo docs showing first clone, fetch, run, offline check, workspace selection, and publish dry-run.
- Extend guardrails to enforce source layout, projection boundaries, and demo commands.
- Extend `scripts/check_package_manager_guardrails.py` to accept legacy `sifr/` fixtures during migration and require `src/` layout for newly generated demos after closeout.
- Run full local validation.

Validation:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
- Demo transcript checked into docs or verification artifacts.

Acceptance:

- A user can clone `sifr-demo-app`, run Sifr commands only, and understand failures from Sifr diagnostics.

## Implementation Order

1. Source layout and `__init__.sifr` source-map rules.
2. Managed Cargo projection and drift diagnostics.
3. Package session and `sifr fetch`/`sifr tree`/package-aware `sifr check`.
4. Compiler import integration.
5. `sifr run` and `sifr test`.
6. Workspace/alias/multiple-version hardening.
7. Publish/vendor/migration/docs/demo closeout.

This order keeps the compiler integration dependent on a stable package source map, and keeps publishing dependent on the final canonical layout.

## Review Requirements

Each milestone requires:

- focused implementation PR;
- local targeted validation;
- `scripts/run_all_tests.sh --profile quick` before PR;
- Claude review pass until READY;
- issue update with status, validation output summary, and PR link.

Full closeout requires:

- final Claude full implementation review until READY;
- demo transcript for `sifr-demo-app` and `sifr-demo-workspace`;
- full `scripts/run_all_tests.sh` unless blocked by a documented infrastructure issue;
- docs and guardrails updated.
