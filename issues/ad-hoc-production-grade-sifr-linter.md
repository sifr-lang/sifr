# Ad Hoc Phase: Production-Grade Sifr Linter

Status: planned on 2026-05-26

## Purpose

Build a production-grade Sifr linter by reusing as much of Ruff's linting architecture as is safe, while keeping Sifr lint semantics, rule IDs, diagnostics, suppressions, and editor behavior Sifr-owned.

The goal is not to port Ruff's Python lint rules. The goal is to avoid rebuilding proven infrastructure: config composition, file discovery, rule selection concepts, phase-gated lint orchestration, suppression mapping, fix application, test patterns, and LSP code-action patterns.

## Source Inputs

This phase is based on:

- Phase 36 tooling contracts in `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`
- Tooling analysis contract in `internal_docs/tooling_analysis.md`
- Tooling reuse strategy in `internal_docs/tooling_reuse_strategy.md`
- Current `sifr_lint` foundation in `crates/sifr_lint`
- Ruff linter docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/linter.md`
- Ruff configuration docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/configuration.md`
- Ruff workspace/config crates in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_workspace`
- Ruff linter crate in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_linter`
- Ruff diagnostics crate in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_diagnostics`
- Ruff server LSP code-action and settings patterns in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_server`
- Local review artifacts:
  - `reviews/sifr-linter-ruff-config-review.md`
  - `reviews/sifr-linter-ruff-registry-rules-review.md`
  - `reviews/sifr-linter-ruff-engine-review.md`
  - `reviews/sifr-linter-ruff-suppression-fixes-review.md`
  - `reviews/sifr-linter-ruff-file-discovery-review.md`
  - `reviews/sifr-linter-ruff-lsp-editor-review.md`
  - `reviews/sifr-linter-ruff-reuse-review-pass-1.md`
  - `reviews/sifr-linter-ruff-reuse-review-pass-2.md`
  - `reviews/sifr-linter-ruff-reuse-review-pass-3.md`

## Quality Contract

Entry criteria:

- Phase 36 is complete.
- `sifr_lint` exists as the Sifr-owned policy-rule crate.
- The Ruff linter reuse audit artifacts above are checked in and reviewed.
- This phase plan is reviewed and approved before implementation starts.

Exit criteria:

- `sifr lint` is production-grade for Sifr policy rules.
- Lint config, rule registry, suppressions, file discovery, fix applicability, LSP diagnostics, and code actions are implemented through Sifr-owned APIs.
- Language-neutral Ruff infrastructure is reused or adapted where practical.
- Python lint semantics, Python rule IDs, and Python project/module behavior do not become Sifr lint authority.
- Full local validation passes or any inherited unrelated gate failure is recorded with proof that this phase did not cause it.

Required quality controls:

- Hard compiler diagnostics remain unsuppressible and cannot be downgraded.
- Policy diagnostics are configurable, suppressible only by explicit Sifr rule IDs, and emitted through `sifr_diagnostics`.
- `sifr_lint` must not import `ruff_linter::rules::*`, `ruff_linter::registry` as production registry, `ruff_linter::linter` as production orchestration, `ruff_python_semantic`, Python project/module resolution, or Ruff Server diagnostic behavior as semantic authority.
- `sifr_lsp` must remain a protocol adapter. Lint diagnostics flow through `sifr_analysis` into `sifr_lint`.
- LSP suppression and fix code actions must be gated by a typed diagnostic class, not diagnostic-code string prefixes.
- Parser-aware suppression ranges must be implemented before adding syntax, HIR, or workspace lint rules.
- The parser-aware suppression gate must be mechanically enforced through Rust types. Syntax, HIR, and workspace lint-rule modules must depend on the parser-aware suppression API at compile time; if that API is absent or bypassed, those modules fail to compile or fail the linter reuse contract check.
- Fix-capable lint rules must define applicability, conflict handling, formatter interaction, idempotence, and safety tests before they are enabled.
- Any adapted Ruff code must be dependency-audited and Sifr-owned at the API boundary.

## Problem Statement

The current `sifr_lint` crate is a Phase 36 foundation. It provides:

