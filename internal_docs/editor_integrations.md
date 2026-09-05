# Sifr Editor Integrations

status: tooling-editor asset layer-implemented

## Shared Rule

All editor integrations delegate semantics to:

```bash
sifr lsp --stdio
```

Editor packages may provide filetype detection, syntax highlighting, commands that call Sifr CLI/LSP surfaces, and setup documentation. They must not implement Sifr parser, type checker, formatter, linter, diagnostics, codegen, symbol, reference, rename, or ownership semantics.

Document and range formatting are standard LSP requests served by `sifr lsp --stdio`.
Manual formatting should use each editor's normal LSP format command, and
format-on-save should be configured through that same LSP client. Editor assets
must not call `sifr fmt` as their primary formatting provider.
Direct `sifr fmt` usage is reserved for CLI, CI, hook, and manual-file workflows
and is documented in [`../docs/formatter.md`](../docs/formatter.md).

Lint diagnostics and lint code actions are also served by `sifr lsp --stdio`.
Editor integrations may expose the standard LSP code-action UI for explicit
Sifr policy suppressions, safe individual fixes, and `source.fixAll.sifr`.
They must not implement rule selection, suppression insertion, fix application,
or policy diagnostics in editor-side code. Users enable or disable lint
diagnostics through the `sifr.lint.enable` LSP setting; command-line linting for
CI and hooks is documented in [`../docs/linter.md`](../docs/linter.md).

## Syntax Asset Source Of Truth

developer tooling surface uses parser-validated syntax assets. TextMate and/or Tree-sitter assets are accepted only when drift checks compare them with `sifr_syntax` tokenization fixtures. Basic highlighting must work before the LSP starts; semantic tokens come from `sifr lsp`.

editor asset layer provides `editor_integrations/syntaxes/sifr.tmLanguage.json` as the
baseline TextMate grammar and `editor_integrations/syntaxes/sifr-token-scope-map.json`
as the validated mapping from `sifr_syntax` token kinds to syntax scopes. The
mapping is validated against `verification/areas/performance/sifr_syntax_token_fixtures/`.

## Required Targets

Neovim:

- filetype detection for `.sifr`
- LSP configuration launching `sifr lsp --stdio`
- manual formatting through `vim.lsp.buf.format`
- format-on-save through the Neovim LSP client
- syntax asset instructions
- no fallback Python tooling
- checked-in assets: `editor_integrations/neovim/ftdetect/sifr.lua` and `editor_integrations/neovim/lsp/sifr.lua`

Zed:

- language extension metadata or contribution-ready config
- LSP command `sifr lsp --stdio`
- LSP-backed formatting and `format_on_save` setup
- syntax strategy notes
- no semantic implementation outside Sifr
- checked-in assets: `editor_integrations/zed/extension.toml` and `editor_integrations/zed/languages/sifr/config.toml`

Helix:

- `languages.toml` contribution-ready config
- language server command `sifr lsp --stdio`
- manual formatting through Helix's LSP format command
- `auto-format = true` compatibility
- syntax/highlighting instructions
- no Python server fallback
- checked-in asset: `editor_integrations/helix/languages.toml`

Emacs:

- major-mode/filetype guidance
- Eglot or lsp-mode command using `sifr lsp --stdio`
- manual formatting through `eglot-format`
- save hooks may call Eglot's LSP formatter
- syntax asset guidance
- no semantic implementation outside Sifr
- checked-in asset: `editor_integrations/emacs/sifr-mode.el`

## Validation

editor asset layer adds checked-in editor assets and `verification/areas/developer_tooling/check_editor_assets.py`.
The check is wired into `scripts/run_all_tests.sh` and has a negative self-test
for bad LSP launch configuration, direct formatter fallback wiring, and syntax
scope drift. tooling lock locks the validation expectation and split-brain rule so
later editor packages cannot add semantics-bearing code.
