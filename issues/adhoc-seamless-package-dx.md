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
- The current plan exposes Cargo-shaped target configuration (`[[bin]]`) in the Sifr manifest even though normal users should get layout-based runnable targets.
- Development/test-only dependencies need a Sifr-native grouping model rather than ad hoc Cargo table leakage.
- Single-file Sifr usage must stay easy: `sifr run main.sifr` and `sifr run any-other-name.sifr` should work without `sifr.toml`.
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
- Do not add scripts or workflow aliases in v1. Shell aliases, `just`, Make, or project-specific tooling can cover local workflows until real Sifr usage proves the needed shape.
- Do not make `[[bin]]` a required or normal Sifr manifest concept. Cargo may still receive generated target configuration in the projection when needed.

## Changes From Phase 37

Phase 37 remains the substrate. This adhoc phase changes the user-facing model and wires the substrate into the compiler/CLI.

Assumption: Sifr has no external stable package ecosystem yet. Phase 37 package layouts, manifest exports, and Cargo alias metadata are treated as internal implementation/demo artifacts rather than compatibility commitments. If an external package appears before this phase lands, the internal fixture migration command can be used as the recovery path, but the production design should still converge on the canonical model below.

Changed behavior:

- `sifr init`: creates the canonical `src/` layout instead of the Phase 37 demo `sifr/<package>/` layout.
- `sifr add`: updates Sifr-facing dependency declarations first, then projects to Cargo dependencies and Cargo dependency renames.
- `sifr remove`: removes the Sifr-facing dependency declaration and its projected Cargo dependency.
- `sifr update`: updates through the Sifr-facing dependency identity, then delegates lockfile mutation to Cargo.
- `sifr run`: can select an implicit `src/main.sifr` target, discover flat `src/bin/*.sifr` targets, run explicit `.sifr` files with or without `sifr.toml`, and fetch missing dependencies in unconstrained online local development.
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
- Normal runnable targets should be discovered from source layout. Manifest entries should be needed only for nonstandard behavior.
- Manifest-less single-file execution remains a first-class learning and scripting path.
- Sifr must not mirror Cargo's full failure taxonomy. Cargo process failures use one stable Sifr wrapper diagnostic that preserves the underlying Cargo error, while Sifr-owned package policy failures get specific Sifr codes.

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
    bin/
      admin.sifr
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

Pure marker lifecycle:

- `sifr init --lib` and `sifr init --bin` create the canonical pure marker when no Rust backend is requested.
- Every package-aware command that derives a package graph validates pure marker contents before compiling Sifr sources.
- If a user edits the marker to add Rust implementation code without declaring Rust-backed trust policy, Sifr reports the existing pure-marker diagnostic (`SIFR-PACKAGE-0501`) before invoking Cargo.
- Sifr must not silently convert a pure Sifr package into a Rust-backed package.
- To intentionally add Rust-backed behavior, the user must configure trust policy and move the package into Rust-backed package mode.

## `sifr.toml` Contract

Minimal library:

```toml
[package]
name = "demo_json"
edition = "2026"
sifr-version = ">=0.3,<0.4"
```

Minimal app:

```toml
[package]
name = "demo_app"
edition = "2026"
sifr-version = ">=0.3,<0.4"
```

Defaults:

- `[source].root` defaults to `src`.
- A library package exposes `src/__init__.sifr` when present.
- `src/main.sifr` is the default app target when present.
- `src/bin/<name>.sifr` defines an additional named app target `<name>`. Nested `src/bin/**` target names are deferred to v2.
- Apps may also include `src/__init__.sifr` if they are importable by other packages.

Canonical package model:

- New package creation commands must generate the `src/` layout.
- Public package API is derived from `__init__.sifr`.
- Manifest-level `[exports] modules = [...]` is not part of the production package model.
- Sifr manifest `[[bin]]` target tables are not part of the production package model. Sifr discovers app targets from `src/main.sifr` and `src/bin/*.sifr`; if Cargo needs target tables, Sifr generates them in the Cargo projection.
- Phase 37 demo layouts and manifest exports are treated as internal implementation fixtures only until this adhoc phase replaces them; there is no external backward-compatibility contract to preserve.
- If a production package uses `[exports].modules` after this phase, Sifr reports `SIFR-PACKAGE-0701` and directs the maintainer to move the public API to `src/__init__.sifr`.
- If a production package uses Sifr manifest `[[bin]]` tables after this phase, Sifr reports `SIFR-PACKAGE-0711` and directs the maintainer to use `src/main.sifr`, `src/bin/*.sifr`, or an explicit `sifr run --bin <name>`.

Initialization semantics:

```text
sifr init --lib demo_json
  creates Cargo.toml with Cargo package name sifr-demo-json
  creates sifr.toml with [source].root = "src" or omits it to use the default
  creates src/__init__.sifr
  creates src/lib.rs with the canonical pure marker

sifr init --bin demo_app
  creates Cargo.toml with Cargo package name sifr-demo-app
  creates sifr.toml with default source root and no explicit binary target table
  creates src/main.sifr
  creates src/lib.rs with the canonical pure marker
  may create src/__init__.sifr only when --importable is requested
```

