# Adhoc Phase: Seamless Package DX And Production Package Management

Related phase: `internal_docs/phases/37_package_management.md`

## Status

- [x] milestone_adhoc_pkg_1: Package UX contract and source layout
- [ ] milestone_adhoc_pkg_2: Sifr-managed Cargo projection
- [ ] milestone_adhoc_pkg_3: Package session and CLI command integration
- [ ] milestone_adhoc_pkg_4: Package-aware compiler imports
- [ ] milestone_adhoc_pkg_5: Workspaces, aliases, and multiple versions
- [ ] milestone_adhoc_pkg_6: Packaging, publishing, vendoring, and release checks
- [ ] milestone_adhoc_pkg_7: Migration, docs, demos, and long-term guardrails

## Milestone Progress

### milestone_adhoc_pkg_1: Package UX contract and source layout

Status: implemented and reviewer-approved. PR: pending.

Delivered:

- Production package manifests now default `[source].root` to `src`, keep legacy `[source].roots` for Phase 37 fixtures, and reject production `[exports].modules` (`SIFR-PACKAGE-0701`) plus production Sifr `[[bin]]` tables (`SIFR-PACKAGE-0711`).
- `PackageSourceMap` derives public namespace APIs from `__init__.sifr`, prefixes production `src/` modules with the Sifr package name, and rejects cross-package imports into private implementation files while preserving legacy export-prefix fixture behavior.
- `parse_init_sifr_reexports` covers explicit relative re-exports, top-level public definitions, public child namespaces, duplicate public API symbol diagnostics (`SIFR-PACKAGE-0713`), and rejection of wildcard/dynamic/assignment exports.
- Manifest-less explicit-file behavior has named tests under the `manifest_less` filter, and `docs/package_management.md` documents the canonical `src/` layout and manifest-less split.

Validation:

- `cargo test -p sifr_package` -> PASS, 46 tests.
- `cargo test -p sifr_package source_layout` -> PASS, 1 test.
- `cargo test -p sifr -- manifest_less` -> PASS, 2 tests.
- `python3 scripts/check_package_manager_guardrails.py` -> PASS.
- `python3 scripts/check_diagnostic_docs_sync.py` -> PASS.
- `python3 scripts/check_diagnostic_code_coverage.py` -> PASS.
- `cargo fmt --check` -> PASS.

Review:

- `reviews/adhoc-package-dx-m1-review-pass-1.md` -> CHANGES_REQUESTED for a stale Phase 37 docs assertion.
- `reviews/adhoc-package-dx-m1-review-pass-2.md` -> READY after updating the assertion and rerunning `cargo test -p sifr_package`.

## Problem

Phase 37 established the Cargo-backed package-management substrate: Cargo metadata parsing, Sifr package graph derivation, package source maps, lock modes, trust policy, workspace selection, publish/archive plans, guardrails, and concrete demo repositories.

The developer experience is still not seamless:

- Users must understand Cargo commands (`cargo fetch`, `cargo metadata`, `cargo check`) to work with a Sifr package app.
- `sifr run`, `sifr check`, and `sifr test` are not package-aware end to end.
- The current demo package layout (`sifr/<package>/*.sifr`) is more verbose than necessary and does not feel like a natural source layout.
- Public package APIs are described through manifest exports today, but Rust-like long-term maintainability is better achieved by declaring API shape in source, and Python-like Sifr ergonomics are better achieved with `__init__.sifr`.
- The current plan exposes Cargo-shaped target configuration (`[[bin]]`) in the Sifr manifest even though normal users should get layout-based runnable targets and simple named workflow scripts.
- Development/test-only dependencies should follow Cargo's manifest model instead of introducing uv-style dependency groups.
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
- Do not support Python package tools (`pyproject.toml`, `uv.lock`, wheels) in this phase. Future Python/uv interop, if added, must lower into the same package session model.
- Do not make every `.sifr` file in a dependency public by default.
- Do not add npm-compatible arbitrary shell scripts in this phase. Sifr supports named workflow scripts, but scripts expand to Sifr command plans, not unparsed shell strings or arbitrary external commands.
- Do not make `[[bin]]` a required or normal Sifr manifest concept. Cargo may still receive generated target configuration in the projection when needed.
- Do not add uv-style or custom dependency groups. Sifr dependency sections stay aligned with Cargo-supported dependency sections.

## Changes From Phase 37

Phase 37 remains the substrate. This adhoc phase changes the user-facing model and wires the substrate into the compiler/CLI.

Sifr has no external stable package ecosystem. Phase 37 package layouts, manifest exports, and Cargo alias metadata are internal implementation/demo artifacts rather than compatibility commitments. The internal fixture migration script exists to move in-repo fixtures and demo repositories into the canonical production model below.

Changed behavior:

- `sifr init`: creates the canonical `src/` layout instead of the Phase 37 demo `sifr/<package>/` layout.
- `sifr add`: updates Sifr-facing dependency declarations first, then projects to Cargo dependencies and Cargo dependency renames.
- `sifr remove`: removes the Sifr-facing dependency declaration and its projected Cargo dependency.
- `sifr update`: updates through the Sifr-facing dependency identity, then delegates lockfile mutation to Cargo.
- `sifr run`: can select an implicit `src/main.sifr` target, discover `src/bin/*.sifr` targets, run explicit `.sifr` files with or without `sifr.toml`, run constrained named workflow scripts, and fetch missing dependencies in unconstrained online local development.
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
- Sifr package CLI flags should match Cargo names and semantics whenever the command delegates to Cargo behavior. Sifr-only flags are allowed only for Sifr-owned concepts that Cargo cannot express.
- Named scripts should be ergonomic like npm scripts but maintainable like compiler command plans: no implicit shell parsing, and script expansion must preserve Cargo-compatible flag semantics for nested Cargo-backed commands.
- Manifest-less single-file execution remains a first-class learning and scripting path.
- Sifr must not mirror Cargo's full failure taxonomy. Cargo process failures use one stable Sifr wrapper diagnostic that preserves the underlying Cargo error, while Sifr-owned package policy failures get specific Sifr codes.
- `sifr --explain <diagnostic-code>` is part of the final diagnostic UX and remains Sifr-owned.
- `sifr repair` is the Sifr-owned projection repair command; Sifr does not reuse Cargo's `fix` name because Cargo `fix` applies compiler suggestions.

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

