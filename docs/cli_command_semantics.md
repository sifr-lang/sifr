# CLI Command Semantics Rules

This document defines stable command-mode behavior for `sifr` CLI commands.

## Command Inputs

- `manifest-less explicit file`: a `.sifr` file with no `sifr.toml` in its
  directory or any ancestor directory. It is compiled in isolation.
- `workspace entry`: a `.sifr` file with a valid `sifr.toml` in its directory
  or an ancestor directory. It is compiled with that workspace's reachable
  modules.

## Mode Resolution Rules

Rules are structural and evaluated in order:

1. Discover the nearest ancestor `sifr.toml` for the input path.
2. If a valid workspace is discovered, use project mode.
3. If no workspace is discovered, use single-file mode.

Notes:

- A discovered malformed workspace manifest is a hard diagnostic. Commands do
  not fall back to single-file mode.
- Workspace `[source].roots` configure user-module lookup after project mode is
  selected. They do not participate in command-mode selection. Package-session
  source-root validation is a separate preflight subject to the known issue
  below.
- Mode selection never depends on the entrypoint filename, source contents,
  import forms, or neighboring module files.
- A manifest-less local import receives the ordinary single-file import
  diagnostic. Add `sifr.toml` when multiple local modules must compile
  together.
- Known issue [#3128](https://github.com/sifr-lang/sifr/issues/3128) affects the
  package-session preflight. If the current directory contains a source-only
  manifest without package metadata, this preflight can fail before these
  rules run. Until the issue is fixed, invoke explicit-file `check`, `run`, and
  `build` from outside that directory. Alternatively, use a complete package
  manifest. `emit` and `trace` do not use this preflight.

## Command Behavior Matrix

| Command | Single-file mode | Project mode |
|---|---|---|
| `sifr run <file>` | `build(source)` then execute binary | `build_project(main.sifr, temp_dir)` then execute binary |
| `sifr build <file> -o <dir>` | `build(source, output_dir)` | `build_project(main.sifr, output_dir)` |
| `sifr check <file>` | frontend/type-check only | frontend/type-check only (file input) |
| `sifr emit <file>` | emit generated Rust for file | emit generated Rust for file |
| `sifr trace <file>` | trace the isolated frontend session | trace the workspace frontend session |
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

- A neighboring invalid `scratch.sifr` file does not affect a manifest-less
  explicit-file command.
- Local import parse/type errors inside a workspace fail `run`, `build`,
  `check`, `emit`, and `trace` consistently.
- All explicit-file compiler commands use the same mode resolver for identical
  input paths.
- Mode selection does not read or parse the entrypoint. Source I/O and parse
  diagnostics occur only after the structural mode has been selected.