Existing directory behavior:

- `sifr init` fails if the target directory contains files and no explicit `--force` flag is provided.
- `--force` may create missing Sifr-owned files but must not overwrite existing user files.
- If an existing `Cargo.toml` or `sifr.toml` is present, `sifr init` reports projection/import-plan diagnostics and suggests `sifr fix --check` or a migration command instead of guessing.
- Non-interactive behavior is required; no prompt-only path may be needed for CI.

App target discovery:

```text
src/main.sifr        -> app target named from [package].name or the package directory
src/bin/admin.sifr   -> app target named "admin"
```

`sifr run` resolution order in a package context:

1. If the first positional argument ends in `.sifr`, or contains a path separator and resolves to a `.sifr` file, compile and run that explicit file as an ephemeral package-local target.
2. If `--bin <name>` is provided, use the matching discovered app target.
3. If the first positional argument has no `.sifr` extension and matches a discovered app target, use that target.
4. If no target argument is provided and `src/main.sifr` exists, use it as the default app target.
5. If no target argument is provided and exactly one discovered app target exists, use that target.
6. Otherwise report `SIFR-PACKAGE-0605` for missing, unknown, or ambiguous runnable target.

`sifr run` with arguments after `--` passes those arguments to the selected app target or explicit `.sifr` file. Arguments after `--` are not parsed as Sifr CLI flags.

Manifest-less single-file execution:

- `sifr run path/to/file.sifr` must work when no `sifr.toml` exists in the current directory or any parent directory.
- Manifest-less mode compiles the explicit file with source root equal to the file's parent directory, no package identity, no external package dependencies, no `__init__.sifr` public API derivation, and no Cargo package graph.
- Manifest-less mode may still use the generated one-file Cargo project or rustc path that the existing non-package CLI uses today.
- If a `sifr.toml` exists in the current directory or a parent directory, explicit `.sifr` file execution runs in package-aware mode when the file is under the selected package source root.
- If an explicit `.sifr` file is outside the selected package source root while a package context exists, Sifr reports `SIFR-PACKAGE-0710` and tells the user to run from outside the package, move the file under `src/`, or pass a future explicit `--standalone` flag if that flag is added.
- `sifr check path/to/file.sifr` and `sifr emit path/to/file.sifr` follow the same manifest-less/package-aware split.

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
- Re-exported public names are extracted from supported relative `from` import forms.
- Definitions written directly in `__init__.sifr` are public names of that namespace when their names do not start with `_`.
- A public namespace path is valid only when every namespace segment is represented by a directory with `__init__.sifr`.
- Filesystem presence confirms possible module locations; the namespace API graph determines what is public.
- A directory without `__init__.sifr` is not a public namespace across package boundaries, regardless of its contents.
- Privacy checks use the derived namespace API graph, not filesystem presence alone.
- A cross-package import into an implementation file that is not reachable through a public namespace API graph reports `SIFR-PACKAGE-0203`.
- The first implementation should reject dynamic or wildcard public API construction in `__init__.sifr` with a stable diagnostic rather than guessing.

`parse_init_sifr_reexports` algorithm:

Input:

```text
namespace_path: DottedModulePath
init_path: PathBuf
package_source_root: PathBuf
```

Output:

```text
NamespaceApi
  namespace: DottedModulePath
  public_symbols: BTreeMap<String, PublicSymbolOrigin>
  public_child_namespaces: BTreeMap<String, DottedModulePath>
  diagnostics: Vec<PackageDiagnostic>
```

Supported public forms in `__init__.sifr`:

```sifr
from .parse import parse_json
from .parse import parse_json as loads
from .value import DemoJsonValue, DemoJsonError
from .codecs import decode_json

class PublicClass:
    ...

def public_factory() -> PublicClass:
    ...

type PublicAlias = dict[str, int]
```

Semantics:

- `from .module import name` exposes `name` at the current namespace when `module` is a local implementation file or public child namespace.
- `from .module import name as alias` exposes `alias`.
- `from . import child_namespace` exposes `child_namespace` as a child namespace only when `child_namespace/__init__.sifr` exists.
- Top-level `class`, `def`, and `type` definitions in `__init__.sifr` expose their names unless the names start with `_`.
- Multiple exports of the same public name must resolve to the same origin or report a duplicate-public-api diagnostic.
- `from .module import *`, dynamic `__all__`-style construction, runtime assignment-based exports, and absolute imports that attempt to define the package API are rejected in `__init__.sifr` for this phase.
- Bare `import module` and `import module as alias` do not define public API in this phase; if Sifr syntax supports them locally, they remain implementation imports only.
- Local package code may still import implementation files directly; these restrictions apply only to cross-package public API derivation.

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

[test-dependencies]
demo_test_support = { package = "sifr-demo-test-support", git = "https://github.com/sifr-lang/sifr-demo-test-support", tag = "v0.1.0" }

[dev-dependencies]
demo_bench_support = { package = "sifr-demo-bench-support", path = "../bench-support" }
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

Dependency groups:

