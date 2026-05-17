# Phase 36 VS Code Extension Production Execution Checklist

Status: in_progress
Source phase: `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`
Phase milestone: `milestone_36_7`
Recommended extension repository: `sifr-lang/sifr-vscode`

This issue tracks the concrete VS Code extension implementation plan for Phase 36. It is not a post-Phase-36 ad hoc phase. Phase 36 cannot close until this checklist is complete or a reviewed planning PR updates the Phase 36 contract.

## Decision

Implement the VS Code extension as a separate repository:

```text
sifr-lang/sifr
  compiler
  CLI
  LSP server
  formatter/linter
  syntax/frontend/analysis
  shared editor contracts and validation gates

sifr-lang/sifr-vscode
  VS Code extension
  package.json
  grammar contribution
  language configuration
  LSP launcher
  commands/settings/tests
  .vsix packaging
```

`milestone_36_1` locked the separate-repository boundary. The main repository records the contract in `verification/tooling/vscode_extension_contract.json`; validation locates the extension repository through `editor_integrations/vscode`, `SIFR_VSCODE_REPO`, or sibling checkout `../sifr-vscode`. The extension remains separate because VS Code extensions have their own Node/TypeScript dependency graph, `.vsix` packaging, marketplace metadata, extension tests, and release cadence.

## Non-Negotiable Boundary

Only the main Sifr toolchain owns semantics.

The VS Code extension must not implement:

- parser logic
- type checking
- ownership/move analysis
- diagnostics derivation
- symbol/reference/rename analysis
- formatter logic
- linter/policy-rule logic
- generated Rust decisions

The extension may only:

- register the `sifr` language and `.sifr` files
- provide syntax-highlighting assets validated against `sifr_syntax`
- launch `sifr lsp --stdio`
- call Sifr LSP requests/commands
- call Sifr CLI commands such as `sifr check`, `sifr fmt`, `sifr lint`, and `sifr test`
- present VS Code UI for results produced by Sifr tooling

## Sequential Position

This work executes after:

1. `milestone_36_1`: repo boundary, extension id, command set, settings schema, syntax asset source of truth, and validation commands are locked.
2. `milestone_36_5`: `sifr lsp --stdio` is production-ready with all required LSP capabilities.
3. `milestone_36_6`: TextMate/Tree-sitter syntax assets and drift checks are ready for extension consumption.

This work completes before:

1. `milestone_36_8`: production verification and performance closeout.
2. Phase 36 exit.
3. Phase 37 package management.

## Repository Setup

- [x] Create or confirm `sifr-lang/sifr-vscode`.
- [x] Record repository ownership, branch protection expectations, release tags, and CI requirements.
- [x] Add `README.md` with install-from-source and local development instructions.
- [x] Add `LICENSE` compatible with the main Sifr repository.
- [x] Add `package.json`, `tsconfig.json`, extension source layout, test layout, and packaging scripts.
- [x] Pin Node/package-manager versions or document the supported version matrix.
- [x] Add CI for install, lint, typecheck, test, and package.

## Extension Manifest And Language Contribution

- [x] Register language id `sifr`.
- [x] Register file extension `.sifr`.
- [x] Register aliases and filename patterns only when justified by Sifr docs.
- [x] Contribute language configuration:
  - comments
  - brackets
  - indentation
  - auto-closing pairs
  - surrounding pairs
  - word pattern
- [x] Contribute TextMate grammar and/or Tree-sitter-backed grammar produced by or validated against Phase 36 syntax assets.
- [x] Add grammar drift tests that consume fixtures from the main repository or a pinned generated artifact.

## LSP Client Launcher

- [x] Launch command defaults to `sifr`.
- [x] Launch args default to `["lsp", "--stdio"]`.
- [x] Setting `sifr.lsp.path` overrides the binary path.
- [x] Setting `sifr.lsp.trace.server` controls protocol tracing.
- [x] Setting `sifr.diagnostics.mode` maps to Sifr LSP `off`, `open-files`, and `workspace` diagnostics modes.
- [x] Setting `sifr.lsp.extraEnv` is added only if needed and must not affect semantics.
- [x] Missing binary startup fails with an actionable setup message.
- [x] Extension never falls back to Python language servers, Pyright, Ruff Server, or ty.
- [x] Restart language server command is implemented.
- [x] Show server logs command is implemented.

