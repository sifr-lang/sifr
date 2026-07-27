# CLI Command Semantics Rules

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

Standalone self-update commands are documented in [`self_update.md`](./self_update.md).
`sifr self update` is available only for official standalone installs with a
schema-versioned receipt. It resolves `alpha`, `beta`, or `stable` through the
canonical schema-v2 governed release index, derives the immutable installer URL
from Sifr's trusted install base, and delegates checksum validation and artifact
replacement to the generated installer. Exact stable pins resolve only active,
non-withdrawn releases.

## Build And Run Output

Successful `sifr build` in the default human diagnostic format writes a
stage-aware summary to stderr. The summary reports the input, mode, release
native target, measured build stages, total elapsed time, binary path, and a
best-effort binary size when the final artifact can be read. Human progress
output is intentionally not a stable scripting API.

`sifr build --quiet` keeps human success output terse:

```text
Finished release build in <duration>
Binary: <path>
```

`sifr run` shares the same build pipeline but prints build progress only when
the generated binary cache misses. It omits the `Binary:` footer because program
stdout follows. Cache hits and `sifr run --quiet` do not print build progress.

Build progress and success banners are emitted only in the default human
diagnostic format and only on stderr. In `--diagnostic-format json` and
`--diagnostic-format compact`, successful builds emit no human progress text on
stdout or stderr; scripts should consume those machine-oriented diagnostic
formats instead of grepping human words such as `Finished`, `Binary`, or phase
labels.

## Diagnostic Output Formats

Compiler-facing commands accept `--diagnostic-format human|json|compact`.
`human` is the default developer-facing format and renders source locations,
snippets, caret highlights, related spans, notes/help, suggestions, and docs
URLs when span data is available. Spanless internal diagnostics use an explicit
`location: <unavailable>` fallback.

`compact` is a stable line-oriented format for CI, agents, and quick terminal
scanning. It emits one severity-only summary line followed by one physical line
per retained diagnostic after recovery limiting:

```text
1 error, 0 warnings, 0 notes
E SIFR-DECIMAL-0001 src/main.sifr:3:30 Decimal() received invalid exact literal '12.34.56'
```

The first four compact fields are stable: severity abbreviation, diagnostic
code, location or `<unknown>`, then the message. Compact mode intentionally
does not emit snippets or docs URLs by default.

`json` preserves the existing `RenderedDiagnostic[]` transport for tools and
editor integrations, including code, severity, message template, args, URL,
spans, children, help, and suggestions.

Formatter commands use the Ruff-backed Sifr formatter documented in
[`formatter.md`](./formatter.md). `sifr fmt [OPTIONS] [FILES]...` defaults to
the current directory, supports write, check, diff, stdin, explicit range,
preview, config, path-selection, and cache controls, and formats only `.sifr`
source. Editor formatting is served through `sifr lsp --stdio`, not through an
editor-owned formatter implementation.

Lint commands use the Sifr-owned policy-rule engine documented in
[`linter.md`](./linter.md). `sifr lint [OPTIONS] [FILES]...` defaults to the
current directory, accepts multiple files or directories, supports stdin with
`-` and `--stdin-filename`, resolves `[lint]`, `[lint.rules]`, and
`[lint.per-file-ignores]` from `sifr.toml`, and applies CLI/global config
overrides after discovered config. The command supports Sifr rule selection
through `--select`, `--extend-select`, and `--ignore`; path filtering through
`--exclude`, `--extend-exclude`, gitignore controls, and force-exclude controls;
and lint-local `--output-format concise|full|json`, `--output-file`,
`--show-files`, `--show-settings`, `--ignore-suppressions`, `--statistics`, and
`--exit-zero`. Safe policy fixes are available through `--fix`, `--fix-only`,
`--diff`, `--show-fixes`, `--fixable`, `--extend-fixable`, `--unfixable`,
`--extend-unfixable`, `--unsafe-fixes`, `--no-unsafe-fixes`, and
`--exit-non-zero-on-fix`.

`sifr lint` emits only suppressible policy diagnostics. It does not run,
downgrade, suppress, or auto-fix hard compiler diagnostics from `sifr check`.
Fix-all is policy-only and safe-by-default.

## Edge Cases

- A neighboring invalid `scratch.sifr` file must not break `run/build` when `main.sifr` has no local imports.
- Local import parse/type errors in actual project mode must fail both `run` and `build` consistently.
- `run` and `build` must use the same mode resolver for identical input paths.
- If `main.sifr` cannot be read or parsed during mode resolution, resolver falls back to single-file mode.
