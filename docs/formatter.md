# Sifr Formatter

`sifr fmt` formats `.sifr` source through the same Sifr Ruff fork parser, AST,
comments, trivia, and formatter used by editor formatting. The formatter is
syntax-aware, deterministic, idempotent, and independent of type checking or
ownership analysis.

## Commands

```bash
sifr fmt [OPTIONS] [FILES]...
```

When no files are supplied, `sifr fmt` formats the current directory. File and
directory inputs discover `.sifr` files, respect formatter excludes, and honor
`.gitignore` by default.

Common modes:

```bash
sifr fmt .
sifr fmt path/to/file.sifr
sifr fmt --check .
sifr fmt --diff path/to/file.sifr
sifr fmt --line-length 100 path/to/file.sifr
sifr fmt --stdin-filename src/main.sifr < src/main.sifr
```

`--check` does not write files and exits nonzero when any file would change.
`--diff` does not write files and prints unified diffs. Stdin formatting writes
the formatted source to stdout. `--range START:END` formats one file over valid
UTF-8 byte offsets and is intended for editor and tooling integrations that
already track source ranges.

Path selection options:

- `--exclude <pattern[,pattern...]>` adds formatter exclude patterns.
- `--respect-gitignore` and `--no-respect-gitignore` control VCS ignore files.
- `--force-exclude` applies excludes to explicit file targets.
- `--no-force-exclude` keeps explicit targets formatable even when excluded.

Cache options:

- `--no-cache` disables formatter cache reads and writes.
- `--cache-dir <path>` overrides the formatter cache directory.

Python-specific Ruff options such as `--target-version` and `--extension` are
not Sifr formatter options. Sifr formats `.sifr` source only.

## Configuration

Canonical formatter config lives in `sifr.toml` under `[format]`.

```toml
[format]
line-length = 88
preview = false
docstring-code-format = false
docstring-code-line-length = "dynamic"
exclude = ["target/**"]
respect-gitignore = true
force-exclude = false
cache = true
cache-dir = ".sifr_cache/formatter"
```

Supported keys are:

- `line-length` or `line_length`
- `preview`
- `docstring-code-format` or `docstring_code_format`
- `docstring-code-line-length` or `docstring_code_line_length`
- `exclude`
- `respect-gitignore` or `respect_gitignore`
- `force-exclude` or `force_exclude`
- `cache`
- `no-cache` or `no_cache`
- `cache-dir` or `cache_dir`
- `extend`

`extend` may be a string or an array of paths. Paths are resolved relative to
the declaring config file, and cycles are reported as formatter diagnostics.
Unknown formatter keys fail deterministically. Python Ruff-only keys
`target-version`, `target_version`, and `extension` are rejected.

CLI flags override discovered config. `--config <path>` applies an explicit
config file, and `--config key=value` applies an inline formatter override.
`--isolated` ignores discovered config files and explicit config-file paths, but
still accepts inline `--config key=value` overrides.

## Style Behavior

The formatter canonicalizes Sifr parameter conventions directly in the Ruff fork:

```sifr
def consume(mut own items: list[int]) -> list[int]:
    return items
```

formats as:

```sifr
def consume(own mut items: list[int]) -> list[int]:
    return items
```

It preserves comments, meaningful blank lines, string contents, and valid source
ranges needed by diagnostics and editor edits. It supports Ruff-style formatter
pragmas where Sifr has matching syntax boundaries:

- `# fmt: off` and `# fmt: on`
- `# fmt: skip`
- `# yapf: disable` and `# yapf: enable`

Preview formatting is disabled by default. Use `--preview`, `--no-preview`, or
`[format].preview` to opt in or out explicitly.

Docstring code formatting is also disabled by default. When
`docstring-code-format = true`, Sifr snippets inside Ruff-recognized Markdown
fences and reStructuredText `code-block` or `sourcecode` directives are
formatted when the language tag is absent, `sifr`, `python`, `py`, `python3`,
or `py3`. Snippets that do not parse as Sifr are left unchanged.

## Editor Formatting

Editor formatting is LSP-first. Editors launch:

```bash
sifr lsp --stdio
```

and request document or range formatting through standard LSP methods. The
server advertises formatting only when `sifr.format.enable` is true. Neovim,
Zed, Helix, Emacs, and VS Code integrations use the LSP formatter for manual
formatting and format-on-save. Editor integrations must not call `sifr fmt` as
their primary document-formatting provider.

## Validation

Formatter coverage is guarded by:

```bash
python3 verification/areas/developer_tooling/check_formatter_ast_coverage.py
python3 verification/areas/developer_tooling/check_formatter_ast_coverage.py --self-test
```

The guardrail fails when a Sifr parser or AST extension lacks both a Ruff fork
formatter fixture and a Sifr wrapper corpus fixture. Performance budgets for
large-file and project formatter checks are part of the local validation lane.
