# Package Management

Cargo is the package substrate for Sifr packages. A distributed Sifr library is a Cargo package that carries Sifr metadata and `.sifr` source files, while Sifr owns the user-facing source layout, package API, and diagnostics.

## Package Shape

Minimum production library layout:

```text
sifr-http/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    __init__.sifr
    client.sifr
    lib.rs
```

`Cargo.toml` owns distribution metadata, dependency resolution, lockfiles, registries, Git/path sources, workspaces, publishing, vendoring, and backend Rust dependencies.

`sifr.toml` owns compiler semantics: Sifr package name, edition, compiler requirement, source root, dependencies, scripts, privacy, and native trust policy.

`[source].root` defaults to `src`. New production packages do not use `[exports].modules` or Sifr manifest `[[bin]]` target tables. Public package APIs are declared in source with `__init__.sifr`.

Minimum app layout:

```text
sifr-app/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    main.sifr
    bin/
      admin.sifr
    lib.rs
```

`src/main.sifr` is the default app target. Additional app targets are discovered from `src/bin/*.sifr`.

## Public APIs

A package root is public when `src/__init__.sifr` exists. Public names are the top-level classes, functions, and type aliases defined there, plus explicit relative re-exports:

```sifr
from .client import HttpClient
from .errors import HttpError as Error
```

A public child namespace is a directory with its own `__init__.sifr`. Implementation files such as `src/client.sifr` are private across package boundaries unless their names are re-exported through an accessible namespace.

Local package code may import its own implementation files directly. Cross-package imports must go through the dependency's public namespace API.

## Manifest-Less Files

Explicit file commands remain first-class for learning and scripting:

```bash
sifr run path/to/file.sifr
sifr check path/to/file.sifr
sifr emit path/to/file.sifr
```

When no `sifr.toml` exists in the current directory or a parent directory, these commands run in manifest-less mode with the file's parent directory as the source root and no package dependency graph. Inside a package, an explicit `.sifr` file must live under the selected package source root.

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
