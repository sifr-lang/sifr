# CLI Command Semantics Contract

This document defines stable command-mode behavior for `sifr` CLI commands.

## Command Inputs

- `single-file input`: any `.sifr` file compiled in isolation.
- `project entry input`: `main.sifr` with at least one resolvable local import of the form `from <module> import ...` where `<module>.sifr` exists in the same directory.

## Mode Resolution Rules

Rules are evaluated in order:

1. If input file stem is not `main`, use single-file mode.
2. If `main.sifr` has no resolvable local imports, use single-file mode.
3. If `main.sifr` has at least one resolvable local import, use project mode.

Notes:

- Stdlib imports (for example `from sifr.math import floor`) do not enable project mode.
- `from typing import ...` and `from enum import ...` do not enable project mode.
- Invalid `main.sifr` source does not enable project mode.
- Missing local-module files do not enable project mode.
- Package-style imports via `pkg/__init__.sifr` are not part of project-mode auto-detect.

## Command Behavior Matrix

| Command | Single-file mode | Project mode |
|---|---|---|
| `sifr run <file>` | `build(source)` then execute binary | `build_project(main.sifr, temp_dir)` then execute binary |
| `sifr build <file> -o <dir>` | `build(source, output_dir)` | `build_project(main.sifr, output_dir)` |
| `sifr check <file>` | frontend/type-check only | frontend/type-check only (file input) |
| `sifr emit <file>` | emit generated Rust for file | emit generated Rust for file |
| `sifr test <dir>` | discover tests and resolve imports against stdlib + local modules in `<dir>` | same |

## Edge Cases

- A neighboring invalid `scratch.sifr` file must not break `run/build` when `main.sifr` has no local imports.
- Local import parse/type errors in actual project mode must fail both `run` and `build` consistently.
- `run` and `build` must use the same mode resolver for identical input paths.
- If `main.sifr` cannot be read or parsed during mode resolution, resolver falls back to single-file mode.
