# Sifr Editor Integrations

status: phase36-m36.6-implemented

## Shared Rule

All editor integrations delegate semantics to:

```bash
sifr lsp --stdio
```

Editor packages may provide filetype detection, syntax highlighting, commands that call Sifr CLI/LSP surfaces, and setup documentation. They must not implement Sifr parser, type checker, formatter, linter, diagnostics, codegen, symbol, reference, rename, or ownership semantics.

## Syntax Asset Source Of Truth

Phase 36 uses parser-validated syntax assets. TextMate and/or Tree-sitter assets are accepted only when drift checks compare them with `sifr_syntax` tokenization fixtures. Basic highlighting must work before the LSP starts; semantic tokens come from `sifr lsp`.

m36.6 provides `editor_integrations/syntaxes/sifr.tmLanguage.json` as the
baseline TextMate grammar and `editor_integrations/syntaxes/sifr-token-scope-map.json`
as the reviewed mapping from `sifr_syntax` token kinds to syntax scopes. The
mapping is validated against `verification/performance/sifr_syntax_token_fixtures/`.

## Required Targets

Neovim:

- filetype detection for `.sifr`
- LSP configuration launching `sifr lsp --stdio`
- syntax asset instructions
- no fallback Python tooling
- checked-in assets: `editor_integrations/neovim/ftdetect/sifr.lua` and `editor_integrations/neovim/lsp/sifr.lua`

Zed:

- language extension metadata or contribution-ready config
- LSP command `sifr lsp --stdio`
- syntax strategy notes
- no semantic implementation outside Sifr
- checked-in assets: `editor_integrations/zed/extension.toml` and `editor_integrations/zed/languages/sifr/config.toml`

Helix:

- `languages.toml` contribution-ready config
- language server command `sifr lsp --stdio`
- syntax/highlighting instructions
- no Python server fallback
- checked-in asset: `editor_integrations/helix/languages.toml`

Emacs:

- major-mode/filetype guidance
- Eglot or lsp-mode command using `sifr lsp --stdio`
- syntax asset guidance
- no semantic implementation outside Sifr
- checked-in asset: `editor_integrations/emacs/sifr-mode.el`

## Validation

m36.6 adds checked-in editor assets and `verification/tooling/check_editor_assets.py`.
The check is wired into `scripts/run_all_tests.sh` and has a negative self-test
for bad LSP launch configuration and syntax scope drift. m36.1 locks the
validation expectation and split-brain rule so later editor packages cannot add
semantics-bearing code.