- `[dependencies]` is the runtime dependency set and is included by `sifr run`, `sifr check`, `sifr build`, `sifr test`, `sifr package`, and `sifr publish`.
- `[test-dependencies]` is the test-only dependency group and is included by `sifr test` by default.
- `[dev-dependencies]` is the local development group and is included by `sifr test` by default, but not by `sifr run`, `sifr build`, `sifr package`, or `sifr publish` unless explicitly requested.
- The lockfile plan should resolve all declared dependency groups together when the lockfile is created or updated, so switching groups does not produce command-specific lock drift.
- Build/package/publish plans compile only the dependency groups selected for the command.
- Custom dependency groups and group composition are deferred to v2.

Dependency command behavior:

- `sifr add --alias name` writes the table key `name` in `sifr.toml [dependencies]` and sets `import = "name"` when the alias differs from the dependency's resolved Sifr package name.
- `sifr add --test <package>` writes `[test-dependencies]`.
- `sifr add --dev <package>` writes `[dev-dependencies]`.
- `sifr remove --test` and `sifr remove --dev` remove from those groups; without a group flag, `sifr remove` removes from `[dependencies]` only and reports a diagnostic if the alias exists only in another group.
- Package commands accept `--group test`, `--group dev`, and `--no-default-groups` where group selection is meaningful. `--only-group`, `--no-group`, and `--all-groups` are deferred to v2.
- Default selected groups:
  - `sifr run`, `sifr check`, and `sifr build`: runtime dependencies only;
  - `sifr test`: runtime, `test`, and `dev`;
  - `sifr package` and `sifr publish`: runtime dependencies only, with preflight checking that excluded groups are not required by public API or selected app targets;
  - `sifr fetch` and `sifr tree`: all groups by default, with group flags available to narrow output/operations.
- `sifr add`, `sifr remove`, and `sifr update` write only `sifr.toml` dependency tables first, then regenerate Sifr-owned Cargo dependency sections.
- Existing Phase 37 alias metadata in `Cargo.toml [package.metadata.sifr.aliases]` is internal transitional data only. This adhoc phase may delete or rewrite it when regenerating package projections; it does not need a user-facing compatibility mode.
- Direct `cargo add` on Sifr-owned dependency sections is not bidirectionally synced. It creates projection drift; `sifr fix --check` reports it and `sifr fix` may either remove the Cargo-only dependency from Sifr-owned sections or ask the user to import it into `sifr.toml`.
- User-owned Cargo dependencies for Rust backend implementation are allowed only in user-owned Cargo sections and must be validated against trust policy when reachable from a Rust-backed Sifr package.
- A manual Cargo dependency that makes Sifr package imports available without a matching `sifr.toml` dependency is not considered a public Sifr dependency and must not be used for Sifr import resolution.

Cargo projection for groups:

- Runtime `[dependencies]` project to Cargo `[dependencies]`.
- `[test-dependencies]` and `[dev-dependencies]` project to Cargo `[dev-dependencies]` when Cargo needs those packages for generated tests or local development builds.
- Cargo cannot distinguish Sifr test-only and dev-only dependencies in its manifest schema. `sifr.toml` remains the source of truth for that distinction.

Alias conflicts:

- Within one importing package scope, two dependency declarations may not expose the same `import` root unless they resolve to the same package instance and the same public API graph.
- Two aliases pointing at different versions or sources with the same `import` root report `SIFR-PACKAGE-0201` or its successor duplicate-import-root diagnostic.
- Two aliases pointing at the same exact package instance with different `import` roots are allowed, but they are treated as two names for one type identity, not two package instances.

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

- `sifr init`, `sifr add`, `sifr remove`, and `sifr update` update Sifr package metadata first for new-layout packages.
- Sifr then updates Sifr-owned Cargo projection sections deterministically.
- Sifr-owned Cargo sections are marked and guarded.
- Advanced users may keep custom Cargo sections for Rust-backed packages, but Sifr validates they do not conflict with the package graph.
- Cargo source IDs are treated as opaque outputs of Cargo metadata.
- Sifr never shells into Cargo internals or links to Cargo private APIs.
- Sifr does not rewrite user-owned Cargo sections except through explicit migration/fix commands.
- Drift is any mismatch inside Sifr-owned projection sections, any missing required Sifr metadata pointer, or any Sifr-owned dependency missing from the Cargo projection.
- Extra user-owned Cargo dependencies are allowed when they are not used as Sifr package dependencies; Rust-backed packages must still satisfy trust policy for reachable backend crates.
- Projection drift always fails package-aware commands before invoking Cargo. `sifr fix --check` reports the same failures without writing; `sifr fix` attempts to repair Sifr-owned projection drift unless `--frozen` is active.

Sifr-owned Cargo section marking:

- Generated sections include stable comments or metadata markers identifying Sifr ownership.
- Guardrails verify those markers exist for generated packages and that projection regeneration is idempotent.
- User-owned sections must be preserved byte-for-byte where practical; semantic TOML rewrites must be documented and tested if byte preservation is not possible.
- Sifr-owned dependency projection includes dependency entries generated from `sifr.toml`, `[package.metadata.sifr]`, include patterns, and the pure marker target when Sifr created it.

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