- Sifr-owned rule metadata
- `RuleSeverity` and `RuleStatus`
- `# sifr: ignore[rule-id]`
- unknown, unused, and blanket suppression diagnostics
- a physical-line `trailing-whitespace` rule
- simple `.sifr` file collection

That is enough to prove the tooling boundary, but it is not a production linter. Production Sifr linting needs:

- real lint config in `sifr.toml`
- rule selection and severity resolution
- per-file ignores and discovery/exclusion parity
- parser-aware suppression mapping
- phase-gated lint orchestration
- syntax, HIR, workspace, and fix-capable rule families
- LSP/editor diagnostics and code actions
- regression checks that prevent Python lint dependencies from leaking into Sifr

Ruff already has battle-tested infrastructure for many of these concerns. Sifr should reuse that work where it is language-neutral, but not inherit Python lint semantics.

## Product Decision

Sifr will build a Sifr-owned linter with a Ruff-inspired architecture.

The architecture is:

```text
.sifr source
  -> sifr_syntax
  -> sifr_frontend / read-only HIR and workspace views
  -> sifr_lint
       - Sifr rule registry
       - Sifr config and rule selection
       - phase-gated lint runner
       - Sifr suppression/fix engine
  -> sifr_analysis
  -> sifr lint and sifr_lsp diagnostics/code actions
```

Ruff linter code is classified as one of:

- `reuse-direct`: use the same crate/API directly because it is language-neutral and already acceptable.
- `adapt`: copy or reimplement the pattern behind a Sifr-owned API after dependency audit.
- `reference-only`: use for design guidance; implementation is Sifr-native.
- `reject`: do not depend on it for production lint behavior.

## Ruff Linter Reuse Matrix

| Ruff area | Decision | Sifr requirement |
| --- | --- | --- |
| Ruff fork parser/AST/token/trivia/source ranges | reuse-direct through `sifr_syntax` | `sifr_lint` consumes syntax through Sifr wrappers, not raw linter entrypoints |
| `ruff_text_size`, source range utilities | reuse-direct where already in workspace | Keep source offsets/ranges compatible with Sifr diagnostics and LSP conversions |
| Ruff workspace config composition | adapt | Implement `sifr.toml` lint config with Ruff-style layering, extends, CLI/editor overrides, diagnostics for unknown keys, and cycle detection |
| Ruff `pyproject.toml` authority | reject | Sifr config authority is `sifr.toml`; Ruff lint config migration requires a separate reviewed migration phase |
| Python `target-version`, per-file target version | reject | No Sifr lint behavior depends on Python versions |
| Ruff plugin settings (`pyflakes`, `pycodestyle`, `isort`, `pydocstyle`, etc.) | reject | No Python plugin config leaks into Sifr |
| File resolver settings: include/exclude/extend-exclude/force-exclude/respect-gitignore | adapt | Use Ruff's product model with `.sifr` defaults and Sifr explicit-target semantics |
| `ignore` walker and `globset` matching | reuse-direct or adapt | Replace naive path matching with robust glob/gitignore discovery |
| Ruff package-root detection (`__init__.py`) | reject | Sifr uses Sifr workspace/package semantics, not Python package roots |
| Ruff cache key primitives | adapt later | Use Sifr cache namespace and keys covering source metadata, config, Sifr version, and rule registry revision |
| Ruff rule registry contents | reject | Sifr owns rule IDs, categories, status, docs URLs, and defaults |
| Ruff registry/code generation pattern | reference-only initially | Static registry is acceptable until rule count justifies macro generation |
| Ruff `RuleSelector` prefix/specificity model | adapt later | Add Sifr rule selectors once rule count/categories require it |
| Ruff rule redirects | adapt | Keep deprecated Sifr rule IDs working with explicit replacement metadata |
| `ruff_linter::rules::*` | reject | Python AST/Python semantic rule implementations are not Sifr lint rules |
| Ruff `SemanticModel` / Pyflakes binding model | reject | Sifr semantic lint rules use `sifr_frontend` and HIR views |
| Ruff linter `SourceKind` | reject | Sifr source kind is `.sifr`; notebooks are out of scope |
| Ruff linter phase ordering | adapt | Build a Sifr phase-gated lint runner: file, token, physical line, syntax, HIR, workspace, suppression, per-file ignores, fixes |
| Ruff AST checker implementation | reference-only | Implement Sifr syntax/HIR checkers natively |
| Ruff logical/physical line checker pattern | adapt | Use for line-based Sifr policy rules |
| Ruff `noqa` syntax | reject | Sifr suppression syntax is `# sifr: ignore[rule-id]` |
| Ruff `NoqaMapping`/directive lookup pattern | adapt | Implement Sifr parser-aware suppression mapping for multi-line statements/ranges |
| Ruff file-level blanket `noqa` | reject unless reviewed later | Blanket suppressions remain forbidden in this phase |
| Ruff diagnostic type | reference-only | Keep `sifr_diagnostics` as canonical diagnostic model |
| Ruff `Fix`, applicability, isolation, apply-fixes algorithm | adapt | Map to Sifr `SuggestionApplicability` and implement Sifr-owned conflict handling/source maps |
| Ruff code-action deferred resolution | adapt | Use for fix-all, rule suppression, and future organize/import actions through `sifr_lsp` |
| Ruff workspace edit tracker | adapt | Add version-aware edit tracking for lint fixes/code actions |
| Ruff fix-all | adapt with policy-only gate | Fix-all applies only safe policy fixes, never hard compiler diagnostics |
| Ruff organize imports/isort | reference-only or reject | Requires a separate Sifr import-organization lint/fix phase after Sifr import semantics are specified |
| Ruff server settings model | adapt | Add editor/global/workspace lint settings with config-preference behavior cleaned of Python options |
| Ruff LSP diagnostic data payload | adapt | Add typed diagnostic class and code-action metadata for hard vs policy diagnostics |

