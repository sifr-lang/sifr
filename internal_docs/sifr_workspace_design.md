# Sifr Workspace Design

Source record: `sifr-workspace-sifr-toml-import-resolution-2026-04-25`

## Scope

Sifr workspaces are discovered from the nearest ancestor `sifr.toml`. The manifest defines the stable workspace root for user module resolution without adding user helpers to the embedded `sifr.*` stdlib registry.

This capability implements native `sifr.toml` only. Future `pyproject.toml` / `[tool.sifr]` compatibility, if approved, must parse into the same internal manifest model and must not fork resolver behavior.

## Manifest Shape

```toml
[package]
name = "leetcode-fixtures"
version = "0.0.0"
edition = "2026"

[workspace]
resolver = "1"
members = ["verification/areas/algorithmic_compatibility/corpora/leetcode/src"]
exclude = ["tmp", "target"]

[source]
root = "src"

[dependencies]
# Reserved for a future package manager.

[profile.dev]
# Reserved for future build/profile behavior.
```

Implemented semantics in this capability:

- Missing `[source]` or `[source].root` defaults to `root = "src"`.
- Missing `[package]` is valid.
- `[package].name`, when present, must be a string.
- `[source].root`, when present, must be one string.
- `[source].roots`, `[exports]`, and `[[bin]]` are unsupported.
- The source root must be relative and non-empty. It must not escape via `..`,
  and it must resolve to an existing directory.

## Resolution

The resolver keeps embedded stdlib resolution separate and highest priority. For user modules it searches:

1. the entry file's parent directory;
2. the configured workspace source root.

The entry parent is an unconditional winner. The workspace source root can hold
flat modules and package directories. Dotted modules such as `helpers.nodes`
map to `helpers/nodes.sifr` or `helpers/nodes/__init__.sifr`.

A flat module and a package directory cannot define the same module. Sifr
reports the collision before it compiles the project.

## Rust Layout

Canonical module IDs remain dotted. The build materializer maps them into nested Rust module files:

- `helpers.nodes` -> `src/helpers/nodes.rs`
- generated namespace file `src/helpers/mod.rs` contains `pub mod nodes;`
- `src/main.rs` declares only the top-level namespace with `mod helpers;`

This keeps HIR and codegen keyed by the canonical dotted module name while producing valid Rust module trees.

## Package Boundary

Package manifests use the same single-root rule. The canonical import root is
the normalized `[package].name`. A root `__init__.sifr` declares the public
package API. The manifest does not contain an export list or binary target
tables.