## Required Commands

- [x] `Sifr: Restart Language Server`
- [x] `Sifr: Show Language Server Logs`
- [x] `Sifr: Locate Sifr Binary`
- [x] `Sifr: Run Check`
- [x] `Sifr: Run Tests`
- [x] `Sifr: Run Lint`
- [x] `Sifr: Check Format`
- [x] `Sifr: Format Document`
- [x] `Sifr: Show Generated Rust`
- [x] `Sifr: Explain Diagnostic`

All commands must call Sifr LSP/CLI surfaces. No command may compute Sifr semantic answers inside the extension.

## Required Editor Features

- [x] Diagnostics from `sifr lsp --stdio`.
- [x] Completion and completion resolve.
- [x] Hover.
- [x] Signature help.
- [x] Go to definition.
- [x] Go to declaration.
- [x] Go to type definition.
- [x] References.
- [x] Prepare rename and rename.
- [x] Document symbols.
- [x] Workspace symbols.
- [x] Semantic tokens.
- [x] Inlay hints.
- [x] Document highlights.
- [x] Folding ranges.
- [x] Code actions from Sifr diagnostic suggestions.
- [x] Document formatting.
- [x] Range formatting.
- [x] Generated Rust preview.
- [x] Explain diagnostic.
- [x] VS Code Test Explorer integration backed by Sifr test discovery and CLI test commands.

## Generated Rust Preview

- [x] Preview command calls the Sifr LSP workspace command or reviewed Sifr CLI surface.
- [x] Preview supports current file.
- [x] Preview supports current selection when Sifr source maps can provide a span.
- [x] Preview preserves source map context or at least identifies the source span.
- [x] Preview fails with a Sifr diagnostic or actionable editor error when codegen cannot produce a preview.
- [x] Extension does not run an editor-owned lowering/codegen path.

## Diagnostics And Code Actions

- [x] Diagnostics preserve Sifr codes, severities, ranges, related information, tags, URLs, child notes/help where LSP can represent them, and structured suggestion applicability.
- [x] Code actions expose Sifr diagnostic suggestions.
- [x] Suppression insertion is offered only for suppressible policy rules.
- [x] Hard correctness diagnostics never offer suppression.
- [x] Unknown/unused suppression diagnostics display normally.
- [x] Diagnostic explain command routes to Sifr.

## Formatting And Linting

- [x] Format document uses Sifr LSP formatting or `sifr fmt`.
- [x] Range formatting uses Sifr LSP range formatting.
- [x] Check-format command uses `sifr fmt --check` if exposed.
- [x] Lint command uses `sifr lint`.
- [x] Extension does not contain a formatter or linter implementation.

## Test Explorer

- [x] Test discovery uses Sifr LSP/CLI test metadata.
- [x] Test item ids are stable enough for VS Code refresh/run flows.
- [x] Run single test.
- [x] Run file/module tests.
- [x] Run workspace tests.
- [x] Surface test diagnostics/output without parsing Sifr semantics in the extension.
- [x] Handle projects without tests by showing an empty test tree, not guessed Python-style tests.

## Packaging And Publication Readiness

- [x] `.vsix` packaging works locally.
- [x] Extension metadata is complete:
  - display name
  - description
  - categories
  - keywords
  - icon
  - repository URL
  - license
  - minimum VS Code engine version
- [x] Changelog is present.
- [x] Marketplace publication checklist is documented.
- [x] Actual marketplace upload is left to Phase 39 release governance if credentials or release approvals are required.

## Cross-Repository Validation

Main `sifr-lang/sifr` repository must own or pin enough validation to prevent extension drift:

