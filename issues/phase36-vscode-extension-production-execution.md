# Phase 36 VS Code Extension Production Execution Checklist

Status: planned
Source phase: `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`
Phase milestone: `milestone_36_7`
Recommended extension repository: `sifr-lang/sifr-vscode`

This issue tracks the concrete VS Code extension implementation plan for Phase 36. It is not a post-Phase-36 ad hoc phase. Phase 36 cannot close until this checklist is complete or a reviewed planning PR updates the Phase 36 contract.

## Decision

Implement the VS Code extension as a separate repository by default:

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

`milestone_36_1` may choose an in-repo extension only if it records a reviewed rationale and preserves the same validation, packaging, and release-boundary requirements. The default is separate repo because VS Code extensions have their own Node/TypeScript dependency graph, `.vsix` packaging, marketplace metadata, extension tests, and release cadence.

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

- [ ] Create or confirm `sifr-lang/sifr-vscode`.
- [ ] Record repository ownership, branch protection expectations, release tags, and CI requirements.
- [ ] Add `README.md` with install-from-source and local development instructions.
- [ ] Add `LICENSE` compatible with the main Sifr repository.
- [ ] Add `package.json`, `tsconfig.json`, extension source layout, test layout, and packaging scripts.
- [ ] Pin Node/package-manager versions or document the supported version matrix.
- [ ] Add CI for install, lint, typecheck, test, and package.

## Extension Manifest And Language Contribution

- [ ] Register language id `sifr`.
- [ ] Register file extension `.sifr`.
- [ ] Register aliases and filename patterns only when justified by Sifr docs.
- [ ] Contribute language configuration:
  - comments
  - brackets
  - indentation
  - auto-closing pairs
  - surrounding pairs
  - word pattern
- [ ] Contribute TextMate grammar and/or Tree-sitter-backed grammar produced by or validated against Phase 36 syntax assets.
- [ ] Add grammar drift tests that consume fixtures from the main repository or a pinned generated artifact.

## LSP Client Launcher

- [ ] Launch command defaults to `sifr`.
- [ ] Launch args default to `["lsp", "--stdio"]`.
- [ ] Setting `sifr.lsp.path` overrides the binary path.
- [ ] Setting `sifr.lsp.trace.server` controls protocol tracing.
- [ ] Setting `sifr.diagnostics.mode` maps to Sifr LSP `off`, `open-files`, and `workspace` diagnostics modes.
- [ ] Setting `sifr.lsp.extraEnv` is added only if needed and must not affect semantics.
- [ ] Missing binary startup fails with an actionable setup message.
- [ ] Extension never falls back to Python language servers, Pyright, Ruff Server, or ty.
- [ ] Restart language server command is implemented.
- [ ] Show server logs command is implemented.

## Required Commands

- [ ] `Sifr: Restart Language Server`
- [ ] `Sifr: Show Language Server Logs`
- [ ] `Sifr: Locate Sifr Binary`
- [ ] `Sifr: Run Check`
- [ ] `Sifr: Run Tests`
- [ ] `Sifr: Format Document`
- [ ] `Sifr: Show Generated Rust`
- [ ] `Sifr: Explain Diagnostic`

All commands must call Sifr LSP/CLI surfaces. No command may compute Sifr semantic answers inside the extension.

## Required Editor Features

- [ ] Diagnostics from `sifr lsp --stdio`.
- [ ] Completion and completion resolve.
- [ ] Hover.
- [ ] Signature help.
- [ ] Go to definition.
- [ ] Go to declaration.
- [ ] Go to type definition.
- [ ] References.
- [ ] Prepare rename and rename.
- [ ] Document symbols.
- [ ] Workspace symbols.
- [ ] Semantic tokens.
- [ ] Inlay hints.
- [ ] Document highlights.
- [ ] Folding ranges.
- [ ] Code actions from Sifr diagnostic suggestions.
- [ ] Document formatting.
- [ ] Range formatting.
- [ ] Generated Rust preview.
- [ ] Explain diagnostic.
- [ ] VS Code Test Explorer integration backed by Sifr test discovery and CLI test commands.

## Generated Rust Preview

- [ ] Preview command calls the Sifr LSP workspace command or reviewed Sifr CLI surface.
- [ ] Preview supports current file.
- [ ] Preview supports current selection when Sifr source maps can provide a span.
- [ ] Preview preserves source map context or at least identifies the source span.
- [ ] Preview fails with a Sifr diagnostic or actionable editor error when codegen cannot produce a preview.
- [ ] Extension does not run an editor-owned lowering/codegen path.

## Diagnostics And Code Actions

- [ ] Diagnostics preserve Sifr codes, severities, ranges, related information, tags, URLs, child notes/help where LSP can represent them, and structured suggestion applicability.
- [ ] Code actions expose Sifr diagnostic suggestions.
- [ ] Suppression insertion is offered only for suppressible policy rules.
- [ ] Hard correctness diagnostics never offer suppression.
- [ ] Unknown/unused suppression diagnostics display normally.
- [ ] Diagnostic explain command routes to Sifr.