## Sifr Lint Architecture Requirements

### Rule ownership

- Sifr rule IDs are Sifr-owned.
- Rule metadata includes ID, summary, docs URL, default severity, status, source location, category, fix availability, and suppression complexity.
- Deprecated rules retain their IDs for at least two minor releases and point to replacements when possible.
- Rule categories must be Sifr concepts, not Flake8 or Ruff plugin categories.

### Diagnostic classes

Every diagnostic surfaced through analysis/LSP must carry a class:

- `Hard`: parse, type, ownership, result/option, runtime-safety, and workspace correctness diagnostics. These are unsuppressible and not part of fix-all.
- `Policy`: `sifr_lint` diagnostics. These can be configured, suppressed by explicit rule ID, and used for policy code actions.

LSP code actions must use this typed class. String-prefix checks such as `SIFR-LINT-*` are not sufficient as the production gate.

### Suppression complexity

Every policy rule must declare one suppression complexity:

1. `physical-line`: diagnostic is tied to a single physical source line.
2. `single-node`: diagnostic is tied to one syntax node.
3. `statement-range`: diagnostic can span a multi-line statement, block arm, function, class, match/case, or ownership/type construct.
4. `symbol-workspace`: diagnostic is tied to a symbol, HIR item, import graph, or workspace result.

Current line-based suppression is valid only for `physical-line` rules. Before any other category ships, `sifr_lint` must attach `# sifr: ignore[rule-id]` comments through `sifr_syntax` statement/range mapping.

### Config ownership

Canonical lint config lives in `sifr.toml`:

```toml
[lint]
preview = false
select = ["default"]
extend-select = []
ignore = []
fixable = []
unfixable = []
unsafe-fixes = "hint"
include = ["*.sifr"]
exclude = []
extend-exclude = []
respect-gitignore = true
force-exclude = false

[lint.rules]
trailing-whitespace = "warn"

[lint.per-file-ignores]
"demos/generated/*.sifr" = ["trailing-whitespace"]
```

Required config semantics:

