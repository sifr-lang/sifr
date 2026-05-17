# Sifr Editor Integrations

status: phase36-contract-locked

## Shared Rule

All editor integrations delegate semantics to:

```bash
sifr lsp --stdio
```

Editor packages may provide filetype detection, syntax highlighting, commands that call Sifr CLI/LSP surfaces, and setup documentation. They must not implement Sifr parser, type checker, formatter, linter, diagnostics, codegen, symbol, reference, rename, or ownership semantics.

## Syntax Asset Source Of Truth

Phase 36 uses parser-validated syntax assets. TextMate and/or Tree-sitter assets are accepted only when drift checks compare them with `sifr_syntax` tokenization fixtures. Basic highlighting must work before the LSP starts; semantic tokens come from `sifr lsp`.

## Required Targets

Neovim:

- filetype detection for `.sifr`
- LSP configuration launching `sifr lsp --stdio`
- syntax asset instructions
- no fallback Python tooling

Zed:

- language extension metadata or contribution-ready config
- LSP command `sifr lsp --stdio`
- syntax strategy notes
- no semantic implementation outside Sifr

Helix:

- `languages.toml` contribution-ready config
- language server command `sifr lsp --stdio`
- syntax/highlighting instructions
- no Python server fallback

Emacs:

- major-mode/filetype guidance
- Eglot or lsp-mode command using `sifr lsp --stdio`
- syntax asset guidance
- no semantic implementation outside Sifr

## Validation

m36.6 adds checked-in editor assets and `verification/tooling/check_editor_assets.py`. m36.1 locks the validation expectation and split-brain rule so later editor packages cannot add semantics-bearing code.
