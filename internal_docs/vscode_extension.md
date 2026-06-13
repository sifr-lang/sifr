# Sifr VS Code Extension Contract

status: phase36-m36.7-implemented

## Repository Boundary

The VS Code extension lives in a separate repository:

```text
sifr-lang/sifr-vscode
```

The main `sifr-lang/sifr` repository owns compiler, CLI, LSP, formatter, linter, syntax assets, editor contracts, and cross-repository validation. The extension repository owns Node/TypeScript packaging, extension tests, `.vsix` packaging, marketplace metadata, and release artifacts.

The repository is expected at `editor_integrations/vscode`, `SIFR_VSCODE_REPO`, or as a sibling checkout `../sifr-vscode`. m36.1 locks the contract. m36.7 makes the extension repository mandatory for package validation.

m36.7 created `sifr-lang/sifr-vscode` and added the initial production extension
scaffold: TypeScript sources, package manifest, language configuration, TextMate
grammar, LSP launcher, settings, commands, test controller, local tests, CI, and
`.vsix` packaging.

## Manifest Contract

The checked-in contract is `verification/areas/developer_tooling/vscode_extension_contract.json`.

Required identity:

- extension id: `sifr-lang.sifr-vscode`
- language id: `sifr`
- file extension: `.sifr`
- minimum VS Code engine: `^1.90.0`

Required default launch:

```json
{
  "command": "sifr",
  "args": ["lsp", "--stdio"]
}
```

The extension must support `sifr.lsp.path` to override the executable path and `sifr.lsp.trace.server` for protocol tracing.
Formatting support is advertised by the Sifr LSP server when
`sifr.format.enable` is true. The VS Code `sifr.formatDocument` command
delegates to `editor.action.formatDocument`, which uses the native LSP document
formatting provider. The extension must not implement formatter logic or use a
direct `sifr fmt` fallback as its document-formatting provider.

Lint diagnostics are advertised by the Sifr LSP server when `sifr.lint.enable`
is true. VS Code quick fixes and source fix-all actions must come from LSP
`textDocument/codeAction` and `codeAction/resolve`. The extension may expose
the `sifr.runLint` CLI command for manual or task-style lint runs, but it must
not implement lint rule logic, suppression insertion, fix conflict resolution,
or fix-all edits in TypeScript.

## Allowed Responsibilities

The extension may:

- register the `sifr` language and `.sifr` files
- contribute syntax-highlighting assets validated by the main repo
- contribute language configuration
- launch `sifr lsp --stdio`
- send Sifr LSP requests and commands
- call Sifr CLI commands for check, fmt, lint, and tests
- present VS Code UI for Sifr-produced results
- present LSP-provided Sifr policy suppressions and safe policy fixes
- package `.vsix` artifacts

## Forbidden Responsibilities

The extension must not implement:

- parser logic
- type checking
- ownership or move analysis
- diagnostics derivation
- symbol, reference, or rename analysis
- formatter logic
- linter or policy-rule logic
- generated Rust decisions
- fallback Python language-server behavior

## Required Commands

- `sifr.restartLanguageServer`
- `sifr.showLanguageServerLogs`
- `sifr.locateBinary`
- `sifr.runCheck`
- `sifr.runTests`
- `sifr.runLint`
- `sifr.checkFormat`
- `sifr.formatDocument`
- `sifr.showGeneratedRust`
- `sifr.explainDiagnostic`

All commands delegate to Sifr LSP or CLI surfaces.

## Versioning Covenant

Before m36.7 closes, the extension must document whether its version is coupled to the Sifr compiler version or declares an explicit supported Sifr version range. Extension releases are gated on the main-repo contract check, `sifr lsp --stdio` smoke tests, extension tests, and `.vsix` packaging.

## Validation

`verification/areas/developer_tooling/check_vscode_extension_contract.py` validates the main-repo contract and, once extension validation is active, checks the extension repository manifest, commands, settings, launch command, package scripts, and forbidden semantic behavior.

`verification/areas/developer_tooling/check_vscode_extension.py` runs m36.7 build/test/package
validation against the real extension checkout: `npm ci` when dependencies are
missing, `npm run lint`, `npm run typecheck`, `npm test`,
`npm run test:extension`, and `npm run package`.