- CLI/editor overrides take precedence over discovered config.
- `sifr.toml` is authoritative.
- Ruff lint config files are not read implicitly.
- Unknown Sifr lint keys are deterministic diagnostics.
- Python-only Ruff keys are deterministic unsupported-option diagnostics only if a future migration mode explicitly reads Ruff configs.
- Extends are path-relative, ordered, cycle-detected, and tested.
- Explicit file targets are linted even when they match excludes unless `force-exclude` is active.
- `unsafe-fixes = "hint"` means unsafe fixes are surfaced as unavailable/user-confirmation-required suggestions but are not applied automatically. Future accepted values are `disabled`, `hint`, and `enabled`; `enabled` still applies only to policy diagnostics and never to hard compiler diagnostics.

### Lint runner phases

The production runner must be phase-gated. It should skip phases that have no enabled rules.

Required phases:

- file/discovery rules
- token/trivia rules
- physical-line rules
- syntax-node rules
- statement-range rules
- HIR/frontend rules
- workspace/import rules
- suppression filtering
- per-file ignore filtering
- fix applicability filtering
- deterministic diagnostic sorting

### Import organization boundary

Import sorting and import organization are not part of this phase unless a milestone explicitly adds Sifr import-order semantics first. Ruff's isort behavior is Python-specific. Any future Sifr import organization rule must define:

- Sifr import/workspace semantics used by the rule
- whether the rule is diagnostic-only or fix-capable
- interaction with package resolution and generated materialization
- formatter interaction and idempotence expectations
- LSP organize-import behavior, if any

### Fixes

Fix-capable rules must not ship until the fix engine supports:

- Sifr `SuggestionApplicability`
- safe vs unsafe policy
- non-overlap and grouped edit isolation
- source-map/edit tracking for LSP
- formatter interaction
- idempotence tests
- fix-all limited to safe policy fixes
- no fixes for hard compiler diagnostics

## Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | Ruff linter reuse matrix is encoded in machine-readable or checkable form before implementation starts |
| AC-2 | `sifr_lint` has a Sifr-owned rule registry with metadata, status, docs URLs, categories, fix availability, and suppression complexity |
| AC-3 | `sifr.toml` lint config supports rule selection, severity overrides, per-file ignores, include/exclude, gitignore, extends, and deterministic diagnostics |
| AC-4 | File discovery uses robust glob/gitignore behavior and preserves explicit-target semantics |
| AC-5 | Parser-aware suppression mapping supports multi-line syntax/HIR diagnostics before any non-physical-line rules ship |
| AC-6 | Hard vs policy diagnostic class is present in analysis/LSP diagnostic data and code-action gating |
| AC-7 | `sifr lint`, `sifr_analysis`, and `sifr_lsp` share one lint engine and produce equivalent policy diagnostics for the same source/options |
| AC-8 | LSP suppression code actions are offered only for policy diagnostics and never for hard compiler diagnostics |
| AC-9 | Fix-capable lint rules have applicability, conflict, source-map, formatter, and idempotence coverage before enabling fix-all |
| AC-10 | Guardrails reject `ruff_linter::rules`, Python semantic/project/runtime dependencies, Ruff rule IDs, and extension/editor-owned linter behavior |
| AC-11 | Docs explain lint config, rules, suppressions, fix safety, editor behavior, and non-reused Ruff/Python behavior |
| AC-12 | Full local validation passes before phase closure |
| AC-13 | A mechanical gate prevents syntax, HIR, and workspace lint rules from shipping before parser-aware suppression support is enabled |
| AC-14 | Unsafe fixes are never applied automatically unless the rule is policy-only, the fix is explicitly enabled, and the edit applicability permits it |

## Milestone Breakdown

Milestones are sequential. Each milestone closes with validation evidence and review before the next starts.

### Milestone 1: `lint_reuse_contract_and_manifests`

Goal: lock the Ruff reuse decisions into enforceable contracts.

Scope:

- create a linter reuse manifest matching this document
- create a lint rule metadata manifest
- create `verification/tooling/check_linter_reuse_contract.py`
- make `check_linter_reuse_contract.py` verify:
  - `crates/sifr_lint/Cargo.toml` does not depend on forbidden Ruff/Python lint crates
  - `cargo tree -p sifr_lint` does not contain `ruff_linter`, `ruff_python_semantic`, Python project/runtime crates, or Ruff Server semantic behavior
  - production Sifr crates do not import `ruff_linter::rules`, `ruff_linter::registry`, `ruff_linter::linter`, `ruff_linter::noqa`, Python `Rule` IDs, or `ruff_python_semantic`
  - seeded negative fixtures fail the check
