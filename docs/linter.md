# Sifr Linter

`sifr lint` runs Sifr-owned policy rules. It does not run the compiler, downgrade
compiler diagnostics, or make hard correctness diagnostics suppressible.

## Command

```bash
sifr lint [OPTIONS] [FILES]...
```

When no files are provided, `sifr lint` checks the current directory. Targets may
be files, directories, or `-` for stdin. Use `--stdin-filename <path>` with stdin
when config lookup, per-file ignores, or diagnostic paths should use a real
workspace path.

Useful command modes:

```bash
sifr lint
sifr lint src demos
sifr lint --output-format json src/main.sifr
sifr lint --show-files
sifr lint --show-settings src/main.sifr
sifr lint --statistics
sifr lint --fix src/main.sifr
sifr lint --diff src/main.sifr
```

## Configuration

Lint configuration is read from `sifr.toml`. Ruff and Python config files are not
Sifr lint authorities.

```toml
[lint]
select = ["trailing-whitespace", "todo-comment"]
ignore = ["todo-comment"]
exclude = ["target/**"]
extend-exclude = ["vendor/**"]
include = ["*.sifr"]
respect-gitignore = true
unsafe-fixes = "disabled"

[lint.rules]
large-parameter-list = "warn"
duplicate-import = "ignore"

[lint.per-file-ignores]
"demos/**" = ["todo-comment"]
```

`extend` may point at another config file. Paths in discovered config are
resolved relative to the config file that declares them. CLI selectors and
config overrides apply after discovered config.

## Rules

Rule IDs are Sifr-owned strings such as `trailing-whitespace`,
`todo-comment`, `boolean-positional-argument`, `large-parameter-list`, and
`duplicate-import`. Rules carry metadata for category, default severity, status,
docs URL, suppression complexity, and fix availability.

Python rule IDs, Ruff rule prefixes, Python plugin option blocks, `# noqa`, and
`pyproject.toml` lint settings do not configure Sifr lint.

## Suppressions

Only policy diagnostics can be suppressed:

```sifr
value = legacy_call()  # sifr: ignore[rule-id]
```

Blanket suppressions are rejected. Unknown and unused suppressions are reported
as policy diagnostics. Parser-aware suppression mapping is used for physical
line, syntax-node, statement-range, HIR, and workspace policy rules.

Use `--ignore-suppressions` to ignore inline Sifr suppression comments for a
single lint run. This does not affect per-file ignores or hard compiler
diagnostics.

## Fixes

Safe policy fixes are available through:

```bash
sifr lint --fix
sifr lint --fix-only
sifr lint --diff
sifr lint --show-fixes
```

`--fix` writes safe fixes to files. `--diff` prints the deterministic patch
without modifying files. `--fix-only` applies fixes without printing remaining
diagnostics. `--show-fixes` prints the fix count by rule.

Fix selection uses Sifr rule selectors:

```bash
sifr lint --fix --fixable trailing-whitespace
sifr lint --fix --unfixable trailing-whitespace
```

Unsafe fixes are disabled by default. `--unsafe-fixes` enables explicitly
unsafe policy fixes; `--no-unsafe-fixes` disables them. Hard compiler
diagnostics are never auto-fixed.

## Exit Status

`sifr lint` exits with:

- `0` when no diagnostics are emitted, or when `--exit-zero` is used.
- `1` when policy diagnostics remain, fixes are available under `--diff`, or
  `--exit-non-zero-on-fix` is used and fixes were applied.
- `2` for usage/configuration errors.

## Editor Behavior

The language server publishes policy diagnostics alongside hard compiler
diagnostics. Policy diagnostics include typed diagnostic data that marks them as
`policy` and includes the Sifr rule ID. Hard diagnostics are marked as `hard`.

Editors may offer suppression and safe-fix code actions only for policy
diagnostics. Fix-all is policy-only, safe-by-default, deferred through
`codeAction/resolve`, and rejected if the document version changed before the
edit is resolved.
