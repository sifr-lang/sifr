# Sifr Editor Integration Assets

These assets are contribution-ready editor integrations for the current Sifr
tooling contract. Every target delegates semantic behavior to:

```bash
sifr lsp --stdio
```

The files in this directory may provide filetype detection, syntax highlighting,
and editor setup metadata. They must not implement parser, type-checker,
diagnostic, formatting, lint, codegen, rename, reference, or ownership logic.

## Targets

- Neovim: `neovim/ftdetect/sifr.lua` and `neovim/lsp/sifr.lua`
- Zed: `zed/extension.toml` and `zed/languages/sifr/config.toml`
- Helix: `helix/languages.toml`
- Emacs: `emacs/sifr-mode.el`

## Syntax

`syntaxes/sifr.tmLanguage.json` provides baseline TextMate highlighting for
`.sifr` files. The grammar is checked against parser-token fixtures through
`verification/tooling/check_editor_assets.py`; semantic tokens still come from
the native LSP server.