- create a placeholder lint config schema manifest
- create `verification/tooling/linter_manifests/suppression_gate.json`
- define the suppression-gate manifest schema:
  - `schema`: integer schema version
  - `gate_state`: `physical_line_only` or `parser_aware`
  - `allowed_rule_families`: array of `physical-line`, `single-node`, `statement-range`, `symbol-workspace`
  - `parser_aware_api`: Rust path that non-physical-line rule modules must depend on
  - `updated_by_milestone`: milestone identifier that last changed the gate
- initialize the suppression-gate manifest with `gate_state = "physical_line_only"` and `allowed_rule_families = ["physical-line"]`
- make `check_linter_reuse_contract.py` validate the suppression-gate manifest path, schema, and state
- update internal docs to link this phase and the reuse audit artifacts

Validation:

- manifest self-tests
- forbidden dependency guardrail and self-test
- `python3 verification/tooling/check_linter_reuse_contract.py`
- `python3 verification/tooling/check_linter_reuse_contract.py --self-test`
- suppression-gate manifest schema validation
- `git diff --check`

Review gate:

- external review confirms there are no unclassified Ruff linter subsystems or hidden Python semantic dependencies

### Milestone 2: `lint_config_and_file_discovery`

Goal: make lint configuration and file discovery production-grade.

Scope:

- implement `[lint]`, `[lint.rules]`, and `[lint.per-file-ignores]` in `sifr.toml`
- implement Ruff-inspired config layering, extends, overrides, unknown-key diagnostics, and cycle detection
- replace naive path matching with robust glob/gitignore discovery
- support include, exclude, extend-exclude, force-exclude, respect-gitignore, and explicit-target behavior
- add negative fixtures for deep directory traversal, ignored directories, symlink loops or cycles where supported by the walker, and pathological file counts within the local validation budget

Validation:

- `cargo test -p sifr_lint`
- config precedence fixtures
- file discovery fixtures and negative tests

Review gate:

- external review confirms config/discovery reuse is language-neutral and Sifr-owned

### Milestone 3: `parser_aware_suppression_engine`

Goal: make suppressions correct for all future rule families.

Scope:

- replace line-only suppression attachment with parser-aware statement/range mapping
- expose a typed parser-aware suppression API, tentatively `sifr_lint::suppression::ParserAwareSuppressions`, that non-physical-line rule modules must use to register suppressible diagnostics
- support physical-line, single-node, statement-range, and symbol/workspace suppression complexity
- keep blanket suppressions forbidden
- keep blanket suppression reporting as a policy diagnostic; any future blanket suppression support requires a reviewed planning update and explicit feature/config gate
- report unknown and unused suppressions deterministically
- add multi-line suppression fixtures for calls, functions, classes, match/case, ownership/type constructs, and HIR diagnostics
- update `verification/tooling/linter_manifests/suppression_gate.json` to `gate_state = "parser_aware"` and `allowed_rule_families = ["physical-line", "single-node", "statement-range", "symbol-workspace"]`
- update `check_linter_reuse_contract.py` so any syntax, HIR, or workspace rule module that bypasses `ParserAwareSuppressions` fails validation

Validation:

- `cargo test -p sifr_lint`
- suppression contract checks and self-tests
- guardrail proving syntax/HIR/workspace rules fail validation if they bypass the parser-aware suppression API
- `python3 verification/tooling/check_linter_reuse_contract.py`
- suppression-gate manifest state transition check

Review gate:

- external review confirms non-trivial rules cannot ship with line-only suppression semantics

### Milestone 4: `phase_gated_lint_engine`

Goal: implement the Ruff-inspired Sifr lint runner.

Scope:

- add phase-gated orchestration for file, token, line, syntax, statement, HIR, workspace, suppression, per-file ignore, fix-filtering, and sorting phases
- preserve current rules through the new runner
- add phase-skip tests proving disabled rule families do not run
- add deterministic ordering and invalid-source behavior

