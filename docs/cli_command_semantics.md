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
- Relative imports such as `from .helper import value` enable project mode when `helper.sifr` exists in the same directory.
- Multi-level relative imports (for example `from ..helper import value`) do not enable project mode.
- Bare relative imports (for example `from . import value`) do not enable project mode.
- Regular import statements (for example `import helper`) do not enable project mode.
- Local files named `typing.sifr` or `enum.sifr` do not enable project mode; these names are treated as stdlib-like imports by auto-detect.

## Resolver Trigger Matrix

| Import form in `main.sifr` | Project-mode activation | Resolver mode | Expected compile result |
|---|---|---|---|
| `from helper import value` with `helper.sifr` sibling | yes | project | success (module resolved) |
| `from .helper import value` with `helper.sifr` sibling | yes | project | success (module resolved) |
| `from .helper import value` without `helper.sifr` sibling | no | single-file | error (`unknown module 'helper'`) |
| `from ..helper import value` | no | single-file | error (`unsupported relative import level 2`) |
| `from . import helper` | no | single-file | error (`unsupported bare relative import`) |
| `import helper` | no | single-file | error (`unsupported import statement`) |
| `from typing import List` | no | single-file | success (type-level import handling) |
| `from enum import Enum` | no | single-file | success (type-level import handling) |

## Command Behavior Matrix

| Command | Single-file mode | Project mode |
|---|---|---|
| `sifr run <file>` | `build(source)` then execute binary | `build_project(main.sifr, temp_dir)` then execute binary |
| `sifr build <file> -o <dir>` | `build(source, output_dir)` | `build_project(main.sifr, output_dir)` |
| `sifr check <file>` | frontend/type-check only | frontend/type-check only (file input) |
| `sifr emit <file>` | emit generated Rust for file | emit generated Rust for file |
| `sifr test <dir>` | discover tests and resolve imports against stdlib + local modules in `<dir>` | same |

Package-management commands use Cargo-backed package coordination as documented in [`package_management.md`](./package_management.md). Cargo owns external dependency resolution, lockfiles, registry/Git/path sources, publishing, and vendoring; Sifr validates package metadata, source roots, exports, trust policy, archive contents, and diagnostics before delegating Cargo-owned behavior.

Formatter commands use the Ruff-backed Sifr formatter documented in
[`formatter.md`](./formatter.md). `sifr fmt [OPTIONS] [FILES]...` defaults to
the current directory, supports write, check, diff, stdin, explicit range,
preview, config, path-selection, and cache controls, and formats only `.sifr`
source. Editor formatting is served through `sifr lsp --stdio`, not through an
editor-owned formatter implementation.

Lint commands use the Sifr-owned policy-rule engine. `sifr lint [OPTIONS]
[FILES]...` defaults to the current directory, accepts multiple files or
directories, supports stdin with `-` and `--stdin-filename`, resolves `[lint]`,
`[lint.rules]`, and `[lint.per-file-ignores]` from `sifr.toml`, and applies
CLI/global config overrides after discovered config. The command supports
Sifr rule selection through `--select`, `--extend-select`, and `--ignore`;
path filtering through `--exclude`, `--extend-exclude`, gitignore controls, and
force-exclude controls; and lint-local `--output-format concise|full|json`,
`--output-file`, `--show-files`, `--show-settings`, and `--exit-zero`.

`sifr lint` emits only suppressible policy diagnostics. It does not run,
downgrade, suppress, or auto-fix hard compiler diagnostics from `sifr check`.
Fix-related lint flags are reserved for the production fix-engine milestone.

## Edge Cases

- A neighboring invalid `scratch.sifr` file must not break `run/build` when `main.sifr` has no local imports.
- Local import parse/type errors in actual project mode must fail both `run` and `build` consistently.
- `run` and `build` must use the same mode resolver for identical input paths.
- If `main.sifr` cannot be read or parsed during mode resolution, resolver falls back to single-file mode.