- [x] `verification/tooling/vscode_extension_contract.json` records language id, extension id, settings, commands, launch command, and repository boundary.
- [x] `verification/tooling/check_vscode_extension_contract.py` validates the main-repo contract against the extension repo.
- [x] `verification/tooling/check_vscode_extension.py` validates extension build/test/package behavior for the located extension repo.
- [x] Main-repo validation locates the extension repo by checking `SIFR_VSCODE_REPO`, then the `editor_integrations/vscode` submodule, then a sibling `../sifr-vscode` checkout relative to the main repo root.
- [x] Main-repo validation fails with an actionable message if the extension repo cannot be found; it must not silently skip once Phase 36 extension validation is active.
- [x] Contract check fails if the extension declares parser/type-checker/formatter/linter/codegen behavior.
- [x] Contract check fails if the extension launch command is not `sifr lsp --stdio`.
- [x] Contract check fails if required settings or commands are missing.
- [x] Contract check fails if package/test commands are not reproducible.

Extension repository validation must include:

- [x] package install
- [x] TypeScript typecheck
- [x] lint
- [x] unit tests
- [x] VS Code extension host tests
- [x] `.vsix` package build
- [x] smoke test launching a locally built `sifr lsp --stdio`

## PR Sequence

All work stays sequential. Do not begin the next PR until the current PR is merged and the checklist is updated.

1. [x] Main repo PR (`milestone_36_1`): lock extension repo boundary, extension contract JSON, validation command, and this issue checklist. Merged PR: <https://github.com/sifr-lang/sifr/pull/2129>.
2. [x] Extension repo PR (`milestone_36_7`): scaffold package, CI, language contribution, and grammar wiring. Merged PR: <https://github.com/sifr-lang/sifr-vscode/pull/1>.
3. [x] Extension repo PR (`milestone_36_7`): LSP launcher, settings, binary discovery, restart/log commands, and smoke tests. Merged PR: <https://github.com/sifr-lang/sifr-vscode/pull/1>.
4. [x] Extension repo PR (`milestone_36_7`): editor feature wiring for LSP diagnostics, completion, hover, navigation, symbols, semantic tokens, inlay hints, folding, highlights, code actions, formatting, and rename. Merged PR: <https://github.com/sifr-lang/sifr-vscode/pull/1>.
5. [x] Extension repo PR (`milestone_36_7`): generated Rust preview and explain diagnostic commands. Merged PR: <https://github.com/sifr-lang/sifr-vscode/pull/1>.
6. [x] Extension repo PR (`milestone_36_7`): VS Code Test Explorer integration. Merged PR: <https://github.com/sifr-lang/sifr-vscode/pull/1>.
7. [x] Main repo PR (`milestone_36_7` -> `milestone_36_8` handoff): `verification/tooling/check_vscode_extension_contract.py`, `verification/tooling/check_vscode_extension.py`, documentation, and validation evidence. Merged PR: <https://github.com/sifr-lang/sifr/pull/2135>.
8. [x] Phase 36 closeout PR (`milestone_36_8`): package evidence, validation evidence, publication checklist, and reviewer approval. Merged PR: <https://github.com/sifr-lang/sifr/pull/2136>.

PR-to-milestone mapping:

- PR 1 belongs to `milestone_36_1`.
- PRs 2 through 6 belong to `milestone_36_7`.
- PR 7 closes the main-repo validation handoff required before `milestone_36_8`.
- PR 8 belongs to `milestone_36_8`.

## Exit Gate

- [x] Extension repository boundary is finalized.
- [x] Extension builds, tests, and packages as `.vsix`.
- [x] Extension launches `sifr lsp --stdio`.
- [x] Extension delegates all Sifr semantics to the LSP/CLI.
- [x] Required commands and settings exist.
- [x] VS Code Test Explorer is backed by Sifr test metadata.
- [x] Generated Rust preview and explain diagnostic commands call Sifr.
- [x] Main repo cross-repo contract checks pass and fail on seeded drift.
- [x] `scripts/run_all_tests.sh --profile quick` passes in the main repo.
- [x] Extension repo CI workflow exists; local CI-equivalent validation passes.
- [x] Reviewer approves the extension as production-grade.
