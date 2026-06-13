# Sifr Workspace Design

Source phase: `plans/issues/archive/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md`

## Scope

Sifr workspaces are discovered from the nearest ancestor `sifr.toml`. The manifest defines the stable workspace root for user module resolution without adding user helpers to the embedded `sifr.*` stdlib registry.

This slice implements native `sifr.toml` only. Future `pyproject.toml` / `[tool.sifr]` compatibility, if approved, must parse into the same internal manifest model and must not fork resolver behavior.

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
roots = ["verification/areas/algorithmic_compatibility/corpora/leetcode/src", "."]

[[bin]]
name = "merge-two-sorted-lists"
path = "verification/areas/algorithmic_compatibility/corpora/leetcode/src/0021_merge_two_sorted_lists.sifr"

[dependencies]
# Reserved for a future package manager.

[profile.dev]
# Reserved for future build/profile behavior.
```

Implemented semantics in this slice:

- Missing `[source]` or `[source].roots` defaults to `roots = ["."]`.
- Missing `[package]` is valid.
- `[package].name`, when present, must be a string.
- `[source].roots`, when present, must be a list of strings.
- Unknown top-level tables and unknown nested keys are accepted and ignored for forward compatibility.
- Source roots must be relative, non-empty, must not escape via `..`, and must resolve to existing directories.

## Resolution

The resolver keeps embedded stdlib resolution separate and highest priority. For user modules it searches:

1. the entry file's parent directory;
2. each configured workspace source root in declaration order.

The entry parent is an unconditional winner. Workspace-root matches are checked for ambiguity, and ambiguous modules fail with `SIFR-WORKSPACE-0102`. Unresolved workspace imports fail with `SIFR-WORKSPACE-0101` and list every attempted path. Dotted modules such as `helpers.nodes` map to `helpers/nodes.sifr`.

Package directories are not implemented in this phase. A graph containing both `helpers.sifr` and `helpers/nodes.sifr` fails with `SIFR-WORKSPACE-0103`.

## Rust Layout

Canonical module IDs remain dotted. The build materializer maps them into nested Rust module files:

- `helpers.nodes` -> `src/helpers/nodes.rs`
- generated namespace file `src/helpers/mod.rs` contains `pub mod nodes;`
- `src/main.rs` declares only the top-level namespace with `mod helpers;`

This keeps HIR and codegen keyed by the canonical dotted module name while producing valid Rust module trees.

## Deferred Work

- `sifr test` workspace discovery remains out of scope.
- Package directories, `__init__.sifr`, re-exports, wildcard imports, package members, dependency fetching, lockfiles, and build profiles remain reserved for later phases.
- No CLI workspace override flag is implemented in this slice.