## Formatting And Linting

- [ ] Format document uses Sifr LSP formatting or `sifr fmt`.
- [ ] Range formatting uses Sifr LSP range formatting.
- [ ] Check-format command uses `sifr fmt --check` if exposed.
- [ ] Lint command uses `sifr lint`.
- [ ] Extension does not contain a formatter or linter implementation.

## Test Explorer

- [ ] Test discovery uses Sifr LSP/CLI test metadata.
- [ ] Test item ids are stable enough for VS Code refresh/run flows.
- [ ] Run single test.
- [ ] Run file/module tests.
- [ ] Run workspace tests.
- [ ] Surface test diagnostics/output without parsing Sifr semantics in the extension.
- [ ] Handle projects without tests by showing an empty test tree, not guessed Python-style tests.

## Packaging And Publication Readiness

- [ ] `.vsix` packaging works locally.
- [ ] Extension metadata is complete:
  - display name
  - description
  - categories
  - keywords
  - icon
  - repository URL
  - license
  - minimum VS Code engine version
- [ ] Changelog is present.
- [ ] Marketplace publication checklist is documented.
- [ ] Actual marketplace upload is left to Phase 39 release governance if credentials or release approvals are required.

## Cross-Repository Validation

Main `sifr-lang/sifr` repository must own or pin enough validation to prevent extension drift:

- [ ] `verification/tooling/vscode_extension_contract.json` records language id, extension id, settings, commands, launch command, and repository boundary.
- [ ] `verification/tooling/check_vscode_extension_contract.py` validates the main-repo contract against the extension repo.
- [ ] `verification/tooling/check_vscode_extension.py` validates extension build/test/package behavior for the located extension repo.
- [ ] Main-repo validation locates the extension repo by first checking `SIFR_VSCODE_REPO` as an absolute path, then a sibling `../sifr-vscode` checkout relative to the main repo root.
- [ ] Main-repo validation fails with an actionable message if the extension repo cannot be found; it must not silently skip once Phase 36 extension validation is active.
- [ ] Contract check fails if the extension declares parser/type-checker/formatter/linter/codegen behavior.
- [ ] Contract check fails if the extension launch command is not `sifr lsp --stdio`.
- [ ] Contract check fails if required settings or commands are missing.
- [ ] Contract check fails if package/test commands are not reproducible.

Extension repository validation must include:

- [ ] package install
- [ ] TypeScript typecheck
- [ ] lint
- [ ] unit tests
- [ ] VS Code extension host tests
- [ ] `.vsix` package build
- [ ] smoke test launching a locally built `sifr lsp --stdio`

## PR Sequence

All work stays sequential. Do not begin the next PR until the current PR is merged and the checklist is updated.

1. [ ] Main repo PR (`milestone_36_1`): lock extension repo boundary, extension contract JSON, validation command, and this issue checklist.
2. [ ] Extension repo PR (`milestone_36_7`): scaffold package, CI, language contribution, and grammar wiring.
3. [ ] Extension repo PR (`milestone_36_7`): LSP launcher, settings, binary discovery, restart/log commands, and smoke tests.
4. [ ] Extension repo PR (`milestone_36_7`): editor feature wiring for LSP diagnostics, completion, hover, navigation, symbols, semantic tokens, inlay hints, folding, highlights, code actions, formatting, and rename.
5. [ ] Extension repo PR (`milestone_36_7`): generated Rust preview and explain diagnostic commands.
6. [ ] Extension repo PR (`milestone_36_7`): VS Code Test Explorer integration.
7. [ ] Main repo PR (`milestone_36_7` -> `milestone_36_8` handoff): `verification/tooling/check_vscode_extension_contract.py`, `verification/tooling/check_vscode_extension.py`, documentation, and validation evidence.
8. [ ] Phase 36 closeout PR (`milestone_36_8`): package evidence, validation evidence, publication checklist, and reviewer approval.

PR-to-milestone mapping:

- PR 1 belongs to `milestone_36_1`.
- PRs 2 through 6 belong to `milestone_36_7`.
- PR 7 closes the main-repo validation handoff required before `milestone_36_8`.
- PR 8 belongs to `milestone_36_8`.

## Exit Gate

- [ ] Extension repository boundary is finalized.
- [ ] Extension builds, tests, and packages as `.vsix`.
- [ ] Extension launches `sifr lsp --stdio`.
- [ ] Extension delegates all Sifr semantics to the LSP/CLI.
- [ ] Required commands and settings exist.
- [ ] VS Code Test Explorer is backed by Sifr test metadata.
- [ ] Generated Rust preview and explain diagnostic commands call Sifr.
- [ ] Main repo cross-repo contract checks pass and fail on seeded drift.
- [ ] `scripts/run_all_tests.sh --profile quick` passes in the main repo.
- [ ] Extension repo CI passes.
- [ ] Reviewer approves the extension as production-grade.