Validation:

- `cargo test -p sifr_lint`
- lint engine phase fixtures
- performance smoke for large files/projects

Review gate:

- external review confirms orchestration reuses Ruff's structure without importing Python checker semantics

### Milestone 5: `sifr_policy_rule_families`

Goal: add production Sifr lint rule families beyond the foundation rule.

Scope:

- add representative token/trivia rules
- add representative syntax rules
- add representative HIR/frontend policy rules
- add workspace/import policy rules only where Sifr workspace/import semantics are already specified
- classify every rule by category, suppression complexity, default severity, status, and fix availability
- keep the static `RULES` slice until the shipped policy-rule count exceeds 50. At that point, implementation must add a reviewed planning update before introducing a `RuleSelector` specificity system or macro-generated registry.
- keep hard correctness diagnostics out of `sifr_lint`
- explicitly exclude import ordering rules unless Sifr import-order semantics are specified in this or a later reviewed phase

Validation:

- targeted rule tests
- snapshot fixtures
- unknown/unused suppression fixtures
- full lint diagnostics parity across CLI and analysis

Review gate:

- external review confirms every rule is Sifr-semantic and no Python rule was ported mechanically

### Milestone 6: `lint_fixes_and_code_actions`

Goal: add safe lint fixes and editor actions.

Scope:

- M6a: implement Sifr-owned fix applicability and edit isolation using Ruff-inspired patterns
- M6a: implement fix conflict resolution, deterministic fix ordering, and synchronous code actions for safe policy fixes and explicit suppressions
- M6a: keep fix-all policy-only and safe-by-default
- M6b: implement source-map/workspace edit tracking
- M6b: add deferred code-action resolution for expensive edits and multi-file/workspace edits
- M6b: add version-aware edit conflict handling for LSP

Validation:

- `cargo test -p sifr_lint`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- fix idempotence and conflict fixtures
- LSP code-action smoke/stress coverage

Review gate:

- external review confirms hard diagnostics cannot be suppressed or auto-fixed

### Milestone 7: `lsp_editor_docs_and_closeout`

Goal: close the phase with editor parity, docs, and production evidence.

Scope:

- update `sifr lint` docs
- update LSP/editor integration docs for lint diagnostics, suppressions, fix actions, and settings
- update VS Code and non-VS Code editor contracts if lint settings or actions change
- update `internal_docs/tooling_analysis.md`, `internal_docs/lsp_server.md`, and verification docs
- run final local validation and production-readiness review

Validation:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
- editor asset checks
- VS Code extension contract/package checks
- final lint reuse/production-readiness review

Review gate:

- final review confirms the linter is production-grade, Sifr-owned, and maximally reuse-informed without Python semantic leakage

## Validation Plan

Every implementation PR must run the narrowest relevant local checks plus targeted lint tests. Phase closure requires:

```bash
cargo test -p sifr_lint
cargo test -p sifr_analysis
cargo test -p sifr_lsp
cargo test -p sifr
python3 verification/tooling/check_linter_reuse_contract.py
python3 verification/tooling/check_linter_reuse_contract.py --self-test
python3 verification/tooling/check_rule_suppression_contract.py
python3 verification/tooling/check_rule_suppression_contract.py --self-test
python3 verification/tooling/check_tooling_dependency_boundaries.py
python3 verification/tooling/check_lsp_split_brain.py
scripts/run_all_tests.sh --profile quick
scripts/run_all_tests.sh
```

## Review Requirements

- This planning phase must be externally reviewed before implementation starts.
- Each milestone must include review focused on Ruff reuse boundaries, Sifr semantic ownership, validation coverage, and docs.
- Phase closure requires final review confirming:
  - no `ruff_linter` Python rule engine dependency
  - no Python semantic/project/runtime authority
  - hard vs policy diagnostics are enforced
  - suppressions are parser-aware for non-line rules
  - LSP/editor behavior uses Sifr-owned lint diagnostics and actions

## Execution Log

- `2026-05-26`: phase drafted after local Ruff linter/config/server scan and Claude subsystem review passes.