[source]
root = "src"
```

Minimal app:

```toml
[package]
name = "demo_app"
edition = "2026"
sifr-version = ">=0.3,<0.4"
default-run = "demo-app"

[scripts]
dev = { command = "run", args = [] }
check-all = { command = "check", args = ["--workspace"] }
offline = { command = "run", args = ["--locked", "--offline"] }
```

Defaults:

- `[source].root` defaults to `src`.
- A library package exposes `src/__init__.sifr` when present.
- `src/main.sifr` is the default app target when present.
- `src/bin/<name>.sifr` defines an additional named app target `<name>`.
- `[package].default-run`, when present, names the app target selected by `sifr run` when multiple targets exist.
- Apps may also include `src/__init__.sifr` if they are importable by other packages.
- `[scripts]` entries are named workflow aliases for Sifr command plans. They are not distributable binaries and do not replace layout-discovered app targets.

Canonical package model:

- New package creation commands must generate the `src/` layout.
- Public package API is derived from `__init__.sifr`.
- Manifest-level `[exports] modules = [...]` is not part of the production package model.
- Sifr manifest `[[bin]]` target tables are not part of the production package model. Sifr discovers app targets from `src/main.sifr` and `src/bin/*.sifr`; if Cargo needs target tables, Sifr generates them in the Cargo projection.
- Phase 37 demo layouts and manifest exports are internal migration fixtures only; there is no external backward-compatibility contract to preserve.
- If a production package uses `[exports].modules`, Sifr reports `SIFR-PACKAGE-0701` and directs the maintainer to move the public API to `src/__init__.sifr`.
- If a production package uses Sifr manifest `[[bin]]` tables, Sifr reports `SIFR-PACKAGE-0711` and directs the maintainer to use `src/main.sifr`, `src/bin/*.sifr`, `[package].default-run`, `[scripts]`, or `sifr run --bin <name>`.

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

`sifr init --name <name>` controls the Sifr package name. The generated Cargo package name remains the projected Cargo package name, `sifr-<kebab-case-name>`, unless an explicit Rust-backed package projection rule says otherwise.

Existing directory behavior:

- `sifr init` fails if the target directory contains files and no explicit `--force` flag is provided.
- `--force` may create missing Sifr-owned files but must not overwrite existing user files.
- If an existing `Cargo.toml` or `sifr.toml` is present, `sifr init` reports projection/import-plan diagnostics and suggests `sifr repair --check` or the internal migration script instead of guessing.
- Non-interactive behavior is required; no prompt-only path may be needed for CI.

App target discovery:

```text
src/main.sifr        -> app target named from [package].name or the package directory
src/bin/admin.sifr   -> app target named "admin"
src/bin/tools/migrate.sifr -> app target named "tools/migrate"
```

Discovered app target names may use alphanumeric characters, `_`, `-`, and `/` as a path separator for nested targets. Empty path segments, `.`/`..`, path separators other than `/`, shell metacharacters, and target names that normalize differently across platforms report `SIFR-PACKAGE-0606`.

`sifr run` resolution order in a package context:

1. If the first positional argument ends in `.sifr`, or contains a path separator and resolves to a `.sifr` file, compile and run that explicit file as an ephemeral package-local target.
2. If `--bin <name>` is provided, use the matching discovered app target.
3. If `--script <name>` is provided, expand the matching `[scripts]` entry into a Sifr command plan.
4. If the first positional argument has no `.sifr` extension and matches both a discovered app target and a `[scripts]` entry, report `SIFR-PACKAGE-0605` and require `--bin` or `--script`.
5. If the first positional argument is provided, has no `.sifr` extension, and matches only a discovered app target, use that target.
6. If the first positional argument has no `.sifr` extension and matches only a `[scripts]` entry, expand that script into a Sifr command plan.
7. If `[package].default-run` is set, use the matching discovered app target.
8. If `src/main.sifr` exists and there is no ambiguity, use it as the default app target.
9. If no positional target is provided and exactly one discovered app target exists, use that target.
10. Otherwise report `SIFR-PACKAGE-0605` for missing or ambiguous runnable target.

`sifr run` with arguments after `--` passes those arguments to the selected app target or explicit `.sifr` file. Arguments after `--` are not parsed as Sifr CLI flags.

Script aliases:

```toml
[scripts]
dev = { command = "run", args = [] }
test-offline = { command = "test", args = ["--locked", "--offline"] }
check-workspace = { command = "check", args = ["--workspace"] }
publish-dry-run = { command = "publish", args = ["--dry-run"] }
```

Script rules:

- `command` must name a Sifr command implemented by this phase (`run`, `check`, `build`, `test`, `fetch`, `tree`, `package`, `publish`, `vendor`, or `repair`) or another command explicitly allowed by the script schema.
- `args` is an argv array, not a shell string.
- Scripts may not invoke arbitrary shell syntax, environment assignment, pipes, redirection, command substitution, platform-specific shell builtins, or external executables.
- Script expansion must be visible in verbose output and in JSON diagnostics. Verbose mode prints `Running script '<name>' -> <command> <args...>` before execution.
- Script names share a namespace with discovered app target names. If a script and app target have the same name, Sifr reports an ambiguity diagnostic and requires `sifr run --bin <name>` for the app target or `sifr run --script <name>` for the script.
- `sifr run --script <name>` always selects a script. `sifr run --bin <name>` always selects an app target.
- Scripts may not call other scripts. Script recursion or nested script expansion reports `SIFR-PACKAGE-0714`.
- After script expansion, any nested Cargo-backed command is validated against the Cargo CLI alignment matrix exactly like a direct command invocation.

Manifest-less single-file execution:

- `sifr run path/to/file.sifr` must work when no `sifr.toml` exists in the current directory or any parent directory.
- Manifest-less mode compiles the explicit file with source root equal to the file's parent directory, no package identity, no external package dependencies, no `__init__.sifr` public API derivation, and no Cargo package graph.
- Manifest-less mode may still use the generated one-file Cargo project or rustc path that the existing non-package CLI uses today.
- If a `sifr.toml` exists in the current directory or a parent directory, explicit `.sifr` file execution runs in package-aware mode when the file is under the selected package source root.
- If an explicit `.sifr` file is outside the selected package source root while a package context exists, Sifr reports `SIFR-PACKAGE-0710` and tells the user to run from outside the package or move the file under `src/`.
- `sifr check path/to/file.sifr` and `sifr emit path/to/file.sifr` follow the same manifest-less/package-aware split.

Explicit-file mode decision tree:

1. Search the current directory and parents for `sifr.toml`.
2. If no `sifr.toml` is found, run in manifest-less mode.
3. If `sifr.toml` is found and the explicit file is under the selected source root, run in package-aware mode.
4. If `sifr.toml` is found and the explicit file is outside the selected source root, report `SIFR-PACKAGE-0710`.

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
- `from .module import *`, dynamic `__all__`-style construction, runtime assignment-based exports, and absolute imports that attempt to define the package API are rejected in `__init__.sifr`.
- Bare `import module` and `import module as alias` do not define public API; if Sifr syntax supports them locally, they remain implementation imports only.
- Absolute imports in `__init__.sifr` that reference local modules are implementation imports and do not contribute to the public API. Only explicit relative `from` forms and top-level `class`, `def`, and `type` definitions define public API.
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

[dev-dependencies]
demo_test_support = { package = "sifr-demo-test-support", git = "https://github.com/sifr-lang/sifr-demo-test-support", tag = "v0.1.0" }
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
- `git`, `tag`, `rev`, `branch`, `path`, `version`, `registry`, `features`, `default-features`, `optional`: projected to Cargo-compatible dependency fields;
- `workspace = true`: allowed only inside Cargo workspaces and projected to Cargo workspace dependency inheritance.

Cargo-compatible dependency sections:

- `[dependencies]` is the runtime dependency set and is included by `sifr run`, `sifr check`, `sifr build`, `sifr test`, `sifr package`, and `sifr publish`.
- `[dev-dependencies]` follows Cargo's dev-dependency model and is used for tests, examples, benchmarks, and local development support. It is included by `sifr test`, but not by normal `sifr run`, `sifr build`, `sifr package`, or `sifr publish`.
- `build-dependencies` and target-specific dependency sections are Cargo-supported concepts for Rust-backed package implementation needs. They are not Sifr import dependencies unless trust policy explicitly wires them into Rust-backed package behavior.
- Sifr does not add `[test-dependencies]`, custom dependency groups, group composition, or uv-style group selection.

Dependency command behavior:

- `sifr add --rename name` follows Cargo's rename flag and writes the table key `name` in `sifr.toml [dependencies]`; it sets `import = "name"` when the rename differs from the dependency's resolved Sifr package name.
- `sifr add --dev <package>` writes `[dev-dependencies]`.
- `sifr add --optional <package>` writes `optional = true`. Optional Cargo dependencies are Sifr feature-gated imports; a selected Sifr feature must enable the optional dependency before its import root can be resolved.
- `sifr remove --dev` removes from `[dev-dependencies]`; without `--dev`, `sifr remove` removes from `[dependencies]` only and reports a diagnostic if the alias exists only in `[dev-dependencies]`.
- `sifr test` includes `[dependencies]` and `[dev-dependencies]`.
- `sifr run`, `sifr check`, `sifr build`, `sifr package`, and `sifr publish` use `[dependencies]` for Sifr import resolution.
- `sifr fetch` and `sifr tree` include both `[dependencies]` and `[dev-dependencies]` by default so a clone can prepare local test/development workflows with one Cargo-backed fetch.
- `sifr add`, `sifr remove`, and `sifr update` write only `sifr.toml` dependency tables first, then regenerate Sifr-owned Cargo dependency sections.
- Existing Phase 37 alias metadata in `Cargo.toml [package.metadata.sifr.aliases]` is internal transitional data only. This adhoc phase may delete or rewrite it when regenerating package projections; it does not need a user-facing compatibility mode.
- Direct `cargo add` on Sifr-owned dependency sections is not bidirectionally synced. It creates projection drift; `sifr repair --check` reports it and `sifr repair` may either remove the Cargo-only dependency from Sifr-owned sections or ask the user to import it into `sifr.toml`.
- User-owned Cargo dependencies for Rust backend implementation are allowed only in user-owned Cargo sections and must be validated against trust policy when reachable from a Rust-backed Sifr package.
- A manual Cargo dependency that makes Sifr package imports available without a matching `sifr.toml` dependency is not considered a public Sifr dependency and must not be used for Sifr import resolution.

Cargo projection:

- Runtime `[dependencies]` project to Cargo `[dependencies]`.
- `[dev-dependencies]` project to Cargo `[dev-dependencies]`.
- Sifr must not invent dependency buckets that Cargo cannot represent directly in the generated manifest.

Alias conflicts:

- Within one importing package scope, two dependency declarations may not expose the same `import` root unless they resolve to the same package instance and the same public API graph.
- Two aliases pointing at different versions or sources with the same `import` root report `SIFR-PACKAGE-0201` or its successor duplicate-import-root diagnostic.
- Two dependency declarations that resolve to the same package instance and expose the same `import` root report `SIFR-PACKAGE-0712` as a duplicate declaration warning.
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
- Sifr does not rewrite user-owned Cargo sections except through explicit internal migration scripts or `sifr repair`.
- Drift is any mismatch inside Sifr-owned projection sections, any missing required Sifr metadata pointer, or any Sifr-owned dependency missing from the Cargo projection.
- Extra user-owned Cargo dependencies are allowed when they are not used as Sifr package dependencies; Rust-backed packages must still satisfy trust policy for reachable backend crates.
- Projection drift always fails package-aware commands before invoking Cargo. `sifr repair --check` reports the same failures without writing; `sifr repair` attempts to repair Sifr-owned projection drift unless `--frozen` is active.

Sifr-owned Cargo section marking:

- Generated sections include stable `# sifr-managed` and `# end sifr-managed` markers identifying Sifr ownership.
- Guardrails verify those markers exist for generated packages and that projection regeneration is idempotent.
- User-owned sections must be preserved byte-for-byte where practical; semantic TOML rewrites must be documented and tested if byte preservation is not possible.
- Sifr-owned dependency projection includes dependency entries generated from `sifr.toml`, `[package.metadata.sifr]`, include patterns, and the pure marker target when Sifr created it.
- Projection idempotency is required: running `sifr repair` twice in a row on a clean projection produces zero file changes.

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
- If Cargo stops surfacing `package.metadata` through stable metadata output, Sifr's Cargo adapter may fall back to scanning selected Cargo package roots for `sifr.toml` as a discovery-only compatibility path. That fallback must stay inside the adapter boundary and must not change `sifr.toml` semantics.
- If Cargo metadata contains a `[package.metadata.sifr].manifest` pointer and a fallback scan finds a different `sifr.toml`, the pointer remains authoritative for discovery and Sifr reports `SIFR-PACKAGE-0703` for the mismatch.
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

- `sifr repair -p|--package <name>` may update Sifr-owned Cargo projection sections, regenerate the pure marker, and repair include patterns.
- `sifr repair` must not overwrite user-owned Rust backend code.
- `--locked`, `--offline`, and `--frozen` commands report drift before running Cargo operations.
- `--frozen` rejects any automatic projection write.
- `sifr repair --check` never writes; it exits non-zero when drift is present.
- `sifr repair` without `--check` writes only Sifr-owned projection sections and refuses to write when `--frozen` is set.
- `sifr repair` does not mutate `Cargo.lock` unless a dependency update operation explicitly requests lockfile mutation.
- If `sifr repair` detects that `src/lib.rs` has been modified from the canonical pure marker while the package is declared pure, it reports `SIFR-PACKAGE-0501` before any Cargo invocation. Repair does not restore user implementation code to the pure marker automatically.

## CLI Commands

Cargo alignment contract:

- Cargo-backed Sifr commands use Cargo flag names and semantics whenever Cargo exposes the corresponding behavior.
- For every delegated Cargo subcommand, Sifr adopts Cargo's stable command grammar for that subcommand. The command listings below highlight required package-management surface; they are not permission to invent different names or meanings.
- Sifr must not introduce alternate names for Cargo flags. For example, dependency renames use `--rename`, package selection uses `-p|--package`, and app argument forwarding uses `--`.
- Sifr-specific command surface is allowed only for Sifr-owned behavior Cargo cannot express: explicit `.sifr` file execution, named workflow scripts, Sifr source layout/privacy, Sifr diagnostics, and projection repair.
- `--manifest-path` means the same thing as Cargo: a path to `Cargo.toml`. The generated Cargo manifest points back to `sifr.toml` through `[package.metadata.sifr]`.
- Unsupported stable Cargo flags for a delegated command block milestone closeout unless the phase explicitly excludes that flag with rationale and reviewer approval.
- Nightly/unstable Cargo flags are not part of the stable package-management surface unless Sifr exposes the same Cargo gate. For example, Cargo's `-Z` behavior must remain nightly-only, and `cargo package --message-format` must require Cargo's `-Zunstable-options` gate if Sifr exposes it.

Authoritative Cargo references for the alignment matrix:

- `cargo run`: https://doc.rust-lang.org/cargo/commands/cargo-run.html
- `cargo check`: https://doc.rust-lang.org/cargo/commands/cargo-check.html
- `cargo build`: https://doc.rust-lang.org/cargo/commands/cargo-build.html
- `cargo test`: https://doc.rust-lang.org/cargo/commands/cargo-test.html
- `cargo init`: https://doc.rust-lang.org/cargo/commands/cargo-init.html
- `cargo fetch`: https://doc.rust-lang.org/cargo/commands/cargo-fetch.html
- `cargo add`: https://doc.rust-lang.org/cargo/commands/cargo-add.html
- `cargo remove`: https://doc.rust-lang.org/cargo/commands/cargo-remove.html
- `cargo update`: https://doc.rust-lang.org/cargo/commands/cargo-update.html
- `cargo tree`: https://doc.rust-lang.org/cargo/commands/cargo-tree.html
- `cargo vendor`: https://doc.rust-lang.org/cargo/commands/cargo-vendor.html
- `cargo package`: https://doc.rust-lang.org/cargo/commands/cargo-package.html
- `cargo publish`: https://doc.rust-lang.org/cargo/commands/cargo-publish.html

Package-aware core commands:

```bash
sifr run [target-or-path-or-script] [-p|--package spec] [--bin name|--script name] [--features f1,f2|-F f1,f2] [--all-features|--no-default-features] [--target triple] [--release|--profile name] [--target-dir dir] [--manifest-path path] [--ignore-rust-version] [--message-format fmt] [--locked|--offline|--frozen] [-j N|--jobs N] [--keep-going] [--] [app-args...]
sifr check [path-or-package] [--workspace] [-p|--package spec] [--exclude spec] [--lib|--bin name|--bins|--all-targets] [--features f1,f2|-F f1,f2] [--all-features|--no-default-features] [--target triple] [--target-dir dir] [--manifest-path path] [--ignore-rust-version] [--message-format fmt] [--locked|--offline|--frozen] [-j N|--jobs N] [--keep-going]
sifr build [path-or-package] [--workspace] [-p|--package spec] [--exclude spec] [--lib|--bin name|--bins|--all-targets] [--features f1,f2|-F f1,f2] [--all-features|--no-default-features] [--target triple] [--release|--profile name] [--target-dir dir] [--manifest-path path] [--ignore-rust-version] [--message-format fmt] [--locked|--offline|--frozen] [-j N|--jobs N] [--keep-going]
sifr test [path-or-package] [testname] [--workspace] [-p|--package spec] [--exclude spec] [--lib|--bin name|--bins|--tests|--all-targets] [--features f1,f2|-F f1,f2] [--all-features|--no-default-features] [--target triple] [--release|--profile name] [--target-dir dir] [--manifest-path path] [--ignore-rust-version] [--message-format fmt] [--locked|--offline|--frozen] [-j N|--jobs N] [--keep-going] [--] [test-args...]
```

Dependency and package operations:

```bash
sifr init [path] [--bin|--lib] [--name name] [--edition edition] [--vcs git|hg|pijul|fossil|none] [--registry registry]
sifr fetch [--target triple] [--manifest-path path] [--locked|--offline|--frozen]
sifr add <crate>... [--git url] [--branch branch|--tag tag|--rev rev] [--path path] [--registry registry] [--dev|--build|--target target] [--dry-run] [--rename name] [--features f1,f2|-F f1,f2] [--default-features|--no-default-features] [--optional|--no-optional] [-p|--package spec] [--manifest-path path] [--ignore-rust-version] [--locked|--offline|--frozen]
sifr remove <crate>... [--dev|--build|--target target] [--dry-run] [-p|--package spec] [--manifest-path path] [--locked|--offline|--frozen]
sifr update [spec...] [-w|--workspace] [--recursive] [--precise precise] [--dry-run] [--manifest-path path] [--ignore-rust-version] [--locked|--offline|--frozen]
sifr tree [--workspace] [-p|--package spec] [--target triple] [--edges kinds] [--invert spec] [--prune spec] [--depth n] [--duplicates] [--no-dedupe] [--manifest-path path] [--locked|--offline|--frozen]
sifr vendor [path] [--sync manifest] [--no-delete] [--respect-source-config] [--versioned-dirs] [--manifest-path path] [--locked|--offline|--frozen]
sifr package [--workspace] [-p|--package spec] [--exclude spec] [--list] [--no-verify] [--no-metadata] [--allow-dirty] [--exclude-lockfile] [--index index] [--registry registry] [--target triple] [--target-dir dir] [--features f1,f2|-F f1,f2] [--all-features|--no-default-features] [--manifest-path path] [--locked|--offline|--frozen] [-j N|--jobs N] [--keep-going]
sifr publish [--dry-run] [--workspace] [-p|--package spec] [--exclude spec] [--registry registry] [--index index] [--token token] [--no-verify] [--allow-dirty] [--target triple] [--target-dir dir] [--features f1,f2|-F f1,f2] [--all-features|--no-default-features] [--manifest-path path] [--locked|--offline|--frozen] [-j N|--jobs N]
sifr repair [-p|--package spec] [--check] [--locked|--offline|--frozen]
sifr --explain <diagnostic-code>
```

Command semantics:

- `sifr run` without lock/network flags may fetch dependencies and update `Cargo.lock`, matching Rust's local development convenience.
- For unconstrained `sifr run`, Sifr checks whether the lockfile and selected package sources are current. If a selected source is absent from Cargo's source cache, Sifr runs the Cargo-backed fetch preflight. If the lockfile can be updated through Cargo without user input, Sifr lets Cargo update it. If the lockfile cannot be updated deterministically, Sifr reports a Sifr-owned recovery diagnostic and asks the user to run `sifr update` explicitly.
- `sifr run path/to/file.sifr` works without `sifr.toml` in manifest-less mode and works inside a package when the file is under the selected source root.
- `sifr run <target-or-path-or-script>` treats a value ending in `.sifr` as an explicit Sifr file path. Otherwise, it matches a discovered app target or named script using Sifr-owned pre-resolution; `--bin name` and `--script name` are explicit disambiguators.
- Arguments after `--` are passed to the selected app target or explicit `.sifr` file and must not be consumed by the Sifr CLI.
- `sifr fetch --locked` fetches exactly the locked graph and fails if the lockfile needs an update.
- `sifr check --locked --offline` never touches the network and fails if any selected dependency source is absent locally.
- `-p` and `--package` are aliases for package selection, matching Cargo naming.
- Cargo common/display options such as `+toolchain`, `--config KEY=VALUE`, `-v|--verbose`, `-q|--quiet`, `--color when`, and `-h|--help` follow Cargo behavior for delegated commands and are omitted from the per-command synopsis only for readability.
- Cargo nightly common options such as `-C PATH` and `-Z flag` remain gated exactly as Cargo gates them; if excluded from the stable package-management surface, the alignment matrix must record the reason.
- `--message-format fmt` follows Cargo's flag name and accepted values for delegated commands. The alignment matrix records accepted values and nightly gates per subcommand. If Sifr cannot faithfully support a Cargo value, it must reject that value explicitly instead of silently changing the meaning.
- `sifr repair` is intentionally Sifr-owned. It is not named `sifr fix` because Cargo's `fix` command applies compiler suggestions, while this command repairs Sifr-owned projection drift.
- `sifr package --dry-run` is intentionally absent because Cargo package has no `--dry-run`; use `sifr package` for archive assembly/verification or `sifr publish --dry-run` for the Cargo dry-run publishing path.
- `sifr --explain <diagnostic-code>` is Sifr-owned diagnostic help. It accepts stable Sifr diagnostic codes, prints the meaning, common causes, docs links, and safe package-manager recovery commands, and never performs package operations.
- `sifr init` does not expose Cargo's template flag. Sifr templates are a Sifr-owned project-scaffolding concern and are outside this package-management command surface.

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
  script_origin: Option<ScriptOrigin>
  writes_projection: bool
  writes_lockfile: bool
  requires_network: bool
  selected_packages: Vec<PackageSelection>
  topological_order: Vec<SifrPackageId>

CargoCommandPlan
  command: metadata | fetch | build | check | run | test | package | publish | vendor | add | remove | update | tree | repair
  current_dir: PathBuf
  targets: Vec<String>
  lock_mode: unlocked | locked | offline | frozen
  features: CargoFeatureSelection
  args: Vec<String>

ScriptOrigin
  name: String
  command: String
  args: Vec<String>
```

Planning rules:

- `--frozen` rejects any operation with `writes_projection`, `writes_lockfile`, or `requires_network`.
- `--locked` rejects operations that would mutate `Cargo.lock`.
- `--offline` rejects plans whose selected package sources are absent locally and reports `SIFR-PACKAGE-0104`.
- Manifest-less explicit `.sifr` file execution bypasses Cargo metadata and package graph derivation, but still uses a small operation plan so lock/network flags, diagnostics, and command output stay consistent.
- Package-aware explicit `.sifr` file execution uses `PackageSession` when the file is under the selected source root.
- Script execution expands to an `OperationPlan` before any Cargo-backed command is invoked; scripts may compose Sifr commands but must not bypass package-session validation.
- Projection drift is checked before Cargo command execution.
- Offline source availability validation reuses the Phase 37 `validate_offline_source_availability` model. Additional constraints must be documented as Sifr-owned preflight checks rather than inferred from Cargo cache internals.
- Sifr validates the Sifr package graph, including package cycles, duplicate import roots, and package selection, before invoking Cargo for package-aware commands.
- Multi-package operations compute package topological order from `SifrPackageGraph`; Cargo may build in its own internal order, but Sifr diagnostics and cache keys use the stable topological order.
- Cargo command stderr/stdout is captured as structured cause data and attached to wrapper diagnostics after credential redaction.

Trust summary:

- accepted: direct Rust-backed dependencies allowed by trust policy and present in Cargo metadata;
- rejected: Rust-backed dependencies reachable from selected Sifr packages but absent from trust policy;
- stale: trust-policy entries whose Cargo package is not present in the resolved Cargo dependency graph.

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
- Data-dependent failures are failures caused by user input or package contents, such as invalid imports or missing package sources. Programmer invariants inside the compiler, such as a generated code map missing an entry that the compiler just created, may use internal assertions because they indicate a compiler bug rather than a user-triggerable runtime failure.
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
- `-p|--package spec`, `--workspace`, and `--exclude spec` follow Cargo package-selection names and broad semantics.
- The package `spec` accepts the Cargo package id format and may also accept an unambiguous Sifr package name when it maps to one Cargo package id.
- Sifr package names must be unique within a Cargo workspace. Duplicate Sifr package names across selected workspace members report `SIFR-PACKAGE-0607`.
- Advanced Phase 37 selector expressions are not part of the public package CLI. Adding them requires a Sifr-owned command or reconciliation with Cargo package-selection semantics in a separate design.
- Root `Cargo.toml`, `Cargo.lock`, and workspace dependency changes invalidate the whole graph.

Virtual workspace root edge case:

- If root `Cargo.toml` is a virtual `[workspace]` with no `[package]` and `sifr.toml` exists directly beside that root `Cargo.toml`, Sifr reports `SIFR-PACKAGE-0706` as a warning.
- The warning says the root `sifr.toml` has no package identity and should be moved to a package member or the workspace root should be converted into a package.
- A root `sifr.toml` in a virtual workspace is not used for package graph derivation.
- Member package `sifr.toml` files under `packages/*` or any workspace member path are not affected by this warning.

## Packaging And Publishing

Sifr packages are Cargo packages with Sifr metadata and source files. The archive format remains Cargo's package archive.

Preflight before `cargo package` or `cargo publish`:

- `sifr.toml` is present and valid.
- `src/**/*.sifr` files required by public API and selected targets are included.
- Required Sifr files are derived from `PackageSourceMap`: every file in the namespace API graph plus every file referenced by selected app targets.
- `src/__init__.sifr` exists for libraries that expose a public API.
- `src/main.sifr`, `src/bin/*.sifr`, or another selected explicit app target exists for apps.
- Pure marker `src/lib.rs` has no implementation code for pure Sifr packages.
- Rust-backed packages declare direct native dependencies in trust policy.
- Archive contents do not contain path traversal entries. Sifr validates this through Cargo's archive assembly output and reports `SIFR-PACKAGE-0404` if traversal is detected.
- Cargo include/exclude does not omit required Sifr files.
- Credentials and private source URLs are redacted in diagnostics.

Commands:

```bash
sifr package
sifr publish --dry-run
sifr publish
```

`sifr publish` delegates upload/authentication to Cargo but owns Sifr preflight diagnostics.

Publish failure boundary:

- Sifr preflight failures happen before `cargo publish` and use Sifr-owned diagnostics.
- Once `cargo publish` starts, Cargo owns upload/authentication/registry mutation semantics.
- If Cargo reports a publish failure after partial registry-side work, Sifr wraps the failure in `SIFR-PACKAGE-0101` and does not attempt rollback unless Cargo exposes a stable supported rollback command.
- Human-readable output must make clear that registry state should be checked with the registry/Cargo tooling named in the underlying Cargo excerpt.
- When `action = publish` and the underlying Cargo failure indicates a network or I/O failure after upload may have started, the `SIFR-PACKAGE-0101` help text tells the user to check registry status with Cargo or the registry interface before retrying.

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
action: metadata | fetch | build | check | run | test | package | publish | vendor | add | remove | update | tree
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

`SIFR-PACKAGE-0104` machine-readable fields:

```text
package
package_instance
dependency_alias
source_kind: path | git | registry | unknown
lock_mode: offline | frozen
recovery_command
```

`SIFR-PACKAGE-0403` machine-readable fields:

```text
cargo_package
manifest
source_kind: path | git | registry | unknown
missing_files: Vec<PathBuf>
```

`source_kind = "unknown"` is used when Sifr cannot determine source kind before invoking Cargo, or when a Cargo failure occurs before usable metadata is available. Human-readable output should omit source-kind-specific advice in that case and rely on the underlying Cargo excerpt plus generic recovery guidance.

Human-readable output:

- Shows the Sifr wrapper heading and stable context.
- Shows the redacted Cargo excerpt under an "Underlying Cargo failure" label.
- Makes clear that `SIFR-PACKAGE-0101` is a wrapper, not an attempt to reinterpret Cargo's full error taxonomy.

Diagnostic docs for `SIFR-PACKAGE-0101`:

- Explains that the code wraps a Cargo command failure.
- Documents where to find the underlying Cargo excerpt in human and JSON output.
- Documents redaction behavior.
- Gives generic next steps: rerun with the displayed Sifr command, inspect the Cargo excerpt, authenticate with Cargo for credential errors, or run `sifr fetch --locked` before offline commands when the help text says so.
- Does not list every possible Cargo failure mode.

`sifr --explain SIFR-PACKAGE-0101`:

- Prints the diagnostic-doc explanation for the code.
- Includes package-manager-specific recovery commands only when they are Sifr-owned and safe.
- Never performs package operations.

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
- Redaction fixtures must cover public registry URLs, private registry URLs with embedded credentials, URL userinfo credentials, `CARGO_REGISTRIES_*` environment-derived text, GitHub token prefixes (`ghs_`, `gho_`, `ghp_`, `ghu_`), `cargo:token`, and base64-like strings that should not be over-redacted.

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
- Dependency alias resolves to no available package instance: `SIFR-PACKAGE-0206`.
- Untrusted Rust backend dependency: `SIFR-PACKAGE-0301`.
- Stale trust entry: `SIFR-PACKAGE-0305`.
- Package archive missing Sifr source: `SIFR-PACKAGE-0403`.
- Package archive contains a path traversal entry: `SIFR-PACKAGE-0404`.
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
- `0707`: layout migration validation failed.
- `0708`: Cargo dependency or alias metadata conflicts with the canonical `sifr.toml [dependencies]` projection.
- `0709`: pure marker missing but user-owned Rust target prevents regeneration.
- `0710`: explicit `.sifr` file target is outside the selected package source root.
- `0711`: production `sifr.toml` uses Sifr manifest `[[bin]]` tables instead of layout-discovered app targets.
- `0712`: duplicate dependency declarations resolve to the same package instance and import root.
- `0713`: duplicate public API symbol in `__init__.sifr`.
- `0714`: script recursion or nested script expansion.

Diagnostic allocation rules:

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

Internal migration script:

```bash
python3 scripts/migrate_sifr_src_layout.py --from sifr-rooted --to src-init
```

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
- The rewrite step is filesystem-only and must not invoke Cargo or the Sifr compiler while files are in a partially migrated state. Validation commands run after file movement completes.

Migration validation:

1. Before migration, snapshot the public API from `__init__.sifr` or legacy `[exports].modules`.
2. After migration, derive the public API from the new `src/__init__.sifr` namespace graph.
3. Diff the public API snapshots and report `SIFR-PACKAGE-0707` if names are lost or added without an explicit flag.
4. Compile/package-check all local imports in the package to catch broken relative paths.
5. Run archive preflight to confirm required Sifr files are included.
6. Do not modify `Cargo.lock` package source revisions during layout migration.

Rollback:

- Before rewriting files, the migration script writes `.sifr/migrations/<timestamp>.json` with source/destination paths, SHA-256 checksums, file modes where supported, and the command options used.
- File contents changed by migration are copied to `.sifr/migrations/<timestamp>/files/` using checksum-addressed names.
- `python3 scripts/migrate_sifr_src_layout.py --rollback <migration-id>` restores only files whose current checksum still matches the post-migration checksum recorded by the descriptor.
- If files changed after migration, rollback reports conflicts and leaves them untouched unless `--force` is supplied.
- The rollback descriptor is preferred over a tar archive to avoid archive traversal risks and to make conflict detection machine-checkable.
- Failed validation leaves the original tree in place unless `--apply-partial` is explicitly passed; `--apply-partial` is intended only for manual repair.

Production schema:

- New packages use `[source].root` singular, defaulting to `src`.
- `source.roots` and `[exports].modules` are not production package schema.
- Sifr manifest `[[bin]]` tables are not production package schema; app targets are discovered from `src/main.sifr` and `src/bin/*.sifr`.
- Named workflow aliases use `[scripts]` with structured Sifr command plans.
- Internal Phase 37 fixtures may continue to exercise the old parser/model until their tests are replaced or migrated, but no user-facing command should generate the old schema.
- By closeout, guardrails should fail newly added package-management fixtures that use `sifr/<package>/` layout, manifest `[exports].modules`, or Sifr manifest `[[bin]]` tables unless the test is explicitly marked as a parser/backfill regression.

Demo and fixture strategy:

- Milestone-level tests use in-tree fixtures under `verification/package_management/src_layout_fixtures/` so implementation can advance before public demo repositories are migrated.
- Published `sifr-demo-*` repositories move to the canonical layout once package-aware compiler integration supports the new layout end to end.
- Milestone 7 migrates the demo repositories to the canonical production layout and records command transcripts for clone, fetch, run, offline check, workspace selection, and publish dry-run.

## Milestones

### milestone_adhoc_pkg_1: Package UX Contract And Source Layout

Scope:

- Finalize the `src/` layout, `src/__init__.sifr` public API rule, namespaced `__init__.sifr` rule, and `src/main.sifr` run target.
- Finalize layout-discovered app targets (`src/main.sifr`, `src/bin/*.sifr`), `[package].default-run`, and the absence of Sifr manifest `[[bin]]` in production schema.
- Finalize `[scripts]` as structured Sifr command-plan aliases, including no-shell parsing rules and app-target ambiguity behavior.
- Finalize manifest-less explicit file semantics for `sifr run/check/emit path/to/file.sifr`.
- Add parser/source-map tests for public root imports, public namespace imports, private implementation rejection, and local private imports.
- Implement `parse_init_sifr_reexports` and namespace API graph derivation in `PackageSourceMap`.
- Add rejection tests for production packages that still use manifest `[exports].modules`.
- Add rejection tests for production packages that still use Sifr manifest `[[bin]]` tables.
- Document the layout in `docs/package_management.md`.

Validation:

- `cargo test -p sifr_package source_layout`
- `cargo test -p sifr -- manifest_less`
- Unit tests for unknown `sifr.toml` top-level tables and nested keys to preserve forward-compatible parsing.
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
- Implement `sifr init --lib`, `sifr init --bin`, and `sifr repair --check` projection paths.

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
- Implement `[scripts]` expansion for structured Sifr command plans, including ambiguity checks with app target names.
- Implement Cargo-compatible dependency section handling for `[dependencies]` and `[dev-dependencies]`.
- Implement Cargo-compatible CLI names and semantics for package selection, target selection, feature selection, lock/network flags, `--message-format`, and `--` app/test argument passing.
- Add a Cargo CLI alignment matrix for every delegated subcommand in this phase, checked against current Cargo docs/help, with no undocumented alternate Sifr names.
- Implement the `SIFR-PACKAGE-0101` Cargo wrapper schema, including action, current directory, redacted args, exit status, lock/network mode, package context, bounded redacted Cargo excerpts, and JSON output fields.
- Implement `sifr --explain <diagnostic-code>` as Sifr-owned diagnostic help, including `SIFR-PACKAGE-0101` wrapper docs, redaction behavior, and safe recovery guidance.
- Retire or supersede credential-specific Cargo-failure codes such as `SIFR-PACKAGE-0105` so credential failures route through `SIFR-PACKAGE-0101`.
- Add redaction tests that prove public URLs and useful Cargo context survive while tokens, passwords, URL credentials, and private hostnames with embedded credentials are removed.

Validation:

- `cargo test -p sifr_package package_session`
- `cargo test -p sifr -- package_cli`
- Cargo CLI alignment matrix checked into docs or test fixtures for `run`, `check`, `build`, `test`, `init`, `fetch`, `add`, `remove`, `update`, `tree`, `vendor`, `package`, and `publish`.
- Targeted CLI tests for `sifr run main.sifr` without `sifr.toml`, `sifr run <script>`, `sifr run --script <name>`, `sifr run --bin <name> -- args`, Cargo-compatible `--message-format` handling, and dev-dependency defaults for `sifr test`.
- Demo command: `sifr fetch --locked` in `sifr-demo-app`.
- Targeted regression tests for `SIFR-PACKAGE-0101` wrapper fields, script-origin JSON diagnostics, redaction, retired credential-code mapping, stdout/stderr capture bounds, and `sifr repair --check` no-write behavior.
- Redaction tests cover public registry URLs, credential-bearing private URLs, URL userinfo credentials, `CARGO_REGISTRIES_*` text, GitHub token prefixes, `cargo:token`, and base64-like strings that should not be over-redacted.

Acceptance:

- A fresh cache can run `sifr fetch --locked` and then `sifr check --locked --offline`.

### milestone_adhoc_pkg_4: Package-Aware Compiler Imports

Scope:

- Connect `PackageSourceMap` to HIR/name resolution/import resolution.
- Support package root imports and public namespace imports.
- Reject private, transitive, and ambiguous imports with stable diagnostics.
- Make `sifr run` compile selected layout-discovered package app targets and explicit package-local `.sifr` file targets.

Validation:

- E2E pass/fail fixtures for local relative imports, public namespace imports, private implementation rejection, ambiguous import roots, transitive import rejection, cross-package `__init__.sifr` re-export chains, and re-export cycle detection.
- Demo command: `sifr run --locked --offline` in `sifr-demo-app`.

Acceptance:

- `sifr-demo-app` can import `demo_json_v1`, `demo_json_v2`, and `demo_http` through Sifr package APIs without direct Cargo commands.

### milestone_adhoc_pkg_5: Workspaces, Aliases, And Multiple Versions

Scope:

- Support `sifr check --workspace`, `-p|--package`, and `--exclude` through `PackageSession` using Cargo-compatible package selection.
- Support aliases for multiple versions through Sifr-facing dependency config.
- Ensure Cargo-compatible dependency sections compose correctly with workspaces, aliases, and multiple versions.
- Validate same package at multiple versions with distinct type identities.
- Ensure workspace dependency inheritance flows through Cargo metadata into Sifr scopes.
- Implement codegen namespace hashing for package instances.
- Add type identity tests that prove aliased package versions do not collide in HIR or generated Rust.

Validation:

- E2E workspace fixtures using `sifr-demo-workspace`.
- Multiple-version alias tests for `demo_json_v1` and `demo_json_v2`.
- Duplicate Sifr package name tests for `SIFR-PACKAGE-0607`.

Acceptance:

- Workspaces and aliases behave consistently across CLI, source map, diagnostics, and cache digests.

### milestone_adhoc_pkg_6: Packaging, Publishing, Vendoring, And Release Checks

Scope:

- Wire `sifr package`, `sifr publish`, and `sifr vendor`.
- Validate archive contents for the new `src/` layout.
- Validate that public API and selected app targets do not depend on `[dev-dependencies]`.
- Keep Cargo upload/authentication delegated.
- Add release dry-run checks for pure, Rust-backed, app, and workspace packages.

Validation:

- `sifr package` and `sifr publish --dry-run` on demo repositories.
- `cargo test -p sifr_package package_publish_vendor`
- Redaction tests for credentials/private source URLs.
- Trust-policy stale-entry tests for `SIFR-PACKAGE-0305`.
- Archive preflight tests for missing required Sifr files and path traversal diagnostics.

Acceptance:

- Sifr packages can be published as Cargo packages without exposing Cargo complexity to normal users.

### milestone_adhoc_pkg_7: Migration, Docs, Demos, And Long-Term Guardrails

Scope:

- Add an internal layout migration script or documented manual migration.
- Update `sifr-demo-*` repositories to canonical `src/` layout.
- Add end-to-end demo docs showing first clone, fetch, run, explicit file run, script run, offline check, workspace selection, dev-dependency behavior, and publish dry-run.
- Extend guardrails to enforce source layout, projection boundaries, and demo commands.
- Extend `scripts/check_package_manager_guardrails.py` to allow old Phase 37 layouts only in explicitly named internal regression fixtures during migration and require `src/` layout, no manifest exports, and no Sifr manifest `[[bin]]` tables for all production demos after closeout.
- Add guardrails that forbid active Cargo stderr taxonomy diagnostics, require projection idempotency tests, validate Sifr-owned Cargo section markers, and ensure pure-marker modifications are caught before Cargo execution.
- Guardrails also enforce that `cargo_metadata` versioning is pinned and audited in `crates/sifr_package/DEPENDENCY_AUDIT.md`, Sifr-owned Cargo sections use `# sifr-managed` markers, scripts contain command plans rather than shell strings or external executables, credential redaction tests cover both overbroad and underinclusive cases, and direct Cargo dependency edits inside Sifr-owned sections are reported as projection drift.
- Document Cargo compatibility assumptions: Sifr relies on stable `cargo metadata --format-version 1` fields and stable Cargo CLI subcommand behavior; if Cargo changes those surfaces, Sifr updates the adapter boundary rather than leaking Cargo internals to users.
- Document and test the discovery-only fallback for Cargo metadata surfaces that stop exposing `package.metadata.sifr`.
- Run full local validation.

Validation:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
- Demo transcript checked into docs or verification artifacts.

Acceptance:

- A user can clone `sifr-demo-app`, run Sifr commands only, and understand failures from Sifr diagnostics.

## Implementation Order

1. Source layout and `__init__.sifr` source-map rules.
2. Layout-discovered app targets, manifest-less explicit-file mode, and structured scripts.
3. Managed Cargo projection and drift diagnostics.
4. Cargo-compatible dev-dependency planning.
5. Package session and `sifr fetch`/`sifr tree`/package-aware `sifr check`.
6. Compiler import integration.
7. `sifr run` and `sifr test`.
8. Workspace/alias/multiple-version hardening.
9. Publish/vendor/migration/docs/demo closeout.

This order keeps the compiler integration dependent on a stable package source map, and keeps publishing dependent on the final canonical layout.

## Review Requirements

Each milestone requires:

- focused implementation PR;
- local targeted validation;
- `scripts/run_all_tests.sh --profile quick` before PR;
- Cargo CLI alignment audit against current stable Cargo docs and local `cargo <subcommand> --help` output for every delegated subcommand touched by that milestone; additions, removals, unstable gates, and intentional exclusions must be recorded in the alignment matrix before closeout;
- Claude review pass until READY;
- issue update with status, validation output summary, and PR link.

Full closeout requires:

- final Claude full implementation review until READY;
- demo transcript for `sifr-demo-app` and `sifr-demo-workspace`;
- full `scripts/run_all_tests.sh` unless blocked by a documented infrastructure issue;
- docs and guardrails updated.
