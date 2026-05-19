# Package Management

Phase 37 makes Cargo the package substrate for Sifr packages. A distributed Sifr library is a Cargo package that carries Sifr metadata and `.sifr` source files.

## Package Shape

Minimum package layout:

```text
sifr-http/
  Cargo.toml
  sifr.toml
  sifr/
    http/
      __init__.sifr
  src/lib.rs
```

`Cargo.toml` owns distribution metadata, dependency resolution, lockfiles, registries, Git/path sources, workspaces, publishing, vendoring, and backend Rust dependencies.

`sifr.toml` owns compiler semantics: Sifr package name, edition, compiler requirement, source roots, exports, import aliases, privacy, and native trust policy.

## Cargo Metadata Hook

Every Sifr package exposes a manifest pointer through Cargo metadata:

```toml
[package.metadata.sifr]
manifest = "sifr.toml"
```

Sifr reads this from `cargo metadata --format-version 1`, resolves `sifr.toml` relative to the selected Cargo package root, and derives a Sifr package graph from Cargo's resolved package graph. Cargo source ids are opaque; Sifr does not parse Cargo registry or Git cache internals.

## Pure Sifr Packages

Pure Sifr packages include a minimal Rust library target because Cargo requires a target:

```rust
// Pure Sifr package marker. Sifr source lives in sifr.toml source roots.
```

Sifr rejects non-trivial Rust marker contents for packages that do not declare Rust-backed behavior.

## Publishing And Vendoring

`sifr package --dry-run` validates Sifr metadata and package archive contents before delegating to Cargo package/publish command plans. Validation rejects missing `sifr.toml`, missing `.sifr` source files, Cargo include/exclude omissions, archive traversal paths, invalid exports, and backend trust violations.

Publishing and vendoring reuse Cargo-compatible behavior. Credentials remain Cargo-owned and must never appear in Sifr diagnostics, generated files, package metadata, or logs.

## Lock Modes

Sifr delegates lock and network semantics to Cargo:

- `--locked` asks Cargo to reject lockfile updates.
- `--offline` asks Cargo to avoid network access and requires selected package sources to be locally available.
- `--frozen` combines locked and offline semantics.

## Python Interop

`pyproject.toml`, `uv.lock`, and uv/Python package coordination are future interop work. They are not part of Phase 37 core package management and must lower into the same Cargo-backed `SifrPackageGraph` and `PackageSourceMap` model when implemented.