Manifest discovery hook:

- `[package.metadata.sifr] manifest = "sifr.toml"` remains required in the generated Cargo projection for package-aware dependencies.
- The pointer exists so `cargo metadata --format-version 1` can tell Sifr which Cargo packages are Sifr-capable without Sifr scanning every Cargo package root.
- The pointer is not the source of truth for package semantics and is not a trust anchor. `sifr.toml` contents remain authoritative after discovery.
- A local workspace/package root may be discovered by finding `sifr.toml` directly before invoking Cargo, but dependency packages discovered through Cargo metadata must carry the pointer to be treated as Sifr packages.
- Missing, unreadable, or unparsable manifests referenced by `[package.metadata.sifr].manifest` report `SIFR-PACKAGE-0703`.
- If the pointer is absent from a package that otherwise looks like a Sifr package, Sifr treats that as projection drift for Sifr-owned packages, not as a Cargo failure.

Pure marker:

```rust
// Pure Sifr package marker. Sifr source lives in sifr.toml source roots.
```

Drift diagnostics:

- `SIFR-PACKAGE-0702`: projected Cargo dependency or alias differs from `sifr.toml` dependency declaration.
- `SIFR-PACKAGE-0703`: `Cargo.toml` is missing `[package.metadata.sifr]`, the manifest pointer path is wrong, or the referenced manifest is unreadable or unparsable.
- `SIFR-PACKAGE-0704`: Cargo include/exclude omits `sifr.toml`, `src/**/*.sifr`, or the required marker/backend files.
- `SIFR-PACKAGE-0705`: `[source].root` points to a missing or non-directory path.
- `SIFR-PACKAGE-0709`: pure package marker is missing and cannot be regenerated because a user-owned Rust target already exists.
- `SIFR-PACKAGE-0710`: explicit `.sifr` file target is outside the selected package source root.
- `SIFR-PACKAGE-0711`: production `sifr.toml` uses Sifr manifest `[[bin]]` tables instead of layout-discovered app targets.

Recovery:

- `sifr fix --package <name>` may update Sifr-owned Cargo projection sections, regenerate the pure marker, and repair include patterns.
- `sifr fix` must not overwrite user-owned Rust backend code.
- `--locked`, `--offline`, and `--frozen` commands report drift before running Cargo operations.
- `--frozen` rejects any automatic projection write.
- `sifr fix --check` never writes; it exits non-zero when drift is present.
- `sifr fix` without `--check` writes only Sifr-owned projection sections and refuses to write when `--frozen` is set.
- `sifr fix` does not mutate `Cargo.lock` unless a dependency update operation explicitly requests lockfile mutation.

## CLI Commands

Package-aware core commands:

```bash
sifr run [target-or-path] [--bin name] [--locked|--offline|--frozen] [--features f1,f2|--all-features|--no-default-features] [--] [app-args...]
sifr check [path-or-package] [--workspace] [-p|--package package] [--filter selector] [--locked|--offline|--frozen]
sifr build [path-or-package] [--workspace] [-p|--package package] [--filter selector] [--locked|--offline|--frozen]
sifr test [path-or-package] [--workspace] [-p|--package package] [--filter selector] [--locked|--offline|--frozen] [--group test|--group dev|--no-default-groups]
```

Dependency and package operations:

```bash
sifr init [--lib|--bin] [name]
sifr fetch [--locked|--offline|--frozen]
sifr add <package> [--git url] [--tag tag] [--path path] [--dev|--test] [--alias name] [--features f1,f2]
sifr remove <package-or-alias> [--dev|--test]
sifr update [package-or-alias]
sifr tree [--workspace|-p|--package package] [--sifr-only|--all] [--depth N] [--group test|--group dev|--no-default-groups]
sifr vendor <dir> [--locked]
sifr package [--dry-run]
sifr publish [--dry-run]
sifr fix [--package name] [--check]
```

Command semantics:

- `sifr run` without lock/network flags may fetch dependencies and update `Cargo.lock`, matching Rust's local development convenience.
- `sifr run path/to/file.sifr` works without `sifr.toml` in manifest-less mode and works inside a package when the file is under the selected source root.
- Arguments after `--` are passed to the selected app target or explicit `.sifr` file and must not be consumed by the Sifr CLI.
- `sifr fetch --locked` fetches exactly the locked graph and fails if the lockfile needs an update.
- `sifr check --locked --offline` never touches the network and fails if any selected dependency source is absent locally.
- `-p` and `--package` are aliases for package selection, matching Cargo naming.
- `--group test` and `--group dev` include those groups alongside command defaults. `--no-default-groups` removes command-specific default groups such as `test` and `dev` from `sifr test`.

## Package Session

All package-aware commands must go through a single orchestration layer:

```text
PackageSession
  workspace_root
  selected_packages
  explicit_file_target
  manifest_less_mode
  lock_mode
  network_mode
  selected_dependency_groups
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
  selected_dependency_groups: Vec<DependencyGroupSelection>
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
- Manifest-less explicit `.sifr` file execution bypasses Cargo metadata and package graph derivation, but still uses a small operation plan so lock/network flags, diagnostics, and command output stay consistent.
- Package-aware explicit `.sifr` file execution uses `PackageSession` when the file is under the selected source root.
- Dependency group selection is computed before projection and Cargo command planning.
- Projection drift is checked before Cargo command execution.
- Offline source availability validation reuses the Phase 37 `validate_offline_source_availability` model. Additional constraints must be documented as Sifr-owned preflight checks rather than inferred from Cargo cache internals.
- Sifr validates the Sifr package graph, including package cycles, duplicate import roots, and package selection, before invoking Cargo for package-aware commands.
- Multi-package operations compute package topological order from `SifrPackageGraph`; Cargo may build in its own internal order, but Sifr diagnostics and cache keys use the stable topological order.
- Cargo command stderr/stdout is captured as structured cause data and attached to wrapper diagnostics after credential redaction.

Responsibilities:

- Discover package/workspace root.
- Detect manifest-less explicit-file execution and short-circuit package graph work when no `sifr.toml` context exists.
- Ensure the Cargo projection is current.
- Run or plan Cargo fetch/metadata/build/package/publish operations.
- Parse and normalize Cargo metadata.
- Derive `SifrPackageGraph`.
- Build the `PackageSourceMap` using `__init__.sifr` namespace rules.
- Validate compiler compatibility, privacy, trust policy, lock modes, and package selection.
- Wrap Cargo process failures in the stable Cargo-failure diagnostic without reclassifying every Cargo error.
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
- This no-panic requirement inherits the Phase 37 generated-runtime safety contract and does not create a package-specific exception.

`SIFR-PACKAGE-0204` machine-readable fields:

```text
expected_package_instance_id
actual_package_instance_id
expected_cargo_package_id
actual_cargo_package_id
import_path
dependency_path
expected_type
actual_type
```

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

- If root `Cargo.toml` is a virtual `[workspace]` with no `[package]` and `sifr.toml` exists directly beside that root `Cargo.toml`, Sifr reports `SIFR-PACKAGE-0706` as a warning.
- The warning says the root `sifr.toml` has no package identity and should be moved to a package member or the workspace root should be converted into a package.
- A root `sifr.toml` in a virtual workspace is not used for package graph derivation.
- Member package `sifr.toml` files under `packages/*` or any workspace member path are not affected by this warning.

Filters:

- `--filter` selector parsing and dependency/dependent closure semantics inherit the Phase 37 selector model unless this phase explicitly changes them.
- Any new selector forms must be added as separate diagnostics/tests rather than modifying existing Phase 37 filter behavior implicitly.

## Packaging And Publishing

Sifr packages are Cargo packages with Sifr metadata and source files. The archive format remains Cargo's package archive.

Preflight before `cargo package` or `cargo publish`:

- `sifr.toml` is present and valid.
- `src/**/*.sifr` files required by public API and selected targets are included.
- `src/__init__.sifr` exists for libraries that expose a public API.
- `src/main.sifr`, `src/bin/*.sifr`, or another selected explicit app target exists for apps.
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

Publish failure boundary:

- Sifr preflight failures happen before `cargo publish` and use Sifr-owned diagnostics.
- Once `cargo publish` starts, Cargo owns upload/authentication/registry mutation semantics.
- If Cargo reports a publish failure after partial registry-side work, Sifr wraps the failure in `SIFR-PACKAGE-0101` and does not attempt rollback unless Cargo exposes a stable supported rollback command.
- Human-readable output must make clear that registry state should be checked with the registry/Cargo tooling named in the underlying Cargo excerpt.

## Diagnostics

### Cargo Failure Boundary

Sifr must avoid inheriting Cargo's complete error taxonomy. The maintainable rule is:

- Sifr-specific policy and compiler failures receive specific Sifr diagnostics.
- Cargo process failures receive a single stable wrapper diagnostic, `SIFR-PACKAGE-0101`.
- The wrapper includes the Cargo subcommand, current directory, redacted arguments, exit status, lock/network mode, and a redacted excerpt of Cargo stderr/stdout.
- The wrapper points users to the underlying Cargo failure text instead of attempting to duplicate Cargo's own explanations.
- Sifr may add targeted help only when the recovery is clearly Sifr-owned, such as "run `sifr fetch --locked` before retrying with `--offline`".
- Sifr must not add a new Sifr diagnostic code for every Cargo resolver, registry, Git, credential, feature, or publish error.
- Credential-related Cargo failures, including `401`, `403`, auth-helper failures, `cargo login` failures, missing registry tokens, and Git credential failures, are still wrapped in `SIFR-PACKAGE-0101`.
- Existing or older credential-specific Cargo-failure codes such as `SIFR-PACKAGE-0105` must be retired, documented as superseded, or mapped to `SIFR-PACKAGE-0101` during this phase. They must not remain as active classification targets for Cargo stderr variants.

Specific Sifr package diagnostics are reserved for failures Sifr can define and validate independently of Cargo internals:

- invalid `sifr.toml`;
- projection drift;
- package source layout/privacy violations;
- Sifr import ambiguity/transitive/private access;
- Sifr compiler compatibility;
- Sifr trust policy violations;
- archive preflight omissions;
- Sifr workspace/package selection errors;
- known offline source absence before invoking Cargo.

Rationale:

- Cargo owns the resolver, registry, Git, credential, feature, and publish error taxonomy.
- Sifr owns redaction, stable command context, package context, and routing users toward the underlying Cargo failure.
- Adding Sifr codes for every recognizable Cargo stderr shape would create a permanent compatibility burden and would go stale as Cargo evolves.

`SIFR-PACKAGE-0101` machine-readable fields:

```text
code: "SIFR-PACKAGE-0101"
action: metadata | fetch | build | check | test | package | publish | vendor | add | remove | update
current_dir: absolute path where Cargo was invoked
args_redacted: redacted Cargo argument vector
exit_status: process exit code when available
lock_mode: unlocked | locked | offline | frozen
network_mode: online | offline
package: selected Sifr package name when available
package_instance: resolved Sifr package instance id when available
dependency_alias: importing dependency alias when available
source_kind: path | git | registry | unknown
stderr_redacted: bounded redacted stderr excerpt
stdout_redacted: bounded redacted stdout excerpt, included only when relevant
help: Sifr-owned recovery hint when one is safe and specific
```

`source_kind = "unknown"` is used when Sifr cannot determine source kind before invoking Cargo, or when a Cargo failure occurs before usable metadata is available. Human-readable output should omit source-kind-specific advice in that case and rely on the underlying Cargo excerpt plus generic recovery guidance.

Human-readable output:

- Shows the Sifr wrapper heading and stable context.
- Shows the redacted Cargo excerpt under an "Underlying Cargo failure" label.
- Makes clear that `SIFR-PACKAGE-0101` is a wrapper, not an attempt to reinterpret Cargo's full error taxonomy.

Diagnostic docs for `SIFR-PACKAGE-0101`:

- Explain that the code wraps a Cargo command failure.
- Document where to find the underlying Cargo excerpt in human output.
- Document redaction behavior.
- Give generic next steps: rerun with the displayed Sifr command, inspect the Cargo excerpt, authenticate with Cargo for credential errors, or run `sifr fetch --locked` before offline commands when the help text says so.
- Do not list every possible Cargo failure mode.

Credential and sensitive data redaction:

- Redaction applies to command arguments, stderr, stdout when captured, environment-derived credential snippets, and generated diagnostics.
- Credential patterns are matched case-insensitively and include `token=`, `bearer`, `gh_`, `gho_`, `ghp_`, `ghs_`, `ghr_`, `ghu_`, `cargo:token`, `secret=`, `password=`, `api_key=`, and `x-token:`.
- URL redaction preserves useful error signal. Public registry URLs such as `https://crates.io/api/v1/crates` remain visible.
- If a URL contains recognized credential material, userinfo and host are redacted while scheme and path are preserved. Example: `https://user:token@private.example.com/pkg` becomes `https://[redacted host]/pkg`.
- URL redaction must use URL parsing or an equivalent structured parser; word-level substring replacement is not sufficient for userinfo redaction.
- File paths, line numbers, package names, registry names without embedded credentials, and Cargo package ids remain visible unless they match credential patterns.
- Stderr is captured by default. Stdout is captured only when stderr is empty or when the Cargo operation is known to emit relevant failure text there.
- Captured excerpts are bounded by line count and byte count to keep diagnostics stable.
- Redaction tests must include both overbroad and underinclusive cases: public URLs must not be removed, and common token/secret/password forms must be removed.

Credential-specific code retirement process:

- `docs/errors/SIFR-PACKAGE-0105.md`, if present, must be rewritten as a superseded-code page that points to `SIFR-PACKAGE-0101`.
- Diagnostic docs sync must allow superseded pages but forbid active code references to retired Cargo-taxonomy variants.
- Existing tests expecting credential-specific Cargo-failure codes must be migrated to assert `SIFR-PACKAGE-0101` plus redacted credential help.
- Guardrails must reject new active diagnostic constants that classify Cargo stderr variants instead of Sifr-owned policy failures.
- The implementation order for milestone 3 must include removing active `SIFR-PACKAGE-0105` emission from Cargo failure mapping before package CLI commands are considered complete.

Required diagnostic behavior:

- Missing offline package source: `SIFR-PACKAGE-0104`, with help to run `sifr fetch --locked`.
- Cargo command failure: `SIFR-PACKAGE-0101`, with redacted command summary and underlying Cargo failure excerpt.
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

- `0701`: manifest-level `[exports].modules` used in a production package instead of `__init__.sifr`.
- `0702`: Cargo projection dependency or alias drift.
- `0703`: missing or incorrect `[package.metadata.sifr]`.
- `0704`: Cargo include/exclude omits required Sifr package files.
- `0705`: invalid source root.
- `0706`: ignored `sifr.toml` at virtual workspace root.
- `0709`: pure marker missing but user-owned Rust target prevents regeneration.
- `0710`: explicit `.sifr` file target is outside the selected package source root.
- `0711`: production `sifr.toml` uses Sifr manifest `[[bin]]` tables instead of layout-discovered app targets.

Future diagnostic allocation:

- New Sifr package diagnostics require an entry in diagnostic docs, code coverage checks, and the package guardrail script.
- Retired diagnostics remain documented as superseded pages and must not be emitted by active code.
- Cargo stderr shape changes must not drive new Sifr diagnostic codes unless the failure is redefined as a Sifr-owned preflight or policy check.

## Internal Fixture Migration Plan

Existing Phase 37 demo/fixture layout:

```text
sifr/demo_json/__init__.sifr
sifr/demo_json/parse.sifr
```

New canonical layout:

```text
src/__init__.sifr
src/parse.sifr
```

V1 migration approach:

- Layout migration is documented as a manual procedure or simple in-repo script for fixtures and demo repositories.
- A dedicated `sifr package migrate-layout` command, migration rollback descriptors, and partial-apply behavior are deferred to v2.

Migration rules:

- Move package source root to `src`.
- Convert package-root `__init__.sifr` into `src/__init__.sifr`.
- Rewrite `[source].root = "sifr"` to `[source].root = "src"` or remove it when `src` is the default.
- Regenerate Cargo include patterns from `sifr/**/*.sifr` to `src/**/*.sifr`.
- Regenerate or verify `src/lib.rs` pure marker for pure packages.
- Rewrite local imports as needed.
- Preserve public API names.
- No external package compatibility mode is required; this migration exists to convert in-repo fixtures and demo repositories to the canonical model.
- Update demo repositories only after the package-aware compiler supports the new layout.

Migration validation:

1. Before migration, snapshot the public API from `__init__.sifr` or legacy `[exports].modules`.
2. After migration, derive the public API from the new `src/__init__.sifr` namespace graph.
3. Diff the public API snapshots and fail the migration script or manual checklist if names are lost or added without an explicit note.
4. Compile/package-check all local imports in the package to catch broken relative paths.
5. Run archive preflight to confirm required Sifr files are included.
6. Do not modify `Cargo.lock` package source revisions during layout migration.

Production schema:

- New packages use `[source].root` singular, defaulting to `src`.
- `source.roots` and `[exports].modules` are not production package schema after this phase.
- Sifr manifest `[[bin]]` tables are not production package schema after this phase; app targets are discovered from `src/main.sifr` and `src/bin/*.sifr`.
- Internal Phase 37 fixtures may continue to exercise the old parser/model until their tests are replaced or migrated, but no user-facing command should generate the old schema.
- After milestone 7, guardrails should fail newly added package-management fixtures that use `sifr/<package>/` layout, manifest `[exports].modules`, or Sifr manifest `[[bin]]` tables unless the test is explicitly marked as a parser/backfill regression.

Demo and fixture strategy:

- Milestone-level tests use in-tree fixtures under `verification/package_management/src_layout_fixtures/` so implementation can advance before public demo repositories are migrated.
- Published `sifr-demo-*` repositories stay on the current internal-demo layout only until package-aware compiler integration supports the new layout end to end.
- Milestone 7 migrates the demo repositories to the canonical production layout and records command transcripts for clone, fetch, run, offline check, workspace selection, and publish dry-run.

## Milestones

### milestone_adhoc_pkg_1: Package UX Contract And Source Layout

Scope:

- Finalize the `src/` layout, `src/__init__.sifr` public API rule, namespaced `__init__.sifr` rule, and `src/main.sifr` run target.
- Finalize layout-discovered app targets (`src/main.sifr`, flat `src/bin/*.sifr`) and the absence of Sifr manifest `[[bin]]` in production schema.
- Finalize manifest-less explicit file semantics for `sifr run/check/emit path/to/file.sifr`.
- Add parser/source-map tests for public root imports, public namespace imports, private implementation rejection, and local private imports.
- Implement `parse_init_sifr_reexports` and namespace API graph derivation in `PackageSourceMap`.
- Add rejection tests for production packages that still use manifest `[exports].modules`.
- Add rejection tests for production packages that still use Sifr manifest `[[bin]]` tables.
- Document the layout in `docs/package_management.md`.

Validation:

- `cargo test -p sifr_package source_layout`
- `cargo test -p sifr -- manifest_less`
- `python3 scripts/check_package_manager_guardrails.py`
- `python3 scripts/check_diagnostic_docs_sync.py`

Acceptance:

- The source map can derive public package APIs solely from `__init__.sifr`.
- No manifest `[exports]` entry is needed for new packages.
- `sifr run any-other-name.sifr` has a documented manifest-less behavior and package-context behavior.

### milestone_adhoc_pkg_2: Sifr-Managed Cargo Projection

Scope:

- Add projection code that creates/updates Cargo package metadata from `sifr.toml`.
- Generate pure marker `src/lib.rs` when absent.
- Preserve user-owned Cargo sections for Rust-backed packages.
- Add drift diagnostics when Cargo projection does not match Sifr package metadata.
- Implement the `[package.metadata.sifr] manifest = "sifr.toml"` discovery hook as generated projection, document it as a discovery pointer, and test `SIFR-PACKAGE-0703`.
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
- Wire manifest-less explicit-file runs through a small non-package operation plan so `sifr run main.sifr` remains available without `sifr.toml`.
- Implement dependency group selection for `runtime`, `test`, and `dev`.
- Implement `-p|--package`, `--group test`, `--group dev`, `--no-default-groups`, and `--` app-argument passing.
- Implement the `SIFR-PACKAGE-0101` Cargo wrapper schema, including action, current directory, redacted args, exit status, lock/network mode, package context, bounded redacted Cargo excerpts, and JSON output fields.
- Retire or supersede credential-specific Cargo-failure codes such as `SIFR-PACKAGE-0105` so credential failures route through `SIFR-PACKAGE-0101`.
- Add redaction tests that prove public URLs and useful Cargo context survive while tokens, passwords, URL credentials, and private hostnames with embedded credentials are removed.

Validation:

- `cargo test -p sifr_package package_session`
- `cargo test -p sifr -- package_cli`
- Targeted CLI tests for `sifr run main.sifr` without `sifr.toml`, `sifr run --bin <name> -- args`, and dependency group defaults.
- Demo command: `sifr fetch --locked` in `sifr-demo-app`.
- Targeted regression tests for `SIFR-PACKAGE-0101` wrapper fields, redaction, retired credential-code mapping, stdout/stderr capture bounds, and `sifr fix --check` no-write behavior.

Acceptance:

- A fresh cache can run `sifr fetch --locked` and then `sifr check --locked --offline`.

### milestone_adhoc_pkg_4: Package-Aware Compiler Imports

Scope:

- Connect `PackageSourceMap` to HIR/name resolution/import resolution.
- Support package root imports and public namespace imports.
- Reject private, transitive, and ambiguous imports with stable diagnostics.
- Make `sifr run` compile selected layout-discovered package app targets and explicit package-local `.sifr` file targets.

Validation:

- E2E pass/fail fixtures for package imports.
- Demo command: `sifr run --locked --offline` in `sifr-demo-app`.

Acceptance:

- `sifr-demo-app` can import `demo_json_v1`, `demo_json_v2`, and `demo_http` through Sifr package APIs without direct Cargo commands.

### milestone_adhoc_pkg_5: Workspaces, Aliases, And Multiple Versions

Scope:

- Support `sifr check --workspace`, `-p`, and `--filter` through `PackageSession`.
- Support aliases for multiple versions through Sifr-facing dependency config.
- Ensure dependency group selection composes correctly with workspaces, aliases, and multiple versions.
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
- Validate that public API and selected app targets do not depend on excluded `test` or `dev` dependencies.
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

- Add documented manual migration or a simple in-repo fixture migration script.
- Update `sifr-demo-*` repositories to canonical `src/` layout.
- Add end-to-end demo docs showing first clone, fetch, run, explicit file run, offline check, workspace selection, dependency group behavior, and publish dry-run.
- Extend guardrails to enforce source layout, projection boundaries, and demo commands.
- Extend `scripts/check_package_manager_guardrails.py` to allow old Phase 37 layouts only in explicitly named internal regression fixtures during migration and require `src/` layout, no manifest exports, and no Sifr manifest `[[bin]]` tables for all production demos after closeout.
- Add guardrails that forbid active Cargo stderr taxonomy diagnostics, require projection idempotency tests, validate Sifr-owned Cargo section markers, and ensure pure-marker modifications are caught before Cargo execution.
- Document Cargo compatibility assumptions: Sifr relies on stable `cargo metadata --format-version 1` fields and stable Cargo CLI subcommand behavior; if Cargo changes those surfaces, Sifr updates the adapter boundary rather than leaking Cargo internals to users.
- Run full local validation.

Validation:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
- Demo transcript checked into docs or verification artifacts.

Acceptance:

- A user can clone `sifr-demo-app`, run Sifr commands only, and understand failures from Sifr diagnostics.

## Implementation Order

1. Source layout and `__init__.sifr` source-map rules.
2. Layout-discovered app targets and manifest-less explicit-file mode.
3. Managed Cargo projection and drift diagnostics.
4. Dependency groups and group-aware package session planning.
5. Package session and `sifr fetch`/`sifr tree`/package-aware `sifr check`.
6. Compiler import integration.
7. `sifr run` and `sifr test`.
8. Workspace/alias/multiple-version hardening.
9. Publish/vendor/migration/docs/demo closeout.

This order keeps the compiler integration dependent on a stable package source map, and keeps publishing dependent on the final canonical layout.

## Deferred To V2

- `[scripts]` named workflow aliases.
- Custom dependency groups (`[dependency-groups.<name>]`) and group composition.
- `[package].default-run`.
- `--message-format json` and a stable machine-readable command-output schema.
- `sifr --explain <diagnostic-code>`.
- `sifr package migrate-layout` with rollback and partial-apply support.
- Nested `src/bin/<dir>/<name>.sifr` target names.
- Additional group-selection flags such as `--only-group`, `--no-group`, and `--all-groups`.
- Cargo metadata fallback scanning if stable `cargo metadata` output stops surfacing `package.metadata`.

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
